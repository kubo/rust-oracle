# Rust-oracle
[![Test](https://img.shields.io/github/workflow/status/kubo/rust-oracle/Run%20tests?label=test)](https://github.com/kubo/rust-oracle/actions/workflows/run-tests.yml)
[![Crates.io](https://img.shields.io/crates/v/oracle.svg)](https://crates.io/crates/oracle)
[![Docs](https://docs.rs/oracle/badge.svg)](https://docs.rs/oracle)
[![Docs (in development)](https://img.shields.io/badge/docs-in_development-486D9F)](https://www.jiubao.org/rust-oracle/oracle/)

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

```rust
use oracle::{Connection, Error};

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
```

Executes select statements and get the first rows:

```rust
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
```

Executes non-select statements:

```rust
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
```

Prints column information:

```rust
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
```

Prepared statement:

```rust
use oracle::Connection;

let conn = Connection::connect("scott", "tiger", "//localhost/XE")?;

// Create a prepared statement
let mut stmt = conn.statement("insert into person values (:1, :2)").build()?;
// Insert one row
stmt.execute(&[&1, &"John"])?;
// Insert another row
stmt.execute(&[&2, &"Smith"])?;
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

```rust
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
```

Note that NLS_LANG must be set before first rust-oracle function execution if
required.

## TODO

* [BFILEs (External LOBs)](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-5834BC49-4053-40FF-BE39-B14342B1201E) (Note: Reading contents of BFILEs as `Vec<u8>` is supported.)
* Scrollable cursors
* Better Oracle object type support
* XML data type
* [JSON data type](https://oracle-base.com/articles/21c/json-data-type-21c)

## Related Projects

Other crates for connecting to Oracle:
* [Sibyl]: an OCI-based interface supporting both blocking (threads) and nonblocking (async) API

Oracle-related crates:
* [r2d2-oracle]: Oracle support for the [r2d2] connection pool
* [bb8-oracle]: [bb8] connection pool support for oracle
* [include-oracle-sql]: an extension of [include-sql] using [Sibyl] for database access

## License

Rust-oracle and ODPI-C bundled in rust-oracle are under the terms of:

1. [the Universal Permissive License v 1.0 or at your option, any later version](http://oss.oracle.com/licenses/upl); and/or
2. [the Apache License v 2.0](http://www.apache.org/licenses/LICENSE-2.0).

[Rust]:                 https://www.rust-lang.org/
[ODPI-C]:               https://oracle.github.io/odpi/
[ODPI-C installation document]: https://oracle.github.io/odpi/doc/installation.html
[Oracle database]: https://www.oracle.com/database/index.html
[NLS_LANG]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-86A29834-AE29-4BA5-8A78-E19C168B690A
[bb8]: https://crates.io/crates/bb8
[bb8-oracle]: https://crates.io/crates/bb8-oracle
[include-sql]: https://crates.io/crates/include-sql
[include-oracle-sql]: https://crates.io/crates/include-oracle-sql
[r2d2]: https://crates.io/crates/r2d2
[r2d2-oracle]: https://crates.io/crates/r2d2-oracle
[Sibyl]: https://crates.io/crates/sibyl
