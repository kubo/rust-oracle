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
  ```rust
  let conn = oracle::Connection::connect("scott", "tiger", "", oracle::AuthMode::Default).unwrap();
  let mut stmt = conn.prepare("select ename, sal, hiredate from emp").unwrap();
  stmt.execute().unwrap();
  // define the hiredate column to fetch it as varchar2(60)
  stmt.define(2, oracle::OracleType::Varchar2(60)).unwrap();
  while let Ok(row) = stmt.fetch() {
      let ename: String = row.get(0).unwrap();
      let sal: f64 = row.get(1).unwrap();
      let hiredate: String = row.get(2).unwrap() // get as String
      ...
  }
  ```

* FromSQL trait to a new fetchable type
  Date columns are fetched as oracle::Timestamp. If you want to fetch it as
  [NaiveDate](https://docs.rs/chrono/0.3.0/chrono/naive/date/struct.NaiveDate.html) add the following code.
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
* CLOB, NCLOB, BLOB and BFILE (LOB columns may be fetched by defining them as `OracleType::Long` or `OracleType::LongRaw` explicity as defining DATE columns as VARCHAR2(60).)
* REF CURSOR
* BOOLEAN
* Bind parameters

[Rust]: https://www.rust-lang.org/
[ODPI-C]: https://oracle.github.io/odpi/
[install-node-oracledb]: https://github.com/oracle/node-oracledb/blob/master/INSTALL.md
