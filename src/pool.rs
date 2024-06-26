// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2021 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

//! Type definitions for connection pooling
//!
//! This module defines types for connection pooling using [Session Pooling in OCI].
//!
//! [Session Pooling in OCI]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-F9662FFB-EAEF-495C-96FC-49C6D1D9625C
use crate::binding::*;
use crate::chkerr;
use crate::conn::Purity;
use crate::connection::CommonCreateParamsBuilder;
use crate::AssertSend;
use crate::AssertSync;
use crate::Connection;
use crate::Context;
use crate::DpiPool;
use crate::Error;
use crate::OdpiStr;
use crate::Privilege;
use crate::Result;
use std::convert::TryInto;
use std::fmt;
use std::ptr;
use std::time::Duration;

/// The mode to use when closing pools
///
/// See [`Pool::close`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CloseMode {
    /// If there are any active connections in the pool an
    /// error is returned.
    Default,

    /// Causes all of the active connections in the pool
    /// to be closed before closing the pool itself.
    Force,
}

impl CloseMode {
    fn to_dpi_value(self) -> dpiPoolCloseMode {
        match self {
            CloseMode::Default => DPI_MODE_POOL_CLOSE_DEFAULT,
            CloseMode::Force => DPI_MODE_POOL_CLOSE_FORCE,
        }
    }
}

/// The mode to use when getting connections from a connection pool
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GetMode {
    /// The caller should block until a
    /// connection is available from the pool.
    Wait,

    /// The caller should return
    /// immediately, regardless of whether a connection is
    /// available in the pool. If a connection is not
    /// available an error is returned.
    NoWait,

    /// A new connection should be created if
    /// all of the connections in the pool are busy, even if
    /// this exceeds the maximum connections allowable for
    /// the connection pool (see [`PoolBuilder::max_connections`])
    ForceGet,

    /// The caller should block until a
    /// connection is available from the pool, but only for
    /// the specified length of time defined in
    /// the tuple field. If a
    /// connection is not available within the specified
    /// period of time an error is returned.
    TimedWait(Duration),
}

impl GetMode {
    fn to_dpi_value(self) -> dpiPoolGetMode {
        match self {
            GetMode::Wait => DPI_MODE_POOL_GET_WAIT as dpiPoolGetMode,
            GetMode::NoWait => DPI_MODE_POOL_GET_NOWAIT as dpiPoolGetMode,
            GetMode::ForceGet => DPI_MODE_POOL_GET_FORCEGET as dpiPoolGetMode,
            GetMode::TimedWait(_) => DPI_MODE_POOL_GET_TIMEDWAIT as dpiPoolGetMode,
        }
    }

    fn to_wait_timeout(self) -> Result<Option<u32>> {
        if let GetMode::TimedWait(ref dur) = self {
            match dur.as_millis().try_into() {
                Ok(msecs) => Ok(Some(msecs)),
                Err(err) => Err(Error::out_of_range(format!(
                    "too long timed wait duration {:?}",
                    dur
                ))
                .add_source(err)),
            }
        } else {
            Ok(None)
        }
    }
}

/// Whether a connection pool is homogeneous or heterogeneous.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PoolType {
    /// The default pool type.
    /// All connections in the pool are authenticated with the
    /// username and password provided during pool creation.
    Homogeneous,

    /// Connections with different authentication contexts can be
    /// created in the same pool. This pool type also supports
    /// external authentication.
    /// [`PoolBuilder::min_connections`] and [`PoolBuilder::connection_increment`]
    /// are ignored in this type.
    Heterogeneous,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct I32Seconds(i32);

impl I32Seconds {
    fn try_from(dur: Option<Duration>, msg: &str) -> Result<I32Seconds> {
        if let Some(dur) = dur {
            match dur.as_secs().try_into() {
                Ok(secs) => Ok(I32Seconds(secs)),
                Err(err) => {
                    Err(Error::out_of_range(format!("too long {} {:?}", msg, dur)).add_source(err))
                }
            }
        } else {
            Ok(I32Seconds(-1))
        }
    }

    fn into(self) -> Option<Duration> {
        match self.0.try_into() {
            Ok(secs) => Some(Duration::from_secs(secs)),
            Err(_) => None,
        }
    }
}

/// Additional options to get a connection from a pool
///
/// This is used as the argument of [`Pool::get_with_options`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolOptions {
    username: String,
    password: String,
    privilege: Option<Privilege>,
    external_auth: bool,
    tag: String,
    match_any_tag: bool,
    purity: Option<Purity>,
    connection_class: String,
}

impl PoolOptions {
    pub fn new() -> PoolOptions {
        PoolOptions {
            username: "".into(),
            password: "".into(),
            privilege: None,
            external_auth: false,
            tag: "".into(),
            match_any_tag: false,
            purity: None,
            connection_class: "".into(),
        }
    }

    pub fn username<S>(mut self, username: S) -> Self
    where
        S: Into<String>,
    {
        self.username = username.into();
        self
    }

    pub fn password<S>(mut self, password: S) -> Self
    where
        S: Into<String>,
    {
        self.password = password.into();
        self
    }

    pub fn privilege(mut self, privilege: Privilege) -> Self {
        self.privilege = Some(privilege);
        self
    }

    pub fn external_auth(mut self, enable: bool) -> Self {
        self.external_auth = enable;
        self
    }

    pub fn tag<S>(mut self, tag: S) -> Self
    where
        S: Into<String>,
    {
        self.tag = tag.into();
        self
    }

    pub fn match_any_tag(mut self, enable: bool) -> Self {
        self.match_any_tag = enable;
        self
    }

    pub fn purity(mut self, purity: Purity) -> Self {
        self.purity = Some(purity);
        self
    }

    pub fn connection_class<S>(mut self, connection_class: S) -> Self
    where
        S: Into<String>,
    {
        self.connection_class = connection_class.into();
        self
    }

    fn to_dpi_conn_create_params(&self, ctxt: &Context) -> dpiConnCreateParams {
        let mut conn_params = ctxt.conn_create_params();

        if let Some(privilege) = self.privilege {
            conn_params.authMode |= privilege.to_dpi();
        }
        conn_params.externalAuth = i32::from(self.external_auth);
        let s = OdpiStr::new(&self.tag);
        conn_params.tag = s.ptr;
        conn_params.tagLength = s.len;
        conn_params.matchAnyTag = i32::from(self.match_any_tag);
        if let Some(purity) = self.purity {
            conn_params.purity = purity.to_dpi();
        }
        let s = OdpiStr::new(&self.connection_class);
        conn_params.connectionClass = s.ptr;
        conn_params.connectionClassLength = s.len;
        conn_params
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct U32Seconds(u32);

impl U32Seconds {
    fn try_from(dur: Duration, msg: &str) -> Result<U32Seconds> {
        match dur.as_secs().try_into() {
            Ok(secs) => Ok(U32Seconds(secs)),
            Err(err) => {
                Err(Error::out_of_range(format!("too long {} {:?}", msg, dur)).add_source(err))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct U32Milliseconds(u32);

impl U32Milliseconds {
    fn try_from(dur: Duration, msg: &str) -> Result<U32Milliseconds> {
        match dur.as_millis().try_into() {
            Ok(secs) => Ok(U32Milliseconds(secs)),
            Err(err) => {
                Err(Error::out_of_range(format!("too long {} {:?}", msg, dur)).add_source(err))
            }
        }
    }
}

/// A bulider to make a connection pool
#[derive(Debug, Clone, PartialEq)]
pub struct PoolBuilder {
    username: String,
    password: String,
    connect_string: String,
    min_connections: Option<u32>,
    max_connections: Option<u32>,
    connection_increment: Option<u32>,
    ping_interval: Option<I32Seconds>,
    ping_timeout: Option<U32Milliseconds>,
    homogeneous: Option<i32>,
    external_auth: Option<bool>,
    get_mode: Option<GetMode>,
    timeout: Option<U32Seconds>,
    max_lifetime_connection: Option<U32Seconds>,
    plsql_fixup_callback: Option<String>,
    max_connections_per_shard: Option<u32>,
    common_params: CommonCreateParamsBuilder,
}

impl PoolBuilder {
    /// Creates a builder to make a connection pool.
    pub fn new<U, P, C>(username: U, password: P, connect_string: C) -> PoolBuilder
    where
        U: Into<String>,
        P: Into<String>,
        C: Into<String>,
    {
        PoolBuilder {
            username: username.into(),
            password: password.into(),
            connect_string: connect_string.into(),
            min_connections: None,
            max_connections: None,
            connection_increment: None,
            ping_interval: None,
            ping_timeout: None,
            homogeneous: None,
            external_auth: None,
            get_mode: None,
            timeout: None,
            max_lifetime_connection: None,
            plsql_fixup_callback: None,
            max_connections_per_shard: None,
            common_params: Default::default(),
        }
    }

    /// Specifies the minimum number of connections to be created by the connection pool.
    /// This value is ignored if the pool type is [`PoolType::Heterogeneous`].
    /// The default value is 1.
    ///
    /// See also [`Pool::reconfigure`].
    pub fn min_connections(&mut self, num: u32) -> &mut PoolBuilder {
        self.min_connections = Some(num);
        self
    }

    /// Specifies the maximum number of connections that can be created by the connection
    /// pool. Values of 1 and higher are acceptable. The default value is 1.
    ///
    /// See also [`Pool::reconfigure`].
    pub fn max_connections(&mut self, num: u32) -> &mut PoolBuilder {
        self.max_connections = Some(num);
        self
    }

    /// Specifies the number of connections that will be created by the connection pool
    /// when more connections are required and the number of connection is less than
    /// the maximum allowed. This value is ignored if the pool type is [`PoolType::Heterogeneous`].
    /// This value added to the [`PoolBuilder::min_connections`] value
    /// must not exceed the [`PoolBuilder::max_connections`].
    /// The default value is 0.
    ///
    /// See also [`Pool::reconfigure`].
    pub fn connection_increment(&mut self, num: u32) -> &mut PoolBuilder {
        self.connection_increment = Some(num);
        self
    }

    /// Specifies the length of time since a connection has last been used
    /// before a ping will be performed to verify that the connection is still
    /// valid. A `None` value disables this check. The default value is 60 seconds.
    ///
    /// See also [`Pool::ping_interval`] and [`Pool::set_ping_interval`].
    pub fn ping_interval(&mut self, dur: Option<Duration>) -> Result<&mut PoolBuilder> {
        self.ping_interval = Some(I32Seconds::try_from(dur, "ping interval")?);
        Ok(self)
    }

    /// Specifies the length of time to wait when performing a ping to
    /// verify the connection is still valid before the connection is considered
    /// invalid and is dropped. The default value is 5 seconds.
    pub fn ping_timeout(&mut self, dur: Duration) -> Result<&mut PoolBuilder> {
        self.ping_timeout = Some(U32Milliseconds::try_from(dur, "ping timeout")?);
        Ok(self)
    }

    /// Specifies whether the pool is homogeneous or heterogeneous. In a homogeneous pool all
    /// connections use the same credentials whereas in a heterogeneous pool other
    /// credentials are permitted. The default value is [`PoolType::Homogeneous`].
    pub fn pool_type(&mut self, pool_type: PoolType) -> &mut PoolBuilder {
        self.homogeneous = Some(match pool_type {
            PoolType::Homogeneous => 1,
            PoolType::Heterogeneous => 0,
        });
        self
    }

    /// Specifies whether external authentication should be used to create the
    /// connections in the pool. If this value is `false`, the user name and password values
    /// must be specified in the call to [`PoolBuilder::new`]; otherwise, the
    /// username and password values must be empty. The default
    /// value is `false`. External authentication cannot be used with homogeneous pools.
    pub fn external_auth(&mut self, b: bool) -> &mut PoolBuilder {
        self.external_auth = Some(b);
        self
    }

    /// Specifies the mode to use when connections are acquired from the pool.
    /// The default value is [`GetMode::NoWait`].
    ///
    /// See also [`Pool::get_mode`] and [`Pool::set_get_mode`].
    pub fn get_mode(&mut self, mode: GetMode) -> &mut PoolBuilder {
        self.get_mode = Some(mode);
        self
    }

    /// Specifies the length of time after which idle connections in the
    /// pool are terminated. Note that termination only occurs when the pool is
    /// accessed. The default value is [`Duration::ZERO`] which means that no idle connections are
    /// terminated.
    ///
    /// See also [`Pool::timeout`] and [`Pool::set_timeout`].
    pub fn timeout(&mut self, dur: Duration) -> Result<&mut PoolBuilder> {
        self.timeout = Some(U32Seconds::try_from(dur, "timeout")?);
        Ok(self)
    }

    /// Specifies the maximum length of time a pooled connection may
    /// exist. Connections in use will not be closed. They become candidates for
    /// termination only when they are released back to the pool and have existed
    /// for longer than max_lifetime_connection. Connection termination only occurs
    /// when the pool is accessed. The default value is [`Duration::ZERO`] which means that there is
    /// no maximum length of time that a pooled connection may exist.
    ///
    /// See also [`Pool::max_lifetime_connection`] and [`Pool::set_max_lifetime_connection`].
    pub fn max_lifetime_connection(&mut self, dur: Duration) -> Result<&mut PoolBuilder> {
        self.max_lifetime_connection = Some(U32Seconds::try_from(dur, "max lifetime connection")?);
        Ok(self)
    }

    /// Specifies the name of a PL/SQL procedure in the format
    /// *schema.package.callback_proc* which will be called when a connection is
    /// checked out from the pool and the requested tag doesn't match the actual
    /// tag assigned to the connection. The procedure accepts the desired
    /// and actual tags as parameters and it is the responsibility of the procedure
    /// to ensure that the connection matches the desired state upon completion. See
    /// the [OCI documentation](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-B853A020-752F-494A-8D88-D0396EF57177)
    /// for more information. This functionality is only available when Oracle Client
    /// is at version 12.2 and higher.
    pub fn plsql_fixup_callback<T>(&mut self, plsql: T) -> &mut PoolBuilder
    where
        T: Into<String>,
    {
        self.plsql_fixup_callback = Some(plsql.into());
        self
    }

    /// Specifies the maximum number of connections that can be created by the connection
    /// pool for each shard in a sharded database. Set this attribute to a value
    /// other than zero to ensure that the pool is balanced towards each shard. A
    /// value of zero will not set any maximum number of connections for each shard.
    /// If the Oracle client library version is less than 18.3, this value is
    /// ignored.
    ///
    /// See also [`Pool::max_connections_per_shard`] and [`Pool::set_max_connections_per_shard`].
    pub fn max_connections_per_shard(&mut self, num: u32) -> &mut PoolBuilder {
        self.max_connections_per_shard = Some(num);
        self
    }

    fn to_dpi_pool_create_params(&self, ctxt: &Context) -> Result<dpiPoolCreateParams> {
        let mut pool_params = ctxt.pool_create_params();

        if let Some(val) = self.min_connections {
            pool_params.minSessions = val;
        }
        if let Some(val) = self.max_connections {
            pool_params.maxSessions = val;
        }
        if let Some(val) = self.connection_increment {
            pool_params.sessionIncrement = val;
        }
        if let Some(val) = self.ping_interval {
            pool_params.pingInterval = val.0;
        }
        if let Some(val) = self.ping_timeout {
            pool_params.pingTimeout = val.0 as i32;
        }
        if let Some(val) = self.homogeneous {
            pool_params.homogeneous = val;
        }
        if let Some(val) = self.external_auth {
            pool_params.externalAuth = i32::from(val);
        }
        if let Some(val) = self.get_mode {
            pool_params.getMode = val.to_dpi_value();
            if let Some(wait_timeout) = val.to_wait_timeout()? {
                pool_params.waitTimeout = wait_timeout;
            }
        }
        if let Some(val) = self.timeout {
            pool_params.timeout = val.0;
        }
        if let Some(val) = self.max_lifetime_connection {
            pool_params.maxLifetimeSession = val.0;
        }
        if let Some(ref val) = self.plsql_fixup_callback {
            let s = OdpiStr::new(val);
            pool_params.plsqlFixupCallback = s.ptr;
            pool_params.plsqlFixupCallbackLength = s.len;
        }
        if let Some(val) = self.max_connections_per_shard {
            pool_params.maxSessionsPerShard = val;
        }
        Ok(pool_params)
    }

    /// Reserved for when advanced queuing (AQ) or continuous query
    /// notification (CQN) is supported.
    pub fn events(&mut self, b: bool) -> &mut PoolBuilder {
        self.common_params.events(b);
        self
    }

    /// Specifies edition of [Edition-Based Redefinition][].
    ///
    /// [Edition-Based Redefinition]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-58DE05A0-5DEF-4791-8FA8-F04D11964906
    pub fn edition<S>(&mut self, edition: S) -> &mut PoolBuilder
    where
        S: Into<String>,
    {
        self.common_params.edition(edition);
        self
    }

    /// Sets the driver name displayed in [V$SESSION_CONNECT_INFO.CLIENT_DRIVER][].
    ///
    /// The default value is "rust-oracle : version number". Only the first 8
    /// chracters "rust-ora" are displayed when the Oracle server version is
    /// lower than 12.0.1.2.
    ///
    /// [V$SESSION_CONNECT_INFO.CLIENT_DRIVER]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-9F0DCAEA-A67E-4183-89E7-B1555DC591CE
    pub fn driver_name<S>(&mut self, driver_name: S) -> &mut PoolBuilder
    where
        S: Into<String>,
    {
        self.common_params.driver_name(driver_name);
        self
    }

    /// Specifies the number of statements to retain in the statement cache. Use a
    /// value of 0 to disable the statement cache completely.
    /// The default value is 20.
    ///
    /// See also [`Pool::stmt_cache_size`] and [`Pool::set_stmt_cache_size`].
    pub fn stmt_cache_size(&mut self, size: u32) -> &mut PoolBuilder {
        self.common_params.stmt_cache_size(size);
        self
    }

    /// Make a connection pool
    pub fn build(&self) -> Result<Pool> {
        let ctxt = Context::new0()?;
        let username = OdpiStr::new(&self.username);
        let password = OdpiStr::new(&self.password);
        let connect_string = OdpiStr::new(&self.connect_string);
        let common_params = self.common_params.build(&ctxt);
        let mut pool_params = self.to_dpi_pool_create_params(&ctxt)?;
        let mut handle = ptr::null_mut();
        chkerr!(
            &ctxt,
            dpiPool_create(
                ctxt.context,
                username.ptr,
                username.len,
                password.ptr,
                password.len,
                connect_string.ptr,
                connect_string.len,
                &common_params,
                &mut pool_params,
                &mut handle
            )
        );
        Ok(Pool {
            ctxt,
            handle: DpiPool::new(handle),
        })
    }
}

/// Connection pool
///
///
/// # Examples
///
/// Get connections in a pool
///
/// ```
/// # use oracle::Error;
/// # use oracle::pool::PoolBuilder;
/// # use oracle::test_util;
/// # let username = test_util::main_user();
/// # let password = test_util::main_password();
/// # let connect_string = test_util::connect_string();
/// // Create a pool
/// let pool = PoolBuilder::new(username, password, connect_string)
///     .max_connections(20)
///     .build()?;
///
/// // Get connections from the pool.
/// let conn1 = pool.get()?;
/// let conn2 = pool.get()?;
///
/// assert_eq!(pool.open_count()?, 2); // Two connections are in the pool.
/// assert_eq!(pool.busy_count()?, 2); // Two connections are in use.
///
/// // Return the connections to the pool.
/// conn1.close()?;
/// conn2.close()?;
///
/// assert_eq!(pool.open_count()?, 2); // Two connections are in the pool.
/// assert_eq!(pool.busy_count()?, 0); // No connections are in use.
/// # Ok::<(), Error>(())
/// ```
///
/// Use a [heterogeneous pool](PoolType::Heterogeneous) to pool different users' connections
///
/// ```
/// # use oracle::Error;
/// # use oracle::Connector;
/// # use oracle::pool::{PoolBuilder, PoolOptions, PoolType};
/// # use oracle::test_util;
/// # let username = test_util::main_user();
/// # let username = username.as_ref();
/// # let password = test_util::main_password();
/// # let another_username = test_util::edition_user();
/// # let another_username = another_username.as_ref();
/// # let another_password = test_util::edition_password();
/// # let connect_string = test_util::connect_string();
/// // Create a pool
/// let pool = PoolBuilder::new(username, password, connect_string)
///     .pool_type(PoolType::Heterogeneous)
///     .max_connections(20)
///     .build()?;
///
/// // Get a connection from the pool.
/// let conn1 = pool.get()?;
/// let conn1_user = conn1.query_row_as::<String>("select lower(user) from dual", &[])?;
/// assert_eq!(conn1_user, username);
///
/// // Get an another user's connection.
/// let opts = PoolOptions::new().username(another_username).password(another_password);
/// let conn2 = pool.get_with_options(&opts)?; // with connector to pass username and password
/// let conn2_user = conn2.query_row_as::<String>("select lower(user) from dual", &[])?;
/// assert_eq!(conn2_user, another_username);
/// # Ok::<(), Error>(())
/// ```
///
/// Get connections with tags (`NAME=VALUE` pairs)
///
/// ```
/// # use oracle::Error;
/// # use oracle::Connector;
/// # use oracle::conn;
/// # use oracle::pool::{PoolBuilder, PoolOptions};
/// # use oracle::test_util;
/// # let username = test_util::main_user();
/// # let password = test_util::main_password();
/// # let connect_string = test_util::connect_string();
/// // Create a pool
/// let pool = PoolBuilder::new(username, password, connect_string)
///     .max_connections(20)
///     .build()?;
///
/// // Create Pool options to specify a tag.
/// let opts = PoolOptions::new().tag("LANG=FRENCH");
///
/// // Get a connection with tag "LANG=FRENCH".
/// // There are no connections with the tag at this point.
/// let conn = pool.get_with_options(&opts)?;
/// assert_eq!(conn.tag_found(), false, "conn.tag_found() (1)"); // new connection
/// // Change the nls_language for later.
/// if !conn.tag_found() {
///   conn.execute("alter session set nls_language = FRENCH", &[])?;
/// }
/// // ...
/// // use the connection
/// // ...
/// // return it to the pool with new tag.
/// conn.close_with_mode(conn::CloseMode::Retag("LANG=FRENCH"))?;
///
/// // Get a connection with tag "LANG=FRENCH" again.
/// // There is one connection with the tag at this point.
/// let conn = pool.get_with_options(&opts)?;
/// assert_eq!(conn.tag_found(), true, "conn.tag_found() (2)");
/// assert_eq!(conn.tag(), "LANG=FRENCH", "conn.tag() (2)");
/// // Check whether this is the connection previously
/// let sql = "select value from nls_session_parameters where parameter = 'NLS_LANGUAGE'";
/// assert_eq!(conn.query_row_as::<String>(sql, &[])?, "FRENCH");
/// // ...
/// // The connection has been tagged already. There is no need to set a new tag.
/// conn.close()?;
///
/// # Ok::<(), Error>(())
/// ```
#[derive(Clone)]
pub struct Pool {
    ctxt: Context,
    handle: DpiPool,
}

impl Pool {
    fn handle(&self) -> *mut dpiPool {
        self.handle.raw()
    }

    fn ctxt(&self) -> &Context {
        &self.ctxt
    }

    /// Gets a connection from the pool with default parameters.
    ///
    /// Use [`Pool::get_with_options`] to get a new one with
    /// additional parameters.
    ///
    /// When the connection is dropped, it backs to the pool
    /// for subsequent calls to this function. The connection
    /// can be returned back to the pool earlier by calling
    /// [`Connection::close`].
    pub fn get(&self) -> Result<Connection> {
        self.get_with_options(&PoolOptions::new())
    }

    /// Acquires a connection from the specified connection pool.
    ///
    /// See also [`Pool::get`].
    pub fn get_with_options(&self, options: &PoolOptions) -> Result<Connection> {
        let ctxt = Context::new()?;
        let username = OdpiStr::new(&options.username);
        let password = OdpiStr::new(&options.password);
        let mut conn_params = options.to_dpi_conn_create_params(&ctxt);
        let mut handle = ptr::null_mut();
        chkerr!(
            &ctxt,
            dpiPool_acquireConnection(
                self.handle(),
                username.ptr,
                username.len,
                password.ptr,
                password.len,
                &mut conn_params,
                &mut handle
            )
        );
        ctxt.set_warning();
        Ok(Connection::from_dpi_handle(ctxt, handle, &conn_params))
    }

    /// Closes the pool and makes it unusable for further activity.
    pub fn close(&self, mode: &CloseMode) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiPool_close(self.handle(), mode.to_dpi_value())
        );
        Ok(())
    }

    /// Returns the number of connections in the pool that are busy.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::pool::PoolBuilder;
    /// # use oracle::test_util;
    /// # let username = test_util::main_user();
    /// # let password = test_util::main_password();
    /// # let connect_string = test_util::connect_string();
    /// let pool = PoolBuilder::new(username, password, connect_string)
    ///     .max_connections(3)
    ///     .build()?;
    /// assert_eq!(pool.busy_count()?, 0);
    /// let conn1 = pool.get()?;
    /// let conn2 = pool.get()?;
    /// assert_eq!(pool.busy_count()?, 2);
    /// conn1.close()?;
    /// conn2.close()?;
    /// assert_eq!(pool.busy_count()?, 0);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn busy_count(&self) -> Result<u32> {
        let mut count = 0;
        chkerr!(self.ctxt(), dpiPool_getBusyCount(self.handle(), &mut count));
        Ok(count)
    }

    /// Returns the mode used for acquiring or getting connections from the pool.
    ///
    /// See also [`PoolBuilder::get_mode`] and [`Pool::set_get_mode`].
    pub fn get_mode(&self) -> Result<GetMode> {
        let mut val = 0;
        chkerr!(self.ctxt(), dpiPool_getGetMode(self.handle(), &mut val));
        match val as u32 {
            DPI_MODE_POOL_GET_WAIT => Ok(GetMode::Wait),
            DPI_MODE_POOL_GET_NOWAIT => Ok(GetMode::NoWait),
            DPI_MODE_POOL_GET_FORCEGET => Ok(GetMode::ForceGet),
            DPI_MODE_POOL_GET_TIMEDWAIT => {
                let mut val = 0;
                chkerr!(self.ctxt(), dpiPool_getWaitTimeout(self.handle(), &mut val));
                Ok(GetMode::TimedWait(Duration::from_millis(val.into())))
            }
            _ => Err(Error::internal_error(format!(
                "unknown dpiPoolGetMode {}",
                val
            ))),
        }
    }

    /// Sets the mode used for acquiring or getting connections from the pool.
    ///
    /// See also [`PoolBuilder::get_mode`] and [`Pool::get_mode`].
    pub fn set_get_mode(&mut self, mode: &GetMode) -> Result<()> {
        let get_mode = mode.to_dpi_value();
        let wait_timeout = mode.to_wait_timeout()?;
        chkerr!(self.ctxt(), dpiPool_setGetMode(self.handle(), get_mode));
        if let Some(msecs) = wait_timeout {
            chkerr!(self.ctxt(), dpiPool_setWaitTimeout(self.handle(), msecs));
        }
        Ok(())
    }

    /// Returns the maximum lifetime a pooled connection may exist.
    ///
    /// See also [`PoolBuilder::max_lifetime_connection`] and [`Pool::set_max_lifetime_connection`].
    pub fn max_lifetime_connection(&self) -> Result<Duration> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiPool_getMaxLifetimeSession(self.handle(), &mut val)
        );
        Ok(Duration::from_secs(val.into()))
    }

    /// Sets the maximum lifetime a pooled connection may exist.
    ///
    /// See also [`PoolBuilder::max_lifetime_connection`] and [`Pool::max_lifetime_connection`].
    pub fn set_max_lifetime_connection(&mut self, dur: Duration) -> Result<()> {
        let val = U32Seconds::try_from(dur, "max lifetime connection")?;
        chkerr!(
            self.ctxt(),
            dpiPool_setMaxLifetimeSession(self.handle(), val.0)
        );
        Ok(())
    }

    /// Returns the maximum connections per shard. This parameter is used for
    /// balancing shards.
    ///
    /// See also [`PoolBuilder::max_connections_per_shard`] and [`Pool::set_max_connections_per_shard`].
    pub fn max_connections_per_shard(&self) -> Result<u32> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiPool_getMaxSessionsPerShard(self.handle(), &mut val)
        );
        Ok(val)
    }

    /// Sets the maximum number of connections per shard.
    ///
    /// See also [`PoolBuilder::max_connections_per_shard`] and [`Pool::max_connections_per_shard`].
    pub fn set_max_connections_per_shard(&mut self, max_connections: u32) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiPool_setMaxSessionsPerShard(self.handle(), max_connections)
        );
        Ok(())
    }

    /// Returns the number of connections in the pool that are open.
    pub fn open_count(&self) -> Result<u32> {
        let mut val = 0;
        chkerr!(self.ctxt(), dpiPool_getOpenCount(self.handle(), &mut val));
        Ok(val)
    }

    /// Returns the ping interval duration, which is used to check the
    /// healthiness of idle connections before getting checked out. A `None`
    /// value indicates this check is disabled.
    ///
    /// See also [`PoolBuilder::ping_interval`] and [`Pool::set_ping_interval`].
    pub fn ping_interval(&self) -> Result<Option<Duration>> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiPool_getPingInterval(self.handle(), &mut val)
        );
        Ok(I32Seconds(val).into())
    }

    /// Sets the ping interval duration which is used to to check for
    /// healthiness of connections. If this time has passed since the last time the
    /// connection was checked out a ping will be performed. A `None` value will
    /// disable this check.
    ///
    /// See also [`PoolBuilder::ping_interval`] and [`Pool::ping_interval`].
    pub fn set_ping_interval(&mut self, interval: Option<Duration>) -> Result<()> {
        let val = I32Seconds::try_from(interval, "ping interval")?;
        chkerr!(self.ctxt(), dpiPool_setPingInterval(self.handle(), val.0));
        Ok(())
    }

    /// Changes pool configuration corresponding to [`PoolBuilder::min_connections`],
    /// [`PoolBuilder::max_connections`] and [`PoolBuilder::connection_increment`]
    /// to the specified values.
    ///
    /// Connections will be created as needed if the value of `min_connections` is
    /// increased. Connections will be dropped from the pool as they are released
    /// back to the pool if `min_connections` is decreased.
    pub fn reconfigure(
        &self,
        min_connections: u32,
        max_connections: u32,
        connection_increment: u32,
    ) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiPool_reconfigure(
                self.handle(),
                min_connections,
                max_connections,
                connection_increment
            )
        );
        Ok(())
    }

    /// Returns whether or not the SODA metadata cache is enabled or not.
    ///
    /// Enabling the SODA metadata cache can significantly improve the
    /// performance of repeated calls to methods [`soda::Database::create_collection`]
    /// (when not specifying a value for the metadata parameter) and
    /// [`soda::Database::open_collection`]. Note that the cache can
    /// become out of date if changes to the metadata of cached collections
    /// are made externally.
    ///
    /// The SODA metadata cache requires Oracle Client 21.3, or later. It is also
    /// available in Oracle Client 19 from 19.11.
    ///
    /// See also [`Pool::set_soda_metadata_cache`].
    #[doc(hidden)] // uncomment when SODA is supported.
    pub fn soda_metadata_cache(&self) -> Result<bool> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiPool_getSodaMetadataCache(self.handle(), &mut val)
        );
        Ok(val != 0)
    }

    /// Sets whether the SODA metadata cache is enabled or not.
    ///
    /// The SODA metadata cache requires Oracle Client 21.3, or later. It is also
    /// available in Oracle Client 19 from 19.11.
    ///
    /// See also [`Pool::soda_metadata_cache`].
    #[doc(hidden)] // uncomment when SODA is supported.
    pub fn set_soda_metadata_cache(&mut self, enabled: bool) -> Result<()> {
        let enabled = i32::from(enabled);
        chkerr!(
            self.ctxt(),
            dpiPool_setSodaMetadataCache(self.handle(), enabled)
        );
        Ok(())
    }

    /// Returns the default size of the statement cache for connections in the pool,
    /// in number of statements.
    ///
    /// See also [`PoolBuilder::stmt_cache_size`] and [`Pool::set_stmt_cache_size`].
    pub fn stmt_cache_size(&self) -> Result<u32> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiPool_getStmtCacheSize(self.handle(), &mut val)
        );
        Ok(val)
    }

    /// Sets the default size of the statement cache for connections in the pool.
    ///
    /// See also [`PoolBuilder::stmt_cache_size`] and [`Pool::stmt_cache_size`].
    pub fn set_stmt_cache_size(&mut self, cache_size: u32) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiPool_setStmtCacheSize(self.handle(), cache_size)
        );
        Ok(())
    }

    /// Returns the length of time after which idle connections in the
    /// pool are terminated. Note that termination only occurs when the pool is
    /// accessed. A value of [`Duration::ZERO`] means that no ide connections are terminated.
    ///
    /// See also [`PoolBuilder::timeout`] and [`Pool::set_timeout`].
    pub fn timeout(&self) -> Result<Duration> {
        let mut val = 0;
        chkerr!(self.ctxt(), dpiPool_getTimeout(self.handle(), &mut val));
        Ok(Duration::from_secs(val.into()))
    }

    /// Sets the amount of time after which idle connections in the
    /// pool are terminated. Note that termination only occurs when the pool is
    /// accessed. A value of [`Duration::ZERO`] will result in no idle connections being terminated.
    ///
    /// See also [`PoolBuilder::timeout`] and [`Pool::timeout`].
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        let val = U32Seconds::try_from(timeout, "timeout")?;
        chkerr!(self.ctxt(), dpiPool_setTimeout(self.handle(), val.0));
        Ok(())
    }
}

impl fmt::Debug for Pool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Poll {{ handle: {:?}", self.handle())
    }
}

impl AssertSync for Pool {}
impl AssertSend for Pool {}
