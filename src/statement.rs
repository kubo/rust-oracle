use std::ptr;
use std::fmt;

use binding::*;
use OdpiStr;
use to_odpi_str;
use oracle_type::OracleType;
use connection::Connection;
use error::error_from_context;
use types::FromSql;
use value_ref::ValueRef;
use Result;
use Error;
use StatementType;

//
// Statement
//

pub struct Statement<'conn> {
    conn: &'conn Connection,
    handle: *mut dpiStmt,
    fetch_array_size: u32,
    is_query: bool,
    is_plsql: bool,
    is_ddl: bool,
    is_dml: bool,
    statement_type: dpiStatementType,
    is_returning: bool,
    // shorthand of column_info.len()
    num_cols: usize,
    // Column information of a query.
    column_info: Vec<ColumnInfo>,
    colums_are_defined: bool,
    bind_count: usize,
    bind_names: Vec<String>,
    column_vars: Vec<Variable>,
    bind_vars: Vec<Variable>,
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
            fetch_array_size: 0,
            is_query: info.isQuery != 0,
            is_plsql: info.isPLSQL != 0,
            is_ddl: info.isDDL != 0,
            is_dml: info.isDML != 0,
            statement_type: info.statementType,
            is_returning: info.isReturning != 0,
            num_cols: 0,
            column_info: Vec::new(),
            colums_are_defined: false,
            bind_count: bind_count,
            bind_names: bind_names,
            column_vars: Vec::new(),
            bind_vars: Variable::new_vec(bind_count),
        })
    }

    // pos: zero-based position
    fn find_defined_column(&self, pos: usize) -> Option<&OracleType> {
        match self.column_vars.get(pos) {
            Some(x) =>
                 match (*x).oratype {
                     OracleType::None => None,
                     _ => Some(&(*x).oratype),
                 },
            None => None,
        }
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
        let var = Variable::new(self.conn, oratype, 1)?;
        chkerr!(self.conn.ctxt,
                bindidx.bind(self.handle, var.handle));
        self.bind_vars[pos] = var;
        Ok(())
    }

    pub fn bind_value<I, T>(&self, bindidx: I) -> Result<T> where I: BindIndex, T: FromSql {
        let pos = bindidx.idx(&self)?;
        let var = &self.bind_vars[pos];
        let mut num = 0;
        let mut data = ptr::null_mut();
        chkerr!(self.conn.ctxt,
                dpiVar_getData(var.handle, &mut num, &mut data));
        ValueRef::new(data, var.oratype.native_type()?, &var.oratype)?.get()
    }

    pub fn define<I>(&mut self, colidx: I, oratype: &OracleType) -> Result<()> where I: ColumnIndex {
        let pos = colidx.idx(&self)?;
        let var = Variable::new(self.conn, oratype, DPI_DEFAULT_FETCH_ARRAY_SIZE)?;
        chkerr!(self.conn.ctxt,
                dpiStmt_define(self.handle, (pos + 1) as u32, var.handle));
        self.column_vars[pos] = var;
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let mut num_query_columns = 0;
        chkerr!(self.conn.ctxt,
                dpiStmt_execute(self.handle, DPI_MODE_EXEC_DEFAULT, &mut num_query_columns));
        chkerr!(self.conn.ctxt,
                dpiStmt_getFetchArraySize(self.handle, &mut self.fetch_array_size));
        if self.is_query() {
            self.num_cols = num_query_columns as usize;
            self.column_info = Vec::with_capacity(self.num_cols);
            for i in 0..self.num_cols {
                let ci = ColumnInfo::new(self, i)?;
                self.column_info.push(ci);
            }
            self.column_vars = Variable::new_vec(self.num_cols);
        }
        Ok(())
    }

    // Define columns when they are not defined explicitly.
    fn define_columns(&mut self) -> Result<()> {
        for i in 0..self.num_cols {
            if self.find_defined_column(i).is_none() {
                let oratype = self.column_info[i].oracle_type();
                let oratype_i64 = OracleType::Int64;
                let oratype = match *oratype {
                    // When the column type is number whose prec is less than 18
                    // and the scale is zero, define it as int64.
                    OracleType::Number(prec, 0) if 0 < prec && prec < DPI_MAX_INT64_PRECISION as i16 =>
                        &oratype_i64,
                    _ =>
                        oratype,
                };
                let var = Variable::new(self.conn, oratype, DPI_DEFAULT_FETCH_ARRAY_SIZE)?;
                chkerr!(self.conn.ctxt,
                        dpiStmt_define(self.handle, (i + 1) as u32, var.handle));
                self.column_vars[i] = var;
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
        self.num_cols
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.column_info.iter().map(|info| info.name().as_str()).collect()
    }

    pub fn column_info(&self) -> &Vec<ColumnInfo> {
        &self.column_info
    }

    pub fn fetch(&mut self) -> Result<Row> {
        if !self.colums_are_defined {
            self.define_columns()?;
            self.colums_are_defined = true;
        }
        let mut found = 0;
        let mut buffer_row_index = 0;
        chkerr!(self.conn.ctxt,
                dpiStmt_fetch(self.handle, &mut found, &mut buffer_row_index));
        if found != 0 {
            Ok(Row::new(self)?)
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

pub struct Row<'stmt> {
    stmt: &'stmt Statement<'stmt>,
    columns: Vec<ValueRef<'stmt>>,
}

impl<'stmt> Row<'stmt> {
    fn new(stmt: &'stmt Statement<'stmt>) -> Result<Row<'stmt>> {
        let mut columns = Vec::<ValueRef>::with_capacity(stmt.num_cols);
        for i in 0..stmt.num_cols {
            let var = &stmt.column_vars[i];
            let mut native_type = 0;
            let mut data = ptr::null_mut();
            chkerr!(stmt.conn.ctxt,
                    dpiStmt_getQueryValue(stmt.handle, (i + 1) as u32, &mut native_type, &mut data));
            columns.push(ValueRef::new(data, native_type, &var.oratype)?);
        }
        Ok(Row {
            stmt: stmt,
            columns: columns,
        })
    }

    pub fn get<I, T>(&self, colidx: I) -> Result<T> where I: ColumnIndex, T: FromSql {
        let pos = colidx.idx(self.stmt)?;
        self.columns[pos].get()
    }

    pub fn columns(&self) -> &Vec<ValueRef> {
        &self.columns
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
            Err(Error::InvalidBindIndex(*self, num))
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
    fn idx(&self, stmt: &Statement) -> Result<usize>;
}

impl ColumnIndex for usize {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        let ncols = stmt.column_count();
        if *self < ncols {
            Ok(*self)
        } else {
            Err(Error::InvalidColumnIndex(*self, ncols))
        }
    }
}

impl<'a> ColumnIndex for &'a str {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        stmt.column_names().iter().position(|&name| name == *self)
            .ok_or_else(|| Error::InvalidColumnName((*self).to_string()))
    }
}

//
// Variable
//

pub struct Variable {
    handle: *mut dpiVar,
    oratype: OracleType,
}

impl Variable {
    fn new(conn: &Connection, oratype: &OracleType, array_size: u32) -> Result<Variable> {
        let mut handle: *mut dpiVar = ptr::null_mut();
        let mut data: *mut dpiData = ptr::null_mut();
        let (oratype_num, native_type, size, size_is_byte) = oratype.var_create_param()?;
        chkerr!(conn.ctxt,
                dpiConn_newVar(conn.handle, oratype_num, native_type, array_size, size, size_is_byte,
                               0, ptr::null_mut(), &mut handle, &mut data));
        Ok(Variable {
            handle: handle,
            oratype: oratype.clone(),
        })
    }

    fn new_vec(size: usize) -> Vec<Variable> {
        vec![Variable {handle: ptr::null_mut(), oratype: OracleType::None} ; size]
    }
}

impl Clone for Variable {
    fn clone(&self) -> Variable {
        if self.handle != ptr::null_mut() {
            unsafe { dpiVar_addRef(self.handle); }
        }
        Variable {
            handle: self.handle,
            oratype: self.oratype.clone(),
        }
    }
}

impl Drop for Variable {
    fn drop(&mut self) {
        if self.handle != ptr::null_mut() {
            unsafe { dpiVar_release(self.handle) };
        }
    }
}
