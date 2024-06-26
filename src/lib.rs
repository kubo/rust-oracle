// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2023 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

#![doc = include_str!("../README.md")]

use std::os::raw::c_char;
use std::ptr;
use std::result;
use std::slice;

#[cfg(feature = "aq_unstable")]
pub mod aq;
mod batch;
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(improper_ctypes)]
mod binding;
pub mod conn;
mod connection;
mod context;
mod error;
pub mod io;
pub mod oci_attr;
pub mod pool;
#[cfg(doctest)]
mod procmacro;
mod row;
pub mod sql_type;
mod sql_value;
mod statement;
mod util;
mod version;

pub use crate::batch::Batch;
pub use crate::batch::BatchBindIndex;
pub use crate::batch::BatchBuilder;
pub use crate::connection::ConnStatus;
pub use crate::connection::Connection;
pub use crate::connection::Connector;
pub use crate::connection::Privilege;
pub use crate::connection::ShutdownMode;
pub use crate::connection::StartupMode;
use crate::context::Context;
pub use crate::context::InitParams;
pub use crate::error::DbError;
pub use crate::error::Error;
pub use crate::error::ErrorKind;
pub use crate::error::ParseOracleTypeError;
pub use crate::row::ResultSet;
pub use crate::row::Row;
pub use crate::row::RowValue;
pub use crate::sql_value::SqlValue;
pub use crate::statement::BindIndex;
pub use crate::statement::ColumnIndex;
pub use crate::statement::ColumnInfo;
pub use crate::statement::Statement;
pub use crate::statement::StatementBuilder;
pub use crate::statement::StatementType;
pub use crate::version::Version;
pub use oracle_procmacro::RowValue;

use crate::binding::*;

pub type Result<T> = result::Result<T, Error>;

macro_rules! define_dpi_data_with_refcount {
    ($name:ident) => {
        define_dpi_data_with_refcount!(__define_struct__, $name);
        paste::item! {
            unsafe impl Send for [<Dpi $name>] {}
            unsafe impl Sync for [<Dpi $name>] {}
        }
    };

    ($name:ident, nosync) => {
        define_dpi_data_with_refcount!(__define_struct__, $name);
        paste::item! {
            unsafe impl Send for [<Dpi $name>] {}
        }
    };

    (__define_struct__, $name:ident) => {
        paste::item! {
            #[derive(Debug)]
            struct [<Dpi $name>] {
                raw: *mut [<dpi $name>],
            }

            impl [<Dpi $name>] {
                fn new(raw: *mut [<dpi $name>]) -> [<Dpi $name>] {
                    [<Dpi $name>] { raw }
                }

                #[allow(dead_code)]
                fn with_add_ref(raw: *mut [<dpi $name>]) -> [<Dpi $name>] {
                    unsafe { [<dpi $name _addRef>](raw) };
                    [<Dpi $name>] { raw }
                }

                #[allow(dead_code)]
                fn null() -> [<Dpi $name>] {
                    [<Dpi $name>] {
                        raw: ptr::null_mut(),
                    }
                }

                #[allow(dead_code)]
                fn is_null(&self) -> bool {
                    self.raw.is_null()
                }

                pub(crate) fn raw(&self) -> *mut [<dpi $name>] {
                    self.raw
                }
            }

            impl Clone for [<Dpi $name>] {
                fn clone(&self) -> [<Dpi $name>] {
                    if !self.is_null() {
                        unsafe { [<dpi $name _addRef>](self.raw()) };
                    }
                    [<Dpi $name>]::new(self.raw())
                }
            }

            impl Drop for [<Dpi $name>] {
                fn drop(&mut self) {
                   if !self.is_null() {
                       unsafe { [<dpi $name _release>](self.raw()) };
                   }
                }
            }
        }
    };
}

// define DpiConn wrapping *mut dpiConn.
define_dpi_data_with_refcount!(Conn);

// define DpiMsgProps wrapping *mut dpiMsgProps.
define_dpi_data_with_refcount!(MsgProps);

// define DpiObjectType wrapping *mut dpiObjectType.
define_dpi_data_with_refcount!(ObjectType);

// define DpiPool wrapping *mut dpiPool.
define_dpi_data_with_refcount!(Pool);

// define DpiObjectAttr wrapping *mut dpiObjectAttr.
define_dpi_data_with_refcount!(ObjectAttr);

// define DpiQueue wrapping *mut dpiQueue.
define_dpi_data_with_refcount!(Queue);

// define DpiObject wrapping *mut dpiObject.
define_dpi_data_with_refcount!(Object, nosync);

// define DpiStmt wrapping *mut dpiStmt.
define_dpi_data_with_refcount!(Stmt, nosync);

// define DpiVar wrapping *mut dpiVar.
struct DpiVar {
    raw: *mut dpiVar,
    data: *mut dpiData,
}

impl DpiVar {
    fn new(raw: *mut dpiVar, data: *mut dpiData) -> DpiVar {
        DpiVar { raw, data }
    }

    fn with_add_ref(raw: *mut dpiVar, data: *mut dpiData) -> DpiVar {
        unsafe { dpiVar_addRef(raw) };
        DpiVar::new(raw, data)
    }

    fn is_null(&self) -> bool {
        self.raw.is_null()
    }
}

impl Drop for DpiVar {
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe { dpiVar_release(self.raw) };
        }
    }
}

unsafe impl Send for DpiVar {}

#[allow(dead_code)]
trait AssertSend: Send {}
#[allow(dead_code)]
trait AssertSync: Sync {}

//
// Utility struct to convert Rust strings from/to ODPI-C strings
//

struct OdpiStr {
    pub ptr: *const c_char,
    pub len: u32,
}

impl OdpiStr {
    fn new<T>(s: T) -> OdpiStr
    where
        T: AsRef<[u8]>,
    {
        let s = s.as_ref();
        if s.is_empty() {
            OdpiStr {
                ptr: ptr::null(),
                len: 0,
            }
        } else {
            OdpiStr {
                ptr: s.as_ptr() as *const c_char,
                len: s.len() as u32,
            }
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        to_rust_str(self.ptr, self.len)
    }

    #[cfg(feature = "aq_unstable")]
    pub fn to_vec(&self) -> Vec<u8> {
        if self.ptr.is_null() {
            Vec::new()
        } else {
            let ptr = self.ptr as *mut u8;
            let len = self.len as usize;
            unsafe { Vec::from_raw_parts(ptr, len, len) }
        }
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

mod private {
    use std::os::raw::c_void;

    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}
    impl Sealed for bool {}
    impl Sealed for str {}
    impl Sealed for [u8] {}
    impl Sealed for *mut c_void {}
    impl Sealed for &str {}
}

#[allow(dead_code)]
#[doc(hidden)]
// #[cfg(doctest)] isn't usable here. See: https://github.com/rust-lang/rust/issues/67295
pub mod test_util {
    use super::*;
    use std::env;

    pub const VER11_2: Version = Version::new(11, 2, 0, 0, 0);
    pub const VER12_1: Version = Version::new(12, 1, 0, 0, 0);
    pub const VER18: Version = Version::new(18, 0, 0, 0, 0);

    fn env_var_or(env_name: &str, default: &str) -> String {
        match env::var_os(env_name) {
            Some(env_var) => env_var.into_string().unwrap(),
            None => String::from(default),
        }
    }

    pub fn main_user() -> String {
        env_var_or("ODPIC_TEST_MAIN_USER", "odpic")
    }

    pub fn main_password() -> String {
        env_var_or("ODPIC_TEST_MAIN_PASSWORD", "welcome")
    }

    pub fn edition_user() -> String {
        env_var_or("ODPIC_TEST_EDITION_USER", "odpic_edition")
    }

    pub fn edition_password() -> String {
        env_var_or("ODPIC_TEST_EDITION_PASSWORD", "welcome")
    }

    pub fn connect_string() -> String {
        env_var_or("ODPIC_TEST_CONNECT_STRING", "localhost/orclpdb")
    }

    pub fn connect() -> Result<Connection> {
        Connection::connect(main_user(), main_password(), connect_string())
    }

    pub fn check_version(
        conn: &Connection,
        client_ver: &Version,
        server_ver: &Version,
    ) -> Result<bool> {
        Ok(&Version::client()? >= client_ver && &conn.server_version()?.0 >= server_ver)
    }
}
