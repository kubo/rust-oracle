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
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::mem::MaybeUninit;
use std::num;
use std::str;
use std::sync;

/// Enum listing possible errors from rust-oracle.
pub enum Error {
    /// Error from an underlying Oracle client library.
    OciError(DbError),

    /// Error from an underlying ODPI-C layer.
    DpiError(DbError),

    /// Error when NULL value is got but the target rust type cannot handle NULL.
    /// Use `Option<...>` in this case.
    NullValue,

    /// Error when conversion from a string to an Oracle value fails
    ParseError(Box<dyn error::Error + Send + Sync>),

    /// Error when conversion from a type to another fails due to out-of-range
    OutOfRange(String),

    /// Error when conversion from a type to another is not allowed.
    InvalidTypeConversion(String, String),

    /// Error when the bind parameter index is out of range. (one based)
    InvalidBindIndex(usize),

    /// Error when the bind parameter name is not in the SQL.
    InvalidBindName(String),

    /// Error when the column index is out of range. (zero based)
    InvalidColumnIndex(usize),

    /// Error when the column name is not in the SQL.
    InvalidColumnName(String),

    /// Error when the specified attribute name is not found.
    InvalidAttributeName(String),

    /// Error when invalid method is called such as calling execute for select statements.
    InvalidOperation(String),

    /// Error when an uninitialized bind value is accessed. Bind values
    /// must be initialized by [Statement.bind][], [Statement.execute][]
    /// or [Connection.execute][] in advance.
    ///
    /// [Statement.bind]: struct.Statement.html#method.bind
    /// [Statement.execute]: struct.Statement.html#method.execute
    /// [Connection.execute]: struct.Connection.html#method.execute
    UninitializedBindValue,

    /// Error when no more rows exist in the SQL.
    NoDataFound,

    /// Internal error. When you get this error, please report it with a test case to reproduce it.
    InternalError(String),
}

impl AssertSend for Error {}
impl AssertSync for Error {}

/// An error when parsing a string into an Oracle type fails.
/// This appears only in boxed data associated with [Error::ParseError][].
///
/// [Error::ParseError]: enum.Error.html#variant.ParseError
#[derive(Eq, PartialEq, Clone)]
pub struct ParseOracleTypeError {
    typename: &'static str,
}

impl ParseOracleTypeError {
    pub fn new(typename: &'static str) -> ParseOracleTypeError {
        ParseOracleTypeError { typename: typename }
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
    fn_name: String,
    action: String,
}

impl DbError {
    pub fn new(
        code: i32,
        offset: u32,
        message: String,
        fn_name: String,
        action: String,
    ) -> DbError {
        DbError {
            code: code,
            offset: offset,
            message: message,
            fn_name: fn_name,
            action: action,
        }
    }

    /// Oracle error code if OciError. always zero if DpiError
    pub fn code(&self) -> i32 {
        self.code
    }

    /// ? (used for Batch Errors?)
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// function name in ODPI-C used by rust-oracle
    pub fn fn_name(&self) -> &str {
        &self.fn_name
    }

    /// action name in ODPI-C used by rust-oracle
    pub fn action(&self) -> &str {
        &self.action
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::OciError(ref err) => write!(f, "OCI Error: {}", err.message),
            Error::DpiError(ref err) => write!(f, "DPI Error: {}", err.message),
            Error::NullValue => write!(f, "NULL value found"),
            Error::ParseError(ref err) => write!(f, "{}", err),
            Error::OutOfRange(ref msg) => write!(f, "out of range: {}", msg),
            Error::InvalidTypeConversion(ref from, ref to) => {
                write!(f, "invalid type conversion from {} to {}", from, to)
            }
            Error::InvalidBindIndex(ref idx) => {
                write!(f, "invalid bind index (one-based): {}", idx)
            }
            Error::InvalidBindName(ref name) => write!(f, "invalid bind name: {}", name),
            Error::InvalidColumnIndex(ref idx) => {
                write!(f, "invalid column index (zero-based): {}", idx)
            }
            Error::InvalidColumnName(ref name) => write!(f, "invalid column name: {}", name),
            Error::InvalidAttributeName(ref name) => write!(f, "invalid attribute name: {}", name),
            Error::InvalidOperation(ref msg) => write!(f, "invalid operation: {}", msg),
            Error::UninitializedBindValue => write!(f, "Try to access uninitialized bind value"),
            Error::NoDataFound => write!(f, "No data found"),
            Error::InternalError(ref msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::OciError(ref err) => write!(f, "OciError({:?})", err),
            Error::DpiError(ref err) => write!(f, "DpiError({:?})", err),
            Error::NullValue => write!(f, "NullValue"),
            Error::ParseError(ref err) => write!(f, "ParseError({:?})", err),
            Error::OutOfRange(ref msg) => write!(f, "OutOfRange({:?})", msg),
            Error::InvalidTypeConversion(ref from, ref to) => {
                write!(f, "InvalidTypeConversion(from: {:?}, to: {:?})", from, to)
            }
            Error::InvalidBindIndex(ref idx) => write!(f, "InvalidBindIndex({:?})", idx),
            Error::InvalidBindName(ref name) => write!(f, "InvalidBindName({:?})", name),
            Error::InvalidColumnIndex(ref idx) => write!(f, "InvalidColumnIndex({:?})", idx),
            Error::InvalidColumnName(ref name) => write!(f, "InvalidColumnName({:?})", name),
            Error::InvalidAttributeName(ref name) => write!(f, "InvalidAttributeName({:?})", name),
            Error::InvalidOperation(ref msg) => write!(f, "InvalidOperation({:?})", msg),
            Error::UninitializedBindValue => write!(f, "UninitializedBindValue"),
            Error::NoDataFound => write!(f, "NoDataFound"),
            Error::InternalError(ref msg) => write!(f, "InternalError({:?})", msg),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::OciError(_) => "Oracle OCI error",
            Error::DpiError(_) => "ODPI-C error",
            Error::NullValue => "NULL value",
            Error::ParseError(_) => "parse error",
            Error::OutOfRange(_) => "out of range",
            Error::InvalidTypeConversion(_, _) => "invalid type conversion",
            Error::InvalidBindIndex(_) => "invalid bind index",
            Error::InvalidBindName(_) => "invalid bind name",
            Error::InvalidColumnIndex(_) => "invalid column index",
            Error::InvalidColumnName(_) => "invalid column name",
            Error::InvalidAttributeName(_) => "invalid attribute name",
            Error::InvalidOperation(_) => "invalid operation",
            Error::UninitializedBindValue => "uninitialided bind value error",
            Error::NoDataFound => "no data found",
            Error::InternalError(_) => "internal error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::ParseError(ref err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<ParseOracleTypeError> for Error {
    fn from(err: ParseOracleTypeError) -> Self {
        Error::ParseError(Box::new(err))
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Self {
        Error::ParseError(Box::new(err))
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(err: num::ParseFloatError) -> Self {
        Error::ParseError(Box::new(err))
    }
}

impl From<num::TryFromIntError> for Error {
    fn from(err: num::TryFromIntError) -> Self {
        Error::ParseError(Box::new(err))
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        Error::ParseError(Box::new(err))
    }
}

impl<T> From<sync::PoisonError<T>> for Error {
    fn from(err: sync::PoisonError<T>) -> Self {
        Error::InternalError(err.to_string())
    }
}

//
// functions to check errors
//

pub fn error_from_dpi_error(err: &dpiErrorInfo) -> Error {
    let err = DbError::new(
        err.code,
        err.offset,
        to_rust_str(err.message, err.messageLength),
        unsafe { CStr::from_ptr(err.fnName) }
            .to_string_lossy()
            .into_owned(),
        unsafe { CStr::from_ptr(err.action) }
            .to_string_lossy()
            .into_owned(),
    );
    if err.message().starts_with("DPI") {
        Error::DpiError(err)
    } else {
        Error::OciError(err)
    }
}

pub(crate) fn error_from_context(ctxt: &Context) -> Error {
    let err = unsafe {
        let mut err = MaybeUninit::uninit();
        dpiContext_getError(ctxt.context, err.as_mut_ptr());
        err.assume_init()
    };
    crate::error::error_from_dpi_error(&err)
}

#[macro_export]
#[doc(hidden)]
macro_rules! chkerr {
    ($ctxt:expr, $code:expr) => {{
        if unsafe { $code } == DPI_SUCCESS as i32 {
            ()
        } else {
            return Err($crate::error::error_from_context($ctxt));
        }
    }};
    ($ctxt:expr, $code:expr, $cleanup:stmt) => {{
        if unsafe { $code } == DPI_SUCCESS as i32 {
            ()
        } else {
            let err = $crate::error::error_from_context($ctxt);
            $cleanup
            return Err(err);
        }
    }};
}
