# Rust-oracle

This is an [Oracle database][] driver for [Rust][] based on [ODPI-C][].

Don't use this until the version number reaches to 0.1.0.

## Build-time Requirements

* Rust 1.19 or later
* C compiler. See `Compile-time Requirements` in [this document](https://github.com/alexcrichton/cc-rs#compile-time-requirements).

## Run-time Requirements

* Oracle client 11.2 or later. See [ODPI-C installation document][].

## Usage

Rust-oracle was published to [crates.io](https://crates.io/crates/oracle).
However it is old. Use rust-oracle in the github.

```text
[dependencies]
oracle = { git = "https://github.com/kubo/rust-oracle.git" }
```

When you need to fetch or bind [chrono](https://docs.rs/chrono/0.4/chrono/)
data types, enable `chrono` feature:

```text
[dependencies]
oracle = { git = "https://github.com/kubo/rust-oracle.git", features = ["chrono"] }
```

## Examples

Select a table:

```rust
extern crate oracle;

fn main() {
    // Connect to a database.
    let conn = oracle::Connection::new("scott", "tiger", "//localhost/XE").unwrap();
    // Select a table with a bind variable.
    let mut stmt = conn.execute("select ename, sal, comm from emp where deptno = :1", &[&30]).unwrap();

    // Print column names
    for info in stmt.column_info() {
        print!(" {:14}|", info.name());
    }
    println!("");

    // Print column types
    for info in stmt.column_info() {
        print!(" {:14}|", info.oracle_type().to_string());
    }
    println!("");

    // Print column values
    println!("---------------|---------------|---------------|");
    while let Ok(row) = stmt.fetch() {
        // get a column value by position (0-based)
        let ename: String = row.get(0).unwrap();
        // get a column by name (case-insensitive)
        let sal: i32 = row.get("sal").unwrap();
        // get a nullable column
        let comm: Option<i32> = row.get(2).unwrap();

        println!(" {:14}| {:>10}    | {:>10}    |",
                 ename,
                 sal,
                 comm.map_or("".to_string(), |v| v.to_string()));
    }
}
```

## NLS_LANG parameter

[NLS_LANG][] consists of three components: [language][], [territory][] and
charset. However the charset component is ignored and UTF-8(AL32UTF8) is used
as charset because rust characters are UTF-8.

The territory component specifies numeric format, date format and so on.
However it affects only conversion in Oracle. See the following example:

```rust
// The territory is France.
std::env::set_var("NLS_LANG", "french_france.AL32UTF8");
let conn = oracle::Connection::new("scott", "tiger", "").unwrap();

// 10.1 is converted to a string in Oracle and fetched as a string.
let mut stmt = conn.execute("select to_char(10.1) from dual", &[]).unwrap();
let row = stmt.fetch().unwrap();
let result: String = row.get(0).unwrap();
assert_eq!(result, "10,1"); // The decimal mark depends on the territory.

// 10.1 is fetched as a number and converted to a string in rust-oracle
let mut stmt = conn.execute("select 10.1 from dual", &[]).unwrap();
let row = stmt.fetch().unwrap();
let result: String = row.get(0).unwrap();
assert_eq!(result, "10.1"); // The decimal mark is always period(.).
```

Note that NLS_LANG must be set before first rust-oracle function execution if
required.

## TODO

* Connection pooling
* Read and write LOB as stream
* REF CURSOR, BOOLEAN
* Autocommit mode
* Scrollable cursors

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
