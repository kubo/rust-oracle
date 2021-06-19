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

use std::cell::RefCell;
use std::fmt;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::rc::Rc;

use crate::binding::*;
use crate::chkerr;
use crate::new_odpi_str;
use crate::private;
use crate::sql_type::FromSql;
use crate::sql_type::OracleType;
use crate::sql_type::ToSql;
use crate::sql_value::BufferRowIndex;
use crate::to_odpi_str;
use crate::to_rust_str;
use crate::Connection;
use crate::Error;
use crate::Result;
use crate::ResultSet;
use crate::Row;
use crate::RowValue;
use crate::SqlValue;

const OCI_ATTR_SQLFNCODE: u32 = 10;

// https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-A251CF91-EB9F-4DBC-8BB8-FB5EA92C20DE
const SQLFNCODE_CREATE_TYPE: u16 = 77;
const SQLFNCODE_ALTER_TYPE: u16 = 80;
const SQLFNCODE_DROP_TYPE: u16 = 78;

/// Parameters to prepare Statement.
#[derive(Debug, Clone, PartialEq)]
pub enum StmtParam {
    /// The array size used for performing fetches.
    ///
    /// This specifies the number of rows allocated before performing
    /// fetches. The default value is 100. Higher value reduces
    /// the number of network round trips to fetch rows but requires
    /// more memory. The preferable value depends on the query and
    /// the environment.
    ///
    /// If the query returns only onw row, you should use
    /// `StmtParam::FetchArraySize(1)`.
    FetchArraySize(u32),

    /// The number of rows that will be prefetched by the Oracle Client
    /// library when a query is executed. The default value is
    /// DPI_DEFAULT_PREFETCH_ROWS (2). Increasing this value may reduce
    /// the number of round-trips to the database that are required in
    /// order to fetch rows, but at the cost of increasing memory
    /// requirements.
    /// Setting this value to 0 will disable prefetch completely,
    /// which may be useful when the timing for fetching rows must be
    /// controlled by the caller.
    PrefetchRows(u32),

    /// Reserved for when statement caching is supported.
    Tag(String),

    /// Reserved for when scrollable cursors are supported.
    Scrollable,
}

/// Statement type returned by [`Statement.statement_type`](Statement#method.statement_type).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StatementType {
    /// SELECT statement
    Select,

    /// INSERT statement
    Insert,

    /// UPDATE statement
    Update,

    /// DELETE statement
    Delete,

    /// [MERGE][] statement
    ///
    /// [MERGE]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-5692CCB7-24D9-4C0E-81A7-A22436DC968F
    Merge,

    /// CREATE statement
    Create,

    /// ALTER statement
    Alter,

    /// DROP statement
    Drop,

    /// PL/SQL statement without declare clause
    Begin,

    /// PL/SQL statement with declare clause
    Declare,

    /// COMMIT statement
    Commit,

    /// ROLLBACK statement
    Rollback,

    /// [EXPLAIN PLAN][] statement
    ///
    /// [EXPLAIN PLAN]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FD540872-4ED3-4936-96A2-362539931BA0
    ExplainPlan,

    /// [CALL][] statement
    ///
    /// [CALL]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-6CD7B9C4-E5DC-4F3C-9B6A-876AD2C63545
    Call,

    /// Unknown statement
    Unknown,
}

impl StatementType {
    pub(crate) fn from_enum(num: dpiStatementType) -> StatementType {
        match num as u32 {
            DPI_STMT_TYPE_SELECT => StatementType::Select,
            DPI_STMT_TYPE_INSERT => StatementType::Insert,
            DPI_STMT_TYPE_UPDATE => StatementType::Update,
            DPI_STMT_TYPE_DELETE => StatementType::Delete,
            DPI_STMT_TYPE_MERGE => StatementType::Merge,
            DPI_STMT_TYPE_CREATE => StatementType::Create,
            DPI_STMT_TYPE_ALTER => StatementType::Alter,
            DPI_STMT_TYPE_DROP => StatementType::Drop,
            DPI_STMT_TYPE_BEGIN => StatementType::Begin,
            DPI_STMT_TYPE_DECLARE => StatementType::Declare,
            DPI_STMT_TYPE_COMMIT => StatementType::Commit,
            DPI_STMT_TYPE_ROLLBACK => StatementType::Rollback,
            DPI_STMT_TYPE_EXPLAIN_PLAN => StatementType::ExplainPlan,
            DPI_STMT_TYPE_CALL => StatementType::Call,
            _ => StatementType::Unknown,
        }
    }
}

impl fmt::Display for StatementType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &StatementType::Select => write!(f, "select"),
            &StatementType::Insert => write!(f, "insert"),
            &StatementType::Update => write!(f, "update"),
            &StatementType::Delete => write!(f, "delete"),
            &StatementType::Merge => write!(f, "merge"),
            &StatementType::Create => write!(f, "create"),
            &StatementType::Alter => write!(f, "alter"),
            &StatementType::Drop => write!(f, "drop"),
            &StatementType::Begin => write!(f, "PL/SQL(begin)"),
            &StatementType::Declare => write!(f, "PL/SQL(declare)"),
            &StatementType::Commit => write!(f, "commit"),
            &StatementType::Rollback => write!(f, "rollback"),
            &StatementType::ExplainPlan => write!(f, "explain plan"),
            &StatementType::Call => write!(f, "call"),
            &StatementType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Statement
pub struct Statement<'conn> {
    pub(crate) conn: &'conn Connection,
    handle: *mut dpiStmt,
    pub(crate) column_info: Vec<ColumnInfo>,
    pub(crate) row: Option<Row>,
    shared_buffer_row_index: Rc<RefCell<u32>>,
    statement_type: StatementType,
    is_returning: bool,
    bind_count: usize,
    bind_names: Vec<String>,
    bind_values: Vec<SqlValue>,
    fetch_array_size: u32,
    prefetch_rows: u32,
}

impl<'conn> Statement<'conn> {
    pub(crate) fn new(
        conn: &'conn Connection,
        sql: &str,
        params: &[StmtParam],
    ) -> Result<Statement<'conn>> {
        let sql = to_odpi_str(sql);
        let mut fetch_array_size = DPI_DEFAULT_FETCH_ARRAY_SIZE;
        let mut prefetch_rows = DPI_DEFAULT_PREFETCH_ROWS;
        let mut scrollable = 0;
        let mut tag = new_odpi_str();
        for param in params {
            match param {
                &StmtParam::FetchArraySize(size) => {
                    fetch_array_size = size;
                }
                &StmtParam::PrefetchRows(rows) => {
                    prefetch_rows = rows;
                }
                &StmtParam::Scrollable => {
                    scrollable = 1;
                }
                &StmtParam::Tag(ref name) => {
                    tag = to_odpi_str(name);
                }
            }
        }
        let mut handle: *mut dpiStmt = ptr::null_mut();
        chkerr!(
            conn.ctxt,
            dpiConn_prepareStmt(
                conn.handle.raw(),
                scrollable,
                sql.ptr,
                sql.len,
                tag.ptr,
                tag.len,
                &mut handle
            )
        );
        let mut info = MaybeUninit::uninit();
        chkerr!(
            conn.ctxt,
            dpiStmt_getInfo(handle, info.as_mut_ptr()),
            unsafe {
                dpiStmt_release(handle);
            }
        );
        let info = unsafe { info.assume_init() };
        let mut num = 0;
        chkerr!(conn.ctxt, dpiStmt_getBindCount(handle, &mut num), unsafe {
            dpiStmt_release(handle);
        });
        let bind_count = num as usize;
        let mut bind_names = Vec::with_capacity(bind_count);
        let mut bind_values = Vec::with_capacity(bind_count);
        if bind_count > 0 {
            let mut names: Vec<*const i8> = vec![ptr::null_mut(); bind_count];
            let mut lengths = vec![0; bind_count];
            chkerr!(
                conn.ctxt,
                dpiStmt_getBindNames(handle, &mut num, names.as_mut_ptr(), lengths.as_mut_ptr()),
                unsafe {
                    dpiStmt_release(handle);
                }
            );
            bind_names = Vec::with_capacity(num as usize);
            for i in 0..(num as usize) {
                bind_names.push(to_rust_str(names[i], lengths[i]));
                bind_values.push(SqlValue::new(conn.ctxt));
            }
        };
        Ok(Statement {
            conn: conn,
            handle: handle,
            column_info: Vec::new(),
            row: None,
            shared_buffer_row_index: Rc::new(RefCell::new(0)),
            statement_type: StatementType::from_enum(info.statementType),
            is_returning: info.isReturning != 0,
            bind_count: bind_count,
            bind_names: bind_names,
            bind_values: bind_values,
            fetch_array_size: fetch_array_size,
            prefetch_rows: prefetch_rows,
        })
    }

    /// Closes the statement before the end of lifetime.
    pub fn close(&mut self) -> Result<()> {
        self.close_internal("")
    }

    fn close_internal(&mut self, tag: &str) -> Result<()> {
        let tag = to_odpi_str(tag);

        chkerr!(self.conn.ctxt, dpiStmt_close(self.handle, tag.ptr, tag.len));
        self.handle = ptr::null_mut();
        Ok(())
    }

    /// Executes the prepared statement and returns a result set containing [`Row`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query(&mut self, params: &[&dyn ToSql]) -> Result<ResultSet<Row>> {
        self.exec(params, true, "query")?;
        Ok(ResultSet::<Row>::new(self))
    }

    /// Executes the prepared statement using named parameters and returns a result set containing [`Row`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_named(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<ResultSet<Row>> {
        self.exec_named(params, true, "query_named")?;
        Ok(ResultSet::<Row>::new(self))
    }

    /// Executes the prepared statement and returns a result set containing [`RowValue`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_as<'a, T>(&'a mut self, params: &[&dyn ToSql]) -> Result<ResultSet<'a, T>>
    where
        T: RowValue,
    {
        self.exec(params, true, "query_as")?;
        Ok(ResultSet::new(self))
    }

    /// Executes the prepared statement using named parameters and returns a result set containing [`RowValue`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_as_named<'a, T>(
        &'a mut self,
        params: &[(&str, &dyn ToSql)],
    ) -> Result<ResultSet<'a, T>>
    where
        T: RowValue,
    {
        self.exec_named(params, true, "query_as_named")?;
        Ok(ResultSet::new(self))
    }

    /// Gets one row from the prepared statement using positoinal bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row(&mut self, params: &[&dyn ToSql]) -> Result<Row> {
        let mut rows = self.query(params)?;
        rows.next().unwrap_or(Err(Error::NoDataFound))
    }

    /// Gets one row from the prepared statement using named bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row_named(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<Row> {
        let mut rows = self.query_named(params)?;
        rows.next().unwrap_or(Err(Error::NoDataFound))
    }

    /// Gets one row from the prepared statement as specified type using positoinal bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row_as<T>(&mut self, params: &[&dyn ToSql]) -> Result<T>
    where
        T: RowValue,
    {
        let mut rows = self.query_as::<T>(params)?;
        rows.next().unwrap_or(Err(Error::NoDataFound))
    }

    /// Gets one row from the prepared statement as specified type using named bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row_as_named<T>(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<T>
    where
        T: RowValue,
    {
        let mut rows = self.query_as_named::<T>(params)?;
        rows.next().unwrap_or(Err(Error::NoDataFound))
    }

    /// Binds values by position and executes the statement.
    /// It will retunrs `Err` when the statemnet is a select statement.
    ///
    /// See also [`Connection.execute`](Connection#method.execute).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // execute a statement without bind parameters
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (113, 'John')", &[])?;
    /// stmt.execute(&[])?;
    ///
    /// // execute a statement with binding parameters by position
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (:1, :2)", &[])?;
    /// stmt.execute(&[&114, &"Smith"])?;
    /// stmt.execute(&[&115, &"Paul"])?;  // execute with other values.
    ///
    /// # Ok::<(), Error>(())
    /// ```
    pub fn execute(&mut self, params: &[&dyn ToSql]) -> Result<()> {
        self.exec(params, false, "execute")
    }

    /// Binds values by name and executes the statement.
    /// It will retunrs `Err` when the statemnet is a select statement.
    ///
    /// See also [Connection.execute_named](Connection#method.execute_named).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // execute a statement with binding parameters by name
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (:id, :name)", &[])?;
    /// stmt.execute_named(&[("id", &114),
    ///                      ("name", &"Smith")])?;
    /// stmt.execute_named(&[("id", &115),
    ///                      ("name", &"Paul")])?; // execute with other values.
    /// # Ok::<(), Error>(())
    /// ```
    pub fn execute_named(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<()> {
        self.exec_named(params, false, "execute_named")
    }

    fn check_stmt_type(&self, must_be_query: bool, method_name: &str) -> Result<()> {
        if must_be_query {
            if self.statement_type == StatementType::Select {
                Ok(())
            } else {
                Err(Error::InvalidOperation(format!(
                    "Could not use the `{}` method for non-select statements",
                    method_name
                )))
            }
        } else {
            if self.statement_type != StatementType::Select {
                Ok(())
            } else {
                Err(Error::InvalidOperation(format!(
                    "Could not use the `{}` method for select statements",
                    method_name
                )))
            }
        }
    }

    pub(crate) fn exec(
        &mut self,
        params: &[&dyn ToSql],
        must_be_query: bool,
        method_name: &str,
    ) -> Result<()> {
        self.check_stmt_type(must_be_query, method_name)?;
        for i in 0..params.len() {
            self.bind(i + 1, params[i])?;
        }
        self.exec_common()
    }

    pub(crate) fn exec_named(
        &mut self,
        params: &[(&str, &dyn ToSql)],
        must_be_query: bool,
        method_name: &str,
    ) -> Result<()> {
        self.check_stmt_type(must_be_query, method_name)?;
        for i in 0..params.len() {
            self.bind(params[i].0, params[i].1)?;
        }
        self.exec_common()
    }

    fn exec_common(&mut self) -> Result<()> {
        let mut num_query_columns = 0;
        let mut exec_mode = DPI_MODE_EXEC_DEFAULT;
        if self.conn.autocommit {
            exec_mode |= DPI_MODE_EXEC_COMMIT_ON_SUCCESS;
        }
        chkerr!(
            self.conn.ctxt,
            dpiStmt_setFetchArraySize(self.handle, self.fetch_array_size)
        );
        chkerr!(
            self.conn.ctxt,
            dpiStmt_setPrefetchRows(self.handle, self.prefetch_rows)
        );
        chkerr!(
            self.conn.ctxt,
            dpiStmt_execute(self.handle, exec_mode, &mut num_query_columns)
        );
        if self.is_ddl() {
            let mut buf = MaybeUninit::uninit();
            let mut len = mem::size_of::<u16>() as u32;
            chkerr!(
                self.conn.ctxt,
                dpiStmt_getOciAttr(self.handle, OCI_ATTR_SQLFNCODE, buf.as_mut_ptr(), &mut len,)
            );
            let fncode = unsafe { buf.assume_init().asUint16 };
            match fncode {
                SQLFNCODE_CREATE_TYPE | SQLFNCODE_ALTER_TYPE | SQLFNCODE_DROP_TYPE => {
                    self.conn.clear_object_type_cache()?
                }
                _ => (),
            }
        }
        if self.statement_type == StatementType::Select {
            if self.row.is_none() {
                let num_cols = num_query_columns as usize;
                let mut column_names = Vec::with_capacity(num_cols);
                let mut column_values = Vec::with_capacity(num_cols);
                self.column_info = Vec::with_capacity(num_cols);

                for i in 0..num_cols {
                    // set column info
                    let ci = ColumnInfo::new(self, i)?;
                    column_names.push(ci.name.clone());
                    self.column_info.push(ci);
                    // setup column value
                    let mut val = SqlValue::new(self.conn.ctxt);
                    val.buffer_row_index =
                        BufferRowIndex::Shared(self.shared_buffer_row_index.clone());
                    let oratype = self.column_info[i].oracle_type();
                    let oratype_i64 = OracleType::Int64;
                    let oratype = match *oratype {
                        // When the column type is number whose prec is less than 18
                        // and the scale is zero, define it as int64.
                        OracleType::Number(prec, 0)
                            if 0 < prec && prec < DPI_MAX_INT64_PRECISION as u8 =>
                        {
                            &oratype_i64
                        }
                        _ => oratype,
                    };
                    val.init_handle(&self.conn.handle, oratype, self.fetch_array_size)?;
                    chkerr!(
                        self.conn.ctxt,
                        dpiStmt_define(self.handle, (i + 1) as u32, val.handle)
                    );
                    column_values.push(val);
                }
                self.row = Some(Row::new(self.conn, column_names, column_values)?);
            }
        }
        if self.is_returning {
            for val in self.bind_values.iter_mut() {
                val.fix_internal_data()?;
            }
        }
        Ok(())
    }

    /// Returns the number of bind variables in the statement.
    ///
    /// In SQL statements this is the total number of bind variables whereas in
    /// PL/SQL statements this is the count of the **unique** bind variables.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // SQL statements
    /// let stmt = conn.prepare("select :val1, :val2, :val1 from dual", &[])?;
    /// assert_eq!(stmt.bind_count(), 3); // val1, val2 and val1
    ///
    /// // PL/SQL statements
    /// let stmt = conn.prepare("begin :val1 := :val1 || :val2; end;", &[])?;
    /// assert_eq!(stmt.bind_count(), 2); // val1(twice) and val2
    /// # Ok::<(), Error>(())
    /// ```
    pub fn bind_count(&self) -> usize {
        self.bind_count
    }

    /// Returns the names of the unique bind variables in the statement.
    ///
    /// The bind variable names in statements are converted to upper-case.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// let stmt = conn.prepare("BEGIN :val1 := :val2 || :val1 || :aàáâãäå; END;", &[])?;
    /// assert_eq!(stmt.bind_count(), 3);
    /// let bind_names = stmt.bind_names();
    /// assert_eq!(bind_names.len(), 3);
    /// assert_eq!(bind_names[0], "VAL1");
    /// assert_eq!(bind_names[1], "VAL2");
    /// assert_eq!(bind_names[2], "AÀÁÂÃÄÅ");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn bind_names(&self) -> Vec<&str> {
        self.bind_names.iter().map(|name| name.as_str()).collect()
    }

    /// Set a bind value in the statement.
    ///
    /// The position starts from one when the bind index type is `usize`.
    /// The variable name is compared case-insensitively when the bind index
    /// type is `&str`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*; use oracle::sql_type::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    /// let mut stmt = conn.prepare("begin :outval := upper(:inval); end;", &[])?;
    ///
    /// // Sets NULL whose data type is VARCHAR2(60) to the first bind value.
    /// stmt.bind(1, &OracleType::Varchar2(60))?;
    ///
    /// // Sets "to be upper-case" to the second by its name.
    /// stmt.bind("inval", &"to be upper-case")?;
    ///
    /// stmt.execute(&[])?;
    /// let outval: String = stmt.bind_value(1)?;
    /// assert_eq!(outval, "TO BE UPPER-CASE");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn bind<I>(&mut self, bindidx: I, value: &dyn ToSql) -> Result<()>
    where
        I: BindIndex,
    {
        let pos = bindidx.idx(&self)?;
        if self.bind_values[pos].init_handle(&self.conn.handle, &value.oratype(self.conn)?, 1)? {
            chkerr!(
                self.conn.ctxt,
                bindidx.bind(self.handle, self.bind_values[pos].handle)
            );
        }
        self.bind_values[pos].set(value)
    }

    /// Gets a bind value in the statement.
    ///
    /// The position starts from one when the bind index type is `usize`.
    /// The variable name is compared case-insensitively when the bind index
    /// type is `&str`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*; use oracle::sql_type::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // Prepares "begin :outval := upper(:inval); end;",
    /// // sets NULL whose data type is VARCHAR2(60) to the first bind variable,
    /// // sets "to be upper-case" to the second and then executes it.
    /// let mut stmt = conn.prepare("begin :outval := upper(:inval); end;", &[])?;
    /// stmt.execute(&[&OracleType::Varchar2(60),
    ///              &"to be upper-case"])?;
    ///
    /// // Get the first bind value by position.
    /// let outval: String = stmt.bind_value(1)?;
    /// assert_eq!(outval, "TO BE UPPER-CASE");
    ///
    /// // Get the first bind value by name.
    /// let outval: String = stmt.bind_value("outval")?;
    /// assert_eq!(outval, "TO BE UPPER-CASE");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn bind_value<I, T>(&self, bindidx: I) -> Result<T>
    where
        I: BindIndex,
        T: FromSql,
    {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].get()
    }

    /// Gets values returned by RETURNING INTO clause.
    ///
    /// When the `bindidx` ponints to a bind variable out of RETURNING INTO clause,
    /// the behavior is undefined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*; use oracle::sql_type::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // create a table using identity column (Oracle 12c feature).
    /// conn.execute("create table people (id number generated as identity, name varchar2(30))", &[])?;
    ///
    /// // insert one person and return the generated id into :id.
    /// let stmt = conn.execute("insert into people(name) values ('Asimov') returning id into :id", &[&None::<i32>])?;
    /// let inserted_id: i32 = stmt.returned_values("id")?[0];
    /// println!("Asimov's ID is {}", inserted_id);
    ///
    /// // insert another person and return the generated id into :id.
    /// let stmt = conn.execute("insert into people(name) values ('Clark') returning id into :id", &[&None::<i32>])?;
    /// let inserted_id: i32 = stmt.returned_values("id")?[0];
    /// println!("Clark's ID is {}", inserted_id);
    ///
    /// // delete all people and return deleted names into :name.
    /// let stmt = conn.execute("delete from people returning name into :name", &[&OracleType::Varchar2(30)])?;
    /// let deleted_names: Vec<String> = stmt.returned_values("name")?;
    /// for name in deleted_names {
    ///     println!("{} is deleted.", name);
    /// }
    ///
    /// // cleanup
    /// conn.execute("drop table people purge", &[])?;
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// [Statement.bind_value()]: #method.bind_value
    pub fn returned_values<I, T>(&self, bindidx: I) -> Result<Vec<T>>
    where
        I: BindIndex,
        T: FromSql,
    {
        let mut rows = 0;
        chkerr!(self.conn.ctxt, dpiStmt_getRowCount(self.handle, &mut rows));
        if rows == 0 {
            return Ok(vec![]);
        }
        let mut sqlval = self.bind_values[bindidx.idx(&self)?].unsafely_clone();
        if rows > sqlval.array_size as u64 {
            rows = sqlval.array_size as u64;
        }
        let mut vec = Vec::with_capacity(rows as usize);
        for i in 0..rows {
            sqlval.buffer_row_index = BufferRowIndex::Owned(i as u32);
            vec.push(sqlval.get()?);
        }
        Ok(vec)
    }

    pub(crate) fn next(&self) -> Option<Result<&Row>> {
        let mut found = 0;
        let mut buffer_row_index = 0;
        if unsafe { dpiStmt_fetch(self.handle, &mut found, &mut buffer_row_index) } == 0 {
            if found != 0 {
                *self.shared_buffer_row_index.borrow_mut() = buffer_row_index;
                // if self.row.is_none(), dpiStmt_fetch() returns non-zero.
                Some(Ok(self.row.as_ref().unwrap()))
            } else {
                None
            }
        } else {
            Some(Err(crate::error::error_from_context(self.conn.ctxt)))
        }
    }

    /// Returns the number of rows fetched when the SQL statement is a query.
    /// Otherwise, the number of rows affected.
    pub fn row_count(&self) -> Result<u64> {
        let mut count = 0;
        chkerr!(self.conn.ctxt, dpiStmt_getRowCount(self.handle, &mut count));
        Ok(count)
    }

    /// Returns statement type
    pub fn statement_type(&self) -> StatementType {
        self.statement_type
    }

    /// Returns true when the SQL statement is a query.
    pub fn is_query(&self) -> bool {
        self.statement_type == StatementType::Select
    }

    /// Returns true when the SQL statement is a PL/SQL block.
    pub fn is_plsql(&self) -> bool {
        match self.statement_type {
            StatementType::Begin | StatementType::Declare | StatementType::Call => true,
            _ => false,
        }
    }

    /// Returns true when the SQL statement is DDL (data definition language).
    pub fn is_ddl(&self) -> bool {
        match self.statement_type {
            StatementType::Create | StatementType::Drop | StatementType::Alter => true,
            _ => false,
        }
    }

    /// Returns true when the SQL statement is DML (data manipulation language).
    pub fn is_dml(&self) -> bool {
        match self.statement_type {
            StatementType::Insert
            | StatementType::Update
            | StatementType::Delete
            | StatementType::Merge => true,
            _ => false,
        }
    }

    /// Returns true when the SQL statement has a `RETURNING INTO` clause.
    pub fn is_returning(&self) -> bool {
        self.is_returning
    }
}

impl<'conn> Drop for Statement<'conn> {
    fn drop(&mut self) {
        unsafe { dpiStmt_release(self.handle) };
    }
}

impl<'conn> fmt::Debug for Statement<'conn> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Statement {{ handle: {:?}, conn: {:?}, stmt_type: {}",
            self.handle,
            self.conn,
            self.statement_type()
        )?;
        if self.column_info.len() != 0 {
            write!(f, ", colum_info: [")?;
            for (idx, colinfo) in (&self.column_info).iter().enumerate() {
                if idx != 0 {
                    write!(f, ", ")?;
                }
                write!(
                    f,
                    "{} {}{}",
                    colinfo.name(),
                    colinfo.oracle_type(),
                    if colinfo.nullable() { "" } else { " NOT NULL" }
                )?;
            }
            write!(f, "], fetch_array_size: {}", self.fetch_array_size)?;
        }
        if self.bind_count != 0 {
            write!(
                f,
                ", bind_count: {}, bind_names: {:?}, bind_values: {:?}",
                self.bind_count, self.bind_names, self.bind_values
            )?;
        }
        if self.is_returning {
            write!(f, ", is_returning: true")?;
        }
        write!(f, " }}")
    }
}

/// Column information in a select statement
///
/// # Examples
///
/// Print column information of `emp` table.
///
/// ```no_run
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
/// let mut stmt = conn.prepare("select * from emp", &[])?;
/// let rows = stmt.query(&[])?;
/// println!(" {:-30} {:-8} {}", "Name", "Null?", "Type");
/// println!(" {:-30} {:-8} {}", "------------------------------", "--------", "----------------------------");
/// for info in rows.column_info() {
///    println!("{:-30} {:-8} {}",
///             info.name(),
///             if info.nullable() {""} else {"NOT NULL"},
///             info.oracle_type());
/// }
/// # Ok::<(), Error>(())
/// ```
///
/// The output is:
///
/// ```text
///  Name                           Null?    Type
///  ------------------------------ -------- ----------------------------
///  EMPNO                          NOT NULL NUMBER(4)
///  ENAME                                   VARCHAR2(10)
///  JOB                                     VARCHAR2(9)
///  MGR                                     NUMBER(4)
///  HIREDATE                                DATE
///  SAL                                     NUMBER(7,2)
///  COMM                                    NUMBER(7,2)
///  DEPTNO                                  NUMBER(2)
/// ```
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    name: String,
    oracle_type: OracleType,
    nullable: bool,
}

impl ColumnInfo {
    fn new(stmt: &Statement, idx: usize) -> Result<ColumnInfo> {
        let mut info = MaybeUninit::uninit();
        chkerr!(
            stmt.conn.ctxt,
            dpiStmt_getQueryInfo(stmt.handle, (idx + 1) as u32, info.as_mut_ptr())
        );
        let info = unsafe { info.assume_init() };
        Ok(ColumnInfo {
            name: to_rust_str(info.name, info.nameLength),
            oracle_type: OracleType::from_type_info(stmt.conn.ctxt, &info.typeInfo)?,
            nullable: info.nullOk != 0,
        })
    }

    /// Gets column name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets Oracle type
    pub fn oracle_type(&self) -> &OracleType {
        &self.oracle_type
    }

    /// Gets whether the column may be NULL.
    /// False when the column is defined as `NOT NULL`.
    pub fn nullable(&self) -> bool {
        self.nullable
    }
}

impl fmt::Display for ColumnInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.nullable {
            write!(f, "{} {}", self.name, self.oracle_type)
        } else {
            write!(f, "{} {} NOT NULL", self.name, self.oracle_type)
        }
    }
}

/// A trait implemented by types that can index into bind values of a statement.
///
/// This trait is sealed and cannot be implemented for types outside of the `oracle` crate.
pub trait BindIndex: private::Sealed {
    /// Returns the index of the bind value specified by `self`.
    #[doc(hidden)]
    fn idx(&self, stmt: &Statement) -> Result<usize>;
    /// Binds the specified value by using a private method.
    #[doc(hidden)]
    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32;
}

impl BindIndex for usize {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        let num = stmt.bind_count();
        if 0 < num && *self <= num {
            Ok(*self - 1)
        } else {
            Err(Error::InvalidBindIndex(*self))
        }
    }

    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        dpiStmt_bindByPos(stmt_handle, *self as u32, var_handle)
    }
}

impl<'a> BindIndex for &'a str {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        let bindname = self.to_uppercase();
        stmt.bind_names()
            .iter()
            .position(|&name| name == bindname)
            .ok_or_else(|| Error::InvalidBindName((*self).to_string()))
    }

    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        let s = to_odpi_str(*self);
        dpiStmt_bindByName(stmt_handle, s.ptr, s.len, var_handle)
    }
}

/// A trait implemented by types that can index into columns of a row.
///
/// This trait is sealed and cannot be implemented for types outside of the `oracle` crate.
pub trait ColumnIndex: private::Sealed {
    /// Returns the index of the column specified by `self`.
    #[doc(hidden)]
    fn idx(&self, column_names: &Vec<String>) -> Result<usize>;
}

impl ColumnIndex for usize {
    fn idx(&self, column_names: &Vec<String>) -> Result<usize> {
        let ncols = column_names.len();
        if *self < ncols {
            Ok(*self)
        } else {
            Err(Error::InvalidColumnIndex(*self))
        }
    }
}

impl<'a> ColumnIndex for &'a str {
    fn idx(&self, column_names: &Vec<String>) -> Result<usize> {
        for (idx, colname) in column_names.iter().enumerate() {
            if colname.as_str().eq_ignore_ascii_case(*self) {
                return Ok(idx);
            }
        }
        Err(Error::InvalidColumnName((*self).to_string()))
    }
}
