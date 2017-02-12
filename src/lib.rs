extern crate core;
extern crate libc;
#[macro_use]
extern crate lazy_static;

use std::fmt;
use std::result;

mod error;
mod ffi;
mod odpi;
mod types;
mod value_ref;

pub use error::Error;
pub use error::DbError;
pub use odpi::AuthMode;
pub use odpi::ColumnInfo;
pub use odpi::OracleType;
pub use odpi::ShutdownMode;
pub use odpi::StartupMode;
pub use odpi::StatementType;
pub use odpi::Timestamp;
pub use odpi::IntervalDS;
pub use odpi::IntervalYM;
pub use types::FromSql;
pub use value_ref::ValueRef;
use odpi::DpiContext;
use odpi::DpiConnection;
use odpi::DpiStatement;

pub type Result<T> = result::Result<T, Error>;

pub fn client_version() -> Result<Version> {
    try!(DpiContext::get()).client_version()
}

//
// Connection
//

pub struct Connection {
    dpi_conn: DpiConnection,
}

impl Connection {
    pub fn connect(username: &str, password: &str, connect_string: &str, auth_mode: AuthMode) -> Result<Connection> {
        let ctxt = try!(DpiContext::get());
        let mut params = ctxt.conn_create_params.clone();
        params.authMode = auth_mode.as_i32();
        if username.len() == 0 && password.len() == 0 {
            params.externalAuth = 1; /* external authorization */
        }
        let dpi_conn = try!(DpiConnection::new(ctxt, username, password, connect_string, &mut params));
        Ok(Connection { dpi_conn: dpi_conn })
    }

    /// break execution of the statement running on the connection
    pub fn break_execution(&self) -> Result<()> {
        self.dpi_conn.break_execution()
    }

    /// change the password for the specified user
    pub fn change_password(&self, username: &str, old_password: &str, new_password: &str) -> Result<()> {
        self.dpi_conn.change_password(username, old_password, new_password)
    }

    /// close the connection now, not when the reference count reaches zero
    pub fn close(&self) -> Result<()> {
        self.dpi_conn.close(ffi::DPI_MODE_CONN_CLOSE_DEFAULT, "")
    }

    /// commits the current active transaction
    /// This feature will be changed later.
    pub fn commit(&self) -> Result<()> {
        self.dpi_conn.commit()
    }

    /// get current schema associated with the connection
    pub fn current_schema(&self) -> Result<String> {
        self.dpi_conn.current_schema()
    }

    /// get edition associated with the connection
    pub fn edition(&self) -> Result<String> {
        self.dpi_conn.edition()
    }

    /// get external name associated with the connection
    pub fn external_name(&self) -> Result<String> {
        self.dpi_conn.external_name()
    }

    /// get internal name associated with the connection
    pub fn internal_name(&self) -> Result<String> {
        self.dpi_conn.internal_name()
    }

    /// return information about the server version in use
    pub fn server_version(&self) -> Result<(String, Version)> {
        self.dpi_conn.server_version()
    }

    /// return the statement cache size
    pub fn statement_cache_size(&self) -> Result<u32> {
        self.dpi_conn.stmt_cache_size()
    }

    /// ping the connection to see if it is still alive
    pub fn ping(&self) -> Result<()> {
        self.dpi_conn.ping()
    }

    /// prepare a statement and return it for subsequent execution/fetching
    pub fn prepare(&self, sql: &str) -> Result<Statement> {
        let dpi_stmt = try!(self.dpi_conn.prepare_statement(false, sql, ""));
        Statement::new(dpi_stmt)
    }

    /// rolls back the current active transaction
    pub fn rollback(&self) -> Result<()> {
        self.dpi_conn.rollback()
    }

    /// set action associated with the connection
    pub fn set_action(&self, action: &str) -> Result<()> {
        self.dpi_conn.set_action(action)
    }

    /// set client identifier associated with the connection
    pub fn set_client_identifier(&self, client_identifier: &str) -> Result<()> {
        self.dpi_conn.set_client_identifier(client_identifier)
    }

    /// set client info associated with the connection
    pub fn set_client_info(&self, client_info: &str) -> Result<()> {
        self.dpi_conn.set_client_info(client_info)
    }

    /// set current schema associated with the connection
    pub fn set_current_schema(&self, current_schema: &str) -> Result<()> {
        self.dpi_conn.set_current_schema(current_schema)
    }

    /// set database operation associated with the connection
    pub fn set_database_operation(&self, database_operation: &str) -> Result<()> {
        self.dpi_conn.set_db_op(database_operation)
    }

    /// set external name associated with the connection
    pub fn set_external_name(&self, external_name: &str) -> Result<()> {
        self.dpi_conn.set_external_name(external_name)
    }

    /// set internal name associated with the connection
    pub fn set_internal_name(&self, internal_name: &str) -> Result<()> {
        self.dpi_conn.set_internal_name(internal_name)
    }

    /// set module associated with the connection
    pub fn set_module(&self, module: &str) -> Result<()> {
        self.dpi_conn.set_module(module)
    }

    /// set the statement cache size
    pub fn set_statement_cache_size(&self, size: u32) -> Result<()> {
        self.dpi_conn.set_stmt_cache_size(size)
    }

    /// Shuts down the database
    pub fn shutdown_database(&self, mode: ShutdownMode) -> Result<()> {
        self.dpi_conn.shutdown_database(mode)
    }

    /// startup the database
    pub fn startup_database(&self, mode: StartupMode) -> Result<()> {
        self.dpi_conn.startup_database(mode)
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
        self.num_cols = try!(self.dpi_stmt.execute(ffi::DPI_MODE_EXEC_DEFAULT));
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
                    OracleType::Number(prec, 0) if 0 < prec && prec < ffi::DPI_MAX_INT64_PRECISION =>
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

//
// Version
//

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: i32,
    minor: i32,
    update: i32,
    patch: i32,
    port_update: i32,
}

impl Version {
    pub fn new(major: i32, minor: i32, update: i32, patch: i32, port_update: i32) -> Version {
        Version { major: major, minor: minor, update: update,
                  patch: patch, port_update: port_update }
    }

    /// 1st part of Oracle version number
    pub fn major(&self) -> i32 {
        self.major
    }

    /// 2nd part of Oracle version number
    pub fn minor(&self) -> i32 {
        self.minor
    }

    /// 3rd part of Oracle version number
    pub fn update(&self) -> i32 {
        self.update
    }

    /// 4th part of Oracle version number
    pub fn patch(&self) -> i32 {
        self.patch
    }

    /// 5th part of Oracle version number
    pub fn port_update(&self) -> i32 {
        self.port_update
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}.{}", self.major, self.minor, self.update, self.patch, self.port_update)
    }
}
