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

Applications using Rust-oracle 0.1.x should use 0.2.x. The incompatibility
between 0.1.x and 0.2.x is trivial so they will work well without modification.
The author continues updating 0.2.x to fix bugs as long as it doesn't
introduce incompatibilities.

New features are added in Rust-oracle 0.3.x or later. There are enormous
incompatibilities between 0.2.x and 0.3.x. They were introduced to follow
Rust way. Some parameters were removed and builder data types were added
instead. Some types were moved to a new module `sql_type`.

Rust-oracle 0.4.x will include breaking changes about [query methods](https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md).
The query methods with `as` may be renamed to and merged into methods
without `as`. The `params` argument of `Connection.prepare` will be
replaced with some kind of the builder pattern.

## Change Log

See [ChangeLog.md](https://github.com/kubo/rust-oracle/blob/master/ChangeLog.md).

## Build-time Requirements

* Rust 1.31.0 or later for rust-oracle 0.3.0 and later.
* Rust 1.19.0 or later for rust-oarcle 0.1.x and 0.2.x.
* C compiler. See `Compile-time Requirements` in [this document](https://github.com/alexcrichton/cc-rs#compile-time-requirements).

## Run-time Requirements

* Oracle client 11.2 or later. See [ODPI-C installation document][].

## Usage

Put this in your `Cargo.toml`:

```text
[dependencies]
oracle = "0.3.0"
```

When you need to fetch or bind [chrono](https://docs.rs/chrono/0.4/chrono/)
data types, enable `chrono` feature:

```text
[dependencies]
oracle = { version = "0.3.0", features = ["chrono"] }
```

Then put this in your crate root:

```rust
extern crate oracle;
```

## Examples

Executes select statements and get rows:

```no_run
# use oracle::*; fn try_main() -> Result<()> {

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
# Ok(())} fn main() { try_main().unwrap(); }
```

Executes select statements and get the first rows:

```no_run
# use oracle::*; fn try_main() -> Result<()> {
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
# Ok(())} fn main() { try_main().unwrap(); }
```

Executes non-select statements:

```no_run
# use oracle::*; fn try_main() -> Result<()> {
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
# Ok(())} fn main() { try_main().unwrap(); }
```

Prints column information:

```no_run
# use oracle::*; fn try_main() -> Result<()> {
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
# Ok(())} fn main() { try_main().unwrap(); }
```

Prepared statement:

```no_run
# use oracle::*; fn try_main() -> Result<()> {
use oracle::Connection;

let conn = Connection::connect("scott", "tiger", "//localhost/XE")?;

// Create a prepared statement
let mut stmt = conn.prepare("insert into person values (:1, :2)", &[])?;
// Insert one row
stmt.execute(&[&1, &"John"])?;
// Insert another row
stmt.execute(&[&2, &"Smith"])?;
# Ok(())} fn main() { try_main().unwrap(); }
```

This is more efficient than two `conn.execute()`.
An SQL statement is executed in the DBMS as follows:

* step 1. Parse the SQL statement and create an execution plan.
* step 2. Execute the plan with bind parameters.

When a prepared statement is used, step 1 is called only once.

## NLS_LANG parameter

[NLS_LANG][] consists of three components: [language][], [territory][] and
charset. However the charset component is ignored and UTF-8(AL32UTF8) is used
as charset because rust characters are UTF-8.

The territory component specifies numeric format, date format and so on.
However it affects only conversion in Oracle. See the following example:

```no_run
# use oracle::*; fn try_main() -> Result<()> {
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
# Ok(())} fn main() { try_main().unwrap(); }
```

Note that NLS_LANG must be set before first rust-oracle function execution if
required.

## TODO

* Connection pooling
* Read and write LOB as stream
* REF CURSOR, BOOLEAN
* Scrollable cursors
* Batch DML
* Better Oracle object type support

## License

Rust-oracle and ODPI-C bundled in rust-oracle are under the terms of:

1. [the Universal Permissive License v 1.0 or at your option, any later version](http://oss.oracle.com/licenses/upl); and/or
2. [the Apache License v 2.0](http://www.apache.org/licenses/LICENSE-2.0).

[Rust]:                 https://www.rust-lang.org/
[ODPI-C]:               https://oracle.github.io/odpi/
[ODPI-C installation document]: https://oracle.github.io/odpi/doc/installation.html
[Oracle database]: https://www.oracle.com/database/index.html
[NLS_LANG]: http://www.oracle.com/technetwork/products/globalization/nls-lang-099431.html
[language]: http://www.oracle.com/technetwork/database/database-technologies/globalization/nls-lang-099431.html#_Toc110410559
[territory]: http://www.oracle.com/technetwork/database/database-technologies/globalization/nls-lang-099431.html#_Toc110410560
*/

use lazy_static::lazy_static;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::ptr;
use std::result;
use std::slice;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(improper_ctypes)]
mod binding;
mod connection;
mod error;
mod row;
pub mod sql_type;
mod sql_value;
mod statement;
mod util;
mod version;

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
pub use crate::statement::StatementType;
pub use crate::statement::StmtParam;
pub use crate::version::Version;

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
                    [<Dpi $name>] { raw: raw }
                }

                #[allow(dead_code)]
                fn with_add_ref(raw: *mut [<dpi $name>]) -> [<Dpi $name>] {
                    unsafe { [<dpi $name _addRef>](raw) };
                    [<Dpi $name>] { raw: raw }
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

// define DpiObjectType wrapping *mut dpiObjectType.
define_dpi_data_with_refcount!(ObjectType);

// define DpiObjectAttr wrapping *mut dpiObjectAttr.
define_dpi_data_with_refcount!(ObjectAttr);

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
    if s.len() == 0 {
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

mod private {
    pub trait Sealed {}

    impl Sealed for usize {}
    impl<'a> Sealed for &'a str {}
}
