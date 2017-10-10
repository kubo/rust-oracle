extern crate core;
extern crate libc;
#[macro_use]
extern crate lazy_static;

use std::ptr;
use std::result;
use std::os::raw::c_char;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(improper_ctypes)]
mod binding;
#[macro_use]
mod error;
mod connection;
mod odpi;
mod types;
mod value_ref;

pub use binding::dpiAuthMode as AuthMode;
pub use binding::dpiStatementType as StatementType;
pub use binding::dpiShutdownMode as ShutdownMode;
pub use binding::dpiStartupMode as StartupMode;
pub use connection::Connector;
pub use connection::Connection;
pub use error::Error;
pub use error::DbError;
pub use odpi::ColumnInfo;
pub use odpi::OracleType;
pub use odpi::Timestamp;
pub use odpi::IntervalDS;
pub use odpi::IntervalYM;
pub use odpi::Version;
pub use types::FromSql;
pub use value_ref::ValueRef;

use binding::*;
use odpi::DpiStatement;
use error::error_from_context;
use error::error_from_dpi_error;

pub type Result<T> = result::Result<T, Error>;

pub fn client_version() -> Result<Version> {
    let mut dpi_ver = Default::default();
    let ctx = Context::get()?;
    chkerr!(ctx,
            dpiContext_getClientVersion(ctx.context, &mut dpi_ver));
    Ok(Version::new_from_dpi_ver(dpi_ver))
}

pub const AUTH_DEFAULT: dpiAuthMode = DPI_MODE_AUTH_DEFAULT;
pub const AUTH_SYSDBA: dpiAuthMode = DPI_MODE_AUTH_SYSDBA;
pub const AUTH_SYSOPER: dpiAuthMode = DPI_MODE_AUTH_SYSOPER;
pub const AUTH_PRELIM: dpiAuthMode = DPI_MODE_AUTH_PRELIM;
pub const AUTH_SYSASM: dpiAuthMode = DPI_MODE_AUTH_SYSASM;

//
// Context
//

pub struct Context {
    pub context: *mut dpiContext,
    pub common_create_params: dpiCommonCreateParams,
    pub conn_create_params: dpiConnCreateParams,
    pub pool_create_params: dpiPoolCreateParams,
    pub subscr_create_params: dpiSubscrCreateParams,
}

enum ContextResult {
    Ok(Context),
    Err(dpiErrorInfo),
}

unsafe impl Sync for ContextResult {}

lazy_static! {
    static ref DPI_CONTEXT: ContextResult = {
        let mut ctxt = Context {
            context: ptr::null_mut(),
            common_create_params: Default::default(),
            conn_create_params: Default::default(),
            pool_create_params: Default::default(),
            subscr_create_params: Default::default(),
        };
        let mut err: dpiErrorInfo = Default::default();
        if unsafe {
            dpiContext_create(DPI_MAJOR_VERSION, DPI_MINOR_VERSION, &mut ctxt.context, &mut err)
        } == DPI_SUCCESS as i32 {
            unsafe {
                let utf8_ptr = "UTF-8\0".as_ptr() as *const c_char;
                let driver_name = concat!("Rust Oracle : ", env!("CARGO_PKG_VERSION"));
                let driver_name_ptr = driver_name.as_ptr() as *const c_char;
                let driver_name_len = driver_name.len() as u32;
                dpiContext_initCommonCreateParams(ctxt.context, &mut ctxt.common_create_params);
                dpiContext_initConnCreateParams(ctxt.context, &mut ctxt.conn_create_params);
                dpiContext_initPoolCreateParams(ctxt.context, &mut ctxt.pool_create_params);
                dpiContext_initSubscrCreateParams(ctxt.context, &mut ctxt.subscr_create_params);
                ctxt.common_create_params.createMode |= DPI_MODE_CREATE_THREADED;
                ctxt.common_create_params.encoding = utf8_ptr;
                ctxt.common_create_params.nencoding = utf8_ptr;
                ctxt.common_create_params.driverName = driver_name_ptr;
                ctxt.common_create_params.driverNameLength = driver_name_len;
            }
            ContextResult::Ok(ctxt)
        } else {
            ContextResult::Err(err)
        }
    };
}

impl Context {
    pub fn get() -> Result<&'static Context> {
        match *DPI_CONTEXT {
            ContextResult::Ok(ref ctxt) => Ok(ctxt),
            ContextResult::Err(ref err) => Err(error_from_dpi_error(err)),
        }
    }
}

//
// Statement
//

pub struct Statement<'conn> {
    dpi_stmt: DpiStatement<'conn>,
    // shorthand of column_info.len()
    num_cols: usize,
    // Column information of a query.
    column_info: Vec<ColumnInfo>,
    // This attribute stores OrcleTypes used to define columns.
    // The OracleType may be different with the OracleType in
    // ColumnInfo.
    defined_columns: Vec<OracleType>,
    colums_are_defined: bool,
}

impl<'conn> Statement<'conn> {
    fn new(dpi_stmt: DpiStatement) -> Result<Statement> {
        Ok(Statement {
            dpi_stmt: dpi_stmt,
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
        self.dpi_stmt.close("")
    }

    pub fn define<T>(&mut self, idx: T, oratype: OracleType) -> Result<()> where T: RowIndex {
        let pos = try!(idx.idx(&self));
        self.defined_columns[pos] = oratype;
        try!(self.dpi_stmt.define(pos + 1, &self.defined_columns[pos]));
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        self.num_cols = try!(self.dpi_stmt.execute(DPI_MODE_EXEC_DEFAULT));
        if self.is_query() {
            self.column_info = Vec::with_capacity(self.num_cols);
            for i in 0..self.num_cols {
                let col_info = try!(self.dpi_stmt.column_info(i + 1));
                self.column_info.push(col_info);
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
                try!(self.dpi_stmt.define(i + 1, &self.defined_columns[i]));
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
            try!(self.define_columns());
            self.colums_are_defined = true;
        }
        let (found, _) = try!(self.dpi_stmt.fetch());
        if found {
            Ok(try!(Row::new(self)))
        } else {
            Err(Error::NoMoreData)
        }
    }

    pub fn is_query(&self) -> bool {
        self.dpi_stmt.is_query
    }

    pub fn is_plsql(&self) -> bool {
        self.dpi_stmt.is_plsql
    }

    pub fn is_ddl(&self) -> bool {
        self.dpi_stmt.is_ddl
    }

    pub fn is_dml(&self) -> bool {
        self.dpi_stmt.is_dml
    }

    pub fn statement_type(&self) -> StatementType {
        self.dpi_stmt.statement_type.clone()
    }

    pub fn is_returning(&self) -> bool {
        self.dpi_stmt.is_returning
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
            let oratype = &stmt.defined_columns[i];
            let data = try!(stmt.dpi_stmt.query_value(i + 1, oratype));
            columns.push(ValueRef::new(data, oratype));
        }
        Ok(Row { stmt: stmt, columns: columns })
    }

    pub fn get<I, T>(&self, idx: I) -> Result<T> where T: FromSql, I: RowIndex {
        let pos = try!(idx.idx(self.stmt));
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
