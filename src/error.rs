// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2018 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use crate::binding::dpiContext_getError;
use crate::binding::dpiErrorInfo;
use crate::to_rust_str;
use crate::AssertSend;
use crate::AssertSync;
use crate::Context;
#[cfg(doc)]
use crate::{Batch, BatchBuilder, Connection, Statement};
use std::borrow::Cow;
use std::error;
use std::ffi::CStr;
use std::fmt;
#[cfg(feature = "struct_error")]
use std::fmt::Display;
use std::mem::MaybeUninit;
use std::num;
use std::str;
use std::sync;

// DPI-1010: not connected
pub(crate) const DPI_ERR_NOT_CONNECTED: i32 = 1010;

// DPI-1019: buffer size of %u is too small
pub(crate) const DPI_ERR_BUFFER_SIZE_TOO_SMALL: i32 = 1019;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
/// A list of error categories.
///
/// It is used with the [`Error`] type.
///
/// Use `_` to match “all other errors” in `match` expression because it has [`#[non_exhaustive]`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute) attribute.
pub enum ErrorKind {
    /// Error from an underlying Oracle client library.
    OciError,

    /// Error from an underlying ODPI-C layer.
    DpiError,

    /// Error when NULL value is got but the target rust type cannot handle NULL.
    /// Use `Option<...>` in this case.
    NullValue,

    /// Error when conversion from a string to an Oracle value fails
    ParseError,

    /// Error when conversion from a type to another fails due to out-of-range
    OutOfRange,

    /// Error when an unacceptable argument is passed
    InvalidArgument,

    /// Error when conversion from a type to another is not allowed.
    InvalidTypeConversion,

    /// Error when the bind parameter index is out of range. (one based)
    InvalidBindIndex,

    /// Error when the bind parameter name is not in the SQL.
    InvalidBindName,

    /// Error when the column index is out of range. (zero based)
    InvalidColumnIndex,

    /// Error when the column name is not in the SQL.
    InvalidColumnName,

    /// Error when the specified attribute name is not found.
    InvalidAttributeName,

    /// Error when invalid method is called such as calling execute for select statements.
    InvalidOperation,

    /// Error when an uninitialized bind value is accessed. Bind values
    /// must be initialized by [`Statement::bind`], [`Statement::execute`]
    /// or [`Connection::execute`] in advance.
    UninitializedBindValue,

    /// Error when no more rows exist in the SQL.
    NoDataFound,

    /// Error when [`BatchBuilder::with_batch_errors`] is set and [`Batch::execute`]
    /// fails by supplied batch data.
    /// See ["Error Handling with batch errors"](Batch#error-handling-with-batch-errors)
    BatchErrors,

    /// Internal error. When you get this error, please report it with a test case to reproduce it.
    InternalError,
}

/// The error type for oracle
#[derive(Debug)]
#[cfg(feature = "struct_error")]
pub struct Error {
    kind: ErrorKind,
    message: Cow<'static, str>,
    // DbError is in Box to reduce the size of this struct.
    // See: https://rust-lang.github.io/rust-clippy/master/index.html#result_large_err
    dberr: Option<Box<DbError>>,
    batch_errors: Option<Vec<DbError>>,
    source: Option<Box<dyn error::Error + Send + Sync>>,
}

/// The error type for oracle
///
/// **Note:** This enum will be changed to struct in the future.
#[non_exhaustive]
#[derive(Debug)]
#[cfg(not(feature = "struct_error"))]
pub enum Error {
    /// Error from an underlying Oracle client library.
    #[deprecated(note = "Use kind() to check the error category. Use db_error() to get DbError.")]
    OciError(DbError),

    /// Error from an underlying ODPI-C layer.
    #[deprecated(note = "Use kind() to check the error category. Use db_error() to get DbError.")]
    DpiError(DbError),

    /// Error when NULL value is got but the target rust type cannot handle NULL.
    /// Use `Option<...>` in this case.
    #[deprecated(note = "Use kind() to check the error category.")]
    NullValue,

    /// Error when conversion from a string to an Oracle value fails
    #[deprecated(
        note = "Use kind() to check the error category. Use source() to get the underlying error."
    )]
    ParseError(Box<dyn error::Error + Send + Sync>),

    /// Error when conversion from a type to another fails due to out-of-range
    #[deprecated(
        note = "Use kind() to check the error category. Use to_string() to get the message."
    )]
    OutOfRange(String),

    /// Error when conversion from a type to another is not allowed.
    #[deprecated(note = "Use kind() to check the error category.")]
    InvalidTypeConversion(String, String),

    /// Error when an unacceptable argument is passed
    #[deprecated]
    InvalidArgument {
        message: Cow<'static, str>,
        source: Option<Box<dyn error::Error + Send + Sync>>,
    },

    /// Error when the bind parameter index is out of range. (one based)
    #[deprecated(note = "Use kind() to check the error category.")]
    InvalidBindIndex(usize),

    /// Error when the bind parameter name is not in the SQL.
    #[deprecated(note = "Use kind() to check the error category.")]
    InvalidBindName(String),

    /// Error when the column index is out of range. (zero based)
    #[deprecated(note = "Use kind() to check the error category.")]
    InvalidColumnIndex(usize),

    /// Error when the column name is not in the SQL.
    #[deprecated(note = "Use kind() to check the error category.")]
    InvalidColumnName(String),

    /// Error when the specified attribute name is not found.
    #[deprecated(note = "Use kind() to check the error category.")]
    InvalidAttributeName(String),

    /// Error when invalid method is called such as calling execute for select statements.
    #[deprecated(
        note = "Use kind() to check the error category. Use to_string() to get the message."
    )]
    InvalidOperation(String),

    /// Error when an uninitialized bind value is accessed. Bind values
    /// must be initialized by [`Statement::bind`], [`Statement::execute`]
    /// or [`Connection::execute`] in advance.
    #[deprecated(note = "Use kind() to check the error category.")]
    UninitializedBindValue,

    /// Error when no more rows exist in the SQL.
    #[deprecated(note = "Use kind() to check the error category.")]
    NoDataFound,

    #[deprecated(
        note = "Use kind() to check the error category. Use batch_errors() to get the db errors."
    )]
    BatchErrors(Vec<DbError>),

    /// Internal error. When you get this error, please report it with a test case to reproduce it.
    #[deprecated(
        note = "Use kind() to check the error category. Use to_string() to get the message."
    )]
    InternalError(String),
}

impl Error {
    pub(crate) fn from_context(ctxt: &Context) -> Error {
        let err = unsafe {
            let mut err = MaybeUninit::uninit();
            dpiContext_getError(ctxt.context, err.as_mut_ptr());
            err.assume_init()
        };
        Error::from_dpi_error(&err)
    }

    pub(crate) fn from_dpi_error(err: &dpiErrorInfo) -> Error {
        Error::from_db_error(DbError::from_dpi_error(err))
    }
}

#[cfg(feature = "struct_error")]
impl Error {
    pub(crate) fn new<M>(kind: ErrorKind, message: M) -> Error
    where
        M: Into<Cow<'static, str>>,
    {
        Error {
            kind,
            message: message.into(),
            dberr: None,
            batch_errors: None,
            source: None,
        }
    }

    pub(crate) fn add_dberr(self, dberr: DbError) -> Error {
        Error {
            dberr: Some(Box::new(dberr)),
            ..self
        }
    }

    pub(crate) fn add_batch_errors(self, batch_errors: Vec<DbError>) -> Error {
        Error {
            batch_errors: Some(batch_errors),
            ..self
        }
    }

    pub(crate) fn add_source<E>(self, source: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error {
            source: Some(source.into()),
            ..self
        }
    }

    pub(crate) fn from_db_error(dberr: DbError) -> Error {
        let (kind, message_prefix) = if dberr.message().starts_with("DPI") {
            (ErrorKind::DpiError, "DPI")
        } else {
            (ErrorKind::OciError, "OCI")
        };
        Error::new(kind, format!("{} Error: {}", message_prefix, dberr.message)).add_dberr(dberr)
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns [`DbError`].
    pub fn db_error(&self) -> Option<&DbError> {
        self.dberr.as_ref().map(|b| b.as_ref())
    }

    /// Returns batch errors.
    /// See ["Error Handling with batch errors"](Batch#error-handling-with-batch-errors)
    pub fn batch_errors(&self) -> Option<&Vec<DbError>> {
        self.batch_errors.as_ref()
    }

    /// Returns Oracle error code.
    /// For example 1 for "ORA-0001: unique constraint violated"
    pub fn oci_code(&self) -> Option<i32> {
        match (self.kind, &self.dberr) {
            (ErrorKind::OciError, Some(dberr)) if dberr.code != 0 => Some(dberr.code),
            _ => None,
        }
    }

    /// Returns [ODPI-C](https://oracle.github.io/odpi/) error code.
    pub fn dpi_code(&self) -> Option<i32> {
        match (self.kind, &self.dberr) {
            (ErrorKind::DpiError, Some(dberr)) => dpi_error_in_message(&dberr.message),
            _ => None,
        }
    }

    pub(crate) fn oci_error(dberr: DbError) -> Error {
        Error::new(ErrorKind::OciError, format!("OCI Error: {}", dberr.message)).add_dberr(dberr)
    }

    pub(crate) fn null_value() -> Error {
        Error::new(ErrorKind::NullValue, "NULL value found")
    }

    pub(crate) fn parse_error<T>(source: T) -> Error
    where
        T: Into<Box<dyn error::Error + Send + Sync>>,
    {
        let source = source.into();
        Error::new(ErrorKind::ParseError, format!("{}", source)).add_source(source)
    }

    pub(crate) fn out_of_range<T>(message: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::new(ErrorKind::OutOfRange, message.into())
    }

    pub(crate) fn invalid_type_conversion<T1, T2>(from: T1, to: T2) -> Error
    where
        T1: Display,
        T2: Display,
    {
        Error::new(
            ErrorKind::InvalidTypeConversion,
            format!("invalid type conversion from {} to {}", from, to),
        )
    }

    pub(crate) fn invalid_bind_index<T>(index: T) -> Error
    where
        T: Display,
    {
        Error::new(
            ErrorKind::InvalidBindIndex,
            format!("invalid bind index {} (one-based)", index),
        )
    }

    pub(crate) fn invalid_bind_name<T>(name: T) -> Error
    where
        T: Display,
    {
        Error::new(
            ErrorKind::InvalidBindName,
            format!("invalid bind name {}", name),
        )
    }

    pub(crate) fn invalid_column_index<T>(index: T) -> Error
    where
        T: Display,
    {
        Error::new(
            ErrorKind::InvalidColumnIndex,
            format!("invalid column index {} (zero-based)", index),
        )
    }

    pub(crate) fn invalid_column_name<T>(name: T) -> Error
    where
        T: Display,
    {
        Error::new(
            ErrorKind::InvalidColumnName,
            format!("invalid column name {}", name),
        )
    }

    pub(crate) fn invalid_attribute_name<T>(name: T) -> Error
    where
        T: Display,
    {
        Error::new(
            ErrorKind::InvalidAttributeName,
            format!("invalid attribute name {}", name),
        )
    }

    pub(crate) fn invalid_operation<T>(message: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::new(ErrorKind::InvalidOperation, message.into())
    }

    pub(crate) fn uninitialized_bind_value() -> Error {
        Error::new(
            ErrorKind::UninitializedBindValue,
            "try to access uninitialized bind value",
        )
    }

    pub(crate) fn no_data_found() -> Error {
        Error::new(ErrorKind::NoDataFound, "no data found")
    }

    pub(crate) fn make_batch_errors(batch_errors: Vec<DbError>) -> Error {
        Error::new(
            ErrorKind::BatchErrors,
            format!("batch error containing {} error(s)", batch_errors.len()),
        )
        .add_batch_errors(batch_errors)
    }

    pub(crate) fn internal_error<T>(message: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::new(ErrorKind::InternalError, message.into())
    }

    pub(crate) fn invalid_argument<M>(message: M) -> Error
    where
        M: Into<Cow<'static, str>>,
    {
        Error::new(ErrorKind::InvalidArgument, message.into())
    }
}

#[allow(deprecated)]
#[cfg(not(feature = "struct_error"))]
impl Error {
    pub(crate) fn from_db_error(dberr: DbError) -> Error {
        if dberr.message().starts_with("DPI") {
            Error::DpiError(dberr)
        } else {
            Error::OciError(dberr)
        }
    }

    /// Returns the corresponding [`ErrorKind`] for this error.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::OciError(_) => ErrorKind::OciError,
            Error::DpiError(_) => ErrorKind::DpiError,
            Error::NullValue => ErrorKind::NullValue,
            Error::ParseError(_) => ErrorKind::ParseError,
            Error::OutOfRange(_) => ErrorKind::OutOfRange,
            Error::InvalidArgument { .. } => ErrorKind::InvalidArgument,
            Error::InvalidTypeConversion(_, _) => ErrorKind::InvalidTypeConversion,
            Error::InvalidBindIndex(_) => ErrorKind::InvalidBindIndex,
            Error::InvalidBindName(_) => ErrorKind::InvalidBindName,
            Error::InvalidColumnIndex(_) => ErrorKind::InvalidColumnIndex,
            Error::InvalidColumnName(_) => ErrorKind::InvalidColumnName,
            Error::InvalidAttributeName(_) => ErrorKind::InvalidAttributeName,
            Error::InvalidOperation(_) => ErrorKind::InvalidOperation,
            Error::UninitializedBindValue => ErrorKind::UninitializedBindValue,
            Error::NoDataFound => ErrorKind::NoDataFound,
            Error::BatchErrors(_) => ErrorKind::BatchErrors,
            Error::InternalError(_) => ErrorKind::InternalError,
        }
    }

    pub(crate) fn add_source<E>(self, source: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        if let Error::InvalidArgument { message, .. } = self {
            Error::InvalidArgument {
                message,
                source: Some(source.into()),
            }
        } else {
            self
        }
    }

    /// Returns [`DbError`].
    pub fn db_error(&self) -> Option<&DbError> {
        match self {
            Error::OciError(err) | Error::DpiError(err) => Some(err),
            _ => None,
        }
    }

    /// Returns batch errors.
    /// See ["Error Handling with batch errors"](Batch#error-handling-with-batch-errors)
    pub fn batch_errors(&self) -> Option<&Vec<DbError>> {
        match self {
            Error::BatchErrors(errs) => Some(errs),
            _ => None,
        }
    }

    /// Returns Oracle error code.
    /// For example 1 for "ORA-0001: unique constraint violated"
    pub fn oci_code(&self) -> Option<i32> {
        if let Error::OciError(dberr) = &self {
            if dberr.code != 0 {
                Some(dberr.code)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns [ODPI-C](https://oracle.github.io/odpi/) error code.
    pub fn dpi_code(&self) -> Option<i32> {
        if let Error::DpiError(dberr) = &self {
            dpi_error_in_message(&dberr.message)
        } else {
            None
        }
    }

    pub(crate) fn oci_error(dberr: DbError) -> Error {
        Error::OciError(dberr)
    }

    pub(crate) fn null_value() -> Error {
        Error::NullValue
    }

    pub(crate) fn parse_error<T>(source: T) -> Error
    where
        T: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error::ParseError(source.into())
    }

    pub(crate) fn out_of_range<T>(message: T) -> Error
    where
        T: Into<String>,
    {
        Error::OutOfRange(message.into())
    }

    pub(crate) fn invalid_type_conversion<T1, T2>(from: T1, to: T2) -> Error
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Error::InvalidTypeConversion(from.into(), to.into())
    }

    pub(crate) fn invalid_bind_index(index: usize) -> Error {
        Error::InvalidBindIndex(index)
    }

    pub(crate) fn invalid_bind_name<T>(name: T) -> Error
    where
        T: Into<String>,
    {
        Error::InvalidBindName(name.into())
    }

    pub(crate) fn invalid_column_index(index: usize) -> Error {
        Error::InvalidColumnIndex(index)
    }

    pub(crate) fn invalid_column_name<T>(name: T) -> Error
    where
        T: Into<String>,
    {
        Error::InvalidColumnName(name.into())
    }

    pub(crate) fn invalid_attribute_name<T>(name: T) -> Error
    where
        T: Into<String>,
    {
        Error::InvalidAttributeName(name.into())
    }

    pub(crate) fn invalid_operation<T>(message: T) -> Error
    where
        T: Into<String>,
    {
        Error::InvalidOperation(message.into())
    }

    pub(crate) fn uninitialized_bind_value() -> Error {
        Error::UninitializedBindValue
    }

    pub(crate) fn no_data_found() -> Error {
        Error::NoDataFound
    }

    pub(crate) fn make_batch_errors(errs: Vec<DbError>) -> Error {
        Error::BatchErrors(errs)
    }

    pub(crate) fn internal_error<T>(message: T) -> Error
    where
        T: Into<String>,
    {
        Error::InternalError(message.into())
    }

    pub(crate) fn invalid_argument<M>(message: M) -> Error
    where
        M: Into<Cow<'static, str>>,
    {
        Error::InvalidArgument {
            message: message.into(),
            source: None,
        }
    }
}

impl AssertSend for Error {}
impl AssertSync for Error {}

/// An error when parsing a string into an Oracle type fails.
/// This appears only in boxed data associated with [`Error::ParseError`].
#[derive(Eq, PartialEq, Clone)]
pub struct ParseOracleTypeError {
    typename: &'static str,
}

impl ParseOracleTypeError {
    pub fn new(typename: &'static str) -> ParseOracleTypeError {
        ParseOracleTypeError { typename }
    }
}

impl fmt::Display for ParseOracleTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} parse error", self.typename)
    }
}

impl fmt::Debug for ParseOracleTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseOracleTypeError")
    }
}

impl error::Error for ParseOracleTypeError {
    fn description(&self) -> &str {
        "Oracle type parse error"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

/// Oracle database error or ODPI-C error
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DbError {
    code: i32,
    offset: u32,
    message: String,
    fn_name: Cow<'static, str>,
    action: Cow<'static, str>,
    is_recoverable: bool,
    is_warning: bool,
}

impl DbError {
    pub(crate) fn from_dpi_error(err: &dpiErrorInfo) -> DbError {
        DbError {
            code: err.code,
            offset: err.offset,
            message: to_rust_str(err.message, err.messageLength),
            fn_name: unsafe { CStr::from_ptr(err.fnName) }.to_string_lossy(),
            action: unsafe { CStr::from_ptr(err.action) }.to_string_lossy(),
            is_recoverable: err.isRecoverable != 0,
            is_warning: err.isWarning != 0,
        }
    }

    pub(crate) fn to_warning(ctxt: &Context) -> Option<DbError> {
        let err = unsafe {
            let mut err = MaybeUninit::uninit();
            dpiContext_getError(ctxt.context, err.as_mut_ptr());
            err.assume_init()
        };
        if err.isWarning != 0 {
            Some(DbError::from_dpi_error(&err))
        } else {
            None
        }
    }

    /// Creates a new DbError. Note that its `is_recoverable` and `is_warning` values are always `false`.
    pub fn new<M, F, A>(code: i32, offset: u32, message: M, fn_name: F, action: A) -> DbError
    where
        M: Into<String>,
        F: Into<Cow<'static, str>>,
        A: Into<Cow<'static, str>>,
    {
        DbError {
            code,
            offset,
            message: message.into(),
            fn_name: fn_name.into(),
            action: action.into(),
            is_recoverable: false,
            is_warning: false,
        }
    }

    /// The OCI error code if an OCI error has taken place. If no OCI error has taken place the value is 0.
    pub fn code(&self) -> i32 {
        self.code
    }

    /// The parse error offset (in bytes) when executing a statement or the row offset when performing bulk operations or fetching batch error information. If neither of these cases are true, the value is 0.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// The error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The public ODPI-C, used by rust-oracle, function name which was called in which the error took place.
    pub fn fn_name(&self) -> &str {
        &self.fn_name
    }

    /// The internal action that was being performed when the error took place.
    pub fn action(&self) -> &str {
        &self.action
    }

    /// A boolean value indicating if the error is recoverable. This always retruns `false` unless both client and server are at release 12.1 or higher.
    pub fn is_recoverable(&self) -> bool {
        self.is_recoverable
    }

    /// A boolean value indicating if the error information is for a warning returned by Oracle that does not prevent the requested operation from proceeding. Examples include connecting to the database with a password that is about to expire (within the grace period) and creating a stored procedure with compilation errors.
    ///
    /// See also [`Connection::last_warning`].
    pub fn is_warning(&self) -> bool {
        self.is_warning
    }
}

#[cfg(feature = "struct_error")]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(not(feature = "struct_error"))]
impl fmt::Display for Error {
    #[allow(deprecated)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::OciError(err) => write!(f, "OCI Error: {}", err.message),
            Error::DpiError(err) => write!(f, "DPI Error: {}", err.message),
            Error::NullValue => write!(f, "NULL value found"),
            Error::ParseError(err) => write!(f, "{}", err),
            Error::OutOfRange(msg) => write!(f, "{}", msg),
            Error::InvalidTypeConversion(from, to) => {
                write!(f, "invalid type conversion from {} to {}", from, to)
            }
            Error::InvalidBindIndex(idx) => {
                write!(f, "invalid bind index {} (one-based)", idx)
            }
            Error::InvalidBindName(name) => write!(f, "invalid bind name {}", name),
            Error::InvalidColumnIndex(idx) => {
                write!(f, "invalid column index {} (zero-based)", idx)
            }
            Error::InvalidColumnName(name) => write!(f, "invalid column name {}", name),
            Error::InvalidAttributeName(name) => write!(f, "invalid attribute name {}", name),
            Error::InvalidOperation(msg) => write!(f, "{}", msg),
            Error::UninitializedBindValue => write!(f, "try to access uninitialized bind value"),
            Error::NoDataFound => write!(f, "no data found"),
            Error::BatchErrors(errs) => {
                write!(f, "batch errors (")?;
                for err in errs {
                    write!(f, "{}, ", err)?;
                }
                write!(f, ")")
            }
            Error::InternalError(msg) => write!(f, "{}", msg),
            Error::InvalidArgument { .. } => todo!(),
        }
    }
}

#[cfg(feature = "struct_error")]
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(err.as_ref())
        } else {
            None
        }
    }
}

#[cfg(not(feature = "struct_error"))]
impl error::Error for Error {
    #[allow(deprecated)]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::ParseError(err) => Some(err.as_ref()),
            Error::InvalidArgument {
                source: Some(source),
                ..
            } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<ParseOracleTypeError> for Error {
    fn from(err: ParseOracleTypeError) -> Self {
        Error::parse_error(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Self {
        Error::parse_error(err)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(err: num::ParseFloatError) -> Self {
        Error::parse_error(err)
    }
}

impl From<num::TryFromIntError> for Error {
    fn from(err: num::TryFromIntError) -> Self {
        Error::parse_error(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        Error::parse_error(err)
    }
}

impl<T> From<sync::PoisonError<T>> for Error {
    fn from(err: sync::PoisonError<T>) -> Self {
        Error::internal_error(err.to_string())
    }
}

fn dpi_error_in_message(message: &str) -> Option<i32> {
    let bytes = message.as_bytes();
    if !bytes.starts_with(b"DPI-") {
        return None;
    }
    let mut code = 0;
    for c in bytes.iter().skip(4) {
        if b'0' <= *c && *c <= b'9' {
            code *= 10;
            code += (*c - b'0') as i32;
        } else if *c == b':' {
            return Some(code);
        } else {
            break;
        }
    }
    None
}

#[macro_export]
#[doc(hidden)]
macro_rules! chkerr {
    ($ctxt:expr, $code:expr) => {{
        #[allow(unused_unsafe)]
        if unsafe { $code } != DPI_SUCCESS as i32 {
            return Err($crate::Error::from_context($ctxt));
        }
    }};
    ($ctxt:expr, $code:expr, $cleanup:stmt) => {{
        #[allow(unused_unsafe)]
        if unsafe { $code } != DPI_SUCCESS as i32 {
            let err = $crate::Error::from_context($ctxt);
            $cleanup
            return Err(err);
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpi_error_in_message() {
        assert_eq!(None, dpi_error_in_message("ORA-1234"));
        assert_eq!(None, dpi_error_in_message("DPI-1234"));
        assert_eq!(Some(1234), dpi_error_in_message("DPI-1234: xxx"));
    }
}
