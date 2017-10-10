extern crate core;
extern crate libc;
#[macro_use]
extern crate lazy_static;

use std::ptr;
use std::result;
use std::os::raw::c_char;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(improper_ctypes)]
mod binding;
#[macro_use]
mod error;
mod connection;
mod statement;
mod odpi;
mod types;
mod value_ref;

pub use binding::dpiAuthMode as AuthMode;
pub use binding::dpiStatementType as StatementType;
pub use binding::dpiShutdownMode as ShutdownMode;
pub use binding::dpiStartupMode as StartupMode;
pub use connection::Connector;
pub use connection::Connection;
pub use statement::Statement;
pub use statement::ColumnInfo;
pub use statement::Row;
pub use statement::RowIndex;
pub use error::Error;
pub use error::DbError;
pub use odpi::OracleType;
pub use odpi::Timestamp;
pub use odpi::IntervalDS;
pub use odpi::IntervalYM;
pub use odpi::Version;
pub use types::FromSql;

use binding::*;
use error::error_from_context;
use error::error_from_dpi_error;

pub type Result<T> = result::Result<T, Error>;

pub fn client_version() -> Result<Version> {
    let mut dpi_ver = Default::default();
    let ctx = Context::get()?;
    chkerr!(ctx,
            dpiContext_getClientVersion(ctx.context, &mut dpi_ver));
    Ok(Version::new_from_dpi_ver(dpi_ver))
}

pub const AUTH_DEFAULT: dpiAuthMode = DPI_MODE_AUTH_DEFAULT;
pub const AUTH_SYSDBA: dpiAuthMode = DPI_MODE_AUTH_SYSDBA;
pub const AUTH_SYSOPER: dpiAuthMode = DPI_MODE_AUTH_SYSOPER;
pub const AUTH_PRELIM: dpiAuthMode = DPI_MODE_AUTH_PRELIM;
pub const AUTH_SYSASM: dpiAuthMode = DPI_MODE_AUTH_SYSASM;

//
// Context
//

pub struct Context {
    pub context: *mut dpiContext,
    pub common_create_params: dpiCommonCreateParams,
    pub conn_create_params: dpiConnCreateParams,
    pub pool_create_params: dpiPoolCreateParams,
    pub subscr_create_params: dpiSubscrCreateParams,
}

enum ContextResult {
    Ok(Context),
    Err(dpiErrorInfo),
}

unsafe impl Sync for ContextResult {}

lazy_static! {
    static ref DPI_CONTEXT: ContextResult = {
        let mut ctxt = Context {
            context: ptr::null_mut(),
            common_create_params: Default::default(),
            conn_create_params: Default::default(),
            pool_create_params: Default::default(),
            subscr_create_params: Default::default(),
        };
        let mut err: dpiErrorInfo = Default::default();
        if unsafe {
            dpiContext_create(DPI_MAJOR_VERSION, DPI_MINOR_VERSION, &mut ctxt.context, &mut err)
        } == DPI_SUCCESS as i32 {
            unsafe {
                let utf8_ptr = "UTF-8\0".as_ptr() as *const c_char;
                let driver_name = concat!("Rust Oracle : ", env!("CARGO_PKG_VERSION"));
                let driver_name_ptr = driver_name.as_ptr() as *const c_char;
                let driver_name_len = driver_name.len() as u32;
                dpiContext_initCommonCreateParams(ctxt.context, &mut ctxt.common_create_params);
                dpiContext_initConnCreateParams(ctxt.context, &mut ctxt.conn_create_params);
                dpiContext_initPoolCreateParams(ctxt.context, &mut ctxt.pool_create_params);
                dpiContext_initSubscrCreateParams(ctxt.context, &mut ctxt.subscr_create_params);
                ctxt.common_create_params.createMode |= DPI_MODE_CREATE_THREADED;
                ctxt.common_create_params.encoding = utf8_ptr;
                ctxt.common_create_params.nencoding = utf8_ptr;
                ctxt.common_create_params.driverName = driver_name_ptr;
                ctxt.common_create_params.driverNameLength = driver_name_len;
            }
            ContextResult::Ok(ctxt)
        } else {
            ContextResult::Err(err)
        }
    };
}

impl Context {
    pub fn get() -> Result<&'static Context> {
        match *DPI_CONTEXT {
            ContextResult::Ok(ref ctxt) => Ok(ctxt),
            ContextResult::Err(ref err) => Err(error_from_dpi_error(err)),
        }
    }
}
