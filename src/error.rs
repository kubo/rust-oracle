use std::ffi::CStr;
use std::error;
use std::fmt;
use std::slice;
use binding::dpiErrorInfo;
use binding::dpiContext_getError;
use Context;

pub enum Error {
    OciError(DbError),
    DpiError(DbError),
    IndexError(IndexError),
    ConversionError(ConversionError),
    UninitializedBindValue,
    NoMoreData,
    InternalError(String),
}

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
    pub fn message(&self) -> &String {
        &self.message
    }

    /// function name in ODPI-C used by rust-oracle
    pub fn fn_name(&self) -> &String {
        &self.fn_name
    }

    /// action name in ODPI-C used by rust-oracle
    pub fn action(&self) -> &String {
        &self.action
    }
}

pub enum IndexError {
    BindIndex(usize),
    BindName(String),
    ColumnIndex(usize),
    ColumnName(String),
}

pub enum ConversionError {
    NullValue,
    ParseError(Box<error::Error>),
    Overflow(String, &'static str),
    UnsupportedType(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::OciError(ref err) =>
                write!(f, "OCI Error: {}", err.message),
            Error::DpiError(ref err) =>
                write!(f, "DPI Error: {}", err.message),
            Error::IndexError(ref err) => {
                match *err {
                    IndexError::BindIndex(ref idx) =>
                        write!(f, "invalid bind index (one-based): {}", idx),
                    IndexError::BindName(ref name) =>
                        write!(f, "invalid bind name: {}", name),
                    IndexError::ColumnIndex(ref idx) =>
                        write!(f, "invalid column index (zero-based): {}", idx),
                    IndexError::ColumnName(ref name) =>
                        write!(f, "invalid column name: {}", name),
                }
            },
            Error::ConversionError(ref err) => {
                match *err {
                    ConversionError::NullValue =>
                        write!(f, "NULL value found"),
                    ConversionError::ParseError(ref err) =>
                        write!(f, "{}", err),
                    ConversionError::Overflow(ref src, dst) =>
                        write!(f, "number too large to convert {} to {}", src, dst),
                    ConversionError::UnsupportedType(ref from, ref to) =>
                        write!(f, "unsupported type conversion from {} to {}", from, to),
                }
            },
            Error::UninitializedBindValue =>
                write!(f, "Try to access uninitialized bind value"),
            Error::NoMoreData =>
                write!(f, "No more data to be fetched"),
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
            Error::IndexError(ref err) => {
                match *err {
                    IndexError::BindIndex(ref idx) =>
                        write!(f, "IndexError {{ bind index: {} }}", idx),
                    IndexError::BindName(ref name) =>
                        write!(f, "IndexError {{ bind name: {} }}", name),
                    IndexError::ColumnIndex(ref idx) =>
                        write!(f, "IndexError {{ column index: {} }}", idx),
                    IndexError::ColumnName(ref name) =>
                        write!(f, "IndexError {{ column name: {} }}", name),
                }
            },
            Error::ConversionError(ref err) => {
                match *err {
                    ConversionError::NullValue =>
                        write!(f, "ConversionError {{ NULL value }}"),
                    ConversionError::ParseError(ref err) =>
                        write!(f, "ConversionError {{ ParseError: {:?} }}", err),
                    ConversionError::Overflow(ref src, dst) =>
                        write!(f, "ConversionError {{ Overflow {{ src: {}, dest: {} }} }}", src, dst),
                    ConversionError::UnsupportedType(ref from, ref to) =>
                        write!(f, "ConversionError {{ UnsupportedType {{ from: {}, to: {} }} }}", from, to),
                }
            },
            Error::UninitializedBindValue |
            Error::NoMoreData |
            Error::InternalError(_) =>
                write!(f, "{}", *self),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::OciError(_) => "Oracle OCI error",
            Error::DpiError(_) => "Oracle DPI Error",
            Error::IndexError(_) => "invalid index",
            Error::ConversionError(_) => "conversion error",
            Error::UninitializedBindValue => "Uninitialided bind value error",
            Error::NoMoreData => "No more data",
            Error::InternalError(_) => "Internal error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ConversionError(ConversionError::ParseError(ref err)) =>
                Some(err.as_ref()),
            _ => None,
        }
    }
}

//
// functions to check errors
//

pub fn error_from_dpi_error(err: &dpiErrorInfo) -> Error {
    let err = DbError::new(err.code, err.offset,
                           String::from_utf8_lossy(unsafe {
                               slice::from_raw_parts(err.message as *mut u8, err.messageLength as usize)
                           }).into_owned(),
                           unsafe { CStr::from_ptr(err.fnName) }.to_string_lossy().into_owned(),
                           unsafe { CStr::from_ptr(err.action) }.to_string_lossy().into_owned());
    if err.message().starts_with("DPI") {
        Error::DpiError(err)
    } else {
        Error::OciError(err)
    }
}

pub fn error_from_context(ctxt: &Context) -> Error {
    let mut err: dpiErrorInfo = Default::default();
    unsafe {
        dpiContext_getError(ctxt.context, &mut err);
    };
    error_from_dpi_error(&err)
}

macro_rules! chkerr {
    ($ctxt:expr, $code:expr) => {{
        if unsafe { $code } == DPI_SUCCESS as i32 {
            ()
        } else {
            return Err(error_from_context($ctxt));
        }
    }};
    ($ctxt:expr, $code:expr, $cleanup:stmt) => {{
        if unsafe { $code } == DPI_SUCCESS as i32 {
            ()
        } else {
            let err = error_from_context($ctxt);
            $cleanup
            return Err(err);
        }
    }};
}
