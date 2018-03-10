# Rust-oracle

This is an [Oracle database][] driver for [Rust][] based on [ODPI-C][].

Don't use this until the version number reaches to 0.1.0.
This will be 0.1.0 if there are no incompatible changes predicted
from planned features.

**Methods for querying rows were changed in 0.0.4.** If you had written
programs using rust-oracle before 0.0.4, enable the `restore-deleted`
feature in `Cargo.toml`. It restores deleted methods in 0.0.4 and disables
statement-type checking in execute methods.

## Change Log

### 0.0.6 (not released)

Methods for establishing connections were changed in order to avoid
incompatible changes when connection pooling is supported in future.

Changes:

* New methods and enums.
  * `Connection::connect()`
  * `ConnParam`

* Deprecated methods.
  * `Connnection::new()`. Use `Connection::connect()` instead.

Incompatible changes:

* Renamed variants.
  * `Error::NoMoreData` &#x2192; `Error::NoDataFound`
* Removed structs and enums.
  * `Connector` (connection builder). Use `ConnParam` in order to specify extra connection parameters instead.
  * `AuthMode`. Use `ConnParam` to specify authentication mode instead.
* Methods whose return type was changed from `&String` to `&str`.
  * `Connection.tag()`
  * `ColumnInfo.name()`
  * `DbError.message()`
  * `DbError.fn_name()`
  * `DbError.action()`
  * `ObjectType.schema()`
  * `ObjectType.name()`
  * `ObjectTypeAttr.name()`
* Methods whose return type was changed from `&Vec<...>` to `&[...]`.
  * `Row.sql_values()`
  * `ResultSet.column_info()`
  * `ObjectType.attributes()`

### 0.0.5

New features:

* Add query methods to `Connection` to fetch rows without using `Statement`.
  * `Connection.query()`
  * `Connection.query_named()`
  * `Connection.query_as()`
  * `Connection.query_as_named()`
* Add query_row methods to `Statement` to fetch a first row without using `ResultSet`.
  * `Statement.query_row()`
  * `Statement.query_row_named()`
  * `Statement.query_row_as()`
  * `Statement.query_row_as_named()`

Incompatible changes:

* Merge `RowResultSet` struct into `RowValueResultSet` and rename it to `ResultSet`.

### 0.0.4

New features:

* Add query methods to `Statement` to fetch rows as iterator.
  * `Statement.query()`
  * `Statement.query_named()`
  * `Statement.query_as()`
  * `Statement.query_as_named()`
* Add query_row methods to `Connection` to fetch a first row without using `Statement`.
  * `Connection.query_row()`
  * `Connection.query_row_named()`
  * `Connection.query_row_as()`
  * `Connection.query_row_as_named()`
* Autocommit mode.

Incompatible changes:

* Execute methods fail for select statements. Use query methods instead.
  * `Connection.execute()`
  * `Connection.execute_named()`
  * `Statement.execute()`
  * `Statement.execute_named()`
* Renamed traits, methods and variants.
  * `ColumnValues` &#x2192; `RowValue`
  * `Row.values()` &#x2192; `Row.get_as()`
  * `Row.columns()` &#x2192; `Row.sql_values()`
  * `Error::Overflow` &#x2192; `Error::OutOfRange`
* Removed methods.
  * Statement.column_count()
  * Statement.column_names()
  * Statement.column_info()
  * Statement.fetch()
  * SqlValue.clone()

## Build-time Requirements

* Rust 1.19 or later
* C compiler. See `Compile-time Requirements` in [this document](https://github.com/alexcrichton/cc-rs#compile-time-requirements).

## Run-time Requirements

* Oracle client 11.2 or later. See [ODPI-C installation document][].

## Usage

Put this in your `Cargo.toml`:

```text
[dependencies]
oracle = "0.0.5"
```

When you need to fetch or bind [chrono](https://docs.rs/chrono/0.4/chrono/)
data types, enable `chrono` feature:

```text
[dependencies]
oracle = { version = "0.0.5", features = ["chrono"] }
```

If you had written programs using rust-oracle before 0.0.4, try
the `restore-deleted` feature. It restores deleted methods in 0.0.4 and
disables statement-type checking in execute methods.

```text
[dependencies]
oracle = { version = "0.0.5", features = ["restore-deleted"] }
```

Then put this in your crate root:

```rust
extern crate oracle;
```

## Examples

Executes select statements and get rows:

```rust
use oracle::{Connection, Error};

// Connect to a database.
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[]).unwrap();

let sql = "select ename, sal, comm from emp where deptno = :1";

// Select a table with a bind variable.
println!("---------------|---------------|---------------|");
let rows = conn.query(sql, &[&30]).unwrap();
for row_result in rows {
    let row = row_result.unwrap();
    // get a column value by position (0-based)
    let ename: String = row.get(0).unwrap();
    // get a column by name (case-insensitive)
    let sal: i32 = row.get("sal").unwrap();
    // Use `Option<...>` to get a nullable column.
    // Otherwise, `Err(Error::NullValue)` is returned
    // for null values.
    let comm: Option<i32> = row.get(2).unwrap();

    println!(" {:14}| {:>10}    | {:>10}    |",
             ename,
             sal,
             comm.map_or("".to_string(), |v| v.to_string()));
}

// Another way to fetch rows.
// The rows iterator returns Result<(String, i32, Option<i32>)>.
println!("---------------|---------------|---------------|");
let rows = conn.query_as::<(String, i32, Option<i32>)>(sql, &[&10]).unwrap();
for row_result in rows {
    let (ename, sal, comm) = row_result.unwrap();
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
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[]).unwrap();

let sql = "select ename, sal, comm from emp where empno = :1";

// Print the first row.
let row = conn.query_row(sql, &[&7369]).unwrap();
let ename: String = row.get("empno").unwrap();
let sal: i32 = row.get("sal").unwrap();
let comm: Option<i32> = row.get("comm").unwrap();
println!("---------------|---------------|---------------|");
println!(" {:14}| {:>10}    | {:>10}    |",
         ename,
         sal,
         comm.map_or("".to_string(), |v| v.to_string()));
// When no rows are found, conn.query_row() returns `Err(Error::NoDataFound)`.

// Get the first row as a tupple
let row = conn.query_row_as::<(String, i32, Option<i32>)>(sql, &[&7566]).unwrap();
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
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[]).unwrap();

conn.execute("create table person (id number(38), name varchar2(40))", &[]).unwrap();

// Execute a statement with positional parameters.
conn.execute("insert into person values (:1, :2)",
             &[&1, // first parameter
               &"John" // second parameter
              ]).unwrap();

// Execute a statement with named parameters.
conn.execute_named("insert into person values (:id, :name)",
                   &[("id", &2), // 'id' parameter
                     ("name", &"Smith"), // 'name' parameter
                    ]).unwrap();

// Commit the transaction.
conn.commit().unwrap();

// Delete rows
conn.execute("delete from person", &[]).unwrap();

// Rollback the transaction.
conn.rollback().unwrap();
```

Prints column information:

```rust
use oracle::Connection;

// Connect to a database.
let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[]).unwrap();

let sql = "select ename, sal, comm from emp where 1 = 2";
let rows = conn.query(sql, &[]).unwrap();

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

let conn = Connection::connect("scott", "tiger", "//localhost/XE", &[]).unwrap();

// Create a prepared statement
let mut stmt = conn.prepare("insert into person values (:1, :2)").unwrap();
// Insert one row
stmt.execute(&[&1, &"John"]).unwrap();
// Insert another row
stmt.execute(&[&2, &"Smith"]).unwrap();
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

```rust
use oracle::Connection;

// The territory is France.
std::env::set_var("NLS_LANG", "french_france.AL32UTF8");
let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();

// 10.1 is converted to a string in Oracle and fetched as a string.
let result = conn.query_row_as::<String>("select to_char(10.1) from dual", &[]).unwrap();
assert_eq!(result, "10,1"); // The decimal mark depends on the territory.

// 10.1 is fetched as a number and converted to a string in rust-oracle
let result = conn.query_row_as::<String>("select 10.1 from dual", &[]).unwrap();
assert_eq!(result, "10.1"); // The decimal mark is always period(.).
```

Note that NLS_LANG must be set before first rust-oracle function execution if
required.

## TODO

* Connection pooling
* Read and write LOB as stream
* REF CURSOR, BOOLEAN
* Scrollable cursors
* Batch DML
* DML returning
* Better Oracle object type support

## License

Rust-oracle itself is under [2-clause BSD-style license](https://opensource.org/licenses/BSD-2-Clause).

ODPI-C bundled in rust-oracle is under the terms of:

1. [the Universal Permissive License v 1.0 or at your option, any later version](http://oss.oracle.com/licenses/upl); and/or
2. [the Apache License v 2.0](http://www.apache.org/licenses/LICENSE-2.0). 

[Rust]:                 https://www.rust-lang.org/
[ODPI-C]:               https://oracle.github.io/odpi/
[ODPI-C installation document]: https://oracle.github.io/odpi/doc/installation.html
[Oracle database]: https://www.oracle.com/database/index.html
[NLS_LANG]: http://www.oracle.com/technetwork/products/globalization/nls-lang-099431.html
[language]: http://www.oracle.com/technetwork/database/database-technologies/globalization/nls-lang-099431.html#_Toc110410559
[territory]: http://www.oracle.com/technetwork/database/database-technologies/globalization/nls-lang-099431.html#_Toc110410560
[Connector]: https://docs.rs/oracle/0.0.5/oracle/struct.Connector.html
