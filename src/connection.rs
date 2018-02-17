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

use std::ptr;

use Version;
use Statement;

use binding::*;
use ColumnValues;
use Context;
use ObjectType;
use Result;
use ToSql;

use new_odpi_str;
use to_odpi_str;
use to_rust_str;

/// Authorization mode
///
/// See [Connector.auth_mode](struct.Connector.html#method.auth_mode).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AuthMode {
    /// connect without system privileges
    Default,

    /// connect as [SYSDBA](https://docs.oracle.com/database/122/ADMQS/administering-user-accounts-and-security.htm#GUID-2033E766-8FE6-4FBA-97E0-2607B083FA2C)
    SYSDBA,

    /// connect as [SYSOPER](https://docs.oracle.com/database/122/ADMQS/administering-user-accounts-and-security.htm#GUID-2033E766-8FE6-4FBA-97E0-2607B083FA2C)
    SYSOPER,

    /// connect as [SYSASM](https://docs.oracle.com/database/122/OSTMG/authenticate-access-asm-instance.htm#OSTMG02600) (Oracle 12c or later)
    SYSASM,

    /// connect as [SYSBACKUP](https://docs.oracle.com/database/122/DBSEG/configuring-privilege-and-role-authorization.htm#DBSEG785) (Oracle 12c or later)
    SYSBACKUP,

    /// connect as [SYSDG](https://docs.oracle.com/database/122/DBSEG/configuring-privilege-and-role-authorization.htm#GUID-5798F976-85B2-4973-92F7-DB3F6BC9D497) (Oracle 12c or later)
    SYSDG,

    /// connect as [SYSKM](https://docs.oracle.com/database/122/DBSEG/configuring-privilege-and-role-authorization.htm#GUID-573B5831-E106-4D8C-9101-CF9C1B74A39C) (Oracle 12c or later)
    SYSKM,

    /// connect as [SYSRAC](https://docs.oracle.com/database/122/DBSEG/configuring-privilege-and-role-authorization.htm#DBSEG-GUID-69D0614C-D24E-4EC1-958A-79D7CCA3FA3A) (Oracle 12c R2 or later)
    SYSRAC,
}

/// Database startup mode
///
/// See [Connection.startup_database](struct.Connection.html#method.startup_database).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StartupMode {
    /// Shuts down a running instance (if there is any) using ABORT before
    /// starting a new one. This mode should be used only in unusual circumstances.
    Force,

    /// Allows database access only to users with both the CREATE SESSION
    /// and RESTRICTED SESSION privileges (normally, the DBA).
    Restrict,
}

/// Database shutdown mode
///
/// See [Connection.shutdown_database](struct.Connection.html#method.shutdown_database).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShutdownMode {
    /// Further connects are prohibited. Waits for users to disconnect from
    /// the database.
    Default,

    /// Further connects are prohibited and no new transactions are allowed.
    /// Waits for active transactions to complete.
    Transactional,

    /// Further connects are prohibited and no new transactions are allowed.
    /// Waits only for local transactions to complete.
    TransactionalLocal,

    /// Does not wait for current calls to complete or users to disconnect
    /// from the database. All uncommitted transactions are terminated and
    /// rolled back.
    Immediate,

    /// Does not wait for current calls to complete or users to disconnect
    /// from the database. All uncommitted transactions are terminated and
    /// are not rolled back. This is the fastest possible way to shut down
    /// the database, but the next database startup may require instance
    /// recovery. Therefore, this option should be used only in unusual
    /// circumstances; for example, if a background process terminates abnormally.
    Abort,

    /// Shuts down the database. Should be used only in the second call
    /// to [shutdown_database](struct.Connection.html#method.shutdown_database) after the database is closed and dismounted.
    Final,
}

#[doc(hidden)] // hiden until connection pooling is supported.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Purity {
    Default,
    New,
    Self_,
}

//
// Connector
//

/// Connection Builder
///
/// A connection is created by two methods. One is [Connection::new][].
/// The other is [connect method][]. Use the former to connect to a database
/// with username, password and connect_string. Use the latter when
/// additional parameters such as `SYSDBA` are required.
/// See examples below.
///
/// [Connection::new]: struct.Connection.html#method.new
/// [connect method]: #method.connect
pub struct Connector {
    username: String,
    password: String,
    connect_string: String,
    events: bool,
    edition: Option<String>,
    driver_name: Option<String>,
    auth_mode: AuthMode,
    prelim_auth: bool,
    connection_class: Option<String>,
    purity: Purity,
    new_password: Option<String>,
    app_context: Vec<String>,
    tag: Option<String>,
    match_any_tag: bool,
}

impl Connector {
    /// Creates a connection builder.
    pub fn new(username: &str, password: &str, connect_string: &str) -> Connector {
        Connector {
            username: username.to_string(),
            password: password.to_string(),
            connect_string: connect_string.to_string(),
            events: false,
            edition: None,
            driver_name: None,
            auth_mode: AuthMode::Default,
            prelim_auth: false,
            connection_class: None,
            purity: Purity::Default,
            new_password: None,
            app_context: Vec::new(),
            tag: None,
            match_any_tag: false,
        }
    }

    /// Establishes a connection.
    pub fn connect(&self) -> Result<Connection> {
        let ctxt = Context::get()?;
        let mut common_params = ctxt.common_create_params;
        let mut conn_params = ctxt.conn_create_params;

        if self.events {
            common_params.createMode |= DPI_MODE_CREATE_EVENTS;
        }
        if let Some(ref edition) = self.edition {
            let s = to_odpi_str(edition);
            common_params.edition = s.ptr;
            common_params.editionLength = s.len;
        }
        if let Some(ref name) = self.driver_name {
            let s = to_odpi_str(name);
            common_params.driverName = s.ptr;
            common_params.driverNameLength = s.len;
        }
        conn_params.authMode = match self.auth_mode {
            AuthMode::Default   => DPI_MODE_AUTH_DEFAULT,
            AuthMode::SYSDBA    => DPI_MODE_AUTH_SYSDBA,
            AuthMode::SYSOPER   => DPI_MODE_AUTH_SYSOPER,
            AuthMode::SYSASM    => DPI_MODE_AUTH_SYSASM,
            AuthMode::SYSBACKUP => DPI_MODE_AUTH_SYSBKP,
            AuthMode::SYSDG     => DPI_MODE_AUTH_SYSDGD,
            AuthMode::SYSKM     => DPI_MODE_AUTH_SYSKMT,
            AuthMode::SYSRAC    => DPI_MODE_AUTH_SYSRAC,
        };
        if self.prelim_auth {
            conn_params.authMode |= DPI_MODE_AUTH_PRELIM;
        }

        if let Some(ref name) = self.connection_class {
            let s = to_odpi_str(name);
            conn_params.connectionClass = s.ptr;
            conn_params.connectionClassLength = s.len;
        }
        if let Some(ref password) = self.new_password {
            let s = to_odpi_str(password);
            conn_params.newPassword = s.ptr;
            conn_params.newPasswordLength = s.len;
        }
        conn_params.purity = match self.purity {
            Purity::Default => DPI_PURITY_DEFAULT,
            Purity::New => DPI_PURITY_NEW,
            Purity::Self_ => DPI_PURITY_SELF,
        };
        let mut app_context = Vec::new();
        if !self.app_context.is_empty() {
            let n = self.app_context.len() / 3;
            app_context = Vec::with_capacity(n);
            for i in 0..n {
                let namespace = to_odpi_str(&self.app_context[i * 3 + 0]);
                let name = to_odpi_str(&self.app_context[i * 3 + 1]);
                let value = to_odpi_str(&self.app_context[i * 3 + 2]);
                app_context.push(
                    dpiAppContext {
                        namespaceName: namespace.ptr,
                        namespaceNameLength: namespace.len,
                        name: name.ptr,
                        nameLength: name.len,
                        value: value.ptr,
                        valueLength: value.len,
                    });
            }
        }
        if self.username.len() == 0 && self.password.len() == 0 {
            conn_params.externalAuth = 1;
        }
        if let Some(ref name) = self.tag {
            let s = to_odpi_str(name);
            conn_params.tag = s.ptr;
            conn_params.tagLength = s.len;
        }
        if self.match_any_tag {
            conn_params.matchAnyTag = 1;
        }
        conn_params.outTag = ptr::null();
        conn_params.outTagLength = 0;
        conn_params.outTagFound = 0;
        conn_params.appContext = app_context.as_mut_ptr();
        conn_params.numAppContext = app_context.len() as u32;
        Connection::connect_internal(ctxt, &self.username, &self.password, &self.connect_string, &common_params, &conn_params)
    }

    /// Sets a system privilege such as SYSDBA.
    ///
    /// ```no_run
    /// // same with `sqlplus system/manager as sysdba` on command line.
    /// let mut connector = oracle::Connector::new("system", "manager", "");
    /// connector.auth_mode(oracle::AuthMode::SYSDBA);
    /// let conn = connector.connect().unwrap();
    /// ```
    ///
    pub fn auth_mode<'a>(&'a mut self, auth_mode: AuthMode) -> &'a mut Connector {
        self.auth_mode = auth_mode;
        self
    }

    /// Sets prelim_auth mode. This is required to connect to an idle instance.
    ///
    /// This is required only when [starting up a database](struct.Connection.html#method.startup_database).
    pub fn prelim_auth<'a>(&'a mut self, prelim_auth: bool) -> &'a mut Connector {
        self.prelim_auth = prelim_auth;
        self
    }

    pub fn events<'a>(&'a mut self, events: bool) -> &'a mut Connector {
        self.events = events;
        self
    }

    // https://docs.oracle.com/database/122/ADFNS/editions.htm#ADFNS020
    pub fn edition<'a>(&'a mut self, edition: &str) -> &'a mut Connector {
        self.edition = Some(edition.to_string());
        self
    }

    /// Sets new password during establishing a connection.
    ///
    /// When a password is expired, you cannot connect to the user.
    /// A new password must be set by other user or set during establishing
    /// a connection.
    ///
    /// # Examples
    ///
    /// Connect to user `scott` with password `tiger`. If the password
    /// is expired, set a new password `jaguar`.
    ///
    /// ```no_run
    /// let conn = match oracle::Connection::new("scott", "tiger", "") {
    ///     Ok(conn) => conn,
    ///     Err(oracle::Error::OciError(ref dberr)) if dberr.code() == 28001 => {
    ///         // ORA-28001: the password has expired
    ///         let mut connector = oracle::Connector::new("scott", "tiger", "");
    ///         connector.new_password("jaguar");
    ///         connector.connect().unwrap()
    ///     },
    ///     Err(err) => panic!(err.to_string()),
    /// };
    /// ```
    pub fn new_password<'a>(&'a mut self, password: &str) -> &'a mut Connector {
        self.new_password = Some(password.to_string());
        self
    }

    /// Sets an application context.
    /// See [Oracle manual](https://docs.oracle.com/database/122/DBSEG/using-application-contexts-to-retrieve-user-information.htm#DBSEG165)
    ///
    /// This is same with [DBMS_SESSION.SET_CONTEXT][] but this can set application contexts before a connection is established.
    /// [DBMS_SESSION.SET_CONTEXT]: https://docs.oracle.com/database/122/ARPLS/DBMS_SESSION.htm#GUID-395C622C-ED79-44CC-9157-6A320934F2A9
    ///
    /// Examples:
    ///
    /// ```no_run
    /// let mut connector = oracle::Connector::new("scott", "tiger", "");
    /// connector.app_context("CLIENTCONTEXT", "foo", "bar");
    /// connector.app_context("CLIENTCONTEXT", "baz", "qux");
    /// let conn = connector.connect().unwrap();
    /// let mut stmt = conn.execute("select sys_context('CLIENTCONTEXT', 'baz') from dual", &[]).unwrap();
    /// let row = stmt.fetch().unwrap();
    /// let val: String = row.get(0).unwrap();
    /// assert_eq!(val, "qux");
    /// ```
    pub fn app_context<'a>(&'a mut self, namespace: &str, name: &str, value: &str) -> &'a mut Connector {
        self.app_context.reserve(3);
        self.app_context.push(namespace.to_string());
        self.app_context.push(name.to_string());
        self.app_context.push(value.to_string());
        self
    }

    // https://docs.oracle.com/database/122/ADFNS/performance-and-scalability.htm#ADFNS494
    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn purity<'a>(&'a mut self, purity: Purity) -> &'a mut Connector {
        self.purity = purity;
        self
    }

    // https://docs.oracle.com/database/122/ADFNS/performance-and-scalability.htm#GUID-EC3DEE61-512C-4CBB-A431-91894D0E1E37
    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn connection_class<'a>(&'a mut self, name: &str) -> &'a mut Connector {
        self.connection_class = Some(name.to_string());
        self
    }

    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn tag<'a>(&'a mut self, name: &str) -> &'a mut Connector {
        self.tag = Some(name.to_string());
        self
    }

    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn match_any_tag<'a>(&'a mut self, b: bool) -> &'a mut Connector {
        self.match_any_tag = b;
        self
    }

    /// Sets the driver name displayed in [V$SESSION_CONNECT_INFO.CLIENT_DRIVER][].
    ///
    /// The default value is "rust-oracle : version number". Only the first 8
    /// chracters "rust-ora" are displayed when the Oracle server version is
    /// lower than 12.0.1.2.
    ///
    /// [V$SESSION_CONNECT_INFO.CLIENT_DRIVER]: https://docs.oracle.com/database/122/REFRN/V-SESSION_CONNECT_INFO.htm
    pub fn driver_name<'a>(&'a mut self, name: &str) -> &'a mut Connector {
        self.driver_name = Some(name.to_string());
        self
    }
}

//
// Connection
//

/// Connection to an Oracle database
///
/// A connection is created by two methods. One is [Connection::new][].
/// The other is [Connector.connect][]. Use the former to connect to a database
/// with username, password and connect_string. Use the latter when
/// additional parameters such as `SYSDBA` are required.
///
/// [Connection::new]: #method.new
/// [Connector.connect]: struct.Connector.html#method.connect
pub struct Connection {
    pub(crate) ctxt: &'static Context,
    pub(crate) handle: *mut dpiConn,
    tag: String,
    tag_found: bool,
}

impl Connection {

    /// Connects to an Oracle database with username, password and connect_string.
    ///
    /// When you need to set more parameters before connecting to the server, see [Connector](struct.Connector.html).
    ///
    /// # Examples
    /// To connect to a local database.
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    /// ```
    ///
    /// To connect to a remote database specified by easy connect naming.
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "server_name:1521/service_name").unwrap();
    /// ```
    pub fn new(username: &str, password: &str, connect_string: &str) -> Result<Connection> {
        Connector::new(username, password, connect_string).connect()
    }

    /// Prepares a statement and returns it for subsequent execution/fetching
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (:id, :name)").unwrap();
    ///
    /// // insert one row. (set parameters by position)
    /// stmt.execute(&[&113, &"John"]).unwrap();
    ///
    /// // insert another row. (set parameters by name)
    /// stmt.execute_named(&[("id", &114),
    ///                      ("name", &"Smith")]).unwrap();
    /// ```
    pub fn prepare(&self, sql: &str) -> Result<Statement> {
        Statement::new(self, false, sql, "")
    }

    /// Prepares a statement, binds values by position and executes it in one call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    ///
    /// // execute a statement without bind parameters
    /// conn.execute("insert into emp(empno, ename) values (113, 'John')", &[]).unwrap();
    ///
    /// // execute a statement with binding parameters by position
    /// conn.execute("insert into emp(empno, ename) values (:1, :2)", &[&114, &"Smith"]).unwrap();
    ///
    /// ```
    pub fn execute(&self, sql: &str, params: &[&ToSql])-> Result<Statement> {
        let mut stmt = self.prepare(sql)?;
        stmt.execute(params)?;
        Ok(stmt)
    }

    /// Prepares a statement, binds values by name and executes it in one call.
    ///
    /// The bind variable names are compared case-insensitively.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    ///
    /// // execute a statement with binding parameters by name
    /// conn.execute_named("insert into emp(empno, ename) values (:id, :name)",
    ///                    &[("id", &114),
    ///                      ("name", &"Smith")]).unwrap();
    ///
    /// ```
    pub fn execute_named(&self, sql: &str, params: &[(&str, &ToSql)])-> Result<Statement> {
        let mut stmt = self.prepare(sql)?;
        stmt.execute_named(params)?;
        Ok(stmt)
    }

    /// Gets one row from a select statement in one call.
    ///
    /// This is same with the combination of [execute][], [fetch][] and [values].
    /// However the former is a bit optimized about memory usage.
    /// The former prepares memory for one row. On the other hand the latter
    /// internally prepares memory for 100 rows in order to reduce the number
    /// of network roundtrips when many rows are fetched.
    ///
    /// Type inference for the return type doesn't work. You need to specify
    /// it explicitly as `conn.select_one::<...>(sql_stmt, bind_parameters)`.
    /// See [ColumnValues][] for available return types.
    ///
    /// [execute][]: #method.execute
    /// [fetch][]: struct.Statement.html#method.fetch
    /// [values][]: struct.Row.html#method.values
    /// [ColumnValues][]: trait.ColumnValues.html
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    ///
    /// // fetch as a tuple whose type is `(i32, String)`.
    /// let sql = "select empno, ename from emp where empno = 7369";
    /// let tuple = conn.select_one::<(i32, String)>(sql, &[]).unwrap();
    /// assert_eq!(tuple.0, 7369);
    /// assert_eq!(tuple.1, "SMITH");
    ///
    /// // fetch same values using a destructuring let and a bind parameter.
    /// let sql = "select empno, ename from emp where empno = :1";
    /// let (empno, ename) = conn.select_one::<(i32, String)>(sql, &[&7369]).unwrap();
    /// assert_eq!(empno, 7369);
    /// assert_eq!(ename, "SMITH");
    ///
    /// ```
    pub fn select_one<T>(&self, sql: &str, params: &[&ToSql]) -> Result<<T>::Item> where T: ColumnValues {
        let mut stmt = self.prepare(sql)?;
        for i in 0..params.len() {
            stmt.bind(i + 1, params[i])?;
        }
        stmt.execute_internal(1)?;
        stmt.fetch()?.values::<T>()
    }

    /// Gets one row from a select statement with named bind parameters in one call.
    ///
    /// See [select_one][] for more detail.
    ///
    /// [select_one][]: #method.select_one
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    ///
    /// // fetch as a tuple whose type is `(i32, String)` with a named bind parameter "empno".
    /// let sql = "select empno, ename from emp where empno = :empno";
    /// let (empno, ename) = conn.select_one_named::<(i32, String)>(sql, &[("empno", &7369)]).unwrap();
    /// assert_eq!(empno, 7369);
    /// assert_eq!(ename, "SMITH");
    ///
    /// ```
    pub fn select_one_named<T>(&self, sql: &str, params: &[(&str, &ToSql)]) -> Result<<T>::Item> where T: ColumnValues {
        let mut stmt = self.prepare(sql)?;
        for i in 0..params.len() {
            stmt.bind(params[i].0, params[i].1)?;
        }
        stmt.execute_internal(1)?;
        stmt.fetch()?.values::<T>()
    }

    /// Cancels execution of running statements in the connection
    pub fn break_execution(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_breakExecution(self.handle));
        Ok(())
    }

    /// Commits the current active transaction
    pub fn commit(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_commit(self.handle));
        Ok(())
    }

    /// Rolls back the current active transaction
    pub fn rollback(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_rollback(self.handle));
        Ok(())
    }

    /// Closes the connection before the end of lifetime.
    ///
    /// This fails when open statements or LOBs exist.
    pub fn close(&self) -> Result<()> {
        self.close_internal(DPI_MODE_CONN_CLOSE_DEFAULT, "")
    }

    /// Gets information about the server version
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    /// let (version, banner) = conn.server_version().unwrap();
    /// println!("Oracle Version: {}", version);
    /// println!("--- Version Banner ---");
    /// println!("{}", banner);
    /// println!("---------------------");
    /// ```
    pub fn server_version(&self) -> Result<(Version, String)> {
        let mut s = new_odpi_str();
        let mut dpi_ver = Default::default();
        chkerr!(self.ctxt,
                dpiConn_getServerVersion(self.handle, &mut s.ptr, &mut s.len,
                                         &mut dpi_ver));
        Ok((Version::new_from_dpi_ver(dpi_ver), s.to_string()))
    }

    /// Changes the password for the specified user
    pub fn change_password(&self, username: &str, old_password: &str, new_password: &str) -> Result<()> {
        let username = to_odpi_str(username);
        let old_password = to_odpi_str(old_password);
        let new_password = to_odpi_str(new_password);
        chkerr!(self.ctxt,
                dpiConn_changePassword(self.handle,
                                       username.ptr, username.len,
                                       old_password.ptr, old_password.len,
                                       new_password.ptr, new_password.len));
        Ok(())
    }

    /// Pings the connection to see if it is still alive
    pub fn ping(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_ping(self.handle));
        Ok(())
    }

    //pub fn dpiConn_deqObject
    //pub fn dpiConn_enqObject

    /// Gets current schema associated with the connection
    pub fn current_schema(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getCurrentSchema(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    /// Sets current schema associated with the connection
    pub fn set_current_schema(&self, current_schema: &str) -> Result<()> {
        let s = to_odpi_str(current_schema);
        chkerr!(self.ctxt,
                dpiConn_setCurrentSchema(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// Gets edition associated with the connection
    pub fn edition(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getEdition(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    /// Gets external name associated with the connection
    pub fn external_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getExternalName(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    /// Sets external name associated with the connection
    pub fn set_external_name(&self, external_name: &str) -> Result<()> {
        let s = to_odpi_str(external_name);
        chkerr!(self.ctxt,
                dpiConn_setExternalName(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// Gets internal name associated with the connection
    pub fn internal_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getInternalName(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    /// Sets internal name associated with the connection
    pub fn set_internal_name(&self, internal_name: &str) -> Result<()> {
        let s = to_odpi_str(internal_name);
        chkerr!(self.ctxt,
                dpiConn_setInternalName(self.handle, s.ptr, s.len));
        Ok(())
    }

    //pub fn dpiConn_getLTXID
    //pub fn dpiConn_getObjectType

    /// Gets the statement cache size
    pub fn stmt_cache_size(&self) -> Result<u32> {
        let mut size = 0u32;
        chkerr!(self.ctxt,
                dpiConn_getStmtCacheSize(self.handle, &mut size));
        Ok(size)
    }

    /// Sets the statement cache size
    pub fn set_stmt_cache_size(&self, size: u32) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_setStmtCacheSize(self.handle, size));
        Ok(())
    }

    //pub fn dpiConn_newDeqOptions
    //pub fn dpiConn_newEnqOptions
    //pub fn dpiConn_newMsgProps
    //pub fn dpiConn_newSubscription
    //pub fn dpiConn_newTempLob
    //pub fn dpiConn_prepareDistribTrans

    /// Sets module associated with the connection
    ///
    /// This is same with calling [DBMS_APPLICATION_INFO.SET_MODULE][] but
    /// without executing a statement. The module name is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_APPLICATION_INFO.SET_MODULE]: https://docs.oracle.com/database/122/ARPLS/DBMS_APPLICATION_INFO.htm#GUID-B2E2BD20-D91D-40DB-A3F6-37A853384F30
    pub fn set_module(&self, module: &str) -> Result<()> {
        let s = to_odpi_str(module);
        chkerr!(self.ctxt,
                dpiConn_setModule(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// Sets action associated with the connection
    ///
    /// This is same with calling [DBMS_APPLICATION_INFO.SET_ACTION][] but
    /// without executing a statement. The action name is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_APPLICATION_INFO.SET_ACTION]: https://docs.oracle.com/database/122/ARPLS/DBMS_APPLICATION_INFO.htm#GUID-90DA860F-BFBE-4539-BA00-2279B02B8F26
    pub fn set_action(&self, action: &str) -> Result<()> {
        let s = to_odpi_str(action);
        chkerr!(self.ctxt,
                dpiConn_setAction(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// Sets client info associated with the connection
    ///
    /// This is same with calling [DBMS_APPLICATION_INFO.SET_CLIENT_INFO][] but
    /// without executing a statement. The client info is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_APPLICATION_INFO.SET_CLIENT_INFO]: https://docs.oracle.com/database/122/ARPLS/DBMS_APPLICATION_INFO.htm#GUID-68A3DF04-BE91-46CC-8D2B-97BA0E89956F
    pub fn set_client_info(&self, client_info: &str) -> Result<()> {
        let s = to_odpi_str(client_info);
        chkerr!(self.ctxt,
                dpiConn_setClientInfo(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// Sets client identifier associated with the connection
    ///
    /// This is same with calling [DBMS_SESSION.SET_IDENTIFIER][] but
    /// without executing a statement. The client identifier is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_SESSION.SET_IDENTIFIER]: https://docs.oracle.com/database/122/ARPLS/DBMS_SESSION.htm#GUID-988EA930-BDFE-4205-A806-E54F05333562
    pub fn set_client_identifier(&self, client_identifier: &str) -> Result<()> {
        let s = to_odpi_str(client_identifier);
        chkerr!(self.ctxt,
                dpiConn_setClientIdentifier(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// Sets name of the database operation to be monitored in the database.
    /// Sets to `''` if you want to end monitoring the current running database operation.
    ///
    /// This is same with calling [DBMS_SQL_MONITOR.BEGIN_OPERATION][] but
    /// without executing a statement. The database operation name is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// See [Monitoring Database Operations][] in Oracle Database SQL Tuning Guide
    ///
    /// [db_op]: https://docs.oracle.com/database/121/TGSQL/glossary.htm#GUID-EB7D5D0A-0439-4336-8DC3-2DA24072977F
    /// [DBMS_SQL_MONITOR.BEGIN_OPERATION]: https://docs.oracle.com/database/122/ARPLS/DBMS_SQL_MONITOR.htm#ARPLS74785
    /// [Monitoring Database Operations]: https://docs.oracle.com/database/122/TGSQL/monitoring-database-operations.htm#TGSQL-GUID-C941CE9D-97E1-42F8-91ED-4949B2B710BF
    pub fn set_db_op(&self, db_op: &str) -> Result<()> {
        let s = to_odpi_str(db_op);
        chkerr!(self.ctxt,
                dpiConn_setDbOp(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// Gets an object type information from name
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    /// let objtype = conn.object_type("MDSYS.SDO_GEOMETRY");
    /// ```
    pub fn object_type(&self, name: &str) -> Result<ObjectType> {
        let name = to_odpi_str(name);
        let mut handle = ptr::null_mut();
        chkerr!(self.ctxt,
                dpiConn_getObjectType(self.handle, name.ptr, name.len, &mut handle));
        let res = ObjectType::from_dpiObjectType(self.ctxt, handle);
        unsafe { dpiObjectType_release(handle); }
        res
    }

    /// Starts up a database
    ///
    /// This corresponds to sqlplus command `startup nomount`.
    /// You need to connect the databas as system privilege in prelim_auth
    /// mode in advance.
    /// After this method is executed, you need to reconnect the server
    /// as system privilege *without* prelim_auth and executes
    /// `alter database mount` and then `alter database open`.
    ///
    /// # Examples
    ///
    /// Connect to an idle instance as sysdba and start up a database
    ///
    /// ```no_run
    /// use oracle::{Connector, AuthMode};
    /// // connect to an idle instance
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///              .prelim_auth(true) // required to connect to an idle instance
    ///              .auth_mode(AuthMode::SYSDBA) // connect as sysdba
    ///              .connect().unwrap();
    ///
    /// // start the instance
    /// conn.startup_database(&[]).unwrap();
    /// conn.close().unwrap();
    ///
    /// // connect again without prelim_auth
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///              .auth_mode(AuthMode::SYSDBA) // connect as sysdba
    ///              .connect().unwrap();
    ///
    /// // mount and open a database
    /// conn.execute("alter database mount", &[]).unwrap();
    /// conn.execute("alter database open", &[]).unwrap();
    /// ```
    ///
    /// Start up a database in restricted mode
    ///
    /// ```ignore
    /// ...
    /// conn.startup_database(&[StartupMode::Restrict]).unwrap();
    /// ...
    /// ```
    ///
    /// If the database is running, shut it down with mode ABORT and then
    /// start up in restricted mode
    ///
    /// ```ignore
    /// ...
    /// conn.startup_database(&[StartupMode::Force, StartupMode::Restrict]).unwrap();
    /// ...
    /// ```
    pub fn startup_database(&self, modes: &[StartupMode]) -> Result<()> {
        let mut mode_num = 0;
        for mode in modes {
            mode_num |= match *mode {
                StartupMode::Force => DPI_MODE_STARTUP_FORCE,
                StartupMode::Restrict => DPI_MODE_STARTUP_RESTRICT,
            };
        }
        chkerr!(self.ctxt,
                dpiConn_startupDatabase(self.handle, mode_num));
        Ok(())
    }

    /// Shuts down a database
    ///
    /// When this method is called with [ShutdownMode::Default][],
    /// [ShutdownMode::Transactional][], [ShutdownMode::TransactionalLocal][]
    /// or [ShutdownMode::Immediate], execute "alter database close normal"
    /// and "alter database dismount" and call this method again with
    /// [ShutdownMode::Final].
    ///
    /// When this method is called with [ShutdownMode::Abort][],
    /// the database is aborted immediately.
    ///
    /// [ShutdownMode::Default]: enum.ShutdownMode.html#variant.Default
    /// [ShutdownMode::Transactional]: enum.ShutdownMode.html#variant.Transactional
    /// [ShutdownMode::TransactionalLocal]: enum.ShutdownMode.html#variant.TransactionalLocal
    /// [ShutdownMode::Immediate]: enum.ShutdownMode.html#variant.Immediate
    /// [ShutdownMode::Abort]: enum.ShutdownMode.html#variant.Abort
    /// [ShutdownMode::Final]: enum.ShutdownMode.html#variant.Final
    ///
    /// # Examples
    ///
    /// Same with `shutdown immediate` on sqlplus.
    ///
    /// ```no_run
    /// use oracle::{Connector, AuthMode, ShutdownMode};
    /// // connect
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///              .auth_mode(AuthMode::SYSDBA) // connect as sysdba
    ///              .connect().unwrap();
    ///
    /// // begin 'shutdown immediate'
    /// conn.shutdown_database(ShutdownMode::Immediate).unwrap();
    ///
    /// // close and dismount the database
    /// conn.execute("alter database close normal", &[]).unwrap();
    /// conn.execute("alter database dismount", &[]).unwrap();
    ///
    /// // finish shutdown
    /// conn.shutdown_database(oracle::ShutdownMode::Final).unwrap();
    /// ```
    ///
    /// Same with `shutdown abort` on sqlplus.
    ///
    /// ```no_run
    /// use oracle::{Connector, AuthMode, ShutdownMode};
    /// // connect
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///              .auth_mode(AuthMode::SYSDBA) // connect as sysdba
    ///              .connect().unwrap();
    ///
    /// // 'shutdown abort'
    /// conn.shutdown_database(ShutdownMode::Abort).unwrap();
    ///
    /// // The database is aborted here.
    /// ```
    pub fn shutdown_database(&self, mode: ShutdownMode) -> Result<()> {
        let mode = match mode {
            ShutdownMode::Default => DPI_MODE_SHUTDOWN_DEFAULT,
            ShutdownMode::Transactional => DPI_MODE_SHUTDOWN_TRANSACTIONAL,
            ShutdownMode::TransactionalLocal => DPI_MODE_SHUTDOWN_TRANSACTIONAL_LOCAL,
            ShutdownMode::Immediate => DPI_MODE_SHUTDOWN_IMMEDIATE,
            ShutdownMode::Abort => DPI_MODE_SHUTDOWN_ABORT,
            ShutdownMode::Final => DPI_MODE_SHUTDOWN_FINAL,
        };
        chkerr!(self.ctxt,
                dpiConn_shutdownDatabase(self.handle, mode));
        Ok(())
    }

    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn tag(&self) -> &String {
        &self.tag
    }

    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn tag_found(&self) -> bool {
        self.tag_found
    }

    pub(crate) fn connect_internal(ctxt: &'static Context, username: &str, password: &str, connect_string: &str, common_param: &dpiCommonCreateParams, conn_param: &dpiConnCreateParams) -> Result<Connection> {
        let username = to_odpi_str(username);
        let password = to_odpi_str(password);
        let connect_string = to_odpi_str(connect_string);
        let mut param = *conn_param;
        let mut handle = ptr::null_mut();
        chkerr!(ctxt,
                dpiConn_create(ctxt.context, username.ptr, username.len,
                               password.ptr, password.len, connect_string.ptr,
                               connect_string.len, common_param,
                               &mut param, &mut handle));
        Ok(Connection{
            ctxt: ctxt,
            handle: handle,
            tag: to_rust_str(conn_param.outTag, conn_param.outTagLength),
            tag_found: conn_param.outTagFound != 0,
        })
    }

    fn close_internal(&self, mode: dpiConnCloseMode, tag: &str) -> Result<()> {
        let tag = to_odpi_str(tag);
        chkerr!(self.ctxt,
                dpiConn_close(self.handle, mode, tag.ptr, tag.len));
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = unsafe { dpiConn_release(self.handle) };
    }
}
