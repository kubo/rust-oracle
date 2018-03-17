// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017-2018 Kubo Takehiro <kubo@jiubao.org>
//
// Redistribution and use in source and binary forms, with or without modification, are
// permitted provided that the following conditions are met:
//
//    1. Redistributions of source code must retain the above copyright notice, this list of
//       conditions and the following disclaimer.
//
//    2. Redistributions in binary form must reproduce the above copyright notice, this list
//       of conditions and the following disclaimer in the documentation and/or other materials
//       provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHORS ''AS IS'' AND ANY EXPRESS OR IMPLIED
// WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
// ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
// ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// The views and conclusions contained in the software and documentation are those of the
// authors and should not be interpreted as representing official policies, either expressed
// or implied, of the authors.

use std::cell::RefCell;
use std::ptr;
use std::fmt;
use std::rc::Rc;

#[allow(unused_imports)]  // Suppress warning when rust verion >= 1.23.
use std::ascii::AsciiExt; // Required when rust verion < 1.23.

use binding::*;

use Connection;
use Error;
use FromSql;
use OracleType;
use Result;
use ResultSet;
use Row;
use RowValue;
use SqlValue;
use ToSql;

use sql_value::BufferRowIndex;
use new_odpi_str;
use to_odpi_str;
use to_rust_str;

/// Parameters to prepare Statement.
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

    /// Reserved for when statement caching is supported.
    Tag(String),

    /// Reserved for when scrollable cursors are supported.
    Scrollable,
}

/// Statement type returned by [Statement.statement_type()](struct.Statement.html#method.statement_type).
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

    /// MERGE statement
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

    /// Undocumented value in [Oracle manual](https://docs.oracle.com/database/122/LNOCI/handle-and-descriptor-attributes.htm#GUID-A251CF91-EB9F-4DBC-8BB8-FB5EA92C20DE__GUID-8D4D4620-9318-4AD3-8E59-231EB71901B8)
    Other(u32),
}

impl fmt::Display for StatementType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
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
            StatementType::Other(ref n) => write!(f, "other({})", n),
        }
    }
}

/// Statement
pub struct Statement<'conn> {
    pub(crate) conn: &'conn Connection,
    handle: *mut dpiStmt,
    pub(crate) column_info: Vec<ColumnInfo>,
    row: Option<Row>,
    shared_buffer_row_index: Rc<RefCell<u32>>,
    statement_type: dpiStatementType,
    is_returning: bool,
    bind_count: usize,
    bind_names: Vec<String>,
    bind_values: Vec<SqlValue>,
    fetch_array_size: u32,
}

impl<'conn> Statement<'conn> {

    pub(crate) fn new(conn: &'conn Connection, sql: &str, params: &[StmtParam]) -> Result<Statement<'conn>> {
        let sql = to_odpi_str(sql);
        let mut fetch_array_size = DPI_DEFAULT_FETCH_ARRAY_SIZE;
        let mut scrollable = 0;
        let mut tag = new_odpi_str();
        for param in params {
            match param {
                &StmtParam::FetchArraySize(size) => {
                    fetch_array_size = size;
                },
                &StmtParam::Scrollable => {
                    scrollable = 1;
                },
                &StmtParam::Tag(ref name) => {
                    tag = to_odpi_str(name);
                },
            }
        }
        let mut handle: *mut dpiStmt = ptr::null_mut();
        chkerr!(conn.ctxt,
                dpiConn_prepareStmt(conn.handle, scrollable, sql.ptr, sql.len,
                                    tag.ptr, tag.len, &mut handle));
        let mut info: dpiStmtInfo = Default::default();
        chkerr!(conn.ctxt,
                dpiStmt_getInfo(handle, &mut info),
                unsafe { dpiStmt_release(handle); });
        let mut num = 0;
        chkerr!(conn.ctxt,
                dpiStmt_getBindCount(handle, &mut num),
                unsafe { dpiStmt_release(handle); });
        let bind_count = num as usize;
        let mut bind_names = Vec::with_capacity(bind_count);
        let mut bind_values = Vec::with_capacity(bind_count);
        if bind_count > 0 {
            let mut names: Vec<*const i8> = vec![ptr::null_mut(); bind_count];
            let mut lengths = vec![0; bind_count];
            chkerr!(conn.ctxt,
                    dpiStmt_getBindNames(handle, &mut num,
                                         names.as_mut_ptr(), lengths.as_mut_ptr()),
                    unsafe { dpiStmt_release(handle); });
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
            statement_type: info.statementType,
            is_returning: info.isReturning != 0,
            bind_count: bind_count,
            bind_names: bind_names,
            bind_values: bind_values,
            fetch_array_size: fetch_array_size,
        })
    }

    /// Closes the statement before the end of lifetime.
    pub fn close(&mut self) -> Result<()> {
        self.close_internal("")
    }

    fn close_internal(&mut self, tag: &str) -> Result<()> {
        let tag = to_odpi_str(tag);

        chkerr!(self.conn.ctxt,
                dpiStmt_close(self.handle, tag.ptr, tag.len));
        self.handle = ptr::null_mut();
        Ok(())
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
    /// # use oracle::{Connection, OracleType};
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    /// let mut stmt = conn.prepare("begin :outval := upper(:inval); end;", &[]).unwrap();
    ///
    /// // Sets NULL whose data type is VARCHAR2(60) to the first bind value.
    /// stmt.bind(1, &OracleType::Varchar2(60)).unwrap();
    ///
    /// // Sets "to be upper-case" to the second by its name.
    /// stmt.bind("inval", &"to be upper-case").unwrap();
    ///
    /// stmt.execute(&[]).unwrap();
    /// let outval: String = stmt.bind_value(1).unwrap();
    /// assert_eq!(outval, "TO BE UPPER-CASE");
    /// ```
    pub fn bind<I>(&mut self, bindidx: I, value: &ToSql) -> Result<()> where I: BindIndex {
        let pos = bindidx.idx(&self)?;
        if self.bind_values[pos].init_handle(self.conn.handle, &value.oratype()?, 1)? {
            chkerr!(self.conn.ctxt,
                    bindidx.bind(self.handle, self.bind_values[pos].handle));
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
    /// # use oracle::{Connection, OracleType};
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    ///
    /// // Prepares "begin :outval := upper(:inval); end;",
    /// // sets NULL whose data type is VARCHAR2(60) to the first bind variable,
    /// // sets "to be upper-case" to the second and then executes it.
    /// let mut stmt = conn.prepare("begin :outval := upper(:inval); end;", &[]).unwrap();
    /// stmt.execute(&[&OracleType::Varchar2(60),
    ///              &"to be upper-case"]).unwrap();
    ///
    /// // Get the first bind value by position.
    /// let outval: String = stmt.bind_value(1).unwrap();
    /// assert_eq!(outval, "TO BE UPPER-CASE");
    ///
    /// // Get the first bind value by name.
    /// let outval: String = stmt.bind_value("outval").unwrap();
    /// assert_eq!(outval, "TO BE UPPER-CASE");
    /// ```
    pub fn bind_value<I, T>(&self, bindidx: I) -> Result<T> where I: BindIndex, T: FromSql {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].get()
    }

    /// Executes the prepared statement and returns an Iterator over rows.
    pub fn query(&mut self, params: &[&ToSql]) -> Result<ResultSet<Row>> {
        self.exec(params, true, "query")?;
        Ok(ResultSet::<Row>::new(self))
    }

    /// Executes the prepared statement and returns an Iterator over rows.
    pub fn query_named(&mut self, params: &[(&str, &ToSql)]) -> Result<ResultSet<Row>> {
        self.exec_named(params, true, "query_named")?;
        Ok(ResultSet::<Row>::new(self))
    }

    /// Executes the prepared statement and returns an Iterator over rows.
    /// The iterator returns `Result<T>` where T is the specified type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::Connection;
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    /// let mut stmt = conn.prepare("select ename, sal, comm from emp where deptno = :1", &[]).unwrap();
    /// let rows = stmt.query_as::<(String, i32, Option<i32>)>(&[&10]).unwrap();
    ///
    /// println!("---------------|---------------|---------------|");
    /// for row_result in rows {
    ///     let (ename, sal, comm) = row_result.unwrap();
    ///     println!(" {:14}| {:>10}    | {:>10}    |",
    ///              ename,
    ///              sal,
    ///              comm.map_or("".to_string(), |v| v.to_string()));
    /// }
    /// ```
    pub fn query_as<'a, T>(&'a mut self, params: &[&ToSql]) -> Result<ResultSet<'a, T>>
        where T: RowValue
    {
        self.exec(params, true, "query_as")?;
        Ok(ResultSet::new(self))
    }

    /// Executes the prepared statement and returns an Iterator over rows.
    /// The iterator returns `Result<T>` where T is the specified type.
    /// Bind parameters are bound by their names.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::Connection;
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    /// let mut stmt = conn.prepare("select ename, sal, comm from emp where deptno = :deptno", &[]).unwrap();
    /// let rows = stmt.query_as_named::<(String, i32, Option<i32>)>(&[("deptno", &10)]).unwrap();
    ///
    /// println!("---------------|---------------|---------------|");
    /// for row_result in rows {
    ///     let (ename, sal, comm) = row_result.unwrap();
    ///     println!(" {:14}| {:>10}    | {:>10}    |",
    ///              ename,
    ///              sal,
    ///              comm.map_or("".to_string(), |v| v.to_string()));
    /// }
    /// ```
    pub fn query_as_named<'a, T>(&'a mut self, params: &[(&str, &ToSql)]) -> Result<ResultSet<'a, T>>
        where T: RowValue
    {
        self.exec_named(params, true, "query_as_named")?;
        Ok(ResultSet::new(self))
    }

    /// Gets the first row from the prepared statement.
    ///
    /// If the query returns more than one row, all rows except the first are ignored.
    /// It returns `Err(Error::NoDataFound)` when no rows are found.
    pub fn query_row(&mut self, params: &[&ToSql]) -> Result<Row> {
        let row = self.query(params)?.next();
        row.unwrap_or(Err(Error::NoDataFound))
    }

    /// Gets one row from the prepared statement using named bind parameters.
    ///
    /// If the query returns more than one row, all rows except the first are ignored.
    /// It returns `Err(Error::NoDataFound)` when no rows are found.
    pub fn query_row_named(&mut self, params: &[(&str, &ToSql)]) -> Result<Row> {
        let row = self.query_named(params)?.next();
        row.unwrap_or(Err(Error::NoDataFound))
    }

    /// Gets one row from the prepared statement as specified type.
    ///
    /// If the query returns more than one row, all rows except the first are ignored.
    /// It returns `Err(Error::NoDataFound)` when no rows are found.
    pub fn query_row_as<T>(&mut self, params: &[&ToSql]) -> Result<<T>::Item> where T: RowValue {
        let row = self.query_as::<T>(params)?.next();
        row.unwrap_or(Err(Error::NoDataFound))
    }

    /// Gets one row from the prepared statement as specified type using named bind parameters.
    ///
    /// If the query returns more than one row, all rows except the first are ignored.
    /// It returns `Err(Error::NoDataFound)` when no rows are found.
    pub fn query_row_as_named<T>(&mut self, params: &[(&str, &ToSql)]) -> Result<<T>::Item> where T: RowValue {
        let row = self.query_as_named::<T>(params)?.next();
        row.unwrap_or(Err(Error::NoDataFound))
    }

    /// Binds values by position and executes the statement.
    /// It will retunrs `Err` when the statemnet is a select statement.
    ///
    /// See also [Connection.execute](struct.Connection.html#method.execute).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::Connection;
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    ///
    /// // execute a statement without bind parameters
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (113, 'John')", &[]).unwrap();
    /// stmt.execute(&[]).unwrap();
    ///
    /// // execute a statement with binding parameters by position
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (:1, :2)", &[]).unwrap();
    /// stmt.execute(&[&114, &"Smith"]).unwrap();
    /// stmt.execute(&[&115, &"Paul"]).unwrap();  // execute with other values.
    ///
    /// ```
    pub fn execute(&mut self, params: &[&ToSql]) -> Result<()> {
        self.exec(params, false, "execute")
    }

    /// Binds values by name and executes the statement.
    /// It will retunrs `Err` when the statemnet is a select statement.
    ///
    /// See also [Connection.execute_named](struct.Connection.html#method.execute_named).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::Connection;
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    ///
    /// // execute a statement with binding parameters by name
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (:id, :name)", &[]).unwrap();
    /// stmt.execute_named(&[("id", &114),
    ///                      ("name", &"Smith")]).unwrap();
    /// stmt.execute_named(&[("id", &115),
    ///                      ("name", &"Paul")]).unwrap(); // execute with other values.
    /// ```
    pub fn execute_named(&mut self, params: &[(&str, &ToSql)]) -> Result<()> {
        self.exec_named(params, false, "execute_named")
    }

    fn check_stmt_type(&self, must_be_query: bool, method_name: &str) -> Result<()> {
        if must_be_query {
            if self.statement_type == DPI_STMT_TYPE_SELECT {
                Ok(())
            } else {
                Err(Error::InvalidOperation(format!("Could not use the `{}` method for non-select statements", method_name)))
            }
        } else {
            if cfg!(feature = "restore-deleted") || self.statement_type != DPI_STMT_TYPE_SELECT {
                Ok(())
            } else {
                Err(Error::InvalidOperation(format!("Could not use the `{}` method for select statements", method_name)))
            }
        }
    }

    pub(crate) fn exec(&mut self, params: &[&ToSql], must_be_query: bool, method_name: &str) -> Result<()> {
        self.check_stmt_type(must_be_query, method_name)?;
        for i in 0..params.len() {
            self.bind(i + 1, params[i])?;
        }
        self.exec_common()
    }

    pub(crate) fn exec_named(&mut self, params: &[(&str, &ToSql)], must_be_query: bool, method_name: &str) -> Result<()> {
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
        chkerr!(self.conn.ctxt,
                dpiStmt_setFetchArraySize(self.handle, self.fetch_array_size));
        chkerr!(self.conn.ctxt,
                dpiStmt_execute(self.handle, exec_mode, &mut num_query_columns));
        if self.statement_type == DPI_STMT_TYPE_SELECT {
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
                    val.buffer_row_index = BufferRowIndex::Shared(self.shared_buffer_row_index.clone());
                    let oratype = self.column_info[i].oracle_type();
                    let oratype_i64 = OracleType::Int64;
                    let oratype = match *oratype {
                        // When the column type is number whose prec is less than 18
                        // and the scale is zero, define it as int64.
                        OracleType::Number(prec, 0) if 0 < prec && prec < DPI_MAX_INT64_PRECISION as u8 =>
                            &oratype_i64,
                        _ =>
                            oratype,
                    };
                    val.init_handle(self.conn.handle, oratype, self.fetch_array_size)?;
                    chkerr!(self.conn.ctxt,
                            dpiStmt_define(self.handle, (i + 1) as u32, val.handle));
                    column_values.push(val);
                }
                self.row = Some(Row::new(self.conn, column_names, column_values)?);
            }
        }
        if self.is_returning {
            for mut val in self.bind_values.iter_mut() {
                val.fix_internal_data()?;
            }
        }
        Ok(())
    }

    /// Gets values returned by RETURNING INTO clause.
    ///
    /// When the `bindidx` ponints to a bind variable out of RETURNING INTO clause,
    /// the behavior is undefined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::{Connection, OracleType};
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    ///
    /// // create a table using identity column (Oracle 12c feature).
    /// conn.execute("create table people (id number generated as identity, name varchar2(30))", &[]).unwrap();
    ///
    /// // insert one person and return the generated id into :id.
    /// let stmt = conn.execute("insert into people(name) values ('Asimov') returning id into :id", &[&None::<i32>]).unwrap();
    /// let inserted_id: i32 = stmt.returned_values("id").unwrap()[0];
    /// println!("Asimov's ID is {}", inserted_id);
    ///
    /// // insert another person and return the generated id into :id.
    /// let stmt = conn.execute("insert into people(name) values ('Clark') returning id into :id", &[&None::<i32>]).unwrap();
    /// let inserted_id: i32 = stmt.returned_values("id").unwrap()[0];
    /// println!("Clark's ID is {}", inserted_id);
    ///
    /// // delete all people and return deleted names into :name.
    /// let stmt = conn.execute("delete from people returning name into :name", &[&OracleType::Varchar2(30)]).unwrap();
    /// let deleted_names: Vec<String> = stmt.returned_values("name").unwrap();
    /// for name in deleted_names {
    ///     println!("{} is deleted.", name);
    /// }
    ///
    /// // cleanup
    /// conn.execute("drop table people purge", &[]).unwrap();
    /// ```
    ///
    /// [Statement.bind_value()]: #method.bind_value
    pub fn returned_values<I, T>(&self, bindidx: I) -> Result<Vec<T>> where I: BindIndex, T: FromSql {
        let mut rows = 0;
        chkerr!(self.conn.ctxt,
                dpiStmt_getRowCount(self.handle, &mut rows));
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

    /// Returns the number of bind variables in the statement.
    ///
    /// In SQL statements this is the total number of bind variables whereas in
    /// PL/SQL statements this is the count of the **unique** bind variables.
    ///
    /// ```no_run
    /// # use oracle::Connection;
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    ///
    /// // SQL statements
    /// let stmt = conn.prepare("select :val1, :val2, :val1 from dual", &[]).unwrap();
    /// assert_eq!(stmt.bind_count(), 3); // val1, val2 and val1
    ///
    /// // PL/SQL statements
    /// let stmt = conn.prepare("begin :val1 := :val1 || :val2; end;", &[]).unwrap();
    /// assert_eq!(stmt.bind_count(), 2); // val1(twice) and val2
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
    /// # use oracle::Connection;
    /// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    ///
    /// let stmt = conn.prepare("BEGIN :val1 := :val2 || :val1 || :aàáâãäå; END;", &[]).unwrap();
    /// assert_eq!(stmt.bind_count(), 3);
    /// let bind_names = stmt.bind_names();
    /// assert_eq!(bind_names.len(), 3);
    /// assert_eq!(bind_names[0], "VAL1");
    /// assert_eq!(bind_names[1], "VAL2");
    /// assert_eq!(bind_names[2], "AÀÁÂÃÄÅ");
    /// ```
    pub fn bind_names(&self) -> Vec<&str> {
        self.bind_names.iter().map(|name| name.as_str()).collect()
    }

    /// Returns the number of columns.
    /// This returns zero for non-query statements.
    #[cfg(feature = "restore-deleted")]
    #[deprecated(since="0.0.4", note="use `column_info` in the return value of `query`, `query_named`, `query_as` or `query_as_named`")]
    #[doc(hidden)]
    pub fn column_count(&self) -> usize {
        self.column_info.len()
    }

    /// Returns the column names.
    /// This returns an empty vector for non-query statements.
    #[cfg(feature = "restore-deleted")]
    #[deprecated(since="0.0.4", note="use `column_info` in the return value of `query`, `query_named`, `query_as` or `query_as_named`")]
    #[doc(hidden)]
    pub fn column_names(&self) -> Vec<&str> {
        self.column_info.iter().map(|info| info.name().as_str()).collect()
    }

    /// Returns column information.
    #[cfg(feature = "restore-deleted")]
    #[deprecated(since="0.0.4", note="use `column_info` in the return value of `query`, `query_named`, `query_as` or `query_as_named`")]
    #[doc(hidden)]
    pub fn column_info(&self) -> &Vec<ColumnInfo> {
        &self.column_info
    }

    /// Fetchs one row from the statement. This returns `Err(Error::NoDataFound)`
    /// when all rows are fetched.
    #[cfg(feature = "restore-deleted")]
    #[deprecated(since="0.0.4", note="use `query`, `query_named`, `query_as` or `query_as_named` instead of `execute` and `fetch`")]
    #[doc(hidden)]
    pub fn fetch(&mut self) -> Result<&Row> {
        self.next().unwrap_or(Err(Error::NoDataFound))
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
            Some(Err(::error::error_from_context(self.conn.ctxt)))
        }
    }

    /// Returns statement type
    pub fn statement_type(&self) -> StatementType {
        match self.statement_type {
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
            _ => StatementType::Other(self.statement_type),
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

/// Column information in a select statement
///
/// # Examples
///
/// Print column information of `emp` table.
///
/// ```no_run
/// # use oracle::Connection;
/// let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
/// let mut stmt = conn.prepare("select * from emp", &[]).unwrap();
/// let rows = stmt.query(&[]).unwrap();
/// println!(" {:-30} {:-8} {}", "Name", "Null?", "Type");
/// println!(" {:-30} {:-8} {}", "------------------------------", "--------", "----------------------------");
/// for info in rows.column_info() {
///    println!("{:-30} {:-8} {}",
///             info.name(),
///             if info.nullable() {""} else {"NOT NULL"},
///             info.oracle_type());
/// }
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
#[derive(Clone)]
pub struct ColumnInfo {
    name: String,
    oracle_type: OracleType,
    nullable: bool,
}

impl ColumnInfo {
    fn new(stmt: &Statement, idx: usize) -> Result<ColumnInfo> {
        let mut info = Default::default();
        chkerr!(stmt.conn.ctxt,
                dpiStmt_getQueryInfo(stmt.handle, (idx + 1) as u32, &mut info));
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
pub trait BindIndex {
    /// Returns the index of the bind value specified by `self`.
    fn idx(&self, stmt: &Statement) -> Result<usize>;
    /// Binds the specified value by using a private method.
    ///
    /// TODO: hide this method.
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
        stmt.bind_names().iter().position(|&name| name == bindname)
            .ok_or_else(|| Error::InvalidBindName((*self).to_string()))
    }

    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        let s = to_odpi_str(*self);
        dpiStmt_bindByName(stmt_handle, s.ptr, s.len, var_handle)
    }
}

/// A trait implemented by types that can index into columns of a row.
pub trait ColumnIndex {
    /// Returns the index of the column specified by `self`.
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
