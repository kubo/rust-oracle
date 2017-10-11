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
    pub(crate) conn: &'conn Connection,
    pub(crate) handle: *mut dpiStmt,
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
    // This attribute stores OrcleTypes used to define columns.
    // The OracleType may be different with the OracleType in
    // ColumnInfo.
    pub(crate) defined_columns: Vec<OracleType>,
    colums_are_defined: bool,
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
            defined_columns: Vec::new(),
            colums_are_defined: false
        })
    }

    // pos: zero-based position
    fn find_defined_column(&self, pos: usize) -> Option<&OracleType> {
        match self.defined_columns.get(pos) {
            Some(x) =>
                 match *x {
                     OracleType::None => None,
                     _ => Some(x),
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

    pub fn define<T>(&mut self, idx: T, oratype: OracleType) -> Result<()> where T: RowIndex {
        let pos = idx.idx(&self)?;
        self.defined_columns[pos] = oratype;
        let var = DpiVar::new(self.conn, &self.defined_columns[pos], DPI_DEFAULT_FETCH_ARRAY_SIZE)?;
        chkerr!(self.conn.ctxt,
                dpiStmt_define(self.handle, (pos + 1) as u32, var.handle));
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let mut num_query_columns = 0;
        chkerr!(self.conn.ctxt,
                dpiStmt_execute(self.handle, DPI_MODE_EXEC_DEFAULT, &mut num_query_columns));
        chkerr!(self.conn.ctxt,
                dpiStmt_getFetchArraySize(self.handle, &mut self.fetch_array_size));
        self.num_cols = num_query_columns as usize;
        if self.is_query() {
            self.column_info = Vec::with_capacity(self.num_cols);
            for i in 0..self.num_cols {
                let ci = ColumnInfo::new(self, i)?;
                self.column_info.push(ci);
            }
            self.defined_columns = Vec::with_capacity(self.num_cols);
            self.defined_columns.resize(self.num_cols, OracleType::None);
        }
        Ok(())
    }

    // Define columns when they are not defined explicitly.
    fn define_columns(&mut self) -> Result<()> {
        for i in 0..self.num_cols {
            if self.find_defined_column(i).is_none() {
                let oratype = self.column_info[i].oracle_type().clone();
                let oratype = match oratype {
                    // When the column type is number whose prec is less than 18
                    // and the scale is zero, define it as int64.
                    OracleType::Number(prec, 0) if 0 < prec && prec < DPI_MAX_INT64_PRECISION as i16 =>
                        OracleType::Int64,
                    _ =>
                        oratype,
                };
                self.defined_columns[i] = oratype;
                let var = DpiVar::new(self.conn, &self.defined_columns[i], DPI_DEFAULT_FETCH_ARRAY_SIZE)?;
                chkerr!(self.conn.ctxt,
                        dpiStmt_define(self.handle, (i + 1) as u32, var.handle));
            }
        }
        Ok(())
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
            columns.push(ValueRef::new(stmt, i)?);
        }
        Ok(Row {
            stmt: stmt,
            columns: columns,
        })
    }

    pub fn get<I, T>(&self, rowidx: I) -> Result<T> where I: RowIndex, T: FromSql {
        let pos = rowidx.idx(self.stmt)?;
        self.columns[pos].get()
    }

    pub fn columns(&self) -> &Vec<ValueRef> {
        &self.columns
    }
}

//
// RowIndex
//

pub trait RowIndex {
    fn idx(&self, stmt: &Statement) -> Result<usize>;
}

impl RowIndex for usize {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        let ncols = stmt.column_count();
        if *self < ncols {
            Ok(*self)
        } else {
            Err(Error::InvalidColumnIndex(*self, ncols))
        }
    }
}

impl<'a> RowIndex for &'a str {
    fn idx(&self, stmt: &Statement) -> Result<usize> {
        stmt.column_names().iter().position(|&name| name == *self)
            .ok_or_else(|| Error::InvalidColumnName((*self).to_string()))
    }
}

//
// DpiVar
//

pub struct DpiVar<'conn> {
    _conn: &'conn Connection,
    handle: *mut dpiVar,
}

impl<'conn> DpiVar<'conn> {
    pub(crate) fn new(conn: &'conn Connection, oratype: &OracleType, array_size: u32) -> Result<DpiVar<'conn>> {
        let mut handle: *mut dpiVar = ptr::null_mut();
        let mut data: *mut dpiData = ptr::null_mut();
        let (oratype, native_type, size, size_is_byte) = try!(oratype.var_create_param());
        chkerr!(conn.ctxt,
                dpiConn_newVar(conn.handle, oratype, native_type, array_size, size, size_is_byte,
                               0, ptr::null_mut(), &mut handle, &mut data));
        Ok(DpiVar {
            _conn: conn,
            handle: handle,
        })
    }
}

impl<'conn> Drop for DpiVar<'conn> {
    fn drop(&mut self) {
        let _ = unsafe { dpiVar_release(self.handle) };
    }
}
