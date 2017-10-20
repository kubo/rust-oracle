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
use error::IndexError;
use types::FromSql;
use types::ToSql;
use types::ToSqlInTuple;
use value::Value;
use Result;
use Error;
use StatementType;

//
// Statement
//

pub struct Statement<'conn> {
    conn: &'conn Connection,
    handle: *mut dpiStmt,
    row: Row,
    fetch_array_size: u32,
    is_query: bool,
    is_plsql: bool,
    is_ddl: bool,
    is_dml: bool,
    statement_type: dpiStatementType,
    is_returning: bool,
    colums_are_defined: bool,
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
            is_query: info.isQuery != 0,
            is_plsql: info.isPLSQL != 0,
            is_ddl: info.isDDL != 0,
            is_dml: info.isDML != 0,
            statement_type: info.statementType,
            is_returning: info.isReturning != 0,
            colums_are_defined: false,
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

    pub fn bind<I>(&mut self, bindidx: I, oratype: &OracleType) -> Result<()> where I: BindIndex {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].init_handle(self.conn, oratype, 1)?;
        chkerr!(self.conn.ctxt,
                bindidx.bind(self.handle, self.bind_values[pos].handle));
        Ok(())
    }

    pub fn set_bind_value<I, T>(&mut self, bindidx: I, value: T) -> Result<()> where I: BindIndex, T: ToSql {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].set(value)
    }

    pub fn set_null_value<I>(&mut self, bindidx: I) -> Result<()> where I: BindIndex {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].set_null()
    }

    pub fn bind_value<I, T>(&self, bindidx: I) -> Result<T> where I: BindIndex, T: FromSql {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].get()
    }

    pub fn is_null_value<I>(&self, bindidx: I) -> Result<bool> where I: BindIndex {
        let pos = bindidx.idx(&self)?;
        self.bind_values[pos].is_null()
    }

    pub fn define<I>(&mut self, colidx: I, oratype: &OracleType) -> Result<()> where I: ColumnIndex {
        let pos = colidx.idx(&self.row.column_info)?;
        self.row.column_values[pos].init_handle(self.conn, oratype, DPI_DEFAULT_FETCH_ARRAY_SIZE)?;
        chkerr!(self.conn.ctxt,
                dpiStmt_define(self.handle, (pos + 1) as u32, self.row.column_values[pos].handle));
        Ok(())
    }

    pub fn execute<T, U>(&mut self, params: &T) -> Result<()> where T: ToSqlInTuple<U> {
        params.bind(self)?;
        let mut num_query_columns = 0;
        chkerr!(self.conn.ctxt,
                dpiStmt_execute(self.handle, DPI_MODE_EXEC_DEFAULT, &mut num_query_columns));
        chkerr!(self.conn.ctxt,
                dpiStmt_getFetchArraySize(self.handle, &mut self.fetch_array_size));
        if self.is_query() {
            let num_cols = num_query_columns as usize;

            self.row.column_info = Vec::with_capacity(num_cols);
            self.row.column_values = vec![Value::new(self.conn.ctxt); num_cols];

            for i in 0..num_cols {
                let ci = ColumnInfo::new(self, i)?;
                self.row.column_info.push(ci);
            }
        }
        Ok(())
    }

    // Define columns when they are not defined explicitly.
    fn define_columns(&mut self) -> Result<()> {
        for (idx, val) in self.row.column_values.iter_mut().enumerate() {
            if !val.initialized() {
                let oratype = self.row.column_info[idx].oracle_type();
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
                        dpiStmt_define(self.handle, (idx + 1) as u32, val.handle));
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
        if !self.colums_are_defined {
            self.define_columns()?;
            self.colums_are_defined = true;
        }
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

    pub fn is_query(&self) -> bool {
        self.is_query
    }

    pub fn is_plsql(&self) -> bool {
        self.is_plsql
    }

    pub fn is_ddl(&self) -> bool {
        self.is_ddl
    }

    pub fn is_dml(&self) -> bool {
        self.is_dml
    }

    pub fn statement_type(&self) -> StatementType {
        self.statement_type.clone()
    }

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
            Err(Error::IndexError(IndexError::BindIndex(*self)))
        }
    }

    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        dpiStmt_bindByPos(stmt_handle, *self as u32, var_handle)
    }
}

impl<'a> BindIndex for &'a str {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        stmt.bind_names().iter().position(|&name| name == *self)
            .ok_or_else(|| Error::IndexError(IndexError::BindName((*self).to_string())))
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
            Err(Error::IndexError(IndexError::ColumnIndex(*self)))
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
        Err(Error::IndexError(IndexError::ColumnName((*self).to_string())))
    }
}
