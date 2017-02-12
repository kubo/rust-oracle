# Rust Oracle - Work in progress

This is an Oracle driver for [Rust][] based on [ODPI-C][].

Note that this is work-in-progress and will not be finished because
I stopped to continue it. If you are interested in this driver, feel
free to take over this task. The code I wrote is in public domain.
You can apply any license.

## Installation

Install Oracle client and set the environment variables `OCI_INC_DIR` and `OCI_LIB_DIR` as [this page][install-node-oracledb].

Check out rust-oracle and run `cargo`.
```shell
$ export OCI_INC_DIR=...
$ export OCI_LIB_DIR=...
$ export LD_LIBRARY_PATH=$OCI_LIB_DIR
$ git clone --recursive https://github.com/kubo/rust-oracle-wip.git
$ cd rust-oracle-wip
$ cargo build
```

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
  as oracle::Timestamp][tsget]. If you need to fetch them as [NaiveDate][], use FromSQL
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
* CLOB, NCLOB, BLOB and BFILE (LOB columns may be fetched by defining them as `OracleType::Long` or `OracleType::LongRaw` explicity as defining a `date` column as `varchar2(60)`.)
* REF CURSOR
* BOOLEAN
* Bind parameters
* Test code to check whether this crate works

## Notes for who takes over this driver

### File strcuture

* odpi - submodule to checkout [ODPI-C]
* src/ffi.rs - one-to-one mapping of functions and macros in [dpi.h][]
* src/odpi.rs - wrapper for functions and macros in src/ffi.rs
* src/main.rs - sample code. This file must be moved to other location.
* ... others

### How columns are defined

Query metadata are retrieved as [dpiQueryInfo][]. [ColumnInfo][] and [OracleType][] are
created from them via [ColumnInfo::new][] and [from_query_info][] respectively. OracleType
is also used to [define column types][stmt.define] by end users. If not all columns are defined explicity,
they are [implicity defined according to the OracleType in ColumnInfo][stmt.define_columns] just before fetch.

ODPI-C requires dpiVar to define columns. OracleType provides [parameters to create a dpiVar][dpiConn_newVar]
via [var_create_param][]. A dpiVar is created via [DpiVar::new] and passed to [dpiStmt_define][] via
[DpiStatement::define].

### How columns are fetched

to be written

### How number columns are defined and fetched

to be written

### Another idea to define columns

to be written

[Rust]:                 https://www.rust-lang.org/

[install-node-oracledb]: https://github.com/oracle/node-oracledb/blob/master/INSTALL.md

[NaiveDate]:            https://docs.rs/chrono/0.3.0/chrono/naive/date/struct.NaiveDate.html

[tsdef]:                https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L251-L252
[tsget]:                https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L1369-L1379

[dpiQueryInfo]:         https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/ffi.rs#L454-L466

[OracleType]:           https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L131-L186
[from_query_info]:      https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L190-L223
[var_create_param]:     https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L226-L288

[ColumnInfo]:           https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L1192-L1196
[ColumnInfo::new]:      https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L1211-L1217

[stmt.define]:          https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/lib.rs#L225-L230
[stmt.define_columns]:  https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/lib.rs#L250-L260

[DpiStatement::define]: https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L1151-L1156

[DpiVar]:               https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L1235-L1239
[DpiVar::new]:          https://github.com/kubo/rust-oracle-wip/blob/18850ec606d1a8815a85491c1ed51fadbcc19de8/src/odpi.rs#L1242-L1254

[ODPI-C]:               https://oracle.github.io/odpi/
[dpiStmt_define]:       https://oracle.github.io/odpi/doc/public_functions/dpiStmt.html#c.dpiStmt_define
[dpiConn_newVar]:       https://oracle.github.io/odpi/doc/public_functions/dpiConn.html#c.dpiConn_newVar
[dpi.h]:                https://github.com/oracle/odpi/blob/master/include/dpi.h
