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

//! This is an [Oracle database][] driver for [Rust][] based on [ODPI-C][].
//!
//! Don't use this until the version number reaches to 0.1.0.
//!
//! ## Build-time Requirements
//!
//! * Rust 1.19 or later
//! * C compiler. See `Compile-time Requirements` in [this document](https://github.com/alexcrichton/cc-rs#compile-time-requirements).
//!
//! ## Run-time Requirements
//!
//! * Oracle client 11.2 or later. See [ODPI-C installation document][].
//!
//! ## Usage
//!
//! Rust-oracle was published to [crates.io](https://crates.io/crates/oracle).
//! However it is old. Use rust-oracle in the github.
//!
//! ```text
//! [dependencies]
//! oracle = { git = "https://github.com/kubo/rust-oracle.git" }
//! ```
//!
//! When you need to fetch or bind [chrono](https://docs.rs/chrono/0.4/chrono/)
//! data types, enable `chrono` feature:
//!
//! ```text
//! [dependencies]
//! oracle = { git = "https://github.com/kubo/rust-oracle.git", features = ["chrono"] }
//! ```
//!
//! ## Examples
//!
//! Select a table:
//!
//! ```no_run
//! extern crate oracle;
//!
//! fn main() {
//!     // Connect to a database.
//!     let conn = oracle::Connection::new("scott", "tiger", "//localhost/XE").unwrap();
//!     // Select a table with a bind variable.
//!     let mut stmt = conn.execute("select ename, sal, comm from emp where deptno = :1", &[&30]).unwrap();
//!
//!     // Print column names
//!     for info in stmt.column_info() {
//!         print!(" {:14}|", info.name());
//!     }
//!     println!("");
//!
//!     // Print column types
//!     for info in stmt.column_info() {
//!         print!(" {:14}|", info.oracle_type().to_string());
//!     }
//!     println!("");
//!
//!     // Print column values
//!     println!("---------------|---------------|---------------|");
//!     while let Ok(row) = stmt.fetch() {
//!         // get a column value by position (0-based)
//!         let ename: String = row.get(0).unwrap();
//!         // get a column by name (case-insensitive)
//!         let sal: i32 = row.get("sal").unwrap();
//!         // get a nullable column
//!         let comm: Option<i32> = row.get(2).unwrap();
//!
//!         println!(" {:14}| {:>10}    | {:>10}    |",
//!                  ename,
//!                  sal,
//!                  comm.map_or("".to_string(), |v| v.to_string()));
//!     }
//! }
//! ```
//!
//! ## NLS_LANG parameter
//!
//! [NLS_LANG][] consists of three components: [language][], [territory][] and
//! charset. However the charset component is ignored and UTF-8(AL32UTF8) is used
//! as charset because rust characters are UTF-8.
//!
//! The territory component specifies numeric format, date format and so on.
//! However it affects only conversion in Oracle. See the following example:
//!
//! ```no_run
//! extern crate oracle;
//!
//! // The territory is France.
//! std::env::set_var("NLS_LANG", "french_france.AL32UTF8");
//! let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
//!
//! // 10.1 is converted to a string in Oracle and fetched as a string.
//! let mut stmt = conn.execute("select to_char(10.1) from dual", &[]).unwrap();
//! let row = stmt.fetch().unwrap();
//! let result: String = row.get(0).unwrap();
//! assert_eq!(result, "10,1"); // The decimal mark depends on the territory.
//!
//! // 10.1 is fetched as a number and converted to a string in rust-oracle
//! let mut stmt = conn.execute("select 10.1 from dual", &[]).unwrap();
//! let row = stmt.fetch().unwrap();
//! let result: String = row.get(0).unwrap();
//! assert_eq!(result, "10.1"); // The decimal mark is always period(.).
//! ```
//!
//! Note that NLS_LANG must be set before first rust-oracle function execution if
//! required.
//!
//! ## TODO
//!
//! * Connection pooling
//! * Read and write LOB as stream
//! * REF CURSOR, BOOLEAN
//! * Scrollable cursors
//!
//! ## License
//!
//! Rust-oracle itself is under [2-clause BSD-style license](https://opensource.org/licenses/BSD-2-Clause).
//!
//! ODPI-C bundled in rust-oracle is under the terms of:
//!
//! 1. [the Universal Permissive License v 1.0 or at your option, any later version](http://oss.oracle.com/licenses/upl); and/or
//! 2. [the Apache License v 2.0](http://www.apache.org/licenses/LICENSE-2.0). 
//!
//! [Rust]:                 https://www.rust-lang.org/
//! [ODPI-C]:               https://oracle.github.io/odpi/
//! [ODPI-C installation document]: https://oracle.github.io/odpi/doc/installation.html
//! [Oracle database]: https://www.oracle.com/database/index.html
//! [NLS_LANG]: http://www.oracle.com/technetwork/products/globalization/nls-lang-099431.html
//! [language]: http://www.oracle.com/technetwork/database/database-technologies/globalization/nls-lang-099431.html#_Toc110410559
//! [territory]: http://www.oracle.com/technetwork/database/database-technologies/globalization/nls-lang-099431.html#_Toc110410560

#[cfg(feature = "chrono")]
extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate try_from;

use std::os::raw::c_char;
use std::ptr;
use std::result;
use std::slice;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(improper_ctypes)]
mod binding;
#[macro_use]
mod error;
mod connection;
mod row;
mod statement;
mod sql_value;
mod types;
mod util;

pub use connection::AuthMode;
pub use connection::StartupMode;
pub use connection::ShutdownMode;
pub use connection::Purity;
pub use connection::Connector;
pub use connection::Connection;
pub use error::Error;
pub use error::ParseOracleTypeError;
pub use error::DbError;
pub use row::Row;
pub use row::Rows;
pub use statement::StatementType;
pub use statement::Statement;
pub use statement::ColumnInfo;
pub use statement::BindIndex;
pub use statement::ColumnIndex;
pub use statement::ColumnValues;
pub use sql_value::SqlValue;
pub use types::FromSql;
pub use types::ToSql;
pub use types::ToSqlNull;
pub use types::object::Collection;
pub use types::object::Object;
pub use types::object::ObjectType;
pub use types::object::ObjectTypeAttr;
pub use types::oracle_type::OracleType;
pub use types::timestamp::Timestamp;
pub use types::interval_ds::IntervalDS;
pub use types::interval_ym::IntervalYM;
pub use types::version::Version;

use binding::*;
use types::oracle_type::NativeType;

pub type Result<T> = result::Result<T, Error>;

/// Returns Oracle client version
///
/// # Examples
///
/// ```
/// let client_ver = oracle::client_version().unwrap();
/// println!("Oracle Client Version: {}", client_ver);
/// ```
pub fn client_version() -> Result<Version> {
    let mut dpi_ver = Default::default();
    let ctx = Context::get()?;
    chkerr!(ctx,
            dpiContext_getClientVersion(ctx.context, &mut dpi_ver));
    Ok(Version::new_from_dpi_ver(dpi_ver))
}

//
// Context
//

struct Context {
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
                let driver_name = concat!("rust-oracle : ", env!("CARGO_PKG_VERSION"));
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
            ContextResult::Err(ref err) => Err(error::error_from_dpi_error(err)),
        }
    }
}

//
// Default value definitions
//

impl Default for dpiCommonCreateParams {
    fn default() -> dpiCommonCreateParams {
        dpiCommonCreateParams {
            createMode: DPI_MODE_CREATE_DEFAULT,
            encoding: ptr::null(),
            nencoding: ptr::null(),
            edition: ptr::null(),
            editionLength: 0,
            driverName: ptr::null(),
            driverNameLength: 0,
        }
    }
}

impl Default for dpiConnCreateParams {
    fn default() -> dpiConnCreateParams {
        dpiConnCreateParams {
            authMode: DPI_MODE_AUTH_DEFAULT,
            connectionClass: ptr::null(),
            connectionClassLength: 0,
            purity: 0,
            newPassword: ptr::null(),
            newPasswordLength: 0,
            appContext: ptr::null_mut(),
            numAppContext: 0,
            externalAuth: 0,
            externalHandle: ptr::null_mut(),
            pool: ptr::null_mut(),
            tag: ptr::null(),
            tagLength: 0,
            matchAnyTag: 0,
            outTag: ptr::null(),
            outTagLength: 0,
            outTagFound: 0,
            shardingKeyColumns: ptr::null_mut(),
            numShardingKeyColumns: 0,
            superShardingKeyColumns: ptr::null_mut(),
            numSuperShardingKeyColumns: 0,
        }
    }
}

impl Default for dpiData {
    fn default() -> dpiData {
        dpiData {
            isNull: 0,
            value: dpiDataBuffer {
                asInt64: 0,
            },
        }
    }
}

impl Default for dpiPoolCreateParams {
    fn default() -> dpiPoolCreateParams {
        dpiPoolCreateParams {
            minSessions: 0,
            maxSessions: 0,
            sessionIncrement: 0,
            pingInterval: 0,
            pingTimeout: 0,
            homogeneous: 0,
            externalAuth: 0,
            getMode: 0,
            outPoolName: ptr::null(),
            outPoolNameLength: 0,
        }
    }
}

impl Default for dpiSubscrCreateParams {
    fn default() -> dpiSubscrCreateParams {
        dpiSubscrCreateParams {
            subscrNamespace: 0,
            protocol: 0,
            qos: dpiSubscrQOS(0),
            operations: dpiOpCode(0),
            portNumber: 0,
            timeout: 0,
            name: ptr::null(),
            nameLength: 0,
            callback: None,
            callbackContext: ptr::null_mut(),
            recipientName: ptr::null(),
            recipientNameLength: 0,
        }
    }
}

impl Default for dpiErrorInfo {
    fn default() -> dpiErrorInfo {
        dpiErrorInfo {
            code: 0,
            offset: 0,
            message: ptr::null(),
            messageLength: 0,
            encoding: ptr::null(),
            fnName: ptr::null(),
            action: ptr::null(),
            sqlState: ptr::null(),
            isRecoverable: 0,
        }
    }
}

impl Default for dpiDataTypeInfo {
    fn default() -> dpiDataTypeInfo {
        dpiDataTypeInfo {
            oracleTypeNum: 0,
            defaultNativeTypeNum: 0,
            ociTypeCode: 0,
            dbSizeInBytes: 0,
            clientSizeInBytes: 0,
            sizeInChars: 0,
            precision: 0,
            scale: 0,
            fsPrecision: 0,
            objectType: ptr::null_mut(),
        }
    }
}

impl Default for dpiObjectAttrInfo {
    fn default() -> dpiObjectAttrInfo {
        dpiObjectAttrInfo {
            name: ptr::null(),
            nameLength: 0,
            typeInfo: Default::default(),
        }
    }
}

impl Default for dpiObjectTypeInfo {
    fn default() -> dpiObjectTypeInfo {
        dpiObjectTypeInfo {
            schema: ptr::null(),
            schemaLength: 0,
            name: ptr::null(),
            nameLength: 0,
            isCollection: 0,
            elementTypeInfo: Default::default(),
            numAttributes: 0,
        }
    }
}

impl Default for dpiQueryInfo {
    fn default() -> dpiQueryInfo {
        dpiQueryInfo {
            name: ptr::null(),
            nameLength: 0,
            typeInfo: Default::default(),
            nullOk: 0,
        }
    }
}

impl Default for dpiVersionInfo {
    fn default() -> dpiVersionInfo {
        dpiVersionInfo {
            versionNum: 0,
            releaseNum: 0,
            updateNum: 0,
            portReleaseNum: 0,
            portUpdateNum: 0,
            fullVersionNum: 0,
        }
    }
}

impl Default for dpiStmtInfo {
    fn default() -> dpiStmtInfo {
        dpiStmtInfo {
            isQuery: 0,
            isPLSQL: 0,
            isDDL: 0,
            isDML: 0,
            statementType: 0,
            isReturning: 0,
        }
    }
}

//
// Utility struct to convert Rust strings from/to ODPI-C strings
//

struct OdpiStr {
    pub ptr: *const c_char,
    pub len: u32,
}

fn new_odpi_str() -> OdpiStr {
    OdpiStr {
        ptr: ptr::null(),
        len: 0,
    }
}

fn to_odpi_str(s: &str) -> OdpiStr {
    OdpiStr {
        ptr: s.as_ptr() as *const c_char,
        len: s.len() as u32,
    }
}

impl OdpiStr {
    pub fn to_string(&self) -> String {
        to_rust_str(self.ptr, self.len)
    }
}

fn to_rust_str(ptr: *const c_char, len: u32) -> String {
    if ptr.is_null() {
        "".to_string()
    } else {
        let s = unsafe { slice::from_raw_parts(ptr as *mut u8, len as usize) };
        String::from_utf8_lossy(s).into_owned()
    }
}

fn to_rust_slice<'a>(ptr: *const c_char, len: u32) -> &'a [u8] {
    if ptr.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(ptr as *mut u8, len as usize) }
    }
}
