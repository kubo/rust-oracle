// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2024 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

//! Type definitions for connection
//!
//! Some types at the top-level module will move here in future.
use crate::to_rust_str;
#[cfg(doc)]
use crate::Connection;
use crate::Error;
use crate::Result;
use odpic_sys::*;

/// The mode to use when closing connections to the database
///
/// See [`Connection::close_with_mode`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CloseMode<'a> {
    /// The connection is returned to the connection pool for
    /// future use.
    Default,

    /// Causes the connection to be dropped from the connection
    /// pool.
    Drop,

    /// Causes the connection to be tagged with the tag information.
    /// An empty tag `""` will cause the tag to be cleared.
    Retag(&'a str),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// [Session Purity](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-12410EEC-FE79-42E2-8F6B-EAA9EDA59665)
pub enum Purity {
    /// Must use a new session
    New,
    /// Reuse a pooled session
    Self_,
}

impl Purity {
    pub(crate) fn to_dpi(self) -> dpiPurity {
        match self {
            Purity::New => DPI_PURITY_NEW,
            Purity::Self_ => DPI_PURITY_SELF,
        }
    }
}

/// The type of server process associated with a connection
///
/// It is only available with Oracle Client libraries 23.4 or higher.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServerType {
    /// A dedicated server process is being used with the connection.
    Dedicated,
    /// A pooled server process (DRCP) is being used with the connection.
    Pooled,
    /// A shared server process is being used with the connection.
    Shared,
    /// The type of server process is unknown.
    Unknown,
}

impl ServerType {
    pub(crate) fn from_dpi(server_type: u8) -> Result<ServerType> {
        match server_type {
            DPI_SERVER_TYPE_DEDICATED => Ok(ServerType::Dedicated),
            DPI_SERVER_TYPE_POOLED => Ok(ServerType::Pooled),
            DPI_SERVER_TYPE_SHARED => Ok(ServerType::Shared),
            DPI_SERVER_TYPE_UNKNOWN => Ok(ServerType::Unknown),
            _ => Err(Error::internal_error(format!(
                "Unknown dpiServerType {}",
                server_type
            ))),
        }
    }
}

/// Information about a connection
///
/// This is a return value of [`Connection::info()`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    /// The name of the Oracle Database Domain name associated with the connection
    ///
    /// This is the same value returned by the SQL expression
    /// `SELECT VALUE FROM V$PARAMETER WHERE NAME = 'db_domain'`.
    pub db_domain: String,

    /// The Oracle Database name associated with the connection
    ///
    /// This is the same value returned by the SQL expression
    /// `SELECT NAME FROM V$DATABASE`.
    /// Note the values may have different cases.
    pub db_name: String,

    /// The Oracle Database instance name associated with the connection
    ///
    /// This is the same value returned by the SQL expression
    /// `SELECT SYS_CONTEXT('USERENV', 'INSTANCE_NAME') FROM DUAL`.
    /// Note the values may have different cases.
    pub instance_name: String,

    /// The Oracle Database service name associated with the connection
    ///
    /// This is the same value returned by the SQL expression
    /// `SELECT SYS_CONTEXT('USERENV', 'SERVICE_NAME') FROM DUAL`.
    pub service_name: String,

    /// The maximum length of identifiers (in bytes) supported by the
    /// database to which the connection has been established
    ///
    /// See [Database Object Naming Rules](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-75337742-67FD-4EC0-985F-741C93D918DA).
    pub max_identifier_length: u32,

    /// The maximum number of cursors that can be opened
    ///
    /// This is the same value returned by the SQL expression
    /// `SELECT VALUE FROM V$PARAMETER WHERE NAME = 'open_cursors'`.
    pub max_open_cursors: u32,

    /// The type of server process used by the connection
    ///
    /// This is only available with Oracle Client libraries 23.4 or higher.
    /// Otherwise, it is always `ServerType::Unknown`.
    pub server_type: ServerType,
}

impl Info {
    pub(crate) fn from_dpi(info: &dpiConnInfo) -> Result<Info> {
        Ok(Info {
            db_domain: to_rust_str(info.dbDomain, info.dbDomainLength),
            db_name: to_rust_str(info.dbName, info.dbNameLength),
            instance_name: to_rust_str(info.instanceName, info.instanceNameLength),
            service_name: to_rust_str(info.serviceName, info.serviceNameLength),
            max_identifier_length: info.maxIdentifierLength,
            max_open_cursors: info.maxOpenCursors,
            server_type: ServerType::from_dpi(info.serverType)?,
        })
    }
}
