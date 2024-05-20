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

//! Rust-oracle is based on ODPI-C using Oracle Call Interface (OCI) internally.
//! OCI treats resources as handles, which have various attributes documented [here].
//!
//! The module defines type parameters to access some OCI attributes and
//! the trait [`OciAttr`] to define your own type parameters to access attributes
//! which are not predefined in this module.
//!
//! [here]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-CB59C987-07E7-42D4-ADDF-96142CBD3D11
use crate::oci_attr::data_type::{DataType, DurationUsecU64, MaxStringSize};
#[cfg(any(doc, test))]
use crate::oci_attr::handle::Server;
use crate::oci_attr::handle::{HandleType, Session, Stmt, SvcCtx};
#[cfg(any(doc, test))]
use crate::oci_attr::mode::Write;
use crate::oci_attr::mode::{Mode, Read, ReadWrite};
#[cfg(doc)]
use crate::Connection;
#[cfg(doc)]
use std::time::Duration;

pub mod data_type;
pub mod handle;
pub mod mode;

pub unsafe trait OciAttr {
    /// [`SvcCtx`], [`Session`], [`Server`] or [`Stmt`].
    /// Other handle and descriptor types are unsupported.
    type HandleType: HandleType;

    /// [`Read`], [`Write`] or [`ReadWrite`]
    type Mode: Mode;

    /// Attribute data type
    ///
    /// The following table is the mapping between basic data types. If
    /// incorrect types are specified, the behavior is undefined.
    ///
    /// Types in Oracle manual | Rust type
    /// ---|---
    /// `ub1*`/`ub1` | [`u8`]
    /// `ub2*`/`ub2` | [`u16`]
    /// `ub4*`/`ub4` | [`u32`]
    /// `ub8*`/`ub8` | [`u64`]
    /// `boolean*`/`boolean` | [`bool`]
    /// pointer types such as `OCISession**`/`OCISession*` | `*mut c_void`
    /// `oratext**`/`oratext*` | [`str`] [^str]
    /// `ub1*`(with length; value is copied; is really a ub1 array) | `[u8]` [^u8slice]
    ///
    /// The following table is the mapping of predefined types based on basic data types.
    /// They are designed for specific attributes.
    ///
    /// Types in Oracle manual | Rust type
    /// ---|---
    /// `ub8*` | [`DurationUsecU64`] which gets u64 values representing microsecods as [`Duration`]
    /// `ub1*` | [`MaxStringSize`] which gets ub1 values as variants of [`MaxStringSize`]
    ///
    /// Look at the source code of [`DurationUsecU64`] and [`MaxStringSize`] as samples
    /// when you need to implement your own data types.
    ///
    /// [^str]: Values are got as [`String`] because [`str`] implements [`ToOwned`] whose associate type `Owned` is [`String`].
    ///
    /// [^u8slice]: Values are got as `Vec<u8>` because `[T]` where T: Clone implements [`ToOwned`] whose associate type `Owned` is `Vec<T>`.
    ///
    type DataType: DataType + ?Sized;

    /// Attribute number defined in `oci.h` included in Oracle Instant Client SDK
    const ATTR_NUM: u32;
}

/// A type parameter for [`Connection::oci_attr`] to get [`OCI_ATTR_VARTYPE_MAXLEN_COMPAT`] as [`MaxStringSize`].
/// which controls the maximum size of `VARCHAR2`, `NVARCHAR` and `RAW`.
///
/// This corresponds to the result of the following SQL statement when
/// a database user has a privilege to access `v$parameter`.
///
/// ```sql
/// select value from v$parameter where name = 'max_string_size'
/// ```
///
/// [`OCI_ATTR_VARTYPE_MAXLEN_COMPAT`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-D8EE68EB-7E38-4068-B06E-DF5686379E5E__GUID-2EFB39BC-6131-4EAD-BBF6-7CA8E5F2BBF4
#[derive(Debug)]
pub struct VarTypeMaxLenCompat;
const OCI_ATTR_VARTYPE_MAXLEN_COMPAT: u32 = 489;
unsafe impl OciAttr for VarTypeMaxLenCompat {
    type HandleType = SvcCtx;
    type Mode = Read;
    type DataType = MaxStringSize;
    const ATTR_NUM: u32 = OCI_ATTR_VARTYPE_MAXLEN_COMPAT;
}

/// A type parameter for [`Connection::oci_attr`] to get [`OCI_ATTR_CALL_TIME`] as [`Duration`][],
/// which is the server-side time for the preceding call
///
/// Set `true` to [`CollectCallTime`] in advance.
///
/// # Examples
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util;
/// use oracle::oci_attr::CallTime;
/// use oracle::oci_attr::CollectCallTime;
/// use std::time::Duration;
/// # let mut conn = test_util::connect()?;
/// # if !test_util::check_version(&conn, &test_util::VER11_2, &test_util::VER18)? {
/// #     return Ok(());
/// # }
///
/// // Enable CollectCallTime
/// conn.set_oci_attr::<CollectCallTime>(&true)?;
///
/// // This SQL consumes one second in the server-side.
/// conn.execute("begin dbms_session.sleep(1); end;", &[])?;
/// let call_time = conn.oci_attr::<CallTime>()?;
/// assert!(call_time >= Duration::from_secs(1), "call_time is {:?}.", call_time);
/// # Ok::<(), Error>(())
/// ```
///
/// [`OCI_ATTR_CALL_TIME`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FB263210-118E-4DB3-A840-1769EF0CB977__GUID-AA22FE4F-8942-4819-B01F-068DCEAE9B72
/// [`Duration`]: std::time::Duration
pub struct CallTime;
const OCI_ATTR_CALL_TIME: u32 = 370;
unsafe impl OciAttr for CallTime {
    type HandleType = Session;
    type Mode = Read;
    type DataType = DurationUsecU64;
    const ATTR_NUM: u32 = OCI_ATTR_CALL_TIME;
}

/// A type parameter for [`Connection::oci_attr`] and [`Connection::set_oci_attr`] to get and set [`OCI_ATTR_COLLECT_CALL_TIME`],
/// which causes the server to measure call time for each subsequent OCI call
///
/// See [`CallTime`].
///
/// [`OCI_ATTR_COLLECT_CALL_TIME`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FB263210-118E-4DB3-A840-1769EF0CB977__GUID-D4B6CBB6-5627-474C-ABE6-F2CE694DE62B
pub struct CollectCallTime;
const OCI_ATTR_COLLECT_CALL_TIME: u32 = 369;
unsafe impl OciAttr for CollectCallTime {
    type HandleType = Session;
    type Mode = ReadWrite;
    type DataType = bool;
    const ATTR_NUM: u32 = OCI_ATTR_COLLECT_CALL_TIME;
}

/// A type parameter for [`Connection::oci_attr`] and [`Connection::set_oci_attr`] to get and set [`OCI_ATTR_DEFAULT_LOBPREFETCH_SIZE`],
/// which specifies the default prefetch buffer size for each LOB locator
///
/// # Examples
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util;
/// # use oracle::sql_type::Clob;
/// use oracle::oci_attr::DefaultLobPrefetchSize;
/// # let mut conn = test_util::connect()?;
///
/// let lob_size = 64 * 1024;
/// conn.set_oci_attr::<DefaultLobPrefetchSize>(&lob_size)?;
///
/// # conn.execute("insert into TestCLOBs values (1, '11111111111111111111111111111')", &[])?;
/// let mut stmt = conn
///     .statement("select CLOBCol from TestCLOBs where IntCol = :1")
///     .lob_locator()
///     .build()?;
/// let lob = stmt.query_row_as::<Clob>(&[&1])?;
/// # Ok::<(), Error>(())
/// ```
///
/// [`OCI_ATTR_DEFAULT_LOBPREFETCH_SIZE`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FB263210-118E-4DB3-A840-1769EF0CB977__GUID-13400E7D-C1E9-49AB-AD9B-132CBF11E16C
pub struct DefaultLobPrefetchSize;
const OCI_ATTR_DEFAULT_LOBPREFETCH_SIZE: u32 = 438;
unsafe impl OciAttr for DefaultLobPrefetchSize {
    type HandleType = Session;
    type Mode = ReadWrite;
    type DataType = u32;
    const ATTR_NUM: u32 = OCI_ATTR_DEFAULT_LOBPREFETCH_SIZE;
}

/// A type parameter for [`Connection::oci_attr`] to get [`OCI_ATTR_MAX_OPEN_CURSORS`],
/// which is the maximum number of SQL statements that can be opened in one session
///
/// This returns the same value with the result of the following SQL statement when
/// a database user has a privilege to access `v$parameter`.
///
/// ```sql
/// select value from v$parameter where name = 'open_cursors'
/// ```
///
/// Note that this attribute returns a proper value only when connected to a 12.1 server or later.
///
/// [`OCI_ATTR_MAX_OPEN_CURSORS`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FB263210-118E-4DB3-A840-1769EF0CB977__GUID-0F30D36A-E9E5-4CDB-BF53-C2C876C09E00
pub struct MaxOpenCursors;
const OCI_ATTR_MAX_OPEN_CURSORS: u32 = 471;
unsafe impl OciAttr for MaxOpenCursors {
    type HandleType = Session;
    type Mode = Read;
    type DataType = u32;
    const ATTR_NUM: u32 = OCI_ATTR_MAX_OPEN_CURSORS;
}

/// A type parameter for [`Connection::oci_attr`] to get [`OCI_ATTR_TRANSACTION_IN_PROGRESS`] as `bool`,
/// which indicates whether the connection has a currently active transaction.
///
/// Note that this requires Oracle client 12.1 or later.
///
/// # Examples
///
/// ```
/// # use oracle::Error;
/// # use oracle::Version;
/// # use oracle::test_util;
/// use oracle::oci_attr::TransactionInProgress;
/// # if Version::client()? < test_util::VER12_1 {
/// #     return Ok(());
/// # }
/// # let mut conn = test_util::connect()?;
/// # conn.execute("drop table test_sql_fn_code purge", &[]);
///
/// // no active transaction at first
/// assert_eq!(conn.oci_attr::<TransactionInProgress>()?, false);
///
/// // start a transaction
/// conn.execute("insert into TestTempTable values (1, 'val1')", &[])?;
/// assert_eq!(conn.oci_attr::<TransactionInProgress>()?, true);
///
/// // rollback the transction
/// conn.rollback()?;
/// assert_eq!(conn.oci_attr::<TransactionInProgress>()?, false);
/// # Ok::<(), Error>(())
/// ```
/// [`OCI_ATTR_TRANSACTION_IN_PROGRESS`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FB263210-118E-4DB3-A840-1769EF0CB977__GUID-BCECC9A1-5B02-428F-8A1D-20C9A7997AE5
pub struct TransactionInProgress;
const OCI_ATTR_TRANSACTION_IN_PROGRESS: u32 = 484;
unsafe impl OciAttr for TransactionInProgress {
    type HandleType = Session;
    type Mode = Read;
    type DataType = bool;
    const ATTR_NUM: u32 = OCI_ATTR_TRANSACTION_IN_PROGRESS;
}

/// A type parameter for [`Statement::oci_attr`] to get [`OCI_ATTR_SQLFNCODE`],
/// which is the function code of the SQL command associated with the statement.
///
/// Note that the attribute must be read after the statement is executed.
///
/// # Examples
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util;
/// use oracle::oci_attr::SqlFnCode;
/// # use std::thread::sleep;
/// # use std::time::Duration;
/// # let mut conn = test_util::connect()?;
///
/// let stmt = conn.execute("insert into TestNumbers values(11, 12, 13, 14, 15)", &[])?;
/// assert_eq!(stmt.oci_attr::<SqlFnCode>()?, 3);
///
/// let stmt = conn.execute("update TestNumbers set NumberCol = 13 where IntCol = 11", &[])?;
/// assert_eq!(stmt.oci_attr::<SqlFnCode>()?, 5);
///
/// let stmt = conn.execute("delete TestNumbers where IntCol = 11", &[])?;
/// assert_eq!(stmt.oci_attr::<SqlFnCode>()?, 9);
///
/// # conn.rollback()?;
/// # Ok::<(), Error>(())
/// ```
///
/// [`Statement::oci_attr`]: crate::Statement::oci_attr
/// [`OCI_ATTR_SQLFNCODE`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-A251CF91-EB9F-4DBC-8BB8-FB5EA92C20DE__GUID-9E3D8A93-DF13-4023-8444-3F06131D26FB
pub struct SqlFnCode;
const OCI_ATTR_SQLFNCODE: u32 = 10;
unsafe impl OciAttr for SqlFnCode {
    type HandleType = Stmt;
    type Mode = Read;
    type DataType = u16;
    const ATTR_NUM: u32 = OCI_ATTR_SQLFNCODE;
}

/// A type parameter for [`Statement::oci_attr`] to get [`OCI_ATTR_STATEMENT`],
/// which is the text of the SQL statement prepared.
///
/// # Examples
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util;
/// use oracle::oci_attr::Statement;
/// # let mut conn = test_util::connect()?;
///
/// let mut stmt = conn.statement("select * from dual").build()?;
/// assert_eq!(stmt.oci_attr::<Statement>()?, "select * from dual");
/// # Ok::<(), Error>(())
/// ```
///
/// [`Statement::oci_attr`]: crate::Statement::oci_attr
/// [`OCI_ATTR_STATEMENT`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-A251CF91-EB9F-4DBC-8BB8-FB5EA92C20DE__GUID-30B1693F-EFC7-4108-8F06-0EC1DC3785FB
pub struct Statement;
const OCI_ATTR_STATEMENT: u32 = 144;
unsafe impl OciAttr for Statement {
    type HandleType = Stmt;
    type Mode = Read;
    type DataType = str;
    const ATTR_NUM: u32 = OCI_ATTR_STATEMENT;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;
    use crate::Result;

    struct StmtCacheSize;
    unsafe impl OciAttr for StmtCacheSize {
        type HandleType = SvcCtx;
        type Mode = ReadWrite;
        type DataType = u32;
        const ATTR_NUM: u32 = 176;
    }

    struct InternalName;
    unsafe impl OciAttr for InternalName {
        type HandleType = Server;
        type Mode = ReadWrite;
        type DataType = str;
        const ATTR_NUM: u32 = 25;
    }

    struct Module;
    unsafe impl OciAttr for Module {
        type HandleType = Session;
        type Mode = Write;
        type DataType = str;
        const ATTR_NUM: u32 = 366;
    }

    #[test]
    fn read_write_svcctx_u32_attr() -> Result<()> {
        let mut conn = test_util::connect()?;
        let size = conn.stmt_cache_size()?;
        assert_eq!(conn.oci_attr::<StmtCacheSize>()?, size);
        let new_size = size + 20;
        conn.set_oci_attr::<StmtCacheSize>(&new_size)?;
        assert_eq!(conn.oci_attr::<StmtCacheSize>()?, new_size);
        Ok(())
    }

    #[test]
    fn read_write_server_str_attr() -> Result<()> {
        let mut conn = test_util::connect()?;
        conn.set_oci_attr::<InternalName>("test internal name")?;
        assert_eq!(conn.oci_attr::<InternalName>()?, "test internal name");
        Ok(())
    }

    #[test]
    fn write_session_str_attr() -> Result<()> {
        let mut conn = test_util::connect()?;
        conn.set_oci_attr::<Module>("test module name")?;
        let module =
            conn.query_row_as::<String>("select sys_context('USERENV', 'MODULE') from dual", &[])?;
        assert_eq!(module, "test module name");
        Ok(())
    }
}
