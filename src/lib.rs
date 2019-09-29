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

/*!
This is an [Oracle database][] driver for [Rust][] based on [ODPI-C][].

## Change Log

See [ChangeLog.md](https://github.com/kubo/rust-oracle/blob/master/ChangeLog.md).

## Build-time Requirements

* Rust 1.19 or later
* C compiler. See `Compile-time Requirements` in [this document](https://github.com/alexcrichton/cc-rs#compile-time-requirements).

## Run-time Requirements

* Oracle client 11.2 or later. See [ODPI-C installation document][].

## Usage

Put this in your `Cargo.toml`:

```text
[dependencies]
oracle = "0.2.0"
```

When you need to fetch or bind [chrono](https://docs.rs/chrono/0.4/chrono/)
data types, enable `chrono` feature:

```text
[dependencies]
oracle = { version = "0.2.0", features = ["chrono"] }
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
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[])?;

let sql = "select ename, sal, comm from emp where deptno = :1";

// Select a table with a bind variable.
println!("---------------|---------------|---------------|");
let rows = conn.query(sql, &[&30])?;
for row_result in &rows {
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
for row_result in &rows {
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
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[])?;

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
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[])?;

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
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[])?;

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

let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[])?;

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
let conn = Connection::connect("scott", "tiger", "", &[])?;

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
mod sql_value;
mod statement;
mod types;
mod util;

pub use connection::ConnParam;
pub use connection::Connection;
pub use connection::ShutdownMode;
pub use connection::StartupMode;
pub use error::DbError;
pub use error::Error;
pub use error::ParseOracleTypeError;
pub use row::ResultSet;
pub use row::Row;
pub use row::RowValue;
pub use sql_value::SqlValue;
pub use statement::BindIndex;
pub use statement::ColumnIndex;
pub use statement::ColumnInfo;
pub use statement::Statement;
pub use statement::StatementType;
pub use statement::StmtParam;
pub use types::interval_ds::IntervalDS;
pub use types::interval_ym::IntervalYM;
pub use types::object::Collection;
pub use types::object::Object;
pub use types::object::ObjectType;
pub use types::object::ObjectTypeAttr;
pub use types::oracle_type::OracleType;
pub use types::timestamp::Timestamp;
pub use types::version::Version;
pub use types::FromSql;
pub use types::ToSql;
pub use types::ToSqlNull;

use binding::*;
use types::oracle_type::NativeType;

pub type Result<T> = result::Result<T, Error>;

struct DpiConn {
    conn: *mut dpiConn,
}

impl DpiConn {
    fn new(conn: *mut dpiConn) -> DpiConn {
        DpiConn { conn: conn }
    }

    pub(crate) fn raw(&self) -> *mut dpiConn {
        self.conn
    }
}

impl Clone for DpiConn {
    fn clone(&self) -> DpiConn {
        unsafe { dpiConn_addRef(self.raw()) };
        DpiConn::new(self.raw())
    }
}

impl Drop for DpiConn {
    fn drop(&mut self) {
        unsafe { dpiConn_release(self.raw()) };
    }
}

// dpiConn is created with the DPI_MODE_CREATE_THREADED flag,
// so that it is suitable for Send and Sync.
unsafe impl Send for DpiConn {}
unsafe impl Sync for DpiConn {}

/// Returns Oracle client version
///
/// # Examples
///
/// ```
/// # use oracle::*; fn try_main() -> Result<()> {
/// let client_ver = oracle::client_version()?;
/// println!("Oracle Client Version: {}", client_ver);
/// # Ok(())} fn main() { try_main().unwrap(); }
/// ```
pub fn client_version() -> Result<Version> {
    let mut dpi_ver = Default::default();
    let ctx = Context::get()?;
    chkerr!(ctx, dpiContext_getClientVersion(ctx.context, &mut dpi_ver));
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

// Only one instance of Context exists and it is read-only.
unsafe impl Sync for Context {}

enum ContextResult {
    Ok(Context),
    Err(dpiErrorInfo),
}

unsafe impl Sync for ContextResult {}

trait AssertSend: Send {}
trait AssertSync: Sync {}

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
            dpiContext_create(
                DPI_MAJOR_VERSION,
                DPI_MINOR_VERSION,
                &mut ctxt.context,
                &mut err,
            )
        } == DPI_SUCCESS as i32
        {
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
            outNewSession: 0,
        }
    }
}

impl Default for dpiData {
    fn default() -> dpiData {
        dpiData {
            isNull: 0,
            value: dpiDataBuffer { asInt64: 0 },
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
            timeout: 0,
            waitTimeout: 0,
            maxLifetimeSession: 0,
            plsqlFixupCallback: ptr::null(),
            plsqlFixupCallbackLength: 0,
        }
    }
}

impl Default for dpiSubscrCreateParams {
    fn default() -> dpiSubscrCreateParams {
        dpiSubscrCreateParams {
            subscrNamespace: 0,
            protocol: 0,
            qos: 0,
            operations: 0,
            portNumber: 0,
            timeout: 0,
            name: ptr::null(),
            nameLength: 0,
            callback: None,
            callbackContext: ptr::null_mut(),
            recipientName: ptr::null(),
            recipientNameLength: 0,
            ipAddress: ptr::null_mut(),
            ipAddressLength: 0,
            groupingClass: 0,
            groupingValue: 0,
            groupingType: 0,
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
