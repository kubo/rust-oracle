// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017-2018 Kubo Takehiro <kubo@jiubao.org>
//
// Redistribution and use in source and binary forms, with or without modification, are
// permitted provided that the following conditions are met:
//
//    1. Redistributions of source code must retain the above copyright notice, this list of
//       conditions and the following disclaimer.
//
//    2. Redistributions in binary form must reproduce the above copyright notice, this list
//       of conditions and the following disclaimer in the documentation and/or other materials
//       provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHORS ''AS IS'' AND ANY EXPRESS OR IMPLIED
// WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
// ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
// ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// The views and conclusions contained in the software and documentation are those of the
// authors and should not be interpreted as representing official policies, either expressed
// or implied, of the authors.

use std::ffi::CStr;
use std::error;
use std::fmt;
use std::num;
use std::str;
use try_from;
use binding::dpiErrorInfo;
use binding::dpiContext_getError;
use Context;

use to_rust_str;

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
    ParseError(Box<error::Error>),

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
        ParseOracleTypeError {
            typename: typename,
        }
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

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// Oracle database error or ODPI-C error
#[derive(Eq, PartialEq, Clone)]
pub struct DbError {
    code: i32,
    offset: u16,
    message: String,
    fn_name: String,
    action: String,
}

impl DbError {
    pub fn new(code: i32, offset: u16, message: String, fn_name: String, action: String) -> DbError {
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
    pub fn offset(&self) -> u16 {
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
            Error::OciError(ref err) =>
                write!(f, "OCI Error: {}", err.message),
            Error::DpiError(ref err) =>
                write!(f, "DPI Error: {}", err.message),
            Error::NullValue =>
                write!(f, "NULL value found"),
            Error::ParseError(ref err) =>
                write!(f, "{}", err),
            Error::OutOfRange(ref msg) =>
                write!(f, "out of range: {}", msg),
            Error::InvalidTypeConversion(ref from, ref to) =>
                write!(f, "invalid type conversion from {} to {}", from, to),
            Error::InvalidBindIndex(ref idx) =>
                write!(f, "invalid bind index (one-based): {}", idx),
            Error::InvalidBindName(ref name) =>
                write!(f, "invalid bind name: {}", name),
            Error::InvalidColumnIndex(ref idx) =>
                write!(f, "invalid column index (zero-based): {}", idx),
            Error::InvalidColumnName(ref name) =>
                write!(f, "invalid column name: {}", name),
            Error::InvalidAttributeName(ref name) =>
                write!(f, "invalid attribute name: {}", name),
            Error::InvalidOperation(ref msg) =>
                write!(f, "invalid operation: {}", msg),
            Error::UninitializedBindValue =>
                write!(f, "Try to access uninitialized bind value"),
            Error::NoDataFound =>
                write!(f, "No data found"),
            Error::InternalError(ref msg) =>
                write!(f, "Internal Error: {}", msg),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::OciError(ref err) =>
                write!(f, "OCI Error: (code: {}, offset: {}, message:{}, fn_name: {}, action: {})",
                       err.code, err.offset, err.message, err.fn_name, err.action),
            Error::DpiError(ref err) =>
                write!(f, "OCI Error: (code: {}, offset: {}, message:{}, fn_name: {}, action: {})",
                       err.code, err.offset, err.message, err.fn_name, err.action),
            Error::NullValue =>
                write!(f, "NULLValue"),
            Error::ParseError(ref err) =>
                write!(f, "ParseError: {:?}", err),
            Error::OutOfRange(ref msg) =>
                write!(f, "OutOfRange {{ msg: {} }}", msg),
            Error::InvalidTypeConversion(ref from, ref to) =>
                write!(f, "InvalidTypeConversion {{ from: {}, to: {} }}", from, to),
            Error::InvalidBindIndex(ref idx) =>
                write!(f, "InvalidBindIndex: {}", idx),
            Error::InvalidBindName(ref name) =>
                write!(f, "InvalidBindName: {}", name),
            Error::InvalidColumnIndex(ref idx) =>
                write!(f, "InvalidColumnIndex: {}", idx),
            Error::InvalidColumnName(ref name) =>
                write!(f, "InvalidColumnName: {}", name),
            Error::InvalidAttributeName(ref name) =>
                write!(f, "InvalidAttributeName: {}", name),
            Error::InvalidOperation(ref msg) =>
                write!(f, "InvalidOperation: {}", msg),
            Error::UninitializedBindValue =>
                write!(f, "UninitializedBindValue"),
            Error::NoDataFound =>
                write!(f, "NoDataFound"),
            Error::InternalError(_) =>
                write!(f, "{}", *self),
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

    fn cause(&self) -> Option<&error::Error> {
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

impl From<try_from::TryFromIntError> for Error {
    fn from(err: try_from::TryFromIntError) -> Self {
        Error::ParseError(Box::new(err))
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        Error::ParseError(Box::new(err))
    }
}

//
// functions to check errors
//

pub fn error_from_dpi_error(err: &dpiErrorInfo) -> Error {
    let err = DbError::new(err.code, err.offset,
                           to_rust_str(err.message, err.messageLength),
                           unsafe { CStr::from_ptr(err.fnName) }.to_string_lossy().into_owned(),
                           unsafe { CStr::from_ptr(err.action) }.to_string_lossy().into_owned());
    if err.message().starts_with("DPI") {
        Error::DpiError(err)
    } else {
        Error::OciError(err)
    }
}

pub(crate) fn error_from_context(ctxt: &Context) -> Error {
    let mut err: dpiErrorInfo = Default::default();
    unsafe {
        dpiContext_getError(ctxt.context, &mut err);
    };
    ::error::error_from_dpi_error(&err)
}

macro_rules! chkerr {
    ($ctxt:expr, $code:expr) => {{
        if unsafe { $code } == DPI_SUCCESS as i32 {
            ()
        } else {
            return Err(::error::error_from_context($ctxt));
        }
    }};
    ($ctxt:expr, $code:expr, $cleanup:stmt) => {{
        if unsafe { $code } == DPI_SUCCESS as i32 {
            ()
        } else {
            let err = ::error::error_from_context($ctxt);
            $cleanup
            return Err(err);
        }
    }};
}
