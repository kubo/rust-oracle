// Rust Oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017 Kubo Takehiro <kubo@jiubao.org>
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

use std::ptr;
use std::fmt;

use binding::*;
use OdpiStr;
use to_odpi_str;
use OracleType;
use Connection;
use types::FromSql;
use types::ToSql;
use types::ToSqlInTuple;
use value::Value;
use Result;
use Error;

//
// StatementType
//

/// Statement type returned by [Statement.statement_type()](struct.Statement.html#method.statement_type).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StatementType {
    /// select statement
    Select,
    /// update statement
    Update,
    /// delete statement
    Delete,
    /// insert statement
    Insert,
    /// create statement
    Create,
    /// drop statement
    Drop,
    /// alter statement
    Alter,
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
            StatementType::Update => write!(f, "update"),
            StatementType::Delete => write!(f, "delete"),
            StatementType::Insert => write!(f, "insert"),
            StatementType::Create => write!(f, "create"),
            StatementType::Drop => write!(f, "drop"),
            StatementType::Alter => write!(f, "alter"),
            StatementType::Begin => write!(f, "PL/SQL(begin)"),
            StatementType::Declare => write!(f, "PL/SQL(declare)"),
            StatementType::Other(ref n) => write!(f, "other({})", n),
        }
    }
}

//
// Statement
//

pub struct Statement<'conn> {
    conn: &'conn Connection,
    handle: *mut dpiStmt,
    row: Row,
    fetch_array_size: u32,
    statement_type: dpiStatementType,
    is_returning: bool,
    bind_count: usize,
    bind_names: Vec<String>,
    bind_values: Vec<Value>,
}

impl<'conn> Statement<'conn> {

    pub fn new(conn: &'conn Connection, scrollable: bool, sql: &str, tag: &str) -> Result<Statement<'conn>> {
        let scrollable = if scrollable { 1 } else { 0 };
        let sql = to_odpi_str(sql);
        let tag = to_odpi_str(tag);
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
        let mut bind_names = Vec::new();
        if bind_count > 0 {
            let mut names: Vec<*const i8> = vec![ptr::null_mut(); bind_count];
            let mut lengths = vec![0; bind_count];
            chkerr!(conn.ctxt,
                    dpiStmt_getBindNames(handle, &mut num,
                                         names.as_mut_ptr(), lengths.as_mut_ptr()),
                    unsafe { dpiStmt_release(handle); });
            bind_names = Vec::with_capacity(num as usize);
            for i in 0..(num as usize) {
                bind_names.push(OdpiStr::new(names[i], lengths[i]).to_string());
            }
        };
        Ok(Statement {
            conn: conn,
            handle: handle,
            row: Row { column_info: Vec::new(), column_values: Vec::new(), },
            fetch_array_size: 0,
            statement_type: info.statementType,
            is_returning: info.isReturning != 0,
            bind_count: bind_count,
            bind_names: bind_names,
            bind_values: vec![Value::new(conn.ctxt); bind_count],
        })
    }

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

    pub fn bind<I, T>(&mut self, bindidx: I, value: T) -> Result<()> where I: BindIndex, T: ToSql {
        let pos = bindidx.idx(&self)?;
        if self.bind_values[pos].init_handle(self.conn, &value.oratype(), 1)? {
            chkerr!(self.conn.ctxt,
                    bindidx.bind(self.handle, self.bind_values[pos].handle));
        }
        self.bind_values[pos].set(value)
    }

    pub fn bind_value<I, T>(&self, bindidx: I) -> Result<T> where I: BindIndex, T: FromSql {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].get()
    }

    pub fn execute<T, U>(&mut self, params: &T) -> Result<()> where T: ToSqlInTuple<U> {
        params.bind(self)?;
        let mut num_query_columns = 0;
        chkerr!(self.conn.ctxt,
                dpiStmt_execute(self.handle, DPI_MODE_EXEC_DEFAULT, &mut num_query_columns));
        chkerr!(self.conn.ctxt,
                dpiStmt_getFetchArraySize(self.handle, &mut self.fetch_array_size));
        if self.statement_type == DPI_STMT_TYPE_SELECT {
            let num_cols = num_query_columns as usize;

            self.row.column_info = Vec::with_capacity(num_cols);
            self.row.column_values = vec![Value::new(self.conn.ctxt); num_cols];

            for i in 0..num_cols {
                // set column info
                let ci = ColumnInfo::new(self, i)?;
                self.row.column_info.push(ci);
                // setup column value
                let mut val = unsafe { self.row.column_values.get_unchecked_mut(i) };
                let oratype = self.row.column_info[i].oracle_type();
                let oratype_i64 = OracleType::Int64;
                let oratype = match *oratype {
                    // When the column type is number whose prec is less than 18
                    // and the scale is zero, define it as int64.
                    OracleType::Number(prec, 0) if 0 < prec && prec < DPI_MAX_INT64_PRECISION as i16 =>
                        &oratype_i64,
                    _ =>
                        oratype,
                };
                val.init_handle(self.conn, oratype, DPI_DEFAULT_FETCH_ARRAY_SIZE)?;
                chkerr!(self.conn.ctxt,
                        dpiStmt_define(self.handle, (i + 1) as u32, val.handle));
            }
        }
        Ok(())
    }

    pub fn bind_count(&self) -> usize {
        self.bind_count
    }

    pub fn bind_names(&self) -> Vec<&str> {
        self.bind_names.iter().map(|name| name.as_str()).collect()
    }

    pub fn column_count(&self) -> usize {
        self.row.column_info.len()
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.row.column_info.iter().map(|info| info.name().as_str()).collect()
    }

    pub fn column_info(&self) -> &Vec<ColumnInfo> {
        &self.row.column_info
    }

    pub fn fetch(&mut self) -> Result<&Row> {
        let mut found = 0;
        let mut buffer_row_index = 0;
        chkerr!(self.conn.ctxt,
                dpiStmt_fetch(self.handle, &mut found, &mut buffer_row_index));
        if found != 0 {
            for val in self.row.column_values.iter_mut() {
                val.buffer_row_index = buffer_row_index;
            }
            Ok(&self.row)
        } else {
            Err(Error::NoMoreData)
        }
    }

    /// Returns statement type
    pub fn statement_type(&self) -> StatementType {
        match self.statement_type {
            DPI_STMT_TYPE_SELECT => StatementType::Select,
            DPI_STMT_TYPE_UPDATE => StatementType::Update,
            DPI_STMT_TYPE_DELETE => StatementType::Delete,
            DPI_STMT_TYPE_INSERT => StatementType::Insert,
            DPI_STMT_TYPE_CREATE => StatementType::Create,
            DPI_STMT_TYPE_DROP => StatementType::Drop,
            DPI_STMT_TYPE_ALTER => StatementType::Alter,
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
        let _ = unsafe { dpiStmt_release(self.handle) };
    }
}

//
// ColumnInfo
//

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
            name: OdpiStr::new(info.name, info.nameLength).to_string(),
            oracle_type: OracleType::from_type_info(&info.typeInfo)?,
            nullable: info.nullOk != 0,
        })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn oracle_type(&self) -> &OracleType {
        &self.oracle_type
    }

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

//
// Row
//

pub struct Row {
    column_info: Vec<ColumnInfo>,
    column_values: Vec<Value>,
}

impl Row {
    pub fn get<I, T>(&self, colidx: I) -> Result<T> where I: ColumnIndex, T: FromSql {
        let pos = colidx.idx(&self.column_info)?;
        self.column_values[pos].get()
    }

    pub fn columns(&self) -> &Vec<Value> {
        &self.column_values
    }
}

//
// BindIndex
//

pub trait BindIndex {
    fn idx(&self, stmt: &Statement) -> Result<usize>;
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
        stmt.bind_names().iter().position(|&name| name == *self)
            .ok_or_else(|| Error::InvalidBindName((*self).to_string()))
    }

    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        let s = to_odpi_str(*self);
        dpiStmt_bindByName(stmt_handle, s.ptr, s.len, var_handle)
    }
}

//
// ColumnIndex
//

pub trait ColumnIndex {
    fn idx(&self, column_info: &Vec<ColumnInfo>) -> Result<usize>;
}

impl ColumnIndex for usize {
    fn idx(&self, column_info: &Vec<ColumnInfo>) -> Result<usize> {
        let ncols = column_info.len();
        if *self < ncols {
            Ok(*self)
        } else {
            Err(Error::InvalidColumnIndex(*self))
        }
    }
}

impl<'a> ColumnIndex for &'a str {
    fn idx(&self, column_info: &Vec<ColumnInfo>) -> Result<usize> {
        for (idx, info) in column_info.iter().enumerate() {
            if info.name().as_str() == *self {
                return Ok(idx);
            }
        }
        Err(Error::InvalidColumnName((*self).to_string()))
    }
}
