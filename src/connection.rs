// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2019 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fmt;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::binding::*;
use crate::chkerr;
use crate::new_odpi_str;
use crate::oci_attr::data_type::{AttrValue, DataType};
use crate::oci_attr::handle::ConnHandle;
use crate::oci_attr::handle::Server;
use crate::oci_attr::mode::Read;
use crate::oci_attr::mode::{ReadMode, WriteMode};
use crate::oci_attr::OciAttr;
use crate::sql_type::ObjectType;
use crate::sql_type::ObjectTypeInternal;
use crate::sql_type::ToSql;
use crate::to_odpi_str;
use crate::to_rust_str;
use crate::util::duration_to_msecs;
use crate::AssertSend;
use crate::AssertSync;
#[cfg(doc)]
use crate::Batch;
use crate::BatchBuilder;
use crate::Context;
use crate::DpiConn;
use crate::DpiObjectType;
use crate::Error;
use crate::Result;
use crate::ResultSet;
use crate::Row;
use crate::RowValue;
use crate::Statement;
use crate::StatementBuilder;
use crate::StmtParam;
use crate::Version;

struct ServerStatus;
const OCI_ATTR_SERVER_STATUS: u32 = 143;
const OCI_SERVER_NOT_CONNECTED: u32 = 0;
const OCI_SERVER_NORMAL: u32 = 1;
unsafe impl OciAttr for ServerStatus {
    type HandleType = Server;
    type Mode = Read;
    type DataType = u32;
    const ATTR_NUM: u32 = OCI_ATTR_SERVER_STATUS;
}

/// Database startup mode
///
/// See [`Connection::startup_database`]
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
/// See [`Connection::shutdown_database`].
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
    /// to [`Connection::shutdown_database`] after the database is closed and dismounted.
    Final,
}

/// [Administrative privilege](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-633842B8-4B19-4F96-A757-783BF62825A7)
///
/// See [`Connector::privilege`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Privilege {
    /// Connects as [SYSDBA](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-BD5D39D1-DBFF-400A-8645-355F8FB9CD31).
    ///
    Sysdba,

    /// Connects as [SYSOPER](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-BD5D39D1-DBFF-400A-8645-355F8FB9CD31).
    Sysoper,

    /// Connects as [SYSASM](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-7396FD18-628B-4026-AA55-79C6D6205EAE) (Oracle 12c or later)
    Sysasm,

    /// Connects as [SYSBACKUP](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-BF12E37F-4606-42BB-B8B6-4CDC5A870EE7)
    Sysbackup,

    /// Connects as [SYSDG](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-5798F976-85B2-4973-92F7-DB3F6BC9D497) (Oracle 12c or later)
    Sysdg,

    /// Connects as [SYSKM](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-573B5831-E106-4D8C-9101-CF9C1B74A39C) (Oracle 12c or later)
    Syskm,

    /// Connects as [SYSRAC](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-69D0614C-D24E-4EC1-958A-79D7CCA3FA3A) (Oracle 12c R2 or later)
    Sysrac,
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// [Session Purity](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-12410EEC-FE79-42E2-8F6B-EAA9EDA59665)
pub enum Purity {
    /// Must use a new session
    New,
    /// Reuse a pooled session
    Self_,
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Connection status
pub enum ConnStatus {
    /// The connection is alive. See [`Connection::status`] for details.
    Normal,
    /// The connection has been terminated. See [`Connection::status`] for details.
    NotConnected,
    /// The connection has been closed by [`Connection::close`].
    Closed,
}

/// Builder data type to create Connection.
///
/// When a connection can be established only with username, password
/// and connect string, use [`Connection::connect`] instead.
#[derive(Debug, Clone, PartialEq)]
pub struct Connector {
    username: String,
    password: String,
    connect_string: String,
    privilege: Option<Privilege>,
    external_auth: bool,
    prelim_auth: bool,
    new_password: String,
    purity: Option<Purity>,
    connection_class: String,
    app_context: Vec<(String, String, String)>,
    tag: String,
    match_any_tag: bool,
    events: bool,
    edition: String,
    driver_name: String,
}

impl Connector {
    /// Create a connector
    pub fn new<U, P, C>(username: U, password: P, connect_string: C) -> Connector
    where
        U: Into<String>,
        P: Into<String>,
        C: Into<String>,
    {
        Connector {
            username: username.into(),
            password: password.into(),
            connect_string: connect_string.into(),
            privilege: None,
            external_auth: false,
            prelim_auth: false,
            new_password: "".into(),
            purity: None,
            connection_class: "".into(),
            app_context: vec![],
            tag: "".into(),
            match_any_tag: false,
            events: false,
            edition: "".into(),
            driver_name: "".into(),
        }
    }

    /// Set [administrative privilege](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-633842B8-4B19-4F96-A757-783BF62825A7).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// // connect system/manager as sysdba
    /// let conn = Connector::new("system", "manager", "")
    ///     .privilege(Privilege::Sysdba)
    ///     .connect()?;
    /// # Ok::<(), Error>(())
    /// ```
    pub fn privilege(&mut self, privilege: Privilege) -> &mut Connector {
        self.privilege = Some(privilege);
        self
    }

    /// Uses external authentication such as [OS authentication][].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connector::new("", "", "")
    ///     .external_auth(true)
    ///     .connect()?;
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// [OS authentication]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-37BECE32-58D5-43BF-A098-97936D66968F
    pub fn external_auth(&mut self, b: bool) -> &mut Connector {
        self.external_auth = b;
        self
    }

    /// Sets prelim auth mode to connect to an idle instance.
    ///
    /// See [starting up a database](Connection::startup_database).
    pub fn prelim_auth(&mut self, b: bool) -> &mut Connector {
        self.prelim_auth = b;
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
    /// # use oracle::*;
    /// let conn = match Connection::connect("scott", "tiger", "") {
    ///     Ok(conn) => conn,
    ///     Err(Error::OciError(dberr)) if dberr.code() == 28001 => {
    ///         // ORA-28001: the password has expired
    ///         Connector::new("scott", "tiger", "")
    ///             .new_password("jaguar")
    ///             .connect()?
    ///     }
    ///     Err(err) => return Err(err),
    /// };
    /// # Ok::<(), Error>(())
    /// ```
    pub fn new_password<P>(&mut self, password: P) -> &mut Connector
    where
        P: Into<String>,
    {
        self.new_password = password.into();
        self
    }

    /// Sets session purity specifying whether an application can reuse a pooled session (`Purity::Self_`) or must use a new session (`Purity::New`) from [DRCP][] pooled sessions.
    ///
    /// [DRCP]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-015CA8C1-2386-4626-855D-CC546DDC1086
    pub fn purity(&mut self, purity: Purity) -> &mut Connector {
        self.purity = Some(purity);
        self
    }

    /// Sets a connection class to restrict sharing [DRCP][] pooled sessions.
    ///
    /// See [here][] for more detail.
    ///
    /// [DRCP]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-015CA8C1-2386-4626-855D-CC546DDC1086
    /// [here]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-EC3DEE61-512C-4CBB-A431-91894D0E1E37
    pub fn connection_class<S>(&mut self, connection_class: S) -> &mut Connector
    where
        S: Into<String>,
    {
        self.connection_class = connection_class.into();
        self
    }

    /// Appends an application context.
    ///
    /// See [Oracle manual](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-5841261E-988F-4A56-A2B4-71114AB3D51D)
    ///
    /// This is same with [DBMS_SESSION.SET_CONTEXT][] but this can set application contexts before a connection is established.
    ///
    /// [DBMS_SESSION.SET_CONTEXT]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-395C622C-ED79-44CC-9157-6A320934F2A9
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connector::new("scott", "tiger", "")
    ///               .app_context("CLIENTCONTEXT", "foo", "bar")
    ///               .app_context("CLIENTCONTEXT", "baz", "qux")
    ///               .connect()?;
    /// let val = conn.query_row_as::<String>("select sys_context('CLIENTCONTEXT', 'foo') from dual", &[])?;
    /// assert_eq!(val, "bar");
    /// let val = conn.query_row_as::<String>("select sys_context('CLIENTCONTEXT', 'baz') from dual", &[])?;
    /// assert_eq!(val, "qux");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn app_context<T1, T2, T3>(&mut self, namespace: T1, name: T2, value: T3) -> &mut Connector
    where
        T1: Into<String>,
        T2: Into<String>,
        T3: Into<String>,
    {
        self.app_context
            .push((namespace.into(), name.into(), value.into()));
        self
    }

    /// Reserved for when connection pooling is supported.
    pub fn tag<S>(&mut self, tag: S) -> &mut Connector
    where
        S: Into<String>,
    {
        self.tag = tag.into();
        self
    }

    /// Reserved for when connection pooling is supported.
    pub fn match_any_tag(&mut self, b: bool) -> &mut Connector {
        self.match_any_tag = b;
        self
    }

    /// Reserved for when advanced queuing (AQ) or continuous query
    /// notification (CQN) is supported.
    pub fn events(&mut self, b: bool) -> &mut Connector {
        self.events = b;
        self
    }

    /// Specifies edition of [Edition-Based Redefinition][].
    ///
    /// [Edition-Based Redefinition]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-58DE05A0-5DEF-4791-8FA8-F04D11964906
    pub fn edition<S>(&mut self, edition: S) -> &mut Connector
    where
        S: Into<String>,
    {
        self.edition = edition.into();
        self
    }

    /// Sets the driver name displayed in [V$SESSION_CONNECT_INFO.CLIENT_DRIVER][].
    ///
    /// The default value is "rust-oracle : version number". Only the first 8
    /// chracters "rust-ora" are displayed when the Oracle server version is
    /// lower than 12.0.1.2.
    ///
    /// [V$SESSION_CONNECT_INFO.CLIENT_DRIVER]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-9F0DCAEA-A67E-4183-89E7-B1555DC591CE
    pub fn driver_name<S>(&mut self, driver_name: S) -> &mut Connector
    where
        S: Into<String>,
    {
        self.driver_name = driver_name.into();
        self
    }

    /// Connect an Oracle server using specified parameters
    pub fn connect(&self) -> Result<Connection> {
        let ctxt = Context::get()?;
        let mut common_params = ctxt.common_create_params();
        let mut conn_params = ctxt.conn_create_params();

        if let Some(ref privilege) = self.privilege {
            conn_params.authMode |= match privilege {
                &Privilege::Sysdba => DPI_MODE_AUTH_SYSDBA,
                &Privilege::Sysoper => DPI_MODE_AUTH_SYSOPER,
                &Privilege::Sysasm => DPI_MODE_AUTH_SYSASM,
                &Privilege::Sysbackup => DPI_MODE_AUTH_SYSBKP,
                &Privilege::Sysdg => DPI_MODE_AUTH_SYSDGD,
                &Privilege::Syskm => DPI_MODE_AUTH_SYSKMT,
                &Privilege::Sysrac => DPI_MODE_AUTH_SYSRAC,
            };
        }
        if self.external_auth {
            conn_params.externalAuth = 1;
        }
        if self.prelim_auth {
            conn_params.authMode |= DPI_MODE_AUTH_PRELIM;
        }
        let s = to_odpi_str(&self.new_password);
        conn_params.newPassword = s.ptr;
        conn_params.newPasswordLength = s.len;
        if let Some(purity) = self.purity {
            conn_params.purity = match purity {
                Purity::New => DPI_PURITY_NEW,
                Purity::Self_ => DPI_PURITY_SELF,
            };
        }
        let s = to_odpi_str(&self.connection_class);
        conn_params.connectionClass = s.ptr;
        conn_params.connectionClassLength = s.len;
        let mut app_context = Vec::with_capacity(self.app_context.len());
        for ac in &self.app_context {
            let namespace = to_odpi_str(&ac.0);
            let name = to_odpi_str(&ac.1);
            let value = to_odpi_str(&ac.2);
            app_context.push(dpiAppContext {
                namespaceName: namespace.ptr,
                namespaceNameLength: namespace.len,
                name: name.ptr,
                nameLength: name.len,
                value: value.ptr,
                valueLength: value.len,
            });
        }
        if app_context.len() != 0 {
            conn_params.appContext = app_context.as_mut_ptr();
            conn_params.numAppContext = app_context.len() as u32;
        }
        let s = to_odpi_str(&self.tag);
        conn_params.tag = s.ptr;
        conn_params.tagLength = s.len;
        if self.match_any_tag {
            conn_params.matchAnyTag = 1;
        }
        if self.events {
            common_params.createMode |= DPI_MODE_CREATE_EVENTS;
        }
        let s = to_odpi_str(&self.edition);
        common_params.edition = s.ptr;
        common_params.editionLength = s.len;
        let s = to_odpi_str(&self.driver_name);
        common_params.driverName = s.ptr;
        common_params.driverNameLength = s.len;
        Connection::connect_internal(
            &self.username,
            &self.password,
            &self.connect_string,
            Some(common_params),
            Some(conn_params),
        )
    }
}

pub(crate) type Conn = Arc<InnerConn>;

pub(crate) struct InnerConn {
    pub(crate) ctxt: &'static Context,
    pub(crate) handle: DpiConn,
    pub(crate) autocommit: Mutex<bool>,
    pub(crate) objtype_cache: Mutex<HashMap<String, Arc<ObjectTypeInternal>>>,
    tag: String,
    tag_found: bool,
}

impl InnerConn {
    pub fn new(
        ctxt: &'static Context,
        handle: *mut dpiConn,
        conn_params: &dpiConnCreateParams,
    ) -> InnerConn {
        InnerConn {
            ctxt: ctxt,
            handle: DpiConn::new(handle),
            autocommit: Mutex::new(false),
            objtype_cache: Mutex::new(HashMap::new()),
            tag: to_rust_str(conn_params.outTag, conn_params.outTagLength),
            tag_found: conn_params.outTagFound != 0,
        }
    }

    pub fn autocommit(&self) -> bool {
        *self.autocommit.lock().unwrap()
    }

    pub fn clear_object_type_cache(&self) -> Result<()> {
        self.objtype_cache.lock()?.clear();
        Ok(())
    }
}

impl fmt::Debug for InnerConn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Conn {{ handle: {:?}, autocommit: {:?}",
            self.handle.raw(),
            self.autocommit,
        )?;
        if self.tag.len() != 0 {
            write!(f, ", tag: {:?}", self.tag)?;
        }
        if self.tag_found {
            write!(f, ", tag_found: {:?}", self.tag_found)?;
        }
        write!(f, " }}")
    }
}

/// Connection to an Oracle database
pub struct Connection {
    pub(crate) conn: Conn,
}

impl AssertSync for Connection {}
impl AssertSend for Connection {}

impl Connection {
    /// Connects to an Oracle server using username, password and connect string.
    ///
    /// If you need to connect the server with additional parameters
    /// such as SYSDBA privilege, use [`Connector`] instead.
    ///
    /// # Examples
    /// Connect to a local database.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Connect to a remote database specified by easy connect naming.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger",
    ///                                "server_name:1521/service_name")?;
    /// # Ok::<(), Error>(())
    /// ```
    pub fn connect<U, P, C>(username: U, password: P, connect_string: C) -> Result<Connection>
    where
        U: AsRef<str>,
        P: AsRef<str>,
        C: AsRef<str>,
    {
        Connection::connect_internal(
            username.as_ref(),
            password.as_ref(),
            connect_string.as_ref(),
            None,
            None,
        )
    }

    pub(crate) fn connect_internal(
        username: &str,
        password: &str,
        connect_string: &str,
        common_params: Option<dpiCommonCreateParams>,
        conn_params: Option<dpiConnCreateParams>,
    ) -> Result<Connection> {
        let ctxt = Context::get()?;
        let common_params = common_params.unwrap_or(ctxt.common_create_params());
        let mut conn_params = conn_params.unwrap_or(ctxt.conn_create_params());
        let username = to_odpi_str(username);
        let password = to_odpi_str(password);
        let connect_string = to_odpi_str(connect_string);
        let mut handle = ptr::null_mut();
        chkerr!(
            ctxt,
            dpiConn_create(
                ctxt.context,
                username.ptr,
                username.len,
                password.ptr,
                password.len,
                connect_string.ptr,
                connect_string.len,
                &common_params,
                &mut conn_params,
                &mut handle
            )
        );
        Ok(Connection {
            conn: Arc::new(InnerConn::new(ctxt, handle, &conn_params)),
        })
    }

    pub(crate) fn from_conn(conn: Conn) -> Connection {
        Connection { conn: conn }
    }

    pub(crate) fn ctxt(&self) -> &'static Context {
        self.conn.ctxt
    }

    pub(crate) fn handle(&self) -> *mut dpiConn {
        self.conn.handle.raw()
    }

    /// Closes the connection before the end of lifetime.
    ///
    /// This fails when open statements or LOBs exist.
    pub fn close(&self) -> Result<()> {
        self.close_internal(DPI_MODE_CONN_CLOSE_DEFAULT, "")
    }

    /// Creates [`StatementBuilder`][] to create a [`Statement`][]
    ///
    /// # Examples
    ///
    /// Executes a SQL statement with different parameters.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// # let conn = Connection::connect("scott", "tiger", "")?;
    /// let mut stmt = conn.statement("insert into emp(empno, ename) values (:id, :name)").build()?;
    ///
    /// let emp_list = [
    ///     (7369, "Smith"),
    ///     (7499, "Allen"),
    ///     (7521, "Ward"),
    /// ];
    ///
    /// // insert rows using positional parameters
    /// for emp in &emp_list {
    ///    stmt.execute(&[&emp.0, &emp.1])?;
    /// }
    ///
    /// let emp_list = [
    ///     (7566, "Jones"),
    ///     (7654, "Martin"),
    ///     (7698, "Blake"),
    /// ];
    ///
    /// // insert rows using named parameters
    /// for emp in &emp_list {
    ///    stmt.execute_named(&[("id", &emp.0), ("name", &emp.1)])?;
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Query methods in Connection allocate memory for 100 rows by default
    /// to reduce the number of network round trips in case that many rows are
    /// fetched. When 100 isn't preferable, use [`StatementBuilder::fetch_array_size`]
    /// to customize it.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// # let conn = Connection::connect("scott", "tiger", "")?;
    /// // fetch top 10 rows.
    /// let mut stmt = conn
    ///     .statement("select empno, ename from emp order by empno fetch first 10 rows only")
    ///     .fetch_array_size(10)
    ///     .build()?;
    /// for row_result in stmt.query_as::<(i32, String)>(&[])? {
    ///     let (empno, ename) = row_result?;
    ///     println!("empno: {}, ename: {}", empno, ename);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// By default, a maximum of 2 rows are returned when the query is first
    /// executed. To modify this, use [`StatementBuilder::prefetch_rows`] to customize
    /// it. For more information on the difference between this and `fetch_array_size`,
    /// see [this writeup](https://blog.dbi-services.com/arraysize-or-rowprefetch-in-sqlplus/)
    /// or [this description](https://oracle.github.io/node-oracledb/doc/api.html#rowfetching).
    ///
    /// ```no_run
    /// # use oracle::*;
    /// # let conn = Connection::connect("scott", "tiger", "")?;
    /// // fetch top 10 rows.
    /// let mut stmt = conn
    ///     .statement("select empno, ename from emp order by empno fetch first 10 rows only")
    ///     .prefetch_rows(11)  // add one to avoid a round-trip to check for end-of-fetch
    ///     .build()?;
    /// for row_result in stmt.query_as::<(i32, String)>(&[])? {
    ///     let (empno, ename) = row_result?;
    ///     println!("empno: {}, ename: {}", empno, ename);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn statement<'conn, 'sql>(&'conn self, sql: &'sql str) -> StatementBuilder<'conn, 'sql> {
        StatementBuilder::new(self, sql)
    }

    /// Creates a [`Statement`][] with parameters
    ///
    /// This will be marked as [deprecated][] at version 0.6.x. Please use [`statement(sql).build()`](#method.statement)
    /// instead. The `statement()` method was added to follow [this API guideline](https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder).
    ///
    /// [deprecated]: https://doc.rust-lang.org/edition-guide/rust-2018/the-compiler/an-attribute-for-deprecation.html
    ///
    /// # Examples
    ///
    /// Executes a SQL statement with different parameters.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// # let conn = Connection::connect("scott", "tiger", "")?;
    /// let mut stmt = conn.prepare("insert into emp(empno, ename) values (:id, :name)", &[])?;
    ///
    /// let emp_list = [
    ///     (7369, "Smith"),
    ///     (7499, "Allen"),
    ///     (7521, "Ward"),
    /// ];
    ///
    /// // insert rows using positional parameters
    /// for emp in &emp_list {
    ///    stmt.execute(&[&emp.0, &emp.1])?;
    /// }
    ///
    /// let emp_list = [
    ///     (7566, "Jones"),
    ///     (7654, "Martin"),
    ///     (7698, "Blake"),
    /// ];
    ///
    /// // insert rows using named parameters
    /// for emp in &emp_list {
    ///    stmt.execute_named(&[("id", &emp.0), ("name", &emp.1)])?;
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Query methods in Connection allocate memory for 100 rows by default
    /// to reduce the number of network round trips in case that many rows are
    /// fetched. When 100 isn't preferable, use `StmtParam::FetchArraySize(u32)`
    /// to customize it.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// # let conn = Connection::connect("scott", "tiger", "")?;
    /// // fetch top 10 rows.
    /// let mut stmt = conn.prepare("select * from (select empno, ename from emp order by empno) where rownum <= 10",
    ///                             &[StmtParam::FetchArraySize(10)])?;
    /// for row_result in stmt.query_as::<(i32, String)>(&[])? {
    ///     let (empno, ename) = row_result?;
    ///     println!("empno: {}, ename: {}", empno, ename);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    ///
    pub fn prepare(&self, sql: &str, params: &[StmtParam]) -> Result<Statement> {
        Statement::from_params(self, sql, params)
    }

    /// Creates [BatchBuilder][]
    ///
    /// See [`Batch`].
    pub fn batch<'conn, 'sql>(
        &'conn self,
        sql: &'sql str,
        max_batch_size: usize,
    ) -> BatchBuilder<'conn, 'sql> {
        BatchBuilder::new(self, sql, max_batch_size)
    }

    /// Executes a select statement and returns a result set containing [`Row`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query(&self, sql: &str, params: &[&dyn ToSql]) -> Result<ResultSet<Row>> {
        let mut stmt = self.statement(sql).build()?;
        stmt.exec(params, true, "query")?;
        Ok(ResultSet::<Row>::from_stmt(stmt.stmt))
    }

    /// Executes a select statement using named parameters and returns a result set containing [`Row`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_named(&self, sql: &str, params: &[(&str, &dyn ToSql)]) -> Result<ResultSet<Row>> {
        let mut stmt = self.statement(sql).build()?;
        stmt.exec_named(params, true, "query_named")?;
        Ok(ResultSet::<Row>::from_stmt(stmt.stmt))
    }

    /// Executes a select statement and returns a result set containing [`RowValue`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_as<T>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<ResultSet<T>>
    where
        T: RowValue,
    {
        let mut stmt = self.statement(sql).build()?;
        stmt.exec(params, true, "query_as")?;
        Ok(ResultSet::<T>::from_stmt(stmt.stmt))
    }

    /// Executes a select statement using named parameters and returns a result set containing [`RowValue`]s.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_as_named<T>(
        &self,
        sql: &str,
        params: &[(&str, &dyn ToSql)],
    ) -> Result<ResultSet<T>>
    where
        T: RowValue,
    {
        let mut stmt = self.statement(sql).build()?;
        stmt.exec_named(params, true, "query_as_named")?;
        Ok(ResultSet::<T>::from_stmt(stmt.stmt))
    }

    /// Gets one row from a query using positoinal bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Row> {
        let mut stmt = self.statement(sql).fetch_array_size(1).build()?;
        if let Err(err) = stmt.query_row(params) {
            return Err(err);
        };
        Ok(mem::replace(&mut stmt.stmt.row, None).unwrap())
    }

    /// Gets one row from a query using named bind parameters.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row_named(&self, sql: &str, params: &[(&str, &dyn ToSql)]) -> Result<Row> {
        let mut stmt = self.statement(sql).fetch_array_size(1).build()?;
        if let Err(err) = stmt.query_row_named(params) {
            return Err(err);
        };
        Ok(mem::replace(&mut stmt.stmt.row, None).unwrap())
    }

    /// Gets one row from a query as specified type.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row_as<T>(&self, sql: &str, params: &[&dyn ToSql]) -> Result<T>
    where
        T: RowValue,
    {
        let mut stmt = self.statement(sql).fetch_array_size(1).build()?;
        stmt.query_row_as::<T>(params)
    }

    /// Gets one row from a query with named bind parameters as specified type.
    ///
    /// See [Query Methods][].
    ///
    /// [Query Methods]: https://github.com/kubo/rust-oracle/blob/master/docs/query-methods.md
    pub fn query_row_as_named<T>(&self, sql: &str, params: &[(&str, &dyn ToSql)]) -> Result<T>
    where
        T: RowValue,
    {
        let mut stmt = self.statement(sql).fetch_array_size(1).build()?;
        stmt.query_row_as_named::<T>(params)
    }

    /// Creates a statement, binds values by position and executes it in one call.
    /// It will retunrs `Err` when the statemnet is a select statement.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // execute a statement without bind parameters
    /// conn.execute("insert into emp(empno, ename) values (113, 'John')", &[])?;
    ///
    /// // execute a statement with binding parameters by position
    /// conn.execute("insert into emp(empno, ename) values (:1, :2)", &[&114, &"Smith"])?;
    ///
    /// # Ok::<(), Error>(())
    /// ```
    pub fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Statement> {
        let mut stmt = self.statement(sql).build()?;
        stmt.execute(params)?;
        Ok(stmt)
    }

    /// Creates a statement, binds values by name and executes it in one call.
    /// It will retunrs `Err` when the statemnet is a select statement.
    ///
    /// The bind variable names are compared case-insensitively.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    ///
    /// // execute a statement with binding parameters by name
    /// conn.execute_named("insert into emp(empno, ename) values (:id, :name)",
    ///                    &[("id", &114),
    ///                      ("name", &"Smith")])?;
    ///
    /// # Ok::<(), Error>(())
    /// ```
    pub fn execute_named(&self, sql: &str, params: &[(&str, &dyn ToSql)]) -> Result<Statement> {
        let mut stmt = self.statement(sql).build()?;
        stmt.execute_named(params)?;
        Ok(stmt)
    }

    /// Commits the current active transaction
    pub fn commit(&self) -> Result<()> {
        chkerr!(self.ctxt(), dpiConn_commit(self.handle()));
        Ok(())
    }

    /// Rolls back the current active transaction
    pub fn rollback(&self) -> Result<()> {
        chkerr!(self.ctxt(), dpiConn_rollback(self.handle()));
        Ok(())
    }

    /// Gets autocommit mode.
    /// It is false by default.
    pub fn autocommit(&self) -> bool {
        self.conn.autocommit()
    }

    /// Enables or disables autocommit mode.
    /// It is disabled by default.
    pub fn set_autocommit(&mut self, autocommit: bool) {
        *self.conn.autocommit.lock().unwrap() = autocommit;
    }

    /// Cancels execution of running statements in the connection
    pub fn break_execution(&self) -> Result<()> {
        chkerr!(self.ctxt(), dpiConn_breakExecution(self.handle()));
        Ok(())
    }

    /// Gets an object type information from name
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    /// let objtype = conn.object_type("MDSYS.SDO_GEOMETRY");
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Note that the object type is cached in the connection.
    /// However when "CREATE TYPE", "ALTER TYPE" or "DROP TYPE"
    /// is executed, the cache clears.
    pub fn object_type(&self, name: &str) -> Result<ObjectType> {
        {
            let guard = self.conn.objtype_cache.lock()?;
            if let Some(rc_objtype) = guard.get(name) {
                return Ok(ObjectType {
                    internal: rc_objtype.clone(),
                });
            }
        }
        let s = to_odpi_str(name);
        let mut handle = ptr::null_mut();
        chkerr!(
            self.ctxt(),
            dpiConn_getObjectType(self.handle(), s.ptr, s.len, &mut handle)
        );
        let res = ObjectType::from_dpi_object_type(self.conn.clone(), DpiObjectType::new(handle));
        if let Ok(ref objtype) = res {
            self.conn
                .objtype_cache
                .lock()?
                .insert(name.to_string(), objtype.internal.clone());
        };
        res
    }

    /// Clear the object type cache in the connection.
    ///
    /// See also [`object_type`](#method.object_type).
    pub fn clear_object_type_cache(&self) -> Result<()> {
        self.conn.clear_object_type_cache()
    }

    #[doc(hidden)]
    pub fn object_type_cache_len(&self) -> usize {
        self.conn.objtype_cache.lock().unwrap().len()
    }

    /// Gets information about the server version
    ///
    /// NOTE: if you connect to Oracle Database 18 or higher with
    /// Oracle client libraries 12.2 or lower, it gets the base
    /// version (such as 18.0.0.0.0) instead of the full version
    /// (such as 18.3.0.0.0).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    /// let (version, banner) = conn.server_version()?;
    /// println!("Oracle Version: {}", version);
    /// println!("--- Version Banner ---");
    /// println!("{}", banner);
    /// println!("---------------------");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn server_version(&self) -> Result<(Version, String)> {
        let mut s = new_odpi_str();
        let mut ver = MaybeUninit::uninit();
        chkerr!(
            self.ctxt(),
            dpiConn_getServerVersion(self.handle(), &mut s.ptr, &mut s.len, ver.as_mut_ptr())
        );
        Ok((
            Version::new_from_dpi_ver(unsafe { ver.assume_init() }),
            s.to_string(),
        ))
    }

    /// Changes the password for the specified user
    pub fn change_password(
        &self,
        username: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let username = to_odpi_str(username);
        let old_password = to_odpi_str(old_password);
        let new_password = to_odpi_str(new_password);
        chkerr!(
            self.ctxt(),
            dpiConn_changePassword(
                self.handle(),
                username.ptr,
                username.len,
                old_password.ptr,
                old_password.len,
                new_password.ptr,
                new_password.len
            )
        );
        Ok(())
    }

    /// Pings the connection to see if it is still alive.
    ///
    /// It checks the connection by making a network round-trip
    /// between the client and the server.
    ///
    /// See also [`Connection::status`].
    pub fn ping(&self) -> Result<()> {
        chkerr!(self.ctxt(), dpiConn_ping(self.handle()));
        Ok(())
    }

    /// Gets the status of the connection.
    ///
    /// It returns `Ok(ConnStatus::Closed)` when the connection was closed
    /// by [`Connection::close`].
    /// Otherwise see bellow.
    ///
    /// **Oracle client 12.2 and later:**
    ///
    /// It checks whether the underlying TCP socket has disconnected
    /// by the server. There is no guarantee that the server is alive
    /// and the network between the client and server has no trouble.
    ///
    /// For example, it returns `Ok(ConnStatus::NotConnected)` when the
    /// database on the server-side OS stopped and the client received
    /// a FIN or RST packet. However it returns `Ok(ConnStatus::Normal)`
    /// when the server-side OS itself crashes or the network is in
    /// trouble.
    ///
    /// **Oracle client 11.2 and 12.1:**
    ///
    /// It returns `Ok(ConnStatus::Normal)` when the last network
    /// round-trip between the client and server went through. Otherwise,
    /// `Ok(ConnStatus::NotConnected)`. There is no guarantee that the
    /// next network round-trip will go through.
    ///
    /// See also [`Connection::ping`].
    pub fn status(&self) -> Result<ConnStatus> {
        match self.oci_attr::<ServerStatus>() {
            Ok(status) => match status {
                OCI_SERVER_NOT_CONNECTED => Ok(ConnStatus::NotConnected),
                OCI_SERVER_NORMAL => Ok(ConnStatus::Normal),
                _ => Err(Error::InternalError(format!(
                    "Unexpected server status: {}",
                    status
                ))),
            },
            Err(Error::DpiError(err)) if err.message() == "DPI-1010: not connected" => {
                Ok(ConnStatus::Closed)
            }
            Err(err) => Err(err),
        }
    }

    /// Gets the statement cache size
    pub fn stmt_cache_size(&self) -> Result<u32> {
        let mut size = 0u32;
        chkerr!(
            self.ctxt(),
            dpiConn_getStmtCacheSize(self.handle(), &mut size)
        );
        Ok(size)
    }

    /// Sets the statement cache size
    pub fn set_stmt_cache_size(&self, size: u32) -> Result<()> {
        chkerr!(self.ctxt(), dpiConn_setStmtCacheSize(self.handle(), size));
        Ok(())
    }

    /// Gets the current call timeout used for round-trips to
    /// the database made with this connection. `None` means that no timeouts
    /// will take place.
    pub fn call_timeout(&self) -> Result<Option<Duration>> {
        let mut value = 0;
        chkerr!(
            self.ctxt(),
            dpiConn_getCallTimeout(self.handle(), &mut value)
        );
        if value != 0 {
            Ok(Some(Duration::from_millis(value.into())))
        } else {
            Ok(None)
        }
    }

    /// Sets the call timeout to be used for round-trips to the
    /// database made with this connection. None means that no timeouts
    /// will take place.
    ///
    /// The call timeout value applies to each database round-trip
    /// individually, not to the sum of all round-trips. Time spent
    /// processing in rust-oracle before or after the completion of each
    /// round-trip is not counted.
    ///
    /// - If the time from the start of any one round-trip to the
    ///   completion of that same round-trip exceeds call timeout,
    ///   then the operation is halted and an exception occurs.
    ///
    /// - In the case where an rust-oracle operation requires more than one
    ///   round-trip and each round-trip takes less than call timeout,
    ///   then no timeout will occur, even if the sum of all round-trip
    ///   calls exceeds call timeout.
    ///
    /// - If no round-trip is required, the operation will never be
    ///   interrupted.
    ///
    /// After a timeout is triggered, rust-oracle attempts to clean up the
    /// internal connection state. The cleanup is allowed to take another
    /// `duration`.
    ///
    /// If the cleanup was successful, an exception DPI-1067 will be
    /// raised but the application can continue to use the connection.
    ///
    /// For small values of call timeout, the connection cleanup may not
    /// complete successfully within the additional call timeout
    /// period. In this case an exception ORA-3114 is raised and the
    /// connection will no longer be usable. It should be closed.
    pub fn set_call_timeout(&self, dur: Option<Duration>) -> Result<()> {
        if let Some(dur) = dur {
            let msecs = duration_to_msecs(dur).ok_or(Error::OutOfRange(format!(
                "Too large duration {:?}. It must be less than 49.7 days",
                dur
            )))?;
            if msecs == 0 {
                return Err(Error::OutOfRange(format!(
                    "Too short duration {:?}. It must not be submilliseconds",
                    dur
                )));
            }
            chkerr!(self.ctxt(), dpiConn_setCallTimeout(self.handle(), msecs));
        } else {
            chkerr!(self.ctxt(), dpiConn_setCallTimeout(self.handle(), 0));
        }
        Ok(())
    }

    /// Gets current schema associated with the connection
    pub fn current_schema(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiConn_getCurrentSchema(self.handle(), &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Sets current schema associated with the connection
    pub fn set_current_schema(&self, current_schema: &str) -> Result<()> {
        let s = to_odpi_str(current_schema);
        chkerr!(
            self.ctxt(),
            dpiConn_setCurrentSchema(self.handle(), s.ptr, s.len)
        );
        Ok(())
    }

    /// Gets edition associated with the connection
    pub fn edition(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiConn_getEdition(self.handle(), &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Gets external name associated with the connection
    pub fn external_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiConn_getExternalName(self.handle(), &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Sets external name associated with the connection
    pub fn set_external_name(&self, external_name: &str) -> Result<()> {
        let s = to_odpi_str(external_name);
        chkerr!(
            self.ctxt(),
            dpiConn_setExternalName(self.handle(), s.ptr, s.len)
        );
        Ok(())
    }

    /// Gets internal name associated with the connection
    pub fn internal_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiConn_getInternalName(self.handle(), &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Sets internal name associated with the connection
    pub fn set_internal_name(&self, internal_name: &str) -> Result<()> {
        let s = to_odpi_str(internal_name);
        chkerr!(
            self.ctxt(),
            dpiConn_setInternalName(self.handle(), s.ptr, s.len)
        );
        Ok(())
    }

    /// Sets module associated with the connection
    ///
    /// This is same with calling [DBMS_APPLICATION_INFO.SET_MODULE][] but
    /// without executing a statement. The module name is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_APPLICATION_INFO.SET_MODULE]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-B2E2BD20-D91D-40DB-A3F6-37A853384F30
    pub fn set_module(&self, module: &str) -> Result<()> {
        let s = to_odpi_str(module);
        chkerr!(self.ctxt(), dpiConn_setModule(self.handle(), s.ptr, s.len));
        Ok(())
    }

    /// Sets action associated with the connection
    ///
    /// This is same with calling [DBMS_APPLICATION_INFO.SET_ACTION][] but
    /// without executing a statement. The action name is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_APPLICATION_INFO.SET_ACTION]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-90DA860F-BFBE-4539-BA00-2279B02B8F26
    pub fn set_action(&self, action: &str) -> Result<()> {
        let s = to_odpi_str(action);
        chkerr!(self.ctxt(), dpiConn_setAction(self.handle(), s.ptr, s.len));
        Ok(())
    }

    /// Sets client info associated with the connection
    ///
    /// This is same with calling [DBMS_APPLICATION_INFO.SET_CLIENT_INFO][] but
    /// without executing a statement. The client info is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_APPLICATION_INFO.SET_CLIENT_INFO]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-68A3DF04-BE91-46CC-8D2B-97BA0E89956F
    pub fn set_client_info(&self, client_info: &str) -> Result<()> {
        let s = to_odpi_str(client_info);
        chkerr!(
            self.ctxt(),
            dpiConn_setClientInfo(self.handle(), s.ptr, s.len)
        );
        Ok(())
    }

    /// Sets client identifier associated with the connection
    ///
    /// This is same with calling [DBMS_SESSION.SET_IDENTIFIER][] but
    /// without executing a statement. The client identifier is piggybacked
    /// to the server with the next network round-trip.
    ///
    /// [DBMS_SESSION.SET_IDENTIFIER]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-988EA930-BDFE-4205-A806-E54F05333562
    pub fn set_client_identifier(&self, client_identifier: &str) -> Result<()> {
        let s = to_odpi_str(client_identifier);
        chkerr!(
            self.ctxt(),
            dpiConn_setClientIdentifier(self.handle(), s.ptr, s.len)
        );
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
    /// [db_op]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-9CE3C342-D210-4690-A7E9-5813EF9D558E
    /// [DBMS_SQL_MONITOR.BEGIN_OPERATION]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-25BE0E79-3A19-4303-9F66-2CFDB87C7F82
    /// [Monitoring Database Operations]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-C941CE9D-97E1-42F8-91ED-4949B2B710BF
    pub fn set_db_op(&self, db_op: &str) -> Result<()> {
        let s = to_odpi_str(db_op);
        chkerr!(self.ctxt(), dpiConn_setDbOp(self.handle(), s.ptr, s.len));
        Ok(())
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
    /// # use oracle::*;
    /// // connect as sysdba with prelim_auth mode
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///     .privilege(Privilege::Sysdba)
    ///     .prelim_auth(true)
    ///     .connect()?;
    ///
    /// // start the instance
    /// conn.startup_database(&[])?;
    /// conn.close()?;
    ///
    /// // connect again without prelim_auth
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///     .privilege(Privilege::Sysdba)
    ///     .connect()?;
    ///
    /// // mount and open a database
    /// conn.execute("alter database mount", &[])?;
    /// conn.execute("alter database open", &[])?;
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Start up a database in restricted mode
    ///
    /// ```ignore
    /// ...
    /// conn.startup_database(&[StartupMode::Restrict])?;
    /// ...
    /// ```
    ///
    /// If the database is running, shut it down with mode ABORT and then
    /// start up in restricted mode
    ///
    /// ```ignore
    /// ...
    /// conn.startup_database(&[StartupMode::Force, StartupMode::Restrict])?;
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
        chkerr!(
            self.ctxt(),
            dpiConn_startupDatabase(self.handle(), mode_num)
        );
        Ok(())
    }

    /// Shuts down a database
    ///
    /// When this method is called with [`ShutdownMode::Default`],
    /// [`ShutdownMode::Transactional`], [`ShutdownMode::TransactionalLocal`]
    /// or [`ShutdownMode::Immediate`], execute "alter database close normal"
    /// and "alter database dismount" and call this method again with
    /// [`ShutdownMode::Final`].
    ///
    /// When this method is called with [`ShutdownMode::Abort`],
    /// the database is aborted immediately.
    ///
    /// # Examples
    ///
    /// Same with `shutdown immediate` on sqlplus.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// // connect as sysdba
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///     .privilege(Privilege::Sysdba)
    ///     .connect()?;
    ///
    /// // begin 'shutdown immediate'
    /// conn.shutdown_database(ShutdownMode::Immediate)?;
    ///
    /// // close and dismount the database
    /// conn.execute("alter database close normal", &[])?;
    /// conn.execute("alter database dismount", &[])?;
    ///
    /// // finish shutdown
    /// conn.shutdown_database(ShutdownMode::Final)?;
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// Same with `shutdown abort` on sqlplus.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// // connect as sysdba
    /// let conn = Connector::new("sys", "change_on_install", "")
    ///     .privilege(Privilege::Sysdba).connect()?;
    ///
    /// // 'shutdown abort'
    /// conn.shutdown_database(ShutdownMode::Abort)?;
    ///
    /// // The database is aborted here.
    /// # Ok::<(), Error>(())
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
        chkerr!(self.ctxt(), dpiConn_shutdownDatabase(self.handle(), mode));
        Ok(())
    }

    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn tag(&self) -> &str {
        &self.conn.tag
    }

    #[doc(hidden)] // hiden until connection pooling is supported.
    pub fn tag_found(&self) -> bool {
        self.conn.tag_found
    }

    fn close_internal(&self, mode: dpiConnCloseMode, tag: &str) -> Result<()> {
        let tag = to_odpi_str(tag);
        chkerr!(
            self.ctxt(),
            dpiConn_close(self.handle(), mode, tag.ptr, tag.len)
        );
        Ok(())
    }

    /// Gets an OCI handle attribute corresponding to the specified type parameter
    /// See the [`oci_attr` module][crate::oci_attr] for details.
    pub fn oci_attr<T>(&self) -> Result<<<T::DataType as DataType>::Type as ToOwned>::Owned>
    where
        T: OciAttr,
        T::HandleType: ConnHandle,
        T::Mode: ReadMode,
    {
        let attr_value = AttrValue::from_conn(self, <T::HandleType>::HANDLE_TYPE, <T>::ATTR_NUM);
        unsafe { <T::DataType>::get(attr_value) }
    }

    /// Sets an OCI handle attribute corresponding to the specified type parameter
    /// See the [`oci_attr` module][crate::oci_attr] for details.
    pub fn set_oci_attr<T>(&mut self, value: &<T::DataType as DataType>::Type) -> Result<()>
    where
        T: OciAttr,
        T::HandleType: ConnHandle,
        T::Mode: WriteMode,
    {
        let mut attr_value =
            AttrValue::from_conn(self, <T::HandleType>::HANDLE_TYPE, <T>::ATTR_NUM);
        unsafe { <T::DataType>::set(&mut attr_value, value) }
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Connection {{ conn: {:?}", self.conn)
    }
}
