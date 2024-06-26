// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2022 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use std::borrow::ToOwned;
use std::fmt;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::binding::*;
use crate::chkerr;
use crate::connection::Conn;
use crate::oci_attr::data_type::{AttrValue, DataType};
use crate::oci_attr::mode::{ReadMode, WriteMode};
use crate::oci_attr::{self, OciAttr, SqlFnCode};
use crate::private;
use crate::sql_type::FromSql;
use crate::sql_type::OracleType;
use crate::sql_type::RefCursor;
use crate::sql_type::ToSql;
#[cfg(doc)]
use crate::sql_type::{Blob, Clob, Nclob};
use crate::sql_value::BufferRowIndex;
use crate::to_rust_str;
use crate::AssertSend;
use crate::Connection;
use crate::Context;
use crate::DpiStmt;
use crate::Error;
use crate::OdpiStr;
use crate::Result;
use crate::ResultSet;
use crate::Row;
use crate::RowValue;
use crate::SqlValue;
#[cfg(doc)]
use std::io::Read;

// https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-A251CF91-EB9F-4DBC-8BB8-FB5EA92C20DE
const SQLFNCODE_CREATE_TYPE: u16 = 77;
const SQLFNCODE_ALTER_TYPE: u16 = 80;
const SQLFNCODE_DROP_TYPE: u16 = 78;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LobBindType {
    Locator,
    Bytes,
}

#[derive(Clone, Debug)]
pub struct QueryParams {
    pub fetch_array_size: u32,
    pub prefetch_rows: Option<u32>,
    pub lob_bind_type: LobBindType,
}

impl QueryParams {
    pub fn new() -> QueryParams {
        QueryParams {
            fetch_array_size: DPI_DEFAULT_FETCH_ARRAY_SIZE,
            prefetch_rows: None,
            lob_bind_type: LobBindType::Bytes,
        }
    }
}

/// A builder to create a [`Statement`][] with various configuration
pub struct StatementBuilder<'conn, 'sql> {
    conn: &'conn Connection,
    sql: &'sql str,
    query_params: QueryParams,
    scrollable: bool,
    tag: String,
    exclude_from_cache: bool,
}

impl<'conn, 'sql> StatementBuilder<'conn, 'sql> {
    pub(crate) fn new(conn: &'conn Connection, sql: &'sql str) -> StatementBuilder<'conn, 'sql> {
        StatementBuilder {
            conn,
            sql,
            query_params: QueryParams::new(),
            scrollable: false,
            tag: "".into(),
            exclude_from_cache: false,
        }
    }

    /// Changes the array size used for performing fetches.
    ///
    /// This specifies the number of rows allocated before performing
    /// fetches. The default value is 100. Higher value reduces
    /// the number of network round trips to fetch rows but requires
    /// more memory. The preferable value depends on the query and
    /// the environment.
    ///
    /// If the query returns only onw row, it is better to change
    /// size to one.
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// let mut stmt = conn
    ///     .statement("select StringCol from TestStrings where IntCol = :1")
    ///     .fetch_array_size(1)
    ///     .build()?;
    /// assert_eq!(stmt.query_row_as::<String>(&[&1])?, "String 1");
    /// assert_eq!(stmt.query_row_as::<String>(&[&2])?, "String 2");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn fetch_array_size(&mut self, size: u32) -> &mut StatementBuilder<'conn, 'sql> {
        self.query_params.fetch_array_size = size;
        self
    }

    /// The number of rows that will be prefetched by the Oracle Client
    /// library when a query is executed. The default value is
    /// DPI_DEFAULT_PREFETCH_ROWS (2). Increasing this value may reduce
    /// the number of round-trips to the database that are required in
    /// order to fetch rows, but at the cost of increasing memory
    /// requirements.
    /// Setting this value to 0 will disable prefetch completely,
    /// which may be useful when the timing for fetching rows must be
    /// controlled by the caller.
    pub fn prefetch_rows(&mut self, size: u32) -> &mut StatementBuilder<'conn, 'sql> {
        self.query_params.prefetch_rows = Some(size);
        self
    }

    /// Enables lob data types to be fetched or bound as [`Clob`], [`Nclob`] or [`Blob`].
    ///
    /// Lob data types are internally bound as string or bytes by default.
    /// It is proper for small data but not for big data. That's because
    /// when a lob contains 1 gigabyte data, the whole data are copied to the client
    /// and consume 1 gigabyte or more memory. When `lob_locator` is set and
    /// a column is fetched as [`Clob`], data are copied using [`Read::read`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Connection;
    /// # use oracle::Error;
    /// # use oracle::sql_type::Clob;
    /// # use oracle::test_util;
    /// # use std::io::{Read, Write};
    /// # let conn = test_util::connect()?;
    /// # conn.execute("delete from TestClobs", &[])?;
    /// # conn.execute("insert into TestClobs values (:1, :2)", &[&1i32, &"clob data"])?;
    /// # let mut out = vec![0u8; 0];
    /// let mut stmt = conn
    ///     .statement("select ClobCol from TestClobs where IntCol = :1")
    ///     .lob_locator()
    ///     .build()?;
    /// let mut clob = stmt.query_row_as::<Clob>(&[&1i32])?;
    ///
    /// // Copy contents of clob using 1MB buffer.
    /// let mut buf = vec![0u8; 1 * 1024 * 1024];
    /// loop {
    ///   let size = clob.read(&mut buf)?;
    ///   if size == 0 {
    ///     break;
    ///   }
    ///   out.write(&buf[0..size]);
    /// }
    /// # Ok::<(), Box::<dyn std::error::Error>>(())
    /// ```
    pub fn lob_locator(&mut self) -> &mut StatementBuilder<'conn, 'sql> {
        self.query_params.lob_bind_type = LobBindType::Locator;
        self
    }

    /// Specifies the key to be used for searching for the statement in the statement cache.
    /// If the key is not found, the SQL text specified by [`Connection::statement`] is used
    /// to create a statement.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    ///
    /// // When both SQL text and a tag are specifed and the tag is not found
    /// // in the statement cache, the SQL text is used to make a statement.
    /// // The statement is backed to the cache with the tag when
    /// // it is closed.
    /// let mut stmt = conn.statement("select 1 from dual").tag("query one").build()?;
    /// assert_eq!(stmt.query_row_as::<i32>(&[])?, 1);
    /// stmt.close()?;
    ///
    /// // When only a tag is specified and the tag is found in the cache,
    /// // the statement with the tag is returned.
    /// let mut stmt = conn.statement("").tag("query one").build()?;
    /// assert_eq!(stmt.query_row_as::<i32>(&[])?, 1);
    /// stmt.close()?;
    ///
    /// // When only a tag is specified and the tag isn't found in the cache,
    /// // ORA-24431 is returned.
    /// let err = conn.statement("").tag("not existing tag").build().unwrap_err();
    /// match err {
    ///   Error::OciError(err) if err.code() == 24431 => {
    ///     // ORA-24431: Statement does not exist in the cache
    ///   },
    ///   _ => panic!("unexpected err {:?}", err),
    /// }
    ///
    /// // WARNING: The SQL statement is not checked when the tag is found.
    /// let mut stmt = conn.statement("select 2 from dual").tag("query one").build()?;
    /// // The result must be 2 if the SQL text is used. However it is 1
    /// // because the statement tagged with "query one" is "select 1 from dual".
    /// assert_eq!(stmt.query_row_as::<i32>(&[])?, 1);
    /// stmt.close()?;
    ///
    /// # // test whether the statement is tagged when it is closed by drop.
    /// # {
    /// #    let mut stmt = conn.statement("select 2 from dual").tag("query two").build()?;
    /// #    assert_eq!(stmt.query_row_as::<i32>(&[])?, 2);
    /// #    // stmt is dropped here.
    /// # }
    /// # let mut stmt = conn.statement("").tag("query two").build()?;
    /// # assert_eq!(stmt.query_row_as::<i32>(&[])?, 2);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn tag<T>(&mut self, tag_name: T) -> &mut StatementBuilder<'conn, 'sql>
    where
        T: Into<String>,
    {
        self.tag = tag_name.into();
        self
    }

    /// Excludes the statement from the cache even when stmt_cache_size is not zero.
    pub fn exclude_from_cache(&mut self) -> &mut StatementBuilder<'conn, 'sql> {
        self.exclude_from_cache = true;
        self
    }

    pub fn build(&self) -> Result<Statement> {
        Statement::new(self)
    }
}

/// Statement type returned by [`Statement::statement_type`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
            StatementType::Select => write!(f, "select"),
            StatementType::Insert => write!(f, "insert"),
            StatementType::Update => write!(f, "update"),
            StatementType::Delete => write!(f, "delete"),
            StatementType::Merge => write!(f, "merge"),
            StatementType::Create => write!(f, "create"),
            StatementType::Alter => write!(f, "alter"),
            StatementType::Drop => write!(f, "drop"),
            StatementType::Begin => write!(f, "PL/SQL(begin)"),
            StatementType::Declare => write!(f, "PL/SQL(declare)"),
            StatementType::Commit => write!(f, "commit"),
            StatementType::Rollback => write!(f, "rollback"),
            StatementType::ExplainPlan => write!(f, "explain plan"),
            StatementType::Call => write!(f, "call"),
            StatementType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Stmt {
    pub(crate) conn: Conn,
    pub(crate) handle: DpiStmt,
    pub(crate) row: Option<Row>,
    shared_buffer_row_index: Arc<AtomicU32>,
    last_buffer_row_index: u32,
    more_rows: bool,
    pub(crate) query_params: QueryParams,
    tag: String,
}

impl Stmt {
    pub(crate) fn new(conn: Conn, handle: DpiStmt, query_params: QueryParams, tag: String) -> Stmt {
        Stmt {
            conn,
            handle,
            row: None,
            shared_buffer_row_index: Arc::new(AtomicU32::new(0)),
            last_buffer_row_index: 0,
            more_rows: false,
            query_params,
            tag,
        }
    }

    pub(crate) fn ctxt(&self) -> &Context {
        self.conn.ctxt()
    }

    pub(crate) fn conn(&self) -> &Conn {
        &self.conn
    }

    pub(crate) fn handle(&self) -> *mut dpiStmt {
        self.handle.raw
    }

    fn close(&mut self) -> Result<()> {
        let tag = OdpiStr::new(&self.tag);
        chkerr!(self.ctxt(), dpiStmt_close(self.handle(), tag.ptr, tag.len));
        Ok(())
    }

    pub(crate) fn init_row(&mut self, num_cols: usize) -> Result<()> {
        self.shared_buffer_row_index.store(0, Ordering::Relaxed);
        self.last_buffer_row_index = 0;
        self.more_rows = true;
        if self.row.is_some() {
            return Ok(());
        }
        let mut column_info = Vec::with_capacity(num_cols);
        let mut column_values = Vec::with_capacity(num_cols);

        for i in 0..num_cols {
            let info = ColumnInfo::new(self, i)?;
            let val = SqlValue::for_column(
                self.conn.clone(),
                self.query_params.clone(),
                self.shared_buffer_row_index.clone(),
                info.oracle_type(),
                self.handle(),
                (i + 1) as u32,
            )?;
            column_info.push(info);
            column_values.push(val);
        }
        self.row = Some(Row::new(column_info, column_values)?);
        Ok(())
    }

    fn try_next(&mut self) -> Result<Option<&Row>> {
        let index = self.shared_buffer_row_index.load(Ordering::Relaxed);
        let last_index = self.last_buffer_row_index;
        if index + 1 < last_index {
            self.shared_buffer_row_index
                .store(index + 1, Ordering::Relaxed);
            Ok(Some(self.row.as_ref().unwrap()))
        } else if self.more_rows && self.fetch_rows()? {
            Ok(Some(self.row.as_ref().unwrap()))
        } else {
            Ok(None)
        }
    }

    pub fn next(&mut self) -> Option<Result<&Row>> {
        self.try_next().transpose()
    }

    pub fn fetch_rows(&mut self) -> Result<bool> {
        let handle = self.handle();
        let row = self.row.as_mut().unwrap();
        for i in 0..(row.column_info.len()) {
            // If fetch array buffer is referenced only by self, it is reusable.
            // Otherwise, a new SqlValue must be created to allocate a new buffer
            // because dpiStmt_fetchRows() overwrites the buffer.
            if row.column_values[i].fetch_array_buffer_shared_count()? > 1 {
                let oratype = row.column_info[i].oracle_type();
                row.column_values[i] = SqlValue::for_column(
                    self.conn.clone(),
                    self.query_params.clone(),
                    self.shared_buffer_row_index.clone(),
                    oratype,
                    handle,
                    (i + 1) as u32,
                )?;
            }
        }
        let mut new_index = 0;
        let mut num_rows = 0;
        let mut more_rows = 0;
        chkerr!(
            self.ctxt(),
            dpiStmt_fetchRows(
                handle,
                self.query_params.fetch_array_size,
                &mut new_index,
                &mut num_rows,
                &mut more_rows
            )
        );
        self.shared_buffer_row_index
            .store(new_index, Ordering::Relaxed);
        self.last_buffer_row_index = new_index + num_rows;
        self.more_rows = more_rows != 0;
        Ok(num_rows != 0)
    }

    pub fn row_count(&self) -> Result<u64> {
        let mut count = 0;
        chkerr!(self.ctxt(), dpiStmt_getRowCount(self.handle(), &mut count));
        Ok(count)
    }
}

impl AssertSend for Stmt {}

impl Drop for Stmt {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Statement
#[derive(Debug)]
pub struct Statement {
    pub(crate) stmt: Stmt,
    statement_type: StatementType,
    is_returning: bool,
    bind_count: usize,
    bind_names: Vec<String>,
    bind_values: Vec<SqlValue<'static>>,
}

impl Statement {
    fn new(builder: &StatementBuilder<'_, '_>) -> Result<Statement> {
        let conn = builder.conn;
        let sql = OdpiStr::new(builder.sql);
        let tag = OdpiStr::new(&builder.tag);
        let mut handle = DpiStmt::null();
        chkerr!(
            conn.ctxt(),
            dpiConn_prepareStmt(
                conn.handle(),
                i32::from(builder.scrollable),
                sql.ptr,
                sql.len,
                tag.ptr,
                tag.len,
                &mut handle.raw
            )
        );
        let mut info = MaybeUninit::uninit();
        chkerr!(conn.ctxt(), dpiStmt_getInfo(handle.raw, info.as_mut_ptr()));
        let info = unsafe { info.assume_init() };
        let mut num = 0;
        chkerr!(conn.ctxt(), dpiStmt_getBindCount(handle.raw, &mut num));
        let bind_count = num as usize;
        let mut bind_names = Vec::with_capacity(bind_count);
        let mut bind_values = Vec::with_capacity(bind_count);
        if bind_count > 0 {
            let mut names: Vec<*const c_char> = vec![ptr::null_mut(); bind_count];
            let mut lengths = vec![0; bind_count];
            chkerr!(
                conn.ctxt(),
                dpiStmt_getBindNames(
                    handle.raw,
                    &mut num,
                    names.as_mut_ptr(),
                    lengths.as_mut_ptr()
                )
            );
            bind_names = Vec::with_capacity(num as usize);
            for i in 0..(num as usize) {
                bind_names.push(to_rust_str(names[i], lengths[i]));
                bind_values.push(SqlValue::for_bind(
                    conn.conn.clone(),
                    builder.query_params.clone(),
                    1,
                ));
            }
        };
        let tag = if builder.exclude_from_cache {
            chkerr!(conn.ctxt(), dpiStmt_deleteFromCache(handle.raw));
            String::new()
        } else {
            builder.tag.clone()
        };
        Ok(Statement {
            stmt: Stmt::new(conn.conn.clone(), handle, builder.query_params.clone(), tag),
            statement_type: StatementType::from_enum(info.statementType),
            is_returning: info.isReturning != 0,
            bind_count,
            bind_names,
            bind_values,
        })
    }

    /// Closes the statement before the end of lifetime.
    pub fn close(&mut self) -> Result<()> {
        self.stmt.close()
    }

    pub(crate) fn ctxt(&self) -> &Context {
        self.conn().ctxt()
    }

    pub(crate) fn conn(&self) -> &Conn {
        &self.stmt.conn
    }

    pub(crate) fn handle(&self) -> *mut dpiStmt {
        self.stmt.handle.raw
    }

    /// Executes the prepared statement and returns a result set containing [`Row`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query(&mut self, params: &[&dyn ToSql]) -> Result<ResultSet<Row>> {
        self.exec(params, true, "query")?;
        Ok(ResultSet::<Row>::new(&mut self.stmt))
    }

    /// Executes the prepared statement using named parameters and returns a result set containing [`Row`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_named(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<ResultSet<Row>> {
        self.exec_named(params, true, "query_named")?;
        Ok(ResultSet::<Row>::new(&mut self.stmt))
    }

    /// Executes the prepared statement and returns a result set containing [`RowValue`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_as<T>(&mut self, params: &[&dyn ToSql]) -> Result<ResultSet<T>>
    where
        T: RowValue,
    {
        self.exec(params, true, "query_as")?;
        Ok(ResultSet::new(&mut self.stmt))
    }

    /// Executes the prepared statement and returns a result set containing [`RowValue`]s.
    ///
    /// This is the same as [`Statement::query_as()`], but takes ownership of the [`Statement`].
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn into_result_set<T>(mut self, params: &[&dyn ToSql]) -> Result<ResultSet<'static, T>>
    where
        T: RowValue,
    {
        self.exec(params, true, "into_result_set")?;
        Ok(ResultSet::from_stmt(self.stmt))
    }

    /// Executes the prepared statement using named parameters and returns a result set containing [`RowValue`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_as_named<T>(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<ResultSet<T>>
    where
        T: RowValue,
    {
        self.exec_named(params, true, "query_as_named")?;
        Ok(ResultSet::new(&mut self.stmt))
    }

    /// Executes the prepared statement using named parameters and returns a result set containing [`RowValue`]s.
    ///
    /// This is the same as [`Statement::query_as_named()`], but takes ownership of the [`Statement`].
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn into_result_set_named<T>(
        mut self,
        params: &[(&str, &dyn ToSql)],
    ) -> Result<ResultSet<'static, T>>
    where
        T: RowValue,
    {
        self.exec_named(params, true, "into_result_set_named")?;
        Ok(ResultSet::from_stmt(self.stmt))
    }

    /// Gets one row from the prepared statement using positoinal bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row(&mut self, params: &[&dyn ToSql]) -> Result<Row> {
        let mut rows = self.query(params)?;
        rows.next().unwrap_or(Err(Error::no_data_found()))
    }

    /// Gets one row from the prepared statement using named bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row_named(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<Row> {
        let mut rows = self.query_named(params)?;
        rows.next().unwrap_or(Err(Error::no_data_found()))
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
        rows.next().unwrap_or(Err(Error::no_data_found()))
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
        rows.next().unwrap_or(Err(Error::no_data_found()))
    }

    /// Binds values by position and executes the statement.
    /// It will retunrs `Err` when the statemnet is a select statement.
    ///
    /// See also [`Connection::execute`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // execute a statement without bind parameters
    /// let mut stmt = conn
    ///     .statement("insert into emp(empno, ename) values (113, 'John')")
    ///     .build()?;
    /// stmt.execute(&[])?;
    ///
    /// // execute a statement with binding parameters by position
    /// let mut stmt = conn
    ///     .statement("insert into emp(empno, ename) values (:1, :2)")
    ///     .build()?;
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
    /// See also [`Connection::execute_named`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // execute a statement with binding parameters by name
    /// let mut stmt = conn
    ///     .statement("insert into emp(empno, ename) values (:id, :name)")
    ///     .build()?;
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
                Err(Error::invalid_operation(format!(
                    "could not use the `{}` method for non-select statements",
                    method_name
                )))
            }
        } else if self.statement_type != StatementType::Select {
            Ok(())
        } else {
            Err(Error::invalid_operation(format!(
                "could not use the `{}` method for select statements",
                method_name
            )))
        }
    }

    pub(crate) fn exec(
        &mut self,
        params: &[&dyn ToSql],
        must_be_query: bool,
        method_name: &str,
    ) -> Result<()> {
        self.check_stmt_type(must_be_query, method_name)?;
        for (i, param) in params.iter().enumerate() {
            self.bind(i + 1, *param)?;
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
        for param in params {
            self.bind(param.0, param.1)?;
        }
        self.exec_common()
    }

    fn exec_common(&mut self) -> Result<()> {
        let mut num_query_columns = 0;
        let mut exec_mode = DPI_MODE_EXEC_DEFAULT;
        if self.conn().autocommit() {
            exec_mode |= DPI_MODE_EXEC_COMMIT_ON_SUCCESS;
        }
        chkerr!(
            self.ctxt(),
            dpiStmt_setFetchArraySize(self.handle(), self.stmt.query_params.fetch_array_size)
        );
        if let Some(prefetch_rows) = self.stmt.query_params.prefetch_rows {
            chkerr!(
                self.ctxt(),
                dpiStmt_setPrefetchRows(self.handle(), prefetch_rows)
            );
        }
        chkerr!(
            self.ctxt(),
            dpiStmt_execute(self.handle(), exec_mode, &mut num_query_columns)
        );
        self.ctxt().set_warning();
        if self.is_ddl() {
            let fncode = self.oci_attr::<SqlFnCode>()?;
            match fncode {
                SQLFNCODE_CREATE_TYPE | SQLFNCODE_ALTER_TYPE | SQLFNCODE_DROP_TYPE => {
                    self.conn().clear_object_type_cache()?
                }
                _ => (),
            }
        }
        if self.statement_type == StatementType::Select {
            self.stmt.init_row(num_query_columns as usize)?;
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
    /// let stmt = conn.statement("select :val1, :val2, :val1 from dual").build()?;
    /// assert_eq!(stmt.bind_count(), 3); // val1, val2 and val1
    ///
    /// // PL/SQL statements
    /// let stmt = conn.statement("begin :val1 := :val1 || :val2; end;").build()?;
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
    /// let stmt = conn.statement("BEGIN :val1 := :val2 || :val1 || :aàáâãäå; END;").build()?;
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
    /// let mut stmt = conn.statement("begin :outval := upper(:inval); end;").build()?;
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
        let pos = bindidx.idx(self)?;
        let conn = Connection::from_conn(self.conn().clone());
        if self.bind_values[pos].init_handle(&value.oratype(&conn)?)? {
            chkerr!(
                self.ctxt(),
                bindidx.bind(self.handle(), self.bind_values[pos].handle()?)
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
    /// let mut stmt = conn.statement("begin :outval := upper(:inval); end;").build()?;
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
        let pos = bindidx.idx(self)?;
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
    pub fn returned_values<I, T>(&self, bindidx: I) -> Result<Vec<T>>
    where
        I: BindIndex,
        T: FromSql,
    {
        let mut rows = 0;
        chkerr!(self.ctxt(), dpiStmt_getRowCount(self.handle(), &mut rows));
        if rows == 0 {
            return Ok(vec![]);
        }
        let mut sqlval = self.bind_values[bindidx.idx(self)?].clone_with_narrow_lifetime()?;
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

    /// Returns the number of rows fetched when the SQL statement is a query.
    /// Otherwise, the number of rows affected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util::{self, check_version, VER12_1};
    /// # let conn = test_util::connect()?;
    /// // number of affected rows
    /// let stmt = conn.execute("update TestStrings set StringCol = StringCol where IntCol >= :1", &[&6])?;
    /// assert_eq!(stmt.row_count()?, 5);
    ///
    /// // number of fetched rows
    /// let mut stmt = conn
    ///     .statement("select * from TestStrings where IntCol >= :1")
    ///     .build()?;
    /// assert_eq!(stmt.row_count()?, 0); // before fetch
    /// let mut nrows = 0;
    /// for _ in stmt.query(&[&6])? {
    ///   nrows += 1;
    /// }
    /// assert_eq!(stmt.row_count()?, nrows); // after fetch
    ///
    /// // fetch again using same stmt with a different bind value.
    /// let mut nrows = 0;
    /// for _ in stmt.query(&[&4])? {
    ///   nrows += 1;
    /// }
    /// assert_eq!(stmt.row_count()?, nrows); // after fetch
    /// # Ok::<(), Error>(())
    /// ```
    pub fn row_count(&self) -> Result<u64> {
        self.stmt.row_count()
    }

    /// Returns the next implicit result returned by [`dbms_sql.return_result()`]
    /// in a PL/SQL block or a stored procedure.
    ///
    /// This feature is available when both the client and server are 12.1 or higher.
    ///
    /// [`dbms_sql.return_result()`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-87562BF3-682C-48A7-B0C1-61075F19382A
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util::{self, check_version, VER12_1};
    /// # let conn = test_util::connect()?;
    /// # if !check_version(&conn, &VER12_1, &VER12_1)? {
    /// #     return Ok(()); // skip this test
    /// # }
    ///
    /// let sql = r#"
    /// declare
    ///   cursor1 SYS_REFCURSOR;
    ///   cursor2 SYS_REFCURSOR;
    /// begin
    ///   open cursor1 for select StringCol from TestStrings where IntCol = :1;
    ///   -- return the first result set
    ///   dbms_sql.return_result(cursor1);
    ///
    ///   open cursor2 for select StringCol from TestStrings where IntCol = :2;
    ///   -- return the second result set
    ///   dbms_sql.return_result(cursor2);
    /// end;
    /// "#;
    ///
    /// let mut stmt = conn.statement(sql).build()?;
    /// stmt.execute(&[&1, &2])?;
    ///
    /// // Get the first result set.
    /// let mut opt_cursor = stmt.implicit_result()?;
    /// assert!(opt_cursor.is_some());
    /// let mut cursor = opt_cursor.unwrap();
    /// assert_eq!(cursor.query_row_as::<String>()?, "String 1");
    ///
    /// // Get the second result set.
    /// let mut opt_cursor = stmt.implicit_result()?;
    /// assert!(opt_cursor.is_some());
    /// let mut cursor = opt_cursor.unwrap();
    /// assert_eq!(cursor.query_row_as::<String>()?, "String 2");
    ///
    /// // No more result sets
    /// let mut opt_cursor = stmt.implicit_result()?;
    /// assert!(opt_cursor.is_none());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn implicit_result(&self) -> Result<Option<RefCursor>> {
        let mut handle = DpiStmt::null();
        chkerr!(
            self.ctxt(),
            dpiStmt_getImplicitResult(self.handle(), &mut handle.raw)
        );
        if handle.is_null() {
            Ok(None)
        } else {
            let cursor = RefCursor::from_handle(
                self.stmt.conn.clone(),
                handle,
                self.stmt.query_params.clone(),
            )?;
            Ok(Some(cursor))
        }
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
        matches!(
            self.statement_type,
            StatementType::Begin | StatementType::Declare | StatementType::Call
        )
    }

    /// Returns true when the SQL statement is DDL (data definition language).
    pub fn is_ddl(&self) -> bool {
        matches!(
            self.statement_type,
            StatementType::Create | StatementType::Drop | StatementType::Alter
        )
    }

    /// Returns true when the SQL statement is DML (data manipulation language).
    pub fn is_dml(&self) -> bool {
        matches!(
            self.statement_type,
            StatementType::Insert
                | StatementType::Update
                | StatementType::Delete
                | StatementType::Merge
        )
    }

    /// Returns true when the SQL statement has a `RETURNING INTO` clause.
    pub fn is_returning(&self) -> bool {
        self.is_returning
    }

    /// Returns the rowid of the last row that was affected by the statement.
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// let mut stmt = conn
    ///     .statement("insert into TestDates values(100, sysdate, null)")
    ///     .build()?;
    /// stmt.execute(&[])?;
    /// // get the rowid inserted by stmt
    /// let rowid1 = stmt.last_row_id()?;
    /// // get the rowid from database
    /// let rowid2 = conn.query_row_as::<String>("select rowid from TestDates where IntCol = 100", &[])?;
    /// assert_eq!(rowid1, Some(rowid2));
    /// # conn.rollback()?;
    /// # Ok::<(), Error>(())
    /// ```
    pub fn last_row_id(&self) -> Result<Option<String>> {
        let mut rowid = ptr::null_mut();
        chkerr!(self.ctxt(), dpiStmt_getLastRowid(self.handle(), &mut rowid));
        if rowid.is_null() {
            Ok(None)
        } else {
            let mut ptr = ptr::null();
            let mut len = 0;
            chkerr!(
                self.ctxt(),
                dpiRowid_getStringValue(rowid, &mut ptr, &mut len)
            );
            Ok(Some(to_rust_str(ptr, len)))
        }
    }

    /// Gets an OCI handle attribute corresponding to the specified type parameter
    /// See the [`oci_attr` module][crate::oci_attr] for details.
    pub fn oci_attr<T>(&self) -> Result<<<T::DataType as DataType>::Type as ToOwned>::Owned>
    where
        T: OciAttr<HandleType = oci_attr::handle::Stmt>,
        T::Mode: ReadMode,
    {
        let attr_value = AttrValue::from_stmt(self, <T>::ATTR_NUM);
        unsafe { <T::DataType>::get(attr_value) }
    }

    /// Sets an OCI handle attribute corresponding to the specified type parameter
    /// See the [`oci_attr` module][crate::oci_attr] for details.
    pub fn set_oci_attr<T>(&mut self, value: &<T::DataType as DataType>::Type) -> Result<()>
    where
        T: OciAttr<HandleType = oci_attr::handle::Stmt>,
        T::Mode: WriteMode,
    {
        let mut attr_value = AttrValue::from_stmt(self, <T>::ATTR_NUM);
        unsafe { <T::DataType>::set(&mut attr_value, value) }
    }
}

impl AssertSend for Statement {}

/// Column information in a select statement
///
/// # Examples
///
/// Print column information of `emp` table.
///
/// ```no_run
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
/// let mut stmt = conn.statement("select * from emp").build()?;
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
    fn new(stmt: &Stmt, idx: usize) -> Result<ColumnInfo> {
        let mut info = MaybeUninit::uninit();
        chkerr!(
            stmt.ctxt(),
            dpiStmt_getQueryInfo(stmt.handle(), (idx + 1) as u32, info.as_mut_ptr())
        );
        let info = unsafe { info.assume_init() };
        Ok(ColumnInfo {
            name: to_rust_str(info.name, info.nameLength),
            oracle_type: OracleType::from_type_info(stmt.conn(), &info.typeInfo)?,
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
        if 0 < num && 1 <= *self && *self <= num {
            Ok(*self - 1)
        } else {
            Err(Error::invalid_bind_index(*self))
        }
    }

    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        dpiStmt_bindByPos(stmt_handle, *self as u32, var_handle)
    }
}

impl BindIndex for &str {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        let bindname = self.to_uppercase();
        stmt.bind_names()
            .iter()
            .position(|&name| name == bindname)
            .ok_or_else(|| Error::invalid_bind_name(*self))
    }

    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        let s = OdpiStr::new(self);
        dpiStmt_bindByName(stmt_handle, s.ptr, s.len, var_handle)
    }
}

/// A trait implemented by types that can index into columns of a row.
///
/// This trait is sealed and cannot be implemented for types outside of the `oracle` crate.
pub trait ColumnIndex: private::Sealed {
    /// Returns the index of the column specified by `self`.
    #[doc(hidden)]
    fn idx(&self, column_info: &[ColumnInfo]) -> Result<usize>;
}

impl ColumnIndex for usize {
    fn idx(&self, column_info: &[ColumnInfo]) -> Result<usize> {
        let ncols = column_info.len();
        if *self < ncols {
            Ok(*self)
        } else {
            Err(Error::invalid_column_index(*self))
        }
    }
}

impl ColumnIndex for &str {
    fn idx(&self, column_info: &[ColumnInfo]) -> Result<usize> {
        for (idx, info) in column_info.iter().enumerate() {
            if info.name.as_str().eq_ignore_ascii_case(self) {
                return Ok(idx);
            }
        }
        Err(Error::invalid_column_name(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn column_info() -> Result<()> {
        let conn = test_util::connect()?;
        let mut stmt = conn.statement("select * from TestDataTypes").build()?;
        let rows = stmt.query(&[])?;
        let colinfo = rows.column_info();
        assert_eq!(colinfo[0].name(), "STRINGCOL");
        assert_eq!(colinfo[0].oracle_type(), &OracleType::Varchar2(100));
        assert_eq!(colinfo[1].name(), "UNICODECOL");
        assert_eq!(colinfo[1].oracle_type(), &OracleType::NVarchar2(100));
        assert_eq!(colinfo[2].name(), "FIXEDCHARCOL");
        assert_eq!(colinfo[2].oracle_type(), &OracleType::Char(100));
        assert_eq!(colinfo[3].name(), "FIXEDUNICODECOL");
        assert_eq!(colinfo[3].oracle_type(), &OracleType::NChar(100));
        assert_eq!(colinfo[4].name(), "RAWCOL");
        assert_eq!(colinfo[4].oracle_type(), &OracleType::Raw(30));
        assert_eq!(colinfo[5].name(), "FLOATCOL");
        assert_eq!(colinfo[5].oracle_type(), &OracleType::Float(126));
        assert_eq!(colinfo[6].name(), "DOUBLEPRECCOL");
        assert_eq!(colinfo[6].oracle_type(), &OracleType::Float(126));
        assert_eq!(colinfo[7].name(), "INTCOL");
        assert_eq!(colinfo[7].oracle_type(), &OracleType::Number(9, 0));
        assert_eq!(colinfo[8].name(), "NUMBERCOL");
        assert_eq!(colinfo[8].oracle_type(), &OracleType::Number(9, 2));
        assert_eq!(colinfo[9].name(), "DATECOL");
        assert_eq!(colinfo[9].oracle_type(), &OracleType::Date);
        assert_eq!(colinfo[10].name(), "TIMESTAMPCOL");
        assert_eq!(colinfo[10].oracle_type(), &OracleType::Timestamp(6));
        assert_eq!(colinfo[11].name(), "TIMESTAMPTZCOL");
        assert_eq!(colinfo[11].oracle_type(), &OracleType::TimestampTZ(6));
        assert_eq!(colinfo[12].name(), "TIMESTAMPLTZCOL");
        assert_eq!(colinfo[12].oracle_type(), &OracleType::TimestampLTZ(6));
        assert_eq!(colinfo[13].name(), "INTERVALDSCOL");
        assert_eq!(colinfo[13].oracle_type(), &OracleType::IntervalDS(2, 6));
        assert_eq!(colinfo[14].name(), "INTERVALYMCOL");
        assert_eq!(colinfo[14].oracle_type(), &OracleType::IntervalYM(2));
        assert_eq!(colinfo[15].name(), "BINARYFLTCOL");
        assert_eq!(colinfo[15].oracle_type(), &OracleType::BinaryFloat);
        assert_eq!(colinfo[16].name(), "BINARYDOUBLECOL");
        assert_eq!(colinfo[16].oracle_type(), &OracleType::BinaryDouble);
        assert_eq!(colinfo[17].name(), "CLOBCOL");
        assert_eq!(colinfo[17].oracle_type(), &OracleType::CLOB);
        assert_eq!(colinfo[18].name(), "NCLOBCOL");
        assert_eq!(colinfo[18].oracle_type(), &OracleType::NCLOB);
        assert_eq!(colinfo[19].name(), "BLOBCOL");
        assert_eq!(colinfo[19].oracle_type(), &OracleType::BLOB);
        assert_eq!(colinfo[20].name(), "BFILECOL");
        assert_eq!(colinfo[20].oracle_type(), &OracleType::BFILE);
        assert_eq!(colinfo[21].name(), "LONGCOL");
        assert_eq!(colinfo[21].oracle_type(), &OracleType::Long);
        assert_eq!(colinfo[22].name(), "UNCONSTRAINEDCOL");
        assert_eq!(colinfo[22].oracle_type(), &OracleType::Number(0, -127));
        assert_eq!(colinfo[23].name(), "SIGNEDINTCOL");
        assert_eq!(colinfo[23].oracle_type(), &OracleType::Number(38, 0));
        assert_eq!(colinfo[24].name(), "SUBOBJECTCOL");
        assert_eq!(
            colinfo[24].oracle_type().to_string(),
            OracleType::Object(conn.object_type("UDT_SUBOBJECT")?).to_string()
        );
        assert_eq!(colinfo.len(), 25);

        let mut stmt = conn.statement("select * from TestLongRaws").build()?;
        let rows = stmt.query(&[])?;
        let colinfo = rows.column_info();
        assert_eq!(colinfo[0].name(), "INTCOL");
        assert_eq!(colinfo[0].oracle_type(), &OracleType::Number(9, 0));
        assert_eq!(colinfo[1].name(), "LONGRAWCOL");
        assert_eq!(colinfo[1].oracle_type(), &OracleType::LongRaw);
        assert_eq!(colinfo.len(), 2);

        let mut stmt = conn.statement("select * from TestXml").build()?;
        let rows = stmt.query(&[])?;
        let colinfo = rows.column_info();
        assert_eq!(colinfo[0].name(), "INTCOL");
        assert_eq!(colinfo[0].oracle_type(), &OracleType::Number(9, 0));
        assert_eq!(colinfo[1].name(), "XMLCOL");
        assert_eq!(colinfo[1].oracle_type(), &OracleType::Xml);
        assert_eq!(colinfo.len(), 2);

        Ok(())
    }

    #[test]
    fn fetch_rows_to_vec() -> Result<()> {
        let conn = test_util::connect()?;
        // The fetch array size must be less than the number of rows in TestStrings
        // in order to make situation that a new fetch array buffer must allocated
        // in Stmt::fetch_rows().
        let mut stmt = conn
            .statement("select IntCol from TestStrings order by IntCol")
            .fetch_array_size(3)
            .build()?;
        let mut rows = Vec::new();
        for row_result in stmt.query(&[])? {
            rows.push(row_result?);
        }
        for (index, row) in rows.iter().enumerate() {
            let int_col: usize = row.get(0)?;
            assert_eq!(int_col, index + 1);
        }
        Ok(())
    }
}
