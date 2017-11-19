# Rust-oracle - Work in progress

This is an [Oracle database][] driver for [Rust][] based on [ODPI-C][].

Don't use this until the version number reaches to 0.1.0.

## Build-time Requirements

* Rust 1.18 or later
* C compiler. See `Compile-time Requirements` in [this document](https://github.com/alexcrichton/cc-rs#compile-time-requirements).

## Run-time Requirements

* Oracle client 11.2 or later. See [ODPI-C installation document][].

## Usage

Rust-oracle has not been published to [crates.io](https://crates.io/).
You need to put this in your Cargo.toml:

```text
[dependencies]
oracle = { git = "https://github.com/kubo/rust-oracle.git" }
```

## NLS_LANG parameter

[NLS_LANG][] consists of three components: [language][], [territory][] and
charset. However the charset component is ignored and UTF-8(AL32UTF8) is used
as charset because rust characters are UTF-8.

The territory component specifies numeric format, date format and so on.
However it affects only conversion in Oralce. See the following example:

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

## Conversion from Oracle types to Rust types

Values in Oracle are converted to Rust type as possible as it can.
The following table indicates supported conversion.

| Oracle Type | Rust Type |
| --- | --- |
| CHAR, NCHAR, VARCHAR2, NVARCHAR2 | String |
| ″ | i8, i16, i32, i64, u8, u16, u32, u64 via `parse()` |
| ... | ... |

This conversion is used also to get values from output parameters.

## Conversion from Rust types to Oracle types

When a rust value is set to an input parameter, its Oracle type is
determined by the rust type.

| Rust Type | Oracle Type |
| --- | --- |
| str, String | NVARCHAR2(length of the rust value) |
| i8, i16, i32, i64, u8, u16, u32, u64, f32, f64 | NUMBER |
| Vec\<u8> | RAW(length of the rust value) |
| chrono::DateTime, Timestamp | TIMESTAMP(9) WITH TIME ZONE |
| chrono::Date | TIMESTAMP(0) WITH TIME ZONE |
| chrono::naive::NaiveDateTime | TIMESTAMP(9) |
| chrono::naive::NaiveDate | TIMESTAMP(0) |
| chrono::Duration, IntervalDS | INTERVAL DAY(9) TO SECOND(9) |
| IntervalYM | INTERVAL YEAR(9) TO MONTH |

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
