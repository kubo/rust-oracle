use std::ptr;

use binding::*;
use Context;
use error::error_from_context;
use Result;

use odpi::Version;
use odpi::OdpiStr;
use odpi::new_odpi_str;
use odpi::to_odpi_str;
use odpi::DpiStatement;
use Statement;

//
// Connector
//

pub struct Connector<'a> {
    ctxt: &'static Context,
    username: &'a str,
    password: &'a str,
    connect_string: &'a str,
    common_params: dpiCommonCreateParams,
    conn_params: dpiConnCreateParams,
    app_ctxt: Vec<dpiAppContext>,
}

impl<'a> Connector<'a> {
    pub fn new(username: &'a str, password: &'a str, connect_string: &'a str) -> Result<Connector<'a>> {
        let ctxt = try!(Context::get());
        Ok(Connector {
            ctxt: ctxt,
            username: username,
            password: password,
            connect_string: connect_string,
            common_params: ctxt.common_create_params,
            conn_params: ctxt.conn_create_params,
            app_ctxt: Vec::new(),
        })
    }

    pub fn events(&'a mut self, b: bool) -> &'a mut Connector {
        if b {
            self.common_params.createMode |= DPI_MODE_CREATE_EVENTS;
        } else {
            self.common_params.createMode &= dpiCreateMode(!DPI_MODE_CREATE_EVENTS.0);
        }
        self
    }

    pub fn edition(&'a mut self, edition: &'a str) -> &'a mut Connector {
        let s = to_odpi_str(edition);
        self.common_params.edition = s.ptr;
        self.common_params.editionLength = s.len;
        self
    }

    pub fn driver_name(&'a mut self, name: &'a str) -> &'a mut Connector {
        let s = to_odpi_str(name);
        self.common_params.driverName = s.ptr;
        self.common_params.driverNameLength = s.len;
        self
    }

    pub fn auth_mode(&'a mut self, mode: dpiAuthMode) -> &'a mut Connector {
        self.conn_params.authMode = mode;
        self
    }

    pub fn connection_class(&'a mut self, name: &'a str) -> &'a mut Connector {
        let s = to_odpi_str(name);
        self.conn_params.connectionClass = s.ptr;
        self.conn_params.connectionClassLength = s.len;
        self
    }

    pub fn purity(&'a mut self, purity: dpiPurity) -> &'a mut Connector {
        self.conn_params.purity = purity;
        self
    }

    pub fn new_password(&'a mut self, password: &'a str) -> &'a mut Connector {
        let s = to_odpi_str(password);
        self.conn_params.newPassword = s.ptr;
        self.conn_params.newPasswordLength = s.len;
        self
    }

    pub fn app_context(&'a mut self, namespace: &'a str, name: &'a str, value: &'a str) -> &'a mut Connector {
        let ns = to_odpi_str(namespace);
        let n = to_odpi_str(name);
        let v = to_odpi_str(value);
        self.app_ctxt.push(dpiAppContext{
            namespaceName: ns.ptr,
            namespaceNameLength: ns.len,
            name: n.ptr,
            nameLength: n.len,
            value: v.ptr,
            valueLength: v.len
        });
        self
    }

    pub fn external_auth(&'a mut self, b: bool) -> &'a mut Connector {
        self.conn_params.externalAuth = if b {1} else {0};
        self
    }

    pub fn tag(&'a mut self, name: &'a str) -> &'a mut Connector {
        let s = to_odpi_str(name);
        self.conn_params.tag = s.ptr;
        self.conn_params.tagLength = s.len;
        self
    }

    pub fn match_any_tag(&'a mut self, b: bool) -> &'a mut Connector {
        self.conn_params.matchAnyTag = if b {1} else {0};
        self
    }

    pub fn connect(&mut self) -> Result<Connection> {
        self.conn_params.appContext = self.app_ctxt.as_mut_ptr();
        self.conn_params.numAppContext = self.app_ctxt.len() as u32;
        self.conn_params.outTag = ptr::null();
        self.conn_params.outTagLength = 0;
        self.conn_params.outTagFound = 0;
        Connection::connect(self.ctxt, self.username, self.password, self.connect_string, &self.common_params, &self.conn_params)
    }
}

//
// Connection
//

pub struct Connection {
    pub ctxt: &'static Context,
    pub handle: *mut dpiConn,
    tag: String,
    tag_found: bool,
}

impl Connection {

    pub fn new(username: &str, password: &str, connect_string: &str) -> Result<Connection> {
        Connector::new(username, password, connect_string)?.connect()
    }

    pub(crate) fn connect(ctxt: &'static Context, username: &str, password: &str, connect_string: &str, common_param: &dpiCommonCreateParams, conn_param: &dpiConnCreateParams) -> Result<Connection> {
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
            tag: OdpiStr::new(conn_param.outTag, conn_param.outTagLength).to_string(),
            tag_found: conn_param.outTagFound != 0,
        })
    }

    pub fn tag(&self) -> &String {
        &self.tag
    }

    pub fn tag_found(&self) -> bool {
        self.tag_found
    }

    /// break execution of the statement running on the connection
    pub fn break_execution(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_breakExecution(self.handle));
        Ok(())
    }

    /// change the password for the specified user
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

    /// close the connection now, not when the reference count reaches zero
    pub fn close(&self) -> Result<()> {
        self.close_internal(DPI_MODE_CONN_CLOSE_DEFAULT, "")
    }

    fn close_internal(&self, mode: dpiConnCloseMode, tag: &str) -> Result<()> {
        let tag = to_odpi_str(tag);
        chkerr!(self.ctxt,
                dpiConn_close(self.handle, mode, tag.ptr, tag.len));
        Ok(())
    }

    /// commits the current active transaction
    /// This feature will be changed later.
    pub fn commit(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_commit(self.handle));
        Ok(())
    }

    //pub fn dpiConn_deqObject
    //pub fn dpiConn_enqObject

    /// get current schema associated with the connection
    pub fn current_schema(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getCurrentSchema(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    /// get edition associated with the connection
    pub fn edition(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getEdition(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    /// get external name associated with the connection
    pub fn external_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getExternalName(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    /// get internal name associated with the connection
    pub fn internal_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(self.ctxt,
                dpiConn_getInternalName(self.handle, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    //pub fn dpiConn_getLTXID
    //pub fn dpiConn_getObjectType

    /// return information about the server version in use
    pub fn server_version(&self) -> Result<(String, Version)> {
        let mut s = new_odpi_str();
        let mut dpi_ver = Default::default();
        chkerr!(self.ctxt,
                dpiConn_getServerVersion(self.handle, &mut s.ptr, &mut s.len,
                                         &mut dpi_ver));
        Ok((s.to_string(), Version::new_from_dpi_ver(dpi_ver)))
    }

    /// return the statement cache size
    pub fn stmt_cache_size(&self) -> Result<u32> {
        let mut size = 0u32;
        chkerr!(self.ctxt,
                dpiConn_getStmtCacheSize(self.handle, &mut size));
        Ok(size)
    }

    //pub fn dpiConn_newDeqOptions
    //pub fn dpiConn_newEnqOptions
    //pub fn dpiConn_newMsgProps
    //pub fn dpiConn_newSubscription
    //pub fn dpiConn_newTempLob
    //pub fn dpiConn_newVar

    /// ping the connection to see if it is still alive
    pub fn ping(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_ping(self.handle));
        Ok(())
    }

    //pub fn dpiConn_prepareDistribTrans

    /// prepare a statement and return it for subsequent execution/fetching
    pub fn prepare(&self, sql: &str) -> Result<Statement> {
        self.prepare_internal(false, sql, "")
    }

    pub fn prepare_internal(&self, scrollable: bool, sql: &str, tag: &str) -> Result<Statement> {
        let scrollable = if scrollable { 1 } else { 0 };
        let sql = to_odpi_str(sql);
        let tag = to_odpi_str(tag);
        let mut stmt: *mut dpiStmt = ptr::null_mut();
        chkerr!(self.ctxt,
                dpiConn_prepareStmt(self.handle, scrollable, sql.ptr, sql.len,
                                    tag.ptr, tag.len, &mut stmt));
        let mut info: dpiStmtInfo = Default::default();
        chkerr!{&self.ctxt,
                dpiStmt_getInfo(stmt, &mut info),
                unsafe { dpiStmt_release(stmt); }}
        Statement::new(DpiStatement{
            ctxt: self.ctxt,
            conn: self,
            stmt: stmt,
            fetch_array_size: 0,
            is_query: info.isQuery != 0,
            is_plsql: info.isPLSQL != 0,
            is_ddl: info.isDDL != 0,
            is_dml: info.isDML != 0,
            statement_type: info.statementType,
            is_returning: info.isReturning != 0,
        })
    }

    /// rolls back the current active transaction
    pub fn rollback(&self) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_rollback(self.handle));
        Ok(())
    }

    /// set action associated with the connection
    pub fn set_action(&self, action: &str) -> Result<()> {
        let s = to_odpi_str(action);
        chkerr!(self.ctxt,
                dpiConn_setAction(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set client identifier associated with the connection
    pub fn set_client_identifier(&self, client_identifier: &str) -> Result<()> {
        let s = to_odpi_str(client_identifier);
        chkerr!(self.ctxt,
                dpiConn_setClientIdentifier(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set client info associated with the connection
    pub fn set_client_info(&self, client_info: &str) -> Result<()> {
        let s = to_odpi_str(client_info);
        chkerr!(self.ctxt,
                dpiConn_setClientInfo(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set current schema associated with the connection
    pub fn set_current_schema(&self, current_schema: &str) -> Result<()> {
        let s = to_odpi_str(current_schema);
        chkerr!(self.ctxt,
                dpiConn_setCurrentSchema(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set database operation associated with the connection
    pub fn set_db_op(&self, db_op: &str) -> Result<()> {
        let s = to_odpi_str(db_op);
        chkerr!(self.ctxt,
                dpiConn_setDbOp(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set external name associated with the connection
    pub fn set_external_name(&self, external_name: &str) -> Result<()> {
        let s = to_odpi_str(external_name);
        chkerr!(self.ctxt,
                dpiConn_setExternalName(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set internal name associated with the connection
    pub fn set_internal_name(&self, internal_name: &str) -> Result<()> {
        let s = to_odpi_str(internal_name);
        chkerr!(self.ctxt,
                dpiConn_setInternalName(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set module associated with the connection
    pub fn set_module(&self, module: &str) -> Result<()> {
        let s = to_odpi_str(module);
        chkerr!(self.ctxt,
                dpiConn_setModule(self.handle, s.ptr, s.len));
        Ok(())
    }

    /// set the statement cache size
    pub fn set_stmt_cache_size(&self, size: u32) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_setStmtCacheSize(self.handle, size));
        Ok(())
    }

    /// Shuts down the database
    pub fn shutdown_database(&self, mode: dpiShutdownMode) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_shutdownDatabase(self.handle, mode as u32));
        Ok(())
    }

    /// startup the database
    pub fn startup_database(&self, mode: dpiStartupMode) -> Result<()> {
        chkerr!(self.ctxt,
                dpiConn_startupDatabase(self.handle, mode as u32));
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = unsafe { dpiConn_release(self.handle) };
    }
}
