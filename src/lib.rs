// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2019 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

/*!
This is an [Oracle database][] driver for [Rust][] based on [ODPI-C][].

## Change Log

See [ChangeLog.md](https://github.com/kubo/rust-oracle/blob/master/ChangeLog.md).

## Build-time Requirements

* C compiler. See `Compile-time Requirements` in [this document](https://github.com/alexcrichton/cc-rs#compile-time-requirements).

## Run-time Requirements

* Oracle client 11.2 or later. See [ODPI-C installation document][].

## Supported Rust Versions

The oracle crate supports **at least** 6 rust minor versions including the stable
release at the time when the crate was released. The MSRV (minimum supported
rust version) may be changed when a patch version is incremented though it will
not be changed frequently. The current MSRV is 1.54.0.

## Usage

Put this in your `Cargo.toml`:

```text
[dependencies]
oracle = "0.5"
```

When you need to fetch or bind [chrono](https://docs.rs/chrono/0.4/chrono/)
data types, enable `chrono` feature:

```text
[dependencies]
oracle = { version = "0.5", features = ["chrono"] }
```

## Examples

Executes select statements and get rows:

```no_run
# use oracle::*;

// Connect to a database.
let conn = Connection::connect("scott", "tiger", "//localhost/XE")?;

let sql = "select ename, sal, comm from emp where deptno = :1";

// Select a table with a bind variable.
println!("---------------|---------------|---------------|");
let rows = conn.query(sql, &[&30])?;
for row_result in rows {
    let row = row_result?;
    // get a column value by position (0-based)
    let ename: String = row.get(0)?;
    // get a column by name (case-insensitive)
    let sal: i32 = row.get("sal")?;
    // Use `Option<...>` to get a nullable column.
    // Otherwise, `Err(Error::NullValue)` is returned
    // for null values.
    let comm: Option<i32> = row.get(2)?;

    println!(" {:14}| {:>10}    | {:>10}    |",
             ename,
             sal,
             comm.map_or("".to_string(), |v| v.to_string()));
}

// Another way to fetch rows.
// The rows iterator returns Result<(String, i32, Option<i32>)>.
println!("---------------|---------------|---------------|");
let rows = conn.query_as::<(String, i32, Option<i32>)>(sql, &[&10])?;
for row_result in rows {
    let (ename, sal, comm) = row_result?;
    println!(" {:14}| {:>10}    | {:>10}    |",
             ename,
             sal,
             comm.map_or("".to_string(), |v| v.to_string()));
}
# Ok::<(), Error>(())
```

Executes select statements and get the first rows:

```no_run
# use oracle::*;
use oracle::Connection;

// Connect to a database.
let conn = Connection::connect("scott", "tiger", "//localhost/XE")?;

let sql = "select ename, sal, comm from emp where empno = :1";

// Print the first row.
let row = conn.query_row(sql, &[&7369])?;
let ename: String = row.get("empno")?;
let sal: i32 = row.get("sal")?;
let comm: Option<i32> = row.get("comm")?;
println!("---------------|---------------|---------------|");
println!(" {:14}| {:>10}    | {:>10}    |",
         ename,
         sal,
         comm.map_or("".to_string(), |v| v.to_string()));
// When no rows are found, conn.query_row() returns `Err(Error::NoDataFound)`.

// Get the first row as a tupple
let row = conn.query_row_as::<(String, i32, Option<i32>)>(sql, &[&7566])?;
println!("---------------|---------------|---------------|");
println!(" {:14}| {:>10}    | {:>10}    |",
         row.0,
         row.1,
         row.2.map_or("".to_string(), |v| v.to_string()));
# Ok::<(), Error>(())
```

Executes non-select statements:

```no_run
# use oracle::*;
use oracle::Connection;

// Connect to a database.
let conn = Connection::connect("scott", "tiger", "//localhost/XE")?;

conn.execute("create table person (id number(38), name varchar2(40))", &[])?;

// Execute a statement with positional parameters.
conn.execute("insert into person values (:1, :2)",
             &[&1, // first parameter
               &"John" // second parameter
              ])?;

// Execute a statement with named parameters.
conn.execute_named("insert into person values (:id, :name)",
                   &[("id", &2), // 'id' parameter
                     ("name", &"Smith"), // 'name' parameter
                    ])?;

// Commit the transaction.
conn.commit()?;

// Delete rows
conn.execute("delete from person", &[])?;

// Rollback the transaction.
conn.rollback()?;
# Ok::<(), Error>(())
```

Prints column information:

```no_run
# use oracle::*;
use oracle::Connection;

// Connect to a database.
let conn = Connection::connect("scott", "tiger", "//localhost/XE")?;

let sql = "select ename, sal, comm from emp where 1 = 2";
let rows = conn.query(sql, &[])?;

// Print column names
for info in rows.column_info() {
    print!(" {:14}|", info.name());
}
println!("");

// Print column types
for info in rows.column_info() {
    print!(" {:14}|", info.oracle_type().to_string());
}
println!("");
# Ok::<(), Error>(())
```

Prepared statement:

```no_run
# use oracle::*;
use oracle::Connection;

let conn = Connection::connect("scott", "tiger", "//localhost/XE")?;

// Create a prepared statement
let mut stmt = conn.statement("insert into person values (:1, :2)").build()?;
// Insert one row
stmt.execute(&[&1, &"John"])?;
// Insert another row
stmt.execute(&[&2, &"Smith"])?;
# Ok::<(), Error>(())
```

This is more efficient than two `conn.execute()`.
An SQL statement is executed in the DBMS as follows:

* step 1. Parse the SQL statement and create an execution plan.
* step 2. Execute the plan with bind parameters.

When a prepared statement is used, step 1 is called only once.

## NLS_LANG parameter

[NLS_LANG][] consists of three components: language, territory and
charset. However the charset component is ignored and UTF-8(AL32UTF8) is used
as charset because rust characters are UTF-8.

The territory component specifies numeric format, date format and so on.
However it affects only conversion in Oracle. See the following example:

```no_run
# use oracle::*;
use oracle::Connection;

// The territory is France.
std::env::set_var("NLS_LANG", "french_france.AL32UTF8");
let conn = Connection::connect("scott", "tiger", "")?;

// 10.1 is converted to a string in Oracle and fetched as a string.
let result = conn.query_row_as::<String>("select to_char(10.1) from dual", &[])?;
assert_eq!(result, "10,1"); // The decimal mark depends on the territory.

// 10.1 is fetched as a number and converted to a string in rust-oracle
let result = conn.query_row_as::<String>("select 10.1 from dual", &[])?;
assert_eq!(result, "10.1"); // The decimal mark is always period(.).
# Ok::<(), Error>(())
```

Note that NLS_LANG must be set before first rust-oracle function execution if
required.

## TODO

* [BFILEs (External LOBs)](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-5834BC49-4053-40FF-BE39-B14342B1201E) (Note: Reading contents of BFILEs as `Vec<u8>` is supported.)
* Scrollable cursors
* Better Oracle object type support
* XML data type
* [JSON data type](https://oracle-base.com/articles/21c/json-data-type-21c)

## License

Rust-oracle and ODPI-C bundled in rust-oracle are under the terms of:

1. [the Universal Permissive License v 1.0 or at your option, any later version](http://oss.oracle.com/licenses/upl); and/or
2. [the Apache License v 2.0](http://www.apache.org/licenses/LICENSE-2.0).

[Rust]:                 https://www.rust-lang.org/
[ODPI-C]:               https://oracle.github.io/odpi/
[ODPI-C installation document]: https://oracle.github.io/odpi/doc/installation.html
[Oracle database]: https://www.oracle.com/database/index.html
[NLS_LANG]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-86A29834-AE29-4BA5-8A78-E19C168B690A
*/

use lazy_static::lazy_static;
use std::mem::MaybeUninit;
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
#[allow(unknown_lints)] // See: https://github.com/rust-lang/rust-bindgen/issues/1651#issuecomment-848479168
#[allow(deref_nullptr)] // ditto
mod binding;
mod binding_impl;
pub mod conn;
mod connection;
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
pub use crate::error::DbError;
pub use crate::error::Error;
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
pub use crate::statement::StmtParam;
pub use crate::version::Version;
pub use oracle_procmacro::RowValue;

use crate::binding::*;

pub type Result<T> = result::Result<T, Error>;

macro_rules! define_dpi_data_with_refcount {
    ($name:ident) => {
        paste::item! {
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

                pub(crate) fn raw(&self) -> *mut [<dpi $name>] {
                    self.raw
                }
            }

            impl Clone for [<Dpi $name>] {
                fn clone(&self) -> [<Dpi $name>] {
                    unsafe { [<dpi $name _addRef>](self.raw()) };
                    [<Dpi $name>]::new(self.raw())
                }
            }

            impl Drop for [<Dpi $name>] {
                fn drop(&mut self) {
                    unsafe { [<dpi $name _release>](self.raw()) };
                }
            }

            unsafe impl Send for [<Dpi $name>] {}
            unsafe impl Sync for [<Dpi $name>] {}
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

//
// Context
//

struct Context {
    pub context: *mut dpiContext,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

enum ContextResult {
    Ok(Context),
    Err(dpiErrorInfo),
}

unsafe impl Sync for ContextResult {}
unsafe impl Send for ContextResult {}

trait AssertSend: Send {}
trait AssertSync: Sync {}

lazy_static! {
    static ref DPI_CONTEXT: ContextResult = {
        let mut ctxt = ptr::null_mut();
        let mut err = MaybeUninit::uninit();
        if unsafe {
            dpiContext_createWithParams(
                DPI_MAJOR_VERSION,
                DPI_MINOR_VERSION,
                ptr::null_mut(),
                &mut ctxt,
                err.as_mut_ptr(),
            )
        } == DPI_SUCCESS as i32
        {
            ContextResult::Ok(Context { context: ctxt })
        } else {
            ContextResult::Err(unsafe { err.assume_init() })
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

    pub fn common_create_params(&self) -> dpiCommonCreateParams {
        let mut params = MaybeUninit::uninit();
        unsafe {
            dpiContext_initCommonCreateParams(self.context, params.as_mut_ptr());
            let mut params = params.assume_init();
            let driver_name: &'static str = concat!("rust-oracle : ", env!("CARGO_PKG_VERSION"));
            params.createMode |= DPI_MODE_CREATE_THREADED;
            params.driverName = driver_name.as_ptr() as *const c_char;
            params.driverNameLength = driver_name.len() as u32;
            params
        }
    }

    pub fn conn_create_params(&self) -> dpiConnCreateParams {
        let mut params = MaybeUninit::uninit();
        unsafe {
            dpiContext_initConnCreateParams(self.context, params.as_mut_ptr());
            params.assume_init()
        }
    }

    pub fn pool_create_params(&self) -> dpiPoolCreateParams {
        let mut params = MaybeUninit::uninit();
        unsafe {
            dpiContext_initPoolCreateParams(self.context, params.as_mut_ptr());
            params.assume_init()
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

impl OdpiStr {
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
    impl<'a> Sealed for &'a str {}
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
        Connection::connect(&main_user(), &main_password(), &connect_string())
    }

    pub fn check_version(
        conn: &Connection,
        client_ver: &Version,
        server_ver: &Version,
    ) -> Result<bool> {
        Ok(&Version::client()? >= client_ver && &conn.server_version()?.0 >= server_ver)
    }
}
