# Rust Oracle - Work in progress

This is an Oracle driver for [Rust][] based on [ODPI-C][].

Supported Oracle version is 11.2 or later.

Note that this is work-in-progress and will not be finished because
I stopped to develop it. If you are interested in this driver, look
at [this file][notes.md] and feel free to take over this task. The code I wrote
is in public domain. You can apply any license.

## Installation

Check out rust-oracle and run `cargo`. You have no need to install Oracle client
to build this.
```shell
$ git clone --recursive https://github.com/kubo/rust-oracle-wip.git
$ cd rust-oracle-wip
$ cargo build
```

You need to install Oracle client to use this crate as described in
the [ODPI-C installation document][]

Look at `src/main.rs` as a sample.

## Supported features

* Select string, number and date columns
  ```rust
  let conn = oracle::Connection::connect("scott", "tiger", "", oracle::AuthMode::Default).unwrap();
  let mut stmt = conn.prepare("select ename, sal, hiredate, mgr from emp").unwrap();
  stmt.execute().unwrap();
  while let Ok(row) = stmt.fetch() {
      let ename: String = row.get(0).unwrap();  // index by 0-based position
      let sal: f64 = row.get("SAL").unwrap(); // index by name (case-sensitive)
      let hiredate: oracle::Timestamp = row.get(2).unwrap();
      let mgr: Option<i32> = row.get(3).unwrap(); // nullable column must be get as Option<...> to avoid panic
      ...
  }
  ```

* Define column types before fetch

  Sample code to define a `date` column as `varchar2(60)` to fetch it as String.
  ```rust
  let conn = oracle::Connection::connect("scott", "tiger", "", oracle::AuthMode::Default).unwrap();
  let mut stmt = conn.prepare("select ename, sal, hiredate from emp").unwrap();
  stmt.execute().unwrap();
  // define the hiredate column as varchar2(60)
  stmt.define(2, oracle::OracleType::Varchar2(60)).unwrap();
  while let Ok(row) = stmt.fetch() {
      let ename: String = row.get(0).unwrap();
      let sal: f64 = row.get(1).unwrap();
      let hiredate: String = row.get(2).unwrap() // fetch it as String
      ...
  }
  ```

* Implement fetchable data types

  `DATE` columns are [defined as timestamp internally][tsdef] and are [fetched
  as oracle::Timestamp][tsget]. If you need to fetch them as [NaiveDate][], use FromSql
  trait to convert oracle::Timestamp to NaiveDate internally.
  (I have not checked whether it works.)
  ```rust
  impl FromSql for NaiveDate {
      fn from(value: ValueRef) -> Result<Self> {
          // get oracle::Timestamp
          let ts = try!(value.as_timestamp());
          // convert oracle::Timestamp to NaiveDate
          Ok(NaiveDate::from_ymd(ts.year(), ts.month(), ts.day()))
      }
      fn type_name() -> String {
          "NaiveDate".to_string()
      }
  }

  ...
  ...
  ...
   
     let mut stmt = conn.prepare("select ename, sal, hiredate from emp").unwrap();
     stmt.execute().unwrap();
     while let Ok(row) = stmt.fetch() {
         let ename: String = row.get(0).unwrap();
         let sal: f64 = row.get(1).unwrap();
         let hiredate: NaiveDate = row.get(2).unwrap() // get as NaiveDate
         ...
     }
   ```

* Basic transaction methods: commit and rollback

## Unsupported features

* Fetch rows as iterator
* CLOB, NCLOB, BLOB and BFILE (LOB columns may be fetched by defining them as `OracleType::Long` or `OracleType::LongRaw` explicitly as defining a `date` column as `varchar2(60)`.)
* REF CURSOR
* BOOLEAN
* Bind parameters
* Test code to check whether this crate works

[Rust]:                 https://www.rust-lang.org/
[ODPI-C]:               https://oracle.github.io/odpi/
[ODPI-C installation document]: https://oracle.github.io/odpi/doc/installation.html
[tsdef]:                https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L251-L252
[tsget]:                https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L1369-L1379
[NaiveDate]:            https://docs.rs/chrono/0.3.0/chrono/naive/date/struct.NaiveDate.html

[notes.md]:             https://github.com/kubo/rust-oracle-wip/blob/master/notes.md
