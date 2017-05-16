//
// ffi code for dpi.h
//
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::c_void;
use libc::c_char;
use libc::c_int;
use libc::c_long;
use libc::c_uint;
use libc::c_float;
use libc::c_double;
use libc::int8_t;
use libc::int16_t;
use libc::int32_t;
use libc::int64_t;
use libc::uint8_t;
use libc::uint16_t;
use libc::uint32_t;
use libc::uint64_t;

// define ODPI-C version information
pub const DPI_MAJOR_VERSION: c_uint = 2;
pub const DPI_MINOR_VERSION: c_uint = 0;

// define default array size to use
pub const DPI_DEFAULT_FETCH_ARRAY_SIZE: uint32_t = 100;

// define ping interval (in seconds) used when getting connections
pub const DPI_DEFAULT_PING_INTERVAL: c_int = 60;

// define ping timeout (in milliseconds) used when getting connections
pub const DPI_DEFAULT_PING_TIMEOUT: c_int = 5000;

// define constants for dequeue wait (AQ)
pub const DPI_DEQ_WAIT_NO_WAIT: uint32_t = 0;
pub const DPI_DEQ_WAIT_FOREVER: uint32_t = 0xffffffff;

// define maximum precision that can be supported by an int64_t value
pub const DPI_MAX_INT64_PRECISION: int16_t =18;

// define constants for success and failure of methods
pub const DPI_SUCCESS: c_int = 0;
pub const DPI_FAILURE: c_int = -1;

//-----------------------------------------------------------------------------
// Enumerations
//-----------------------------------------------------------------------------

// connection/pool authorization modes
pub type dpiAuthMode = c_int;
pub const DPI_MODE_AUTH_DEFAULT:c_int = 0x00000000;         // OCI_DEFAULT
pub const DPI_MODE_AUTH_SYSDBA:c_int = 0x00000002;          // OCI_SYSDBA
pub const DPI_MODE_AUTH_SYSOPER:c_int = 0x00000004;         // OCI_SYSOPER
pub const DPI_MODE_AUTH_PRELIM:c_int = 0x00000008;          // OCI_PRELIM_AUTH
pub const DPI_MODE_AUTH_SYSASM:c_int = 0x00008000;          // OCI_SYSASM

// connection close modes
pub type dpiConnCloseMode = c_int;
pub const DPI_MODE_CONN_CLOSE_DEFAULT:c_int = 0x0000;       // OCI_DEFAULT
pub const DPI_MODE_CONN_CLOSE_DROP:c_int = 0x0001;          // OCI_SESSRLS_DROPSESS
pub const DPI_MODE_CONN_CLOSE_RETAG:c_int = 0x0002;         // OCI_SESSRLS_RETAG

// connection/pool creation modes
pub type dpiCreateMode = c_int;
pub const DPI_MODE_CREATE_DEFAULT:c_int = 0x00000000;       // OCI_DEFAULT
pub const DPI_MODE_CREATE_THREADED:c_int = 0x00000001;      // OCI_THREADED
pub const DPI_MODE_CREATE_EVENTS:c_int = 0x00000004;        // OCI_EVENTS

// dequeue modes for advanced queuing
pub type dpiDeqMode = c_int;
pub const DPI_MODE_DEQ_BROWSE:c_int = 1;                    // OCI_DEQ_BROWSE
pub const DPI_MODE_DEQ_LOCKED:c_int = 2;                    // OCI_DEQ_LOCKED
pub const DPI_MODE_DEQ_REMOVE:c_int = 3;                    // OCI_DEQ_REMOVE
pub const DPI_MODE_DEQ_REMOVE_NO_DATA:c_int = 4;            // OCI_DEQ_REMOVE_NODATA

// dequeue navigation flags for advanced queuing
pub type dpiDeqNavigation = c_int;
pub const DPI_DEQ_NAV_FIRST_MSG:c_int = 1;                  // OCI_DEQ_FIRST_MSG
pub const DPI_DEQ_NAV_NEXT_TRANSACTION:c_int = 2;           // OCI_DEQ_NEXT_TRANSACTION
pub const DPI_DEQ_NAV_NEXT_MSG:c_int = 3;                   // OCI_DEQ_NEXT_MSG

// event types
pub type dpiEventType = c_int;
pub const DPI_EVENT_NONE:c_int = 0;                         // OCI_EVENT_NONE
pub const DPI_EVENT_STARTUP:c_int = 1;                      // OCI_EVENT_STARTUP
pub const DPI_EVENT_SHUTDOWN:c_int = 2;                     // OCI_EVENT_SHUTDOWN
pub const DPI_EVENT_SHUTDOWN_ANY:c_int = 3;                 // OCI_EVENT_SHUTDOWN_ANY
pub const DPI_EVENT_DROP_DB:c_int = 4;                      // OCI_EVENT_DROP_DB
pub const DPI_EVENT_DEREG:c_int = 5;                        // OCI_EVENT_DEREG
pub const DPI_EVENT_OBJCHANGE:c_int = 6;                    // OCI_EVENT_OBJCHANGE
pub const DPI_EVENT_QUERYCHANGE:c_int = 7;                  // OCI_EVENT_QUERYCHANGE

// statement execution modes
pub type dpiExecMode = c_int;
pub const DPI_MODE_EXEC_DEFAULT:c_int = 0x00000000;             // OCI_DEFAULT
pub const DPI_MODE_EXEC_DESCRIBE_ONLY:c_int = 0x00000010;       // OCI_DESCRIBE_ONLY
pub const DPI_MODE_EXEC_COMMIT_ON_SUCCESS:c_int = 0x00000020;   // OCI_COMMIT_ON_SUCCESS
pub const DPI_MODE_EXEC_BATCH_ERRORS:c_int = 0x00000080;        // OCI_BATCH_ERRORS
pub const DPI_MODE_EXEC_PARSE_ONLY:c_int = 0x00000100;          // OCI_PARSE_ONLY
pub const DPI_MODE_EXEC_ARRAY_DML_ROWCOUNTS:c_int = 0x00100000; // OCI_RETURN_ROW_COUNT_ARRAY

// statement fetch modes
pub type dpiFetchMode = c_int;
pub const DPI_MODE_FETCH_NEXT:c_int = 0x00000002;           // OCI_FETCH_NEXT
pub const DPI_MODE_FETCH_FIRST:c_int = 0x00000004;          // OCI_FETCH_FIRST
pub const DPI_MODE_FETCH_LAST:c_int = 0x00000008;           // OCI_FETCH_LAST
pub const DPI_MODE_FETCH_PRIOR:c_int = 0x00000010;          // OCI_FETCH_PRIOR
pub const DPI_MODE_FETCH_ABSOLUTE:c_int = 0x00000020;       // OCI_FETCH_ABSOLUTE
pub const DPI_MODE_FETCH_RELATIVE:c_int = 0x00000040;       // OCI_FETCH_RELATIVE

// message delivery modes in advanced queuing
pub type dpiMessageDeliveryMode = c_int;
pub const DPI_MODE_MSG_PERSISTENT:c_int = 1;                // OCI_MSG_PERSISTENT
pub const DPI_MODE_MSG_BUFFERED:c_int = 2;                  // OCI_MSG_BUFFERED
pub const DPI_MODE_MSG_PERSISTENT_OR_BUFFERED:c_int = 3;    // OCI_MSG_PERSISTENT_OR_BUFFERED

// dpiMessageState: message states in advanced queuing
pub type dpiMessageState = c_int;
pub const DPI_MSG_STATE_READY:c_int = 0;                    // OCI_MSG_READY
pub const DPI_MSG_STATE_WAITING:c_int = 1;                  // OCI_MSG_WAITING
pub const DPI_MSG_STATE_PROCESSED:c_int = 2;                // OCI_MSG_PROCESSED
pub const DPI_MSG_STATE_EXPIRED:c_int = 3;                  // OCI_MSG_EXPIRED

// native C types
pub type dpiNativeTypeNum = c_int;
pub const DPI_NATIVE_TYPE_INT64:c_int       = 3000;
pub const DPI_NATIVE_TYPE_UINT64:c_int      = 3001;
pub const DPI_NATIVE_TYPE_FLOAT:c_int       = 3002;
pub const DPI_NATIVE_TYPE_DOUBLE:c_int      = 3003;
pub const DPI_NATIVE_TYPE_BYTES:c_int       = 3004;
pub const DPI_NATIVE_TYPE_TIMESTAMP:c_int   = 3005;
pub const DPI_NATIVE_TYPE_INTERVAL_DS:c_int = 3006;
pub const DPI_NATIVE_TYPE_INTERVAL_YM:c_int = 3007;
pub const DPI_NATIVE_TYPE_LOB:c_int         = 3008;
pub const DPI_NATIVE_TYPE_OBJECT:c_int      = 3009;
pub const DPI_NATIVE_TYPE_STMT:c_int        = 3010;
pub const DPI_NATIVE_TYPE_BOOLEAN:c_int     = 3011;
pub const DPI_NATIVE_TYPE_ROWID:c_int       = 3012;

// operation codes (database change and continuous query notification)
pub type dpiOpCode = c_int;
pub const DPI_OPCODE_ALL_OPS:c_int = 0x0;                   // OCI_OPCODE_ALLOPS
pub const DPI_OPCODE_ALL_ROWS:c_int = 0x1;                  // OCI_OPCODE_ALLROWS
pub const DPI_OPCODE_INSERT:c_int = 0x2;                    // OCI_OPCODE_INSERT
pub const DPI_OPCODE_UPDATE:c_int = 0x4;                    // OCI_OPCODE_UPDATE
pub const DPI_OPCODE_DELETE:c_int = 0x8;                    // OCI_OPCODE_DELETE
pub const DPI_OPCODE_ALTER:c_int = 0x10;                    // OCI_OPCODE_ALTER
pub const DPI_OPCODE_DROP:c_int = 0x20;                     // OCI_OPCODE_DROP
pub const DPI_OPCODE_UNKNOWN:c_int = 0x40;                  // OCI_OPCODE_UNKNOWN

// dpiOracleTypeNum: Oracle types
pub type dpiOracleTypeNum = c_int;
pub const DPI_ORACLE_TYPE_NONE:c_int          = 2000;
pub const DPI_ORACLE_TYPE_VARCHAR:c_int       = 2001;
pub const DPI_ORACLE_TYPE_NVARCHAR:c_int      = 2002;
pub const DPI_ORACLE_TYPE_CHAR:c_int          = 2003;
pub const DPI_ORACLE_TYPE_NCHAR:c_int         = 2004;
pub const DPI_ORACLE_TYPE_ROWID:c_int         = 2005;
pub const DPI_ORACLE_TYPE_RAW:c_int           = 2006;
pub const DPI_ORACLE_TYPE_NATIVE_FLOAT:c_int  = 2007;
pub const DPI_ORACLE_TYPE_NATIVE_DOUBLE:c_int = 2008;
pub const DPI_ORACLE_TYPE_NATIVE_INT:c_int    = 2009;
pub const DPI_ORACLE_TYPE_NUMBER:c_int        = 2010;
pub const DPI_ORACLE_TYPE_DATE:c_int          = 2011;
pub const DPI_ORACLE_TYPE_TIMESTAMP:c_int     = 2012;
pub const DPI_ORACLE_TYPE_TIMESTAMP_TZ:c_int  = 2013;
pub const DPI_ORACLE_TYPE_TIMESTAMP_LTZ:c_int = 2014;
pub const DPI_ORACLE_TYPE_INTERVAL_DS:c_int   = 2015;
pub const DPI_ORACLE_TYPE_INTERVAL_YM:c_int   = 2016;
pub const DPI_ORACLE_TYPE_CLOB:c_int          = 2017;
pub const DPI_ORACLE_TYPE_NCLOB:c_int         = 2018;
pub const DPI_ORACLE_TYPE_BLOB:c_int          = 2019;
pub const DPI_ORACLE_TYPE_BFILE:c_int         = 2020;
pub const DPI_ORACLE_TYPE_STMT:c_int          = 2021;
pub const DPI_ORACLE_TYPE_BOOLEAN:c_int       = 2022;
pub const DPI_ORACLE_TYPE_OBJECT:c_int        = 2023;
pub const DPI_ORACLE_TYPE_LONG_VARCHAR:c_int  = 2024;
pub const DPI_ORACLE_TYPE_LONG_RAW:c_int      = 2025;
pub const DPI_ORACLE_TYPE_NATIVE_UINT:c_int   = 2026;
pub const DPI_ORACLE_TYPE_MAX:c_int           = 2027;

// session pool close modes
pub type dpiPoolCloseMode = c_int;
pub const DPI_MODE_POOL_CLOSE_DEFAULT:c_int = 0x0000;       // OCI_DEFAULT
pub const DPI_MODE_POOL_CLOSE_FORCE:c_int = 0x0001;         // OCI_SPD_FORCE

// modes used when acquiring a connection from a session pool
pub type dpiPoolGetMode = c_int;
pub const DPI_MODE_POOL_GET_WAIT:c_int = 0;                 // OCI_SPOOL_ATTRVAL_WAIT
pub const DPI_MODE_POOL_GET_NOWAIT:c_int = 1;               // OCI_SPOOL_ATTRVAL_NOWAIT
pub const DPI_MODE_POOL_GET_FORCEGET:c_int = 2;             // OCI_SPOOL_ATTRVAL_FORCEGET

// purity values when acquiring a connection from a pool
pub type dpiPurity = c_int;
pub const DPI_PURITY_DEFAULT:c_int = 0;                     // OCI_ATTR_PURITY_DEFAULT
pub const DPI_PURITY_NEW:c_int = 1;                         // OCI_ATTR_PURITY_NEW
pub const DPI_PURITY_SELF:c_int = 2;                        // OCI_ATTR_PURITY_SELF

// database shutdown modes
pub type dpiShutdownMode = c_int;
pub const DPI_MODE_SHUTDOWN_DEFAULT:c_int = 0;              // OCI_DEFAULT
pub const DPI_MODE_SHUTDOWN_TRANSACTIONAL:c_int = 1;        // OCI_DBSHUTDOWN_TRANSACTIONAL
pub const DPI_MODE_SHUTDOWN_TRANSACTIONAL_LOCAL:c_int = 2;  // OCI_DBSHUTDOWN_TRANSACTIONAL_LOCAL
pub const DPI_MODE_SHUTDOWN_IMMEDIATE:c_int = 3;            // OCI_DBSHUTDOWN_IMMEDIATE
pub const DPI_MODE_SHUTDOWN_ABORT:c_int = 4;                // OCI_DBSHUTDOWN_ABORT
pub const DPI_MODE_SHUTDOWN_FINAL:c_int = 5;                // OCI_DBSHUTDOWN_FINAL

// database startup modes
pub type dpiStartupMode = c_int;
pub const DPI_MODE_STARTUP_DEFAULT:c_int = 0;               // OCI_DEFAULT
pub const DPI_MODE_STARTUP_FORCE:c_int = 1;                 // OCI_DBSTARTUPFLAG_FORCE
pub const DPI_MODE_STARTUP_RESTRICT:c_int = 2;              // OCI_DBSTARTUPFLAG_RESTRICT

// statement types
pub type dpiStatementType = c_int;
pub const DPI_STMT_TYPE_UNKNOWN:c_int = 0; // Not in dpi.h
pub const DPI_STMT_TYPE_SELECT:c_int = 1;                   // OCI_STMT_SELECT
pub const DPI_STMT_TYPE_UPDATE:c_int = 2;                   // OCI_STMT_UPDATE
pub const DPI_STMT_TYPE_DELETE:c_int = 3;                   // OCI_STMT_DELETE
pub const DPI_STMT_TYPE_INSERT:c_int = 4;                   // OCI_STMT_INSERT
pub const DPI_STMT_TYPE_CREATE:c_int = 5;                   // OCI_STMT_CREATE
pub const DPI_STMT_TYPE_DROP:c_int = 6;                     // OCI_STMT_DROP
pub const DPI_STMT_TYPE_ALTER:c_int = 7;                    // OCI_STMT_ALTER
pub const DPI_STMT_TYPE_BEGIN:c_int = 8;                    // OCI_STMT_BEGIN
pub const DPI_STMT_TYPE_DECLARE:c_int = 9;                  // OCI_STMT_DECLARE
pub const DPI_STMT_TYPE_CALL:c_int = 10;                    // OCI_STMT_CALL

// subscription namespaces
pub type dpiSubscrNamespace = c_int;
pub const DPI_SUBSCR_NAMESPACE_DBCHANGE:c_int = 2;          // OCI_SUBSCR_NAMESPACE_DBCHANGE

// subscription protocols
pub type dpiSubscrProtocol = c_int;
pub const DPI_SUBSCR_PROTO_CALLBACK:c_int = 0;              // OCI_SUBSCR_PROTO_OCI
pub const DPI_SUBSCR_PROTO_MAIL:c_int = 1;                  // OCI_SUBSCR_PROTO_MAIL
pub const DPI_SUBSCR_PROTO_PLSQL:c_int = 2;                 // OCI_SUBSCR_PROTO_SERVER
pub const DPI_SUBSCR_PROTO_HTTP:c_int = 3;                  // OCI_SUBSCR_PROTO_HTTP

// subscription quality of service
pub type dpiSubscrQOS = c_int;
pub const DPI_SUBSCR_QOS_RELIABLE:c_int = 0x01;
pub const DPI_SUBSCR_QOS_DEREG_NFY:c_int = 0x02;
pub const DPI_SUBSCR_QOS_ROWIDS:c_int = 0x04;
pub const DPI_SUBSCR_QOS_QUERY:c_int = 0x08;
pub const DPI_SUBSCR_QOS_BEST_EFFORT:c_int = 0x10;

// visibility of messages in advanced queuing
pub type dpiVisibility = c_int;
pub const DPI_VISIBILITY_IMMEDIATE:c_int = 1;               // OCI_DEQ_IMMEDIATE
pub const DPI_VISIBILITY_ON_COMMIT:c_int = 2;               // OCI_DEQ_ON_COMMIT

//-----------------------------------------------------------------------------
// Handle Types
//-----------------------------------------------------------------------------
pub enum dpiConn {}
pub enum dpiPool {}
pub enum dpiStmt {}
pub enum dpiVar {}
pub enum dpiLob {}
pub enum dpiObject {}
pub enum dpiObjectAttr {}
pub enum dpiObjectType {}
pub enum dpiRowid {}
pub enum dpiSubscr {}
pub enum dpiDeqOptions {}
pub enum dpiEnqOptions {}
pub enum dpiMsgProps {}

//-----------------------------------------------------------------------------
// Complex Native Data Types (used for transferring data to/from ODPI-C)
//-----------------------------------------------------------------------------

// structure used for transferring byte strings to/from ODPI-C
#[repr(C)]
pub struct dpiBytes {
    pub ptr: *mut c_char,
    pub length: uint32_t,
    pub encoding: *const c_char,
}

// structure used for transferring day/seconds intervals to/from ODPI-C
#[repr(C)]
pub struct dpiIntervalDS {
    pub days: int32_t,
    pub hours: int32_t,
    pub minutes: int32_t,
    pub seconds: int32_t,
    pub fseconds: int32_t,
}

// structure used for transferring years/months intervals to/from ODPI-C
#[repr(C)]
pub struct dpiIntervalYM {
    pub years: int32_t,
    pub months: int32_t,
}

// structure used for transferring dates to/from ODPI-C
#[repr(C)]
pub struct dpiTimestamp {
    pub year: int16_t,
    pub month: uint8_t,
    pub day: uint8_t,
    pub hour: uint8_t,
    pub minute: uint8_t,
    pub second: uint8_t,
    pub fsecond: uint32_t,
    pub tzHourOffset: int8_t,
    pub tzMinuteOffset: int8_t,
}

//-----------------------------------------------------------------------------
// Other Types
//-----------------------------------------------------------------------------

pub enum dpiContext {}

// structure used for application context
#[repr(C)]
pub struct dpiAppContext {
    namespaceName: *const c_char,
    namespaceNameLength: uint32_t,
    name: *const c_char,
    nameLength: uint32_t,
    value: *const c_char,
    valueLength: uint32_t,
}

// structure used for common parameters used for creating standalone
// connections and session pools
#[repr(C)]
pub struct dpiCommonCreateParams {
    pub createMode: dpiCreateMode,
    pub encoding: *const c_char,
    pub nencoding: *const c_char,
    pub edition: *const c_char,
    pub editionLength: uint32_t,
    pub driverName: *const c_char,
    pub driverNameLength: uint32_t,
}

// structure used for creating connections
#[repr(C)]
#[derive(Clone)]
pub struct dpiConnCreateParams {
    pub authMode: dpiAuthMode,
    pub connectionClass: *const c_char,
    pub connectionClassLength: uint32_t,
    pub purity: dpiPurity,
    pub newPassword: *const c_char,
    pub newPasswordLength: uint32_t,
    pub appContext: *mut dpiAppContext,
    pub numAppContext: uint32_t,
    pub externalAuth: c_int,
    pub externalHandle: *mut c_void,
    pub pool: *mut dpiPool,
    pub tag: *const c_char,
    pub tagLength: uint32_t,
    pub matchAnyTag: c_int,
    pub outTag: *const c_char,
    pub outTagLength: uint32_t,
    pub outTagFound: c_int,
}

// structure used for transferring data to/from ODPI-C
#[repr(C)]
pub struct dpiData {
    pub isNull: c_int,
    //union {
    //    asBoolean: c_int,
    //    asInt64: int64_t,
    //    asUint64: uint64_t,
    //    asFloat: c_float,
    //    asDouble: c_double,
    //    asBytes: dpiBytes,
    //    asTimestamp: dpiTimestamp,
    //    asIntervalDS: dpiIntervalDS,
    //    asIntervalYM: dpiIntervalYM,
    //    asLOB: *mut dpiLob,
    //    asObject: *mut dpiObject,
    //    asStmt: *mut dpiStmt,
    //    asRowid: *mut dpiRowid,
    //} value,
}

// structure used for transferring encoding information from ODPI-C
#[repr(C)]
pub struct dpiEncodingInfo {
    encoding: *const c_char,
    maxBytesPerCharacter: int32_t,
    nencoding: *const c_char,
    nmaxBytesPerCharacter: int32_t,
}

// structure used for transferring error information from ODPI-C
#[repr(C)]
pub struct dpiErrorInfo {
    pub code: int32_t,
    pub offset: uint16_t,
    pub message: *const c_char,
    pub messageLength: uint32_t,
    pub encoding: *const c_char,
    pub fnName: *const c_char,
    pub action: *const c_char,
    pub sqlState: *const c_char,
    pub isRecoverable: c_int,
}

// structure used for transferring object attribute information from ODPI-C
#[repr(C)]
pub struct dpiObjectAttrInfo {
    name: *const c_char,
    nameLength: uint32_t ,
    oracleTypeNum: dpiOracleTypeNum,
    defaultNativeTypeNum: dpiNativeTypeNum,
    objectType: *mut dpiObjectType,
}

// structure used for transferring object type information from ODPI-C
#[repr(C)]
pub struct dpiObjectTypeInfo {
    schema: *const c_char,
    schemaLength: uint32_t,
    name: *const c_char,
    nameLength: uint32_t,
    isCollection: c_int,
    elementOracleTypeNum: dpiOracleTypeNum,
    elementDefaultNativeTypeNum: dpiNativeTypeNum,
    elementObjectType: *mut dpiObjectType,
    numAttributes: uint16_t,
}

// structure used for creating pools
#[repr(C)]
pub struct dpiPoolCreateParams {
    pub minSessions: uint32_t,
    pub maxSessions: uint32_t,
    pub sessionIncrement: uint32_t,
    pub pingInterval: c_int,
    pub pingTimeout: c_int,
    pub homogeneous: c_int,
    pub externalAuth: c_int,
    pub getMode: dpiPoolGetMode,
    pub outPoolName: *const c_char,
    pub outPoolNameLength: uint32_t,
}

// structure used for transferring query metadata from ODPI-C
#[repr(C)]
pub struct dpiQueryInfo {
    pub name: *const c_char,
    pub nameLength: uint32_t,
    pub oracleTypeNum: dpiOracleTypeNum,
    pub defaultNativeTypeNum: dpiNativeTypeNum,
    pub dbSizeInBytes: uint32_t,
    pub clientSizeInBytes: uint32_t,
    pub sizeInChars: uint32_t,
    pub precision: int16_t,
    pub scale: int8_t,
    pub nullOk: c_int,
    pub objectType: *mut dpiObjectType,
}

// structure used for transferring statement information from ODPI-C
#[repr(C)]
#[derive(Default)]
pub struct dpiStmtInfo {
    pub isQuery: c_int,
    pub isPLSQL: c_int,
    pub isDDL: c_int,
    pub isDML: c_int,
    pub statementType: dpiStatementType,
    pub isReturning: c_int,
}

// callback for subscriptions
pub type dpiSubscrCallback = extern "C" fn(context: *mut c_void, message: *mut dpiSubscrMessage);

// structure used for creating subscriptions
#[repr(C)]
pub struct dpiSubscrCreateParams {
    pub subscrNamespace: dpiSubscrNamespace,
    pub protocol: dpiSubscrProtocol,
    pub qos: dpiSubscrQOS,
    pub operations: dpiOpCode,
    pub portNumber: uint32_t,
    pub timeout: uint32_t,
    pub name: *const c_char,
    pub nameLength: uint32_t,
    pub callback: Option<dpiSubscrCallback>,
    pub callbackContext: *mut c_void,
    pub recipientName: *const c_char,
    pub recipientNameLength: uint32_t,
}

// structure used for transferring messages in subscription callbacks
#[repr(C)]
pub struct dpiSubscrMessage {
    eventType: dpiEventType,
    dbName: *const c_char,
    dbNameLength: uint32_t,
    tables: *mut dpiSubscrMessageTable,
    numTables: uint32_t,
    queries: *mut dpiSubscrMessageQuery,
    numQueries: uint32_t,
    errorInfo: *mut dpiErrorInfo,
}

// structure used for transferring query information in messages in
// subscription callbacks (continuous query notification)
#[repr(C)]
pub struct dpiSubscrMessageQuery {
    id: uint64_t ,
    operation: dpiOpCode,
    tables: *mut dpiSubscrMessageTable,
    numTables: uint32_t,
}

// structure used for transferring version information
#[repr(C)]
#[derive(Default)]
pub struct dpiVersionInfo {
    pub versionNum: c_int,
    pub releaseNum: c_int,
    pub updateNum: c_int,
    pub portReleaseNum: c_int,
    pub portUpdateNum: c_int,
    pub fullVersionNum: uint32_t,
}

// structure used for transferring row information in messages in
// subscription callbacks
#[repr(C)]
pub struct dpiSubscrMessageRow {
    operation: dpiOpCode,
    rowid: *const c_char,
    rowidLength: uint32_t,
}

// structure used for transferring table information in messages in
// subscription callbacks
#[repr(C)]
pub struct dpiSubscrMessageTable {
    operation: dpiOpCode,
    name: *const c_char,
    nameLength: uint32_t,
    rows: *mut dpiSubscrMessageRow,
    numRows: uint32_t,
}

#[link(name = "odpic")]
extern "C" {

    //-----------------------------------------------------------------------------
    // Context Methods (dpiContext)
    //-----------------------------------------------------------------------------

    // create a context handle and validate the version information
    pub fn dpiContext_create(majorVersion: c_uint, minorVersion:c_uint,
                             context: *mut *mut dpiContext, errorInfo: *mut dpiErrorInfo) -> c_int;

    // destroy context handle
    pub fn dpiContext_destroy(context: *mut dpiContext) -> c_int;

    // return the OCI client version in use
    pub fn dpiContext_getClientVersion(context: *const dpiContext,
                                       versionInfo: *mut dpiVersionInfo) -> c_int;

    // get error information
    pub fn dpiContext_getError(context: *const dpiContext, errorInfo: *mut dpiErrorInfo);

    // initialize context parameters to default values
    pub fn dpiContext_initCommonCreateParams(context: *const dpiContext,
                                             params: *mut dpiCommonCreateParams) -> c_int;

    // initialize connection create parameters to default values
    pub fn dpiContext_initConnCreateParams(context: *const dpiContext,
                                           params: *mut dpiConnCreateParams) -> c_int;

    // initialize pool create parameters to default values
    pub fn dpiContext_initPoolCreateParams(context: *const dpiContext,
                                           params: *mut dpiPoolCreateParams) -> c_int;

    // initialize subscription create parameters to default values
    pub fn dpiContext_initSubscrCreateParams(context: *const dpiContext,
                                             params: *mut dpiSubscrCreateParams) -> c_int;

    //-----------------------------------------------------------------------------
    // Connection Methods (dpiConn)
    //-----------------------------------------------------------------------------

    // add a reference to a connection
    pub fn dpiConn_addRef(conn: *mut dpiConn) -> c_int;

    // begin a distributed transaction
    pub fn dpiConn_beginDistribTrans(conn: *mut dpiConn, formatId: c_long,
                                     transactionId: *const c_char, transactionIdLength: uint32_t,
                                     branchId: *const c_char, branchIdLength: uint32_t) -> c_int;

    // break execution of the statement running on the connection
    pub fn dpiConn_breakExecution(conn: *mut dpiConn) -> c_int;

    // change the password for the specified user
    pub fn dpiConn_changePassword(conn: *mut dpiConn, userName: *const c_char,
                                  userNameLength: uint32_t, oldPassword: *const c_char,
                                  oldPasswordLength: uint32_t, newPassword: *const c_char,
                                  newPasswordLength: uint32_t) -> c_int;

    // close the connection now, not when the reference count reaches zero
    pub fn dpiConn_close(conn: *mut dpiConn, mode: dpiConnCloseMode, tag: *const c_char,
                         tagLength: uint32_t) -> c_int;

    // commits the current active transaction
    pub fn dpiConn_commit(conn: *mut dpiConn) -> c_int;

    // create a connection and return a reference to it
    pub fn dpiConn_create(context: *const dpiContext,
                          userName: *const c_char, userNameLength: uint32_t,
                          password: *const c_char, passwordLength: uint32_t,
                          connectString: *const c_char, connectStringLength: uint32_t,
                          commonParams: *const dpiCommonCreateParams,
                          createParams: *mut dpiConnCreateParams, conn: *mut *mut dpiConn) -> c_int;

    // dequeue a message from a queue
    pub fn dpiConn_deqObject(conn: *mut dpiConn, queueName: *const c_char,
                             queueNameLength: uint32_t, options: *mut dpiDeqOptions, props: *mut dpiMsgProps,
                             payload: *mut dpiObject, msgId: *mut *const c_char, msgIdLength: *mut uint32_t) -> c_int;

    // enqueue a message to a queue
    pub fn dpiConn_enqObject(conn: *mut dpiConn, queueName: *const c_char,
                             queueNameLength: uint32_t, options: *mut dpiEnqOptions, props: *mut dpiMsgProps,
                             payload: *mut dpiObject, msgId: *mut *const c_char, msgIdLength: *mut uint32_t) -> c_int;

    // get current schema associated with the connection
    pub fn dpiConn_getCurrentSchema(conn: *mut dpiConn, value: *mut *const c_char,
                                    valueLength: *mut uint32_t) -> c_int;

    // get edition associated with the connection
    pub fn dpiConn_getEdition(conn: *mut dpiConn, value: *mut *const c_char,
                              valueLength: *mut uint32_t) -> c_int;

    // return the encoding information used by the connection
    pub fn dpiConn_getEncodingInfo(conn: *mut dpiConn, info: *mut dpiEncodingInfo) -> c_int;

    // get external name associated with the connection
    pub fn dpiConn_getExternalName(conn: *mut dpiConn, value: *mut *const c_char,
                                   valueLength: *mut uint32_t) -> c_int;

    // get the OCI service context handle associated with the connection
    pub fn dpiConn_getHandle(conn: *mut dpiConn, handle: *mut *mut c_void) -> c_int;

    // get internal name associated with the connection
    pub fn dpiConn_getInternalName(conn: *mut dpiConn, value: *mut *const c_char,
                                   valueLength: *mut uint32_t) -> c_int;

    // get logical transaction id associated with the connection
    pub fn dpiConn_getLTXID(conn: *mut dpiConn, value: *mut *const c_char, valueLength: *mut uint32_t) -> c_int;

    // create a new object type and return it for subsequent object creation
    pub fn dpiConn_getObjectType(conn: *mut dpiConn, name: *const c_char, nameLength: uint32_t,
                                 objType: *mut *mut dpiObjectType) -> c_int;

    // return information about the server version in use
    pub fn dpiConn_getServerVersion(conn: *mut dpiConn, releaseString: *mut *const c_char,
                                    releaseStringLength: *mut uint32_t, versionInfo: *mut dpiVersionInfo) -> c_int;

    // return the statement cache size
    pub fn dpiConn_getStmtCacheSize(conn: *mut dpiConn, cacheSize: *mut uint32_t) -> c_int;

    // create a new dequeue options object and return it
    pub fn dpiConn_newDeqOptions(conn: *mut dpiConn, options: *mut *mut dpiDeqOptions) -> c_int;

    // create a new enqueue options object and return it
    pub fn dpiConn_newEnqOptions(conn: *mut dpiConn, options: *mut *mut dpiEnqOptions) -> c_int;

    // create a new message properties object and return it
    pub fn dpiConn_newMsgProps(conn: *mut dpiConn, props: *mut *mut dpiMsgProps) -> c_int;

    // create a new subscription for events
    pub fn dpiConn_newSubscription(conn: *mut dpiConn, params: *mut dpiSubscrCreateParams,
                                   subscr: *mut *mut dpiSubscr, subscrId: *mut uint32_t) -> c_int;

    // create a new temporary LOB
    pub fn dpiConn_newTempLob(conn: *mut dpiConn, lobType: dpiOracleTypeNum, lob: *mut *mut dpiLob) -> c_int;

    // create a new variable and return it for subsequent binding/defining
    pub fn dpiConn_newVar(conn: *mut dpiConn, oracleTypeNum: dpiOracleTypeNum,
                          nativeTypeNum: dpiNativeTypeNum, maxArraySize: uint32_t, size: uint32_t,
                          sizeIsBytes: c_int, isArray: c_int, objType: *mut dpiObjectType, var: *mut *mut dpiVar,
                          data: *mut *mut dpiData) -> c_int;

    // ping the connection to see if it is still alive
    pub fn dpiConn_ping(conn: *mut dpiConn) -> c_int;

    // prepare a distributed transaction for commit
    pub fn dpiConn_prepareDistribTrans(conn: *mut dpiConn, commitNeeded: *mut c_int) -> c_int;

    // prepare a statement and return it for subsequent execution/fetching
    pub fn dpiConn_prepareStmt(conn: *mut dpiConn, scrollable: c_int, sql: *const c_char,
                               sqlLength: uint32_t, tag: *const c_char, tagLength: uint32_t,
                               stmt: *mut *mut dpiStmt) -> c_int;

    // release a reference to the connection
    pub fn dpiConn_release(conn: *mut dpiConn) -> c_int;

    // rolls back the current active transaction
    pub fn dpiConn_rollback(conn: *mut dpiConn) -> c_int;

    // set action associated with the connection
    pub fn dpiConn_setAction(conn: *mut dpiConn, value: *const c_char, valueLength: uint32_t) -> c_int;

    // set client identifier associated with the connection
    pub fn dpiConn_setClientIdentifier(conn: *mut dpiConn, value: *const c_char,
                                       valueLength: uint32_t) -> c_int;

    // set client info associated with the connection
    pub fn dpiConn_setClientInfo(conn: *mut dpiConn, value: *const c_char,
                                 valueLength: uint32_t) -> c_int;

    // set current schema associated with the connection
    pub fn dpiConn_setCurrentSchema(conn: *mut dpiConn, value: *const c_char,
                                    valueLength: uint32_t) -> c_int;

    // set database operation associated with the connection
    pub fn dpiConn_setDbOp(conn: *mut dpiConn, value: *const c_char, valueLength: uint32_t) -> c_int;

    // set external name associated with the connection
    pub fn dpiConn_setExternalName(conn: *mut dpiConn, value: *const c_char,
                                   valueLength: uint32_t) -> c_int;

    // set internal name associated with the connection
    pub fn dpiConn_setInternalName(conn: *mut dpiConn, value: *const c_char,
                                   valueLength: uint32_t) -> c_int;

    // set module associated with the connection
    pub fn dpiConn_setModule(conn: *mut dpiConn, value: *const c_char, valueLength: uint32_t) -> c_int;

    // set the statement cache size
    pub fn dpiConn_setStmtCacheSize(conn: *mut dpiConn, cacheSize: uint32_t) -> c_int;

    // shutdown the database
    pub fn dpiConn_shutdownDatabase(conn: *mut dpiConn, mode: dpiShutdownMode) -> c_int;

    // startup the database
    pub fn dpiConn_startupDatabase(conn: *mut dpiConn, mode: dpiStartupMode) -> c_int;


    //-----------------------------------------------------------------------------
    // Data Methods (dpiData)
    //-----------------------------------------------------------------------------

    // return the boolean portion of the data
    pub fn dpiData_getBool(data: *mut dpiData) -> c_int;

    // return the bytes portion of the data
    pub fn dpiData_getBytes(data: *mut dpiData) -> *mut dpiBytes;

    // return the double portion of the data
    pub fn dpiData_getDouble(data: *mut dpiData) -> c_double;

    // return the float portion of the data
    pub fn dpiData_getFloat(data: *mut dpiData) -> c_float;

    // return the integer portion of the data
    pub fn dpiData_getInt64(data: *mut dpiData) -> int64_t;

    // return the interval (days/seconds) portion of the data
    pub fn dpiData_getIntervalDS(data: *mut dpiData) -> &dpiIntervalDS;

    // return the interval (years/months) portion of the data
    pub fn dpiData_getIntervalYM(data: *mut dpiData) -> &dpiIntervalYM;

    // return the LOB portion of the data
    pub fn dpiData_getLOB(data: *mut dpiData) -> &dpiLob;

    // return the object portion of the data
    pub fn dpiData_getObject(data: *mut dpiData) -> &dpiObject;

    // return the statement portion of the data
    pub fn dpiData_getStmt(data: *mut dpiData) -> &dpiStmt;

    // return the timestamp portion of the data
    pub fn dpiData_getTimestamp(data: *const dpiData) -> &dpiTimestamp;

    // return the unsigned integer portion of the data
    pub fn dpiData_getUint64(data: *mut dpiData) -> uint64_t;

    // set the boolean portion of the data
    pub fn dpiData_setBool(data: *mut dpiData, value: c_int);

    // set the bytes portion of the data
    pub fn dpiData_setBytes(data: *mut dpiData, ptr: *mut c_char, length: uint32_t);

    // set the double portion of the data
    pub fn dpiData_setDouble(data: *mut dpiData, value: c_double);

    // set the float portion of the data
    pub fn dpiData_setFloat(data: *mut dpiData, value: c_float);

    // set the integer portion of the data
    pub fn dpiData_setInt64(data: *mut dpiData, value: int64_t);

    // set the interval (days/seconds) portion of the data
    pub fn dpiData_setIntervalDS(data: *mut dpiData, days: int32_t, hours: int32_t,
        minutes: int32_t, seconds: int32_t, fsceconds: int32_t);

    // set the interval (years/months) portion of the data
    pub fn dpiData_setIntervalYM(data: *mut dpiData, years: int32_t , months: int32_t);

    // set the LOB portion of the data
    pub fn dpiData_setLOB(data: *mut dpiData, lob: *mut dpiLob);

    // set the object portion of the data
    pub fn dpiData_setObject(data: *mut dpiData, obj: *mut dpiObject);

    // set the statement portion of the data
    pub fn dpiData_setStmt(data: *mut dpiData, stmt: *mut dpiStmt);

    // set the timestamp portion of the data
    pub fn dpiData_setTimestamp(data: *mut dpiData, year: int16_t, month: uint8_t ,
                                day: uint8_t, hour: uint8_t, minute: uint8_t, second: uint8_t,
                                fsecond: uint32_t, tzHourOffset: int8_t, tzMinuteOffset: int8_t);

    // set the unsigned integer portion of the data
    pub fn dpiData_setUint64(data: *mut dpiData, value: uint64_t);

    //-----------------------------------------------------------------------------
    // Dequeue Option Methods (dpiDeqOptions)
    //-----------------------------------------------------------------------------

    // add a reference to dequeue options
    pub fn dpiDeqOptions_addRef(options: *mut dpiDeqOptions) -> c_int;

    // return condition associated with dequeue options
    pub fn dpiDeqOptions_getCondition(options: *mut dpiDeqOptions, value: *mut *const c_char,
                                      valueLength: *mut uint32_t) -> c_int;

    // return consumer name associated with dequeue options
    pub fn dpiDeqOptions_getConsumerName(options: *mut dpiDeqOptions, value: *mut *const c_char,
                                         valueLength: *mut uint32_t) -> c_int;

    // return correlation associated with dequeue options
    pub fn dpiDeqOptions_getCorrelation(options: *mut dpiDeqOptions, value: *mut *const c_char,
                                        valueLength: *mut uint32_t) -> c_int;

    // return mode associated with dequeue options
    pub fn dpiDeqOptions_getMode(options: *mut dpiDeqOptions, value: *mut dpiDeqMode) -> c_int;

    // return message id associated with dequeue options
    pub fn dpiDeqOptions_getMsgId(options: *mut dpiDeqOptions, value: *mut *const c_char,
                                  valueLength: *mut uint32_t) -> c_int;

    // return navigation associated with dequeue options
    pub fn dpiDeqOptions_getNavigation(options: *mut dpiDeqOptions,
                                       value: *mut dpiDeqNavigation) -> c_int;

    // return transformation associated with dequeue options
    pub fn dpiDeqOptions_getTransformation(options: *mut dpiDeqOptions, value: *mut *const c_char,
                                           valueLength: *mut uint32_t) -> c_int;

    // return visibility associated with dequeue options
    pub fn dpiDeqOptions_getVisibility(options: *mut dpiDeqOptions, value: *mut dpiVisibility) -> c_int;

    // return wait time associated with dequeue options
    pub fn dpiDeqOptions_getWait(options: *mut dpiDeqOptions, value: *mut uint32_t) -> c_int;

    // release a reference from dequeue options
    pub fn dpiDeqOptions_release(options: *mut dpiDeqOptions) -> c_int;

    // set condition associated with dequeue options
    pub fn dpiDeqOptions_setCondition(options: *mut dpiDeqOptions, value: *const c_char,
                                      valueLength: uint32_t) -> c_int;

    // set consumer name associated with dequeue options
    pub fn dpiDeqOptions_setConsumerName(options: *mut dpiDeqOptions, value: *const c_char,
                                         valueLength: uint32_t) -> c_int;

    // set correlation associated with dequeue options
    pub fn dpiDeqOptions_setCorrelation(options: *mut dpiDeqOptions, value: *const c_char,
                                        valueLength: uint32_t) -> c_int;

    // set delivery mode associated with dequeue options
    pub fn dpiDeqOptions_setDeliveryMode(options: *mut dpiDeqOptions,
                                         value: dpiMessageDeliveryMode) -> c_int;

    // set mode associated with dequeue options
    pub fn dpiDeqOptions_setMode(options: *mut dpiDeqOptions, value: dpiDeqMode) -> c_int;

    // set message id associated with dequeue options
    pub fn dpiDeqOptions_setMsgId(options: *mut dpiDeqOptions, value: *const c_char,
                                  valueLength: uint32_t) -> c_int;

    // set navigation associated with dequeue options
    pub fn dpiDeqOptions_setNavigation(options: *mut dpiDeqOptions,
                                       value: dpiDeqNavigation) -> c_int;

    // set transformation associated with dequeue options
    pub fn dpiDeqOptions_setTransformation(options: *mut dpiDeqOptions, value: *const c_char,
                                           valueLength: uint32_t) -> c_int;

    // set visibility associated with dequeue options
    pub fn dpiDeqOptions_setVisibility(options: *mut dpiDeqOptions, value: dpiVisibility) -> c_int;

    // set wait time associated with dequeue options
    pub fn dpiDeqOptions_setWait(options: *mut dpiDeqOptions, value: uint32_t) -> c_int;


    //-----------------------------------------------------------------------------
    // Enqueue Option Methods (dpiEnqOptions)
    //-----------------------------------------------------------------------------

    // add a reference to enqueue options
    pub fn dpiEnqOptions_addRef(options: *mut dpiEnqOptions) -> c_int;

    // return transformation associated with enqueue options
    pub fn dpiEnqOptions_getTransformation(options: *mut dpiEnqOptions, value: *mut *const c_char,
                                           valueLength: *mut uint32_t) -> c_int;

    // return visibility associated with enqueue options
    pub fn dpiEnqOptions_getVisibility(options: *mut dpiEnqOptions, value: *mut dpiVisibility) -> c_int;

    // release a reference from enqueue options
    pub fn dpiEnqOptions_release(options: *mut dpiEnqOptions) -> c_int;

    // set delivery mode associated with enqueue options
    pub fn dpiEnqOptions_setDeliveryMode(options: *mut dpiEnqOptions,
                                         value: dpiMessageDeliveryMode) -> c_int;

    // set transformation associated with enqueue options
    pub fn dpiEnqOptions_setTransformation(options: *mut dpiEnqOptions, value: *const c_char,
                                           valueLength: uint32_t) -> c_int;

    // set visibility associated with enqueue options
    pub fn dpiEnqOptions_setVisibility(options: *mut dpiEnqOptions, value: dpiVisibility) -> c_int;


    //-----------------------------------------------------------------------------
    // LOB Methods (dpiLob)
    //-----------------------------------------------------------------------------

    // add a reference to the LOB
    pub fn dpiLob_addRef(lob: *mut dpiLob) -> c_int;

    // close the LOB
    pub fn dpiLob_close(lob: *mut dpiLob) -> c_int;

    // close the LOB's resources
    pub fn dpiLob_closeResource(lob: *mut dpiLob) -> c_int;

    // create a copy of the LOB
    pub fn dpiLob_copy(lob: *mut dpiLob, copiedLob: *mut *mut dpiLob) -> c_int;

    // flush buffers for the LOB
    pub fn dpiLob_flushBuffer(lob: *mut dpiLob) -> c_int;

    pub fn dpiLob_getBufferSize(lob: *mut dpiLob, sizeInChars: uint64_t,
                                sizeInBytes: *mut uint64_t) -> c_int;

    // return the chunk size for the LOB
    pub fn dpiLob_getChunkSize(lob: *mut dpiLob, size: *mut uint32_t) -> c_int;

    // return the directory alias name and file name of a BFILE LOB
    pub fn dpiLob_getDirectoryAndFileName(lob: *mut dpiLob, directoryAlias: *mut *const c_char,
                                          directoryAliasLength: *mut uint32_t, fileName: *mut *const c_char,
                                          fileNameLength: *mut uint32_t) -> c_int;

    // return if the file associated with a BFILE LOB exists
    pub fn dpiLob_getFileExists(lob: *mut dpiLob, exists: *mut c_int) -> c_int;

    // return if the LOB's resources are currently open
    pub fn dpiLob_getIsResourceOpen(lob: *mut dpiLob, isOpen: *mut c_int) -> c_int;

    // return the current size of the LOB
    pub fn dpiLob_getSize(lob: *mut dpiLob, size: *mut uint64_t) -> c_int;

    // open the LOB's resources (used to improve performance of multiple
    // read/writes operations)
    pub fn dpiLob_openResource(lob: *mut dpiLob) -> c_int;

    // read bytes from the LOB at the specified offset
    pub fn dpiLob_readBytes(lob: *mut dpiLob, offset: uint64_t, amount: uint64_t,
                            value: *mut c_char, valueLength: *mut uint64_t) -> c_int;

    // release a reference to the LOB
    pub fn dpiLob_release(lob: *mut dpiLob) -> c_int;

    // set the directory name and file name of the BFILE LOB
    pub fn dpiLob_setDirectoryAndFileName(lob: *mut dpiLob, directoryAlias: *const c_char,
                                          directoryAliasLength: uint32_t, fileName: *const c_char,
                                          fileNameLength: uint32_t) -> c_int;

    // sets the contents of a LOB from a byte string
    pub fn dpiLob_setFromBytes(lob: *mut dpiLob, value: *const c_char, valueLength: uint64_t) -> c_int;

    // trim the LOB to the specified size
    pub fn dpiLob_trim(lob: *mut dpiLob, newSize: uint64_t) -> c_int;

    // write bytes to the LOB at the specified offset
    pub fn dpiLob_writeBytes(lob: *mut dpiLob, offset: uint64_t, value: *const c_char,
                             valueLength: uint64_t) -> c_int;

    //-----------------------------------------------------------------------------
    // Message Properties Methods (dpiMsgProps)
    //-----------------------------------------------------------------------------

    // add a reference to message properties
    pub fn dpiMsgProps_addRef(props: *mut dpiMsgProps) -> c_int;

    // return the number of attempts made to deliver the message
    pub fn dpiMsgProps_getNumAttempts(props: *mut dpiMsgProps, value: *mut int32_t) -> c_int;

    // return correlation associated with the message
    pub fn dpiMsgProps_getCorrelation(props: *mut dpiMsgProps, value: *mut *const c_char,
                                      valueLength: *mut uint32_t) -> c_int;

    // return the number of seconds the message was delayed
    pub fn dpiMsgProps_getDelay(props: *mut dpiMsgProps, value: *mut int32_t) -> c_int;

    // return the mode used for delivering the message
    pub fn dpiMsgProps_getDeliveryMode(props: *mut dpiMsgProps,
                                       value: *mut dpiMessageDeliveryMode) -> c_int;

    // return the time the message was enqueued
    pub fn dpiMsgProps_getEnqTime(props: *mut dpiMsgProps, value: *mut dpiTimestamp) -> c_int;

    // return the name of the exception queue associated with the message
    pub fn dpiMsgProps_getExceptionQ(props: *mut dpiMsgProps, value: *mut *const c_char,
                                     valueLength: *mut uint32_t) -> c_int;

    // return the number of seconds until the message expires
    pub fn dpiMsgProps_getExpiration(props: *mut dpiMsgProps, value: *mut int32_t) -> c_int;

    // return the original message id for the message
    pub fn dpiMsgProps_getOriginalMsgId(props: *mut dpiMsgProps, value: *mut *const c_char,
                                        valueLength: *mut uint32_t) -> c_int;

    // return the priority of the message
    pub fn dpiMsgProps_getPriority(props: *mut dpiMsgProps, value: *mut int32_t) -> c_int;

    // return the state of the message
    pub fn dpiMsgProps_getState(props: *mut dpiMsgProps, value: *mut dpiMessageState) -> c_int;

    // release a reference from message properties
    pub fn dpiMsgProps_release(props: *mut dpiMsgProps) -> c_int;

    // set correlation associated with the message
    pub fn dpiMsgProps_setCorrelation(props: *mut dpiMsgProps, value: *const c_char,
                                      valueLength: uint32_t) -> c_int;

    // set the number of seconds to delay the message
    pub fn dpiMsgProps_setDelay(props: *mut dpiMsgProps, value: int32_t) -> c_int;

    // set the name of the exception queue associated with the message
    pub fn dpiMsgProps_setExceptionQ(props: *mut dpiMsgProps, value: *const c_char,
                                     valueLength: uint32_t) -> c_int;

    // set the number of seconds until the message expires
    pub fn dpiMsgProps_setExpiration(props: *mut dpiMsgProps, value: int32_t) -> c_int;

    // set the original message id for the message
    pub fn dpiMsgProps_setOriginalMsgId(props: *mut dpiMsgProps, value: *const c_char,
                                        valueLength: uint32_t) -> c_int;

    // set the priority of the message
    pub fn dpiMsgProps_setPriority(props: *mut dpiMsgProps, value: int32_t) -> c_int;


    //-----------------------------------------------------------------------------
    // Object Methods (dpiObject)
    //-----------------------------------------------------------------------------

    // add a reference to the object
    pub fn dpiObject_addRef(obj: *mut dpiObject) -> c_int;

    // append an element to the collection
    pub fn dpiObject_appendElement(obj: *mut dpiObject, nativeTypeNum: dpiNativeTypeNum,
                                   value: *mut dpiData) -> c_int;

    // copy the object and return the copied object
    pub fn dpiObject_copy(obj: *mut dpiObject, copiedObj: *mut *mut dpiObject) -> c_int;

    // delete an element from the collection
    pub fn dpiObject_deleteElementByIndex(obj: *mut dpiObject, index: int32_t) -> c_int;

    // get the value of the specified attribute
    pub fn dpiObject_getAttributeValue(obj: *mut dpiObject, attr: *mut dpiObjectAttr,
                                       nativeTypeNum: dpiNativeTypeNum, value: *mut dpiData) -> c_int;

    // return whether an element exists in a collection at the specified index
    pub fn dpiObject_getElementExistsByIndex(obj: *mut dpiObject, index: int32_t,
                                             exists: *mut c_int) -> c_int;

    // get the value of the element in a collection at the specified index
    pub fn dpiObject_getElementValueByIndex(obj: *mut dpiObject, index: int32_t,
                                            nativeTypeNum: dpiNativeTypeNum, value: *mut dpiData) -> c_int;

    // return the first index used in a collection
    pub fn dpiObject_getFirstIndex(obj: *mut dpiObject, index: *mut int32_t, exists: *mut c_int) -> c_int;

    // return the last index used in a collection
    pub fn dpiObject_getLastIndex(obj: *mut dpiObject, index: *mut int32_t, exists: *mut c_int) -> c_int;

    // return the next index used in a collection given an index
    pub fn dpiObject_getNextIndex(obj: *mut dpiObject, index: int32_t, nextIndex: *mut int32_t,
                                  exists: *mut c_int) -> c_int;

    // return the previous index used in a collection given an index
    pub fn dpiObject_getPrevIndex(obj: *mut dpiObject, index: int32_t, prevIndex: *mut int32_t,
                                  exists: *mut c_int) -> c_int;

    // return the number of elements in a collection
    pub fn dpiObject_getSize(obj: *mut dpiObject, size: *mut int32_t) -> c_int;

    // release a reference to the object
    pub fn dpiObject_release(obj: *mut dpiObject) -> c_int;

    // set the value of the specified attribute
    pub fn dpiObject_setAttributeValue(obj: *mut dpiObject, attr: *mut dpiObjectAttr,
                                       nativeTypeNum: dpiNativeTypeNum, value: *mut dpiData) -> c_int;

    // set the value of the element in a collection at the specified index
    pub fn dpiObject_setElementValueByIndex(obj: *mut dpiObject, index: int32_t,
                                            nativeTypeNum: dpiNativeTypeNum, value: *mut dpiData) -> c_int;

    // trim a number of elements from the end of a collection
    pub fn dpiObject_trim(obj: *mut dpiObject, numToTrim: uint32_t) -> c_int;


    //-----------------------------------------------------------------------------
    // Object Type Attribute Methods (dpiObjectAttr)
    //-----------------------------------------------------------------------------

    // add a reference to the attribute
    pub fn dpiObjectAttr_addRef(attr: *mut dpiObjectAttr) -> c_int;

    // return the name of the attribute
    pub fn dpiObjectAttr_getInfo(attr: *mut dpiObjectAttr, info: *mut dpiObjectAttrInfo) -> c_int;

    // release a reference to the attribute
    pub fn dpiObjectAttr_release(attr: *mut dpiObjectAttr) -> c_int;


    //-----------------------------------------------------------------------------
    // Object Type Methods (dpiObjectType)
    //-----------------------------------------------------------------------------

    // add a reference to the object type
    pub fn dpiObjectType_addRef(objType: *mut dpiObjectType) -> c_int;

    // create an object of the specified type and return it
    pub fn dpiObjectType_createObject(objType: *mut dpiObjectType, obj: *mut *mut dpiObject) -> c_int;

    // return the attributes available on the object type
    pub fn dpiObjectType_getAttributes(objType: *mut dpiObjectType, numAttributes: uint16_t,
                                       attributes: *mut *mut dpiObjectAttr) -> c_int;

    // return information about the object type
    pub fn dpiObjectType_getInfo(objType: *mut dpiObjectType, info: *mut dpiObjectTypeInfo) -> c_int;

    // release a reference to the object type
    pub fn dpiObjectType_release(objType: *mut dpiObjectType) -> c_int;


    //-----------------------------------------------------------------------------
    // Session Pools Methods (dpiPool)
    //-----------------------------------------------------------------------------

    // acquire a connection from the pool and return it
    pub fn dpiPool_acquireConnection(pool: *mut dpiPool, userName: *const c_char,
                                     userNameLength: uint32_t, password: *const c_char, passwordLength: uint32_t,
                                     createParams: *mut dpiConnCreateParams, conn: *mut *mut dpiConn) -> c_int;

    // add a reference to a pool
    pub fn dpiPool_addRef(pool: *mut dpiPool) -> c_int;

    // destroy the pool now, not when its reference count reaches zero
    pub fn dpiPool_close(pool: *mut dpiPool, closeMode: dpiPoolCloseMode) -> c_int;

    // create a session pool and return it
    pub fn dpiPool_create(context: *const dpiContext, userName: *const c_char,
                          userNameLength: uint32_t, password: *const c_char, passwordLength: uint32_t,
                          connectString: *const c_char, connectStringLength: uint32_t,
                          commonParams: *const dpiCommonCreateParams,
                          createParams: *mut dpiPoolCreateParams, pool: *mut *mut dpiPool) -> c_int;

    // get the pool's busy count
    pub fn dpiPool_getBusyCount(pool: *mut dpiPool, value: *mut uint32_t) -> c_int;

    // return the encoding information used by the session pool
    pub fn dpiPool_getEncodingInfo(pool: *mut dpiPool, info: *mut dpiEncodingInfo) -> c_int;

    // get the pool's "get" mode
    pub fn dpiPool_getGetMode(pool: *mut dpiPool, value: *mut dpiPoolGetMode) -> c_int;

    // get the pool's maximum lifetime session
    pub fn dpiPool_getMaxLifetimeSession(pool: *mut dpiPool, value: *mut uint32_t) -> c_int;

    // get the pool's open count
    pub fn dpiPool_getOpenCount(pool: *mut dpiPool, value: *mut uint32_t) -> c_int;

    // return the statement cache size
    pub fn dpiPool_getStmtCacheSize(pool: *mut dpiPool, cacheSize: *mut uint32_t) -> c_int;

    // get the pool's timeout value
    pub fn dpiPool_getTimeout(pool: *mut dpiPool, value: *mut uint32_t) -> c_int;

    // release a reference to the pool
    pub fn dpiPool_release(pool: *mut dpiPool) -> c_int;

    // set the pool's "get" mode
    pub fn dpiPool_setGetMode(pool: *mut dpiPool, value: dpiPoolGetMode) -> c_int;

    // set the pool's maximum lifetime session
    pub fn dpiPool_setMaxLifetimeSession(pool: *mut dpiPool, value: uint32_t) -> c_int;

    // set the statement cache size
    pub fn dpiPool_setStmtCacheSize(pool: *mut dpiPool, cacheSize: uint32_t) -> c_int;

    // set the pool's timeout value
    pub fn dpiPool_setTimeout(pool: *mut dpiPool, value: uint32_t) -> c_int;

    //-----------------------------------------------------------------------------
    // Statement Methods (dpiStmt)
    //-----------------------------------------------------------------------------

    // add a reference to a statement
    pub fn dpiStmt_addRef(stmt: *mut dpiStmt) -> c_int;

    // bind a variable to the statement using the given name
    pub fn dpiStmt_bindByName(stmt: *mut dpiStmt, name: *const c_char, nameLength: uint32_t,
                              var: *mut dpiVar) -> c_int;

    // bind a variable to the statement at the given position
    // positions are determined by the order in which names are introduced
    pub fn dpiStmt_bindByPos(stmt: *mut dpiStmt, pos: uint32_t, var: *mut dpiVar) -> c_int;

    // bind a value to the statement using the given name
    // this creates the variable by looking at the type and then binds it
    pub fn dpiStmt_bindValueByName(stmt: *mut dpiStmt, name: *const c_char,
                                   nameLength: uint32_t, nativeTypeNum: dpiNativeTypeNum, data: *mut dpiData) -> c_int;

    // bind a value to the statement at the given position
    // this creates the variable by looking at the type and then binds it
    pub fn dpiStmt_bindValueByPos(stmt: *mut dpiStmt, pos: uint32_t,
                                  nativeTypeNum: dpiNativeTypeNum, data: *mut dpiData) -> c_int;

    // close the statement now, not when its reference count reaches zero
    pub fn dpiStmt_close(stmt: *mut dpiStmt, tag: *const c_char, tagLength: uint32_t) -> c_int;

    // define a variable to accept the data for the specified column (1 based)
    pub fn dpiStmt_define(stmt: *mut dpiStmt, pos: uint32_t, var: *mut dpiVar) -> c_int;

    // execute the statement and return the number of query columns
    // zero implies the statement is not a query
    pub fn dpiStmt_execute(stmt: *mut dpiStmt, mode: dpiExecMode,
                           numQueryColumns: *mut uint32_t) -> c_int;

    // execute the statement multiple times (queries not supported)
    pub fn dpiStmt_executeMany(stmt: *mut dpiStmt, mode: dpiExecMode, numIters: uint32_t) -> c_int;

    // fetch a single row and return the index into the defined variables
    // this will internally perform any execute and array fetch as needed
    pub fn dpiStmt_fetch(stmt: *mut dpiStmt, found: *mut c_int, bufferRowIndex: *mut uint32_t) -> c_int;

    // return the number of rows that are available in the defined variables
    // up to the maximum specified; this will internally perform execute/array
    // fetch only if no rows are available in the defined variables and there are
    // more rows available to fetch
    pub fn dpiStmt_fetchRows(stmt: *mut dpiStmt, maxRows: uint32_t,
                             bufferRowIndex: *mut uint32_t, numRowsFetched: *mut uint32_t, moreRows: *mut c_int) -> c_int;

    // get the number of batch errors that took place in the previous execution
    pub fn dpiStmt_getBatchErrorCount(stmt: *mut dpiStmt, count: *mut uint32_t) -> c_int;

    // get the batch errors that took place in the previous execution
    pub fn dpiStmt_getBatchErrors(stmt: *mut dpiStmt, numErrors: uint32_t,
                                  errors: *mut dpiErrorInfo) -> c_int;

    // get the number of bind variables that are in the prepared statement
    pub fn dpiStmt_getBindCount(stmt: *mut dpiStmt, count: *mut uint32_t) -> c_int;

    // get the names of the bind variables that are in the prepared statement
    pub fn dpiStmt_getBindNames(stmt: *mut dpiStmt, numBindNames: uint32_t,
                                bindNames: *mut *const c_char, bindNameLengths: *mut uint32_t) -> c_int;

    // get the number of rows to (internally) fetch at one time
    pub fn dpiStmt_getFetchArraySize(stmt: *mut dpiStmt, arraySize: *mut uint32_t) -> c_int;

    // get next implicit result from previous execution; NULL if no more exist
    pub fn dpiStmt_getImplicitResult(stmt: *mut dpiStmt, implicitResult: *mut *mut dpiStmt) -> c_int;

    // return information about the statement
    pub fn dpiStmt_getInfo(stmt: *mut dpiStmt, info: *mut dpiStmtInfo) -> c_int;

    // get the number of query columns (zero implies the statement is not a query)
    pub fn dpiStmt_getNumQueryColumns(stmt: *mut dpiStmt, numQueryColumns: *mut uint32_t) -> c_int;

    // return metadata about the column at the specified position (1 based)
    pub fn dpiStmt_getQueryInfo(stmt: *mut dpiStmt, pos: uint32_t, info: *mut dpiQueryInfo) -> c_int;

    // get the value for the specified column of the current row fetched
    pub fn dpiStmt_getQueryValue(stmt: *mut dpiStmt, pos: uint32_t,
                                 nativeTypeNum: *mut dpiNativeTypeNum, data: *mut *mut dpiData) -> c_int;

    // get the row count for the statement
    // for queries, this is the number of rows that have been fetched so far
    // for non-queries, this is the number of rows affected by the last execution
    pub fn dpiStmt_getRowCount(stmt: *mut dpiStmt, count: *mut uint64_t) -> c_int;

    // get the number of rows affected for each DML operation just executed
    // using the mode DPI_MODE_EXEC_ARRAY_DML_ROWCOUNTS
    pub fn dpiStmt_getRowCounts(stmt: *mut dpiStmt, numRowCounts: *mut uint32_t,
                                rowCounts: *mut *mut uint64_t) -> c_int;

    // get subscription query id for continuous query notification
    pub fn dpiStmt_getSubscrQueryId(stmt: *mut dpiStmt, queryId: *mut uint64_t) -> c_int;

    // release a reference to the statement
    pub fn dpiStmt_release(stmt: *mut dpiStmt) -> c_int;

    // scroll the statement to the desired row
    // this is only valid for scrollable statements
    pub fn dpiStmt_scroll(stmt: *mut dpiStmt, mode: dpiFetchMode, offset: int32_t,
                          rowCountOffset: int32_t) -> c_int;

    // set the number of rows to (internally) fetch at one time
    pub fn dpiStmt_setFetchArraySize(stmt: *mut dpiStmt, arraySize: uint32_t) -> c_int;

    //-----------------------------------------------------------------------------
    // Rowid Methods (dpiRowid)
    //-----------------------------------------------------------------------------

    // add a reference to the rowid
    pub fn dpiRowid_addRef(rowid: *mut dpiRowid) -> c_int;

    // get string representation from rowid
    pub fn dpiRowid_getStringValue(rowid: *mut dpiRowid, value: *mut *const c_char,
                                   valueLength: *mut uint32_t) -> c_int;

    // release a reference to the rowid
    pub fn dpiRowid_release(subscr: *mut dpiRowid) -> c_int;

    //-----------------------------------------------------------------------------
    // Subscription Methods (dpiSubscr)
    //-----------------------------------------------------------------------------

    // add a reference to the subscription
    pub fn dpiSubscr_addRef(subscr: *mut dpiSubscr) -> c_int;

    // close the subscription
    pub fn dpiSubscr_close(subscr: *mut dpiSubscr) -> c_int;

    // prepare statement for registration with subscription
    pub fn dpiSubscr_prepareStmt(subscr: *mut dpiSubscr, sql: *const c_char,
                                 sqlLength: uint32_t, stmt: *mut *mut dpiStmt) -> c_int;

    // release a reference to the subscription
    pub fn dpiSubscr_release(subscr: *mut dpiSubscr) -> c_int;

    //-----------------------------------------------------------------------------
    // Variable Methods (dpiVar)
    //-----------------------------------------------------------------------------

    // add a reference to the variable
    pub fn dpiVar_addRef(var: *mut dpiVar) -> c_int;

    // copy the data from one variable to another variable
    pub fn dpiVar_copyData(var: *mut dpiVar, pos: uint32_t, sourceVar: *mut dpiVar,
                           sourcePos: uint32_t) -> c_int;

    // return pointer to array of dpiData structures for transferring data
    // this is needed for DML returning where the number of elements is modified
    pub fn dpiVar_getData(var: *mut dpiVar, numElements: *mut uint32_t, data: *mut *mut dpiData) -> c_int;

    // return the number of elements in a PL/SQL index-by table
    pub fn dpiVar_getNumElementsInArray(var: *mut dpiVar, numElements: *mut uint32_t) -> c_int;

    // return the size in bytes of the buffer used for fetching/binding
    pub fn dpiVar_getSizeInBytes(var: *mut dpiVar, sizeInBytes: *mut uint32_t) -> c_int;

    // release a reference to the variable
    pub fn dpiVar_release(var: *mut dpiVar) -> c_int;

    // set the value of the variable from a byte string
    pub fn dpiVar_setFromBytes(var: *mut dpiVar, pos: uint32_t, value: *const c_char,
                               valueLength: uint32_t) -> c_int;

    // set the value of the variable from a LOB
    pub fn dpiVar_setFromLob(var: *mut dpiVar, pos: uint32_t, lob: *mut dpiLob) -> c_int;

    // set the value of the variable from an object
    pub fn dpiVar_setFromObject(var: *mut dpiVar, pos: uint32_t, obj: *mut dpiObject) -> c_int;

    // set the value of the variable from a rowid
    pub fn dpiVar_setFromRowid(var: *mut dpiVar, pos: uint32_t, rowid: *mut dpiRowid) -> c_int;

    // set the value of the variable from a statement
    pub fn dpiVar_setFromStmt(var: *mut dpiVar, pos: uint32_t, stmt: *mut dpiStmt) -> c_int;

    // set the number of elements in a PL/SQL index-by table
    pub fn dpiVar_setNumElementsInArray(var: *mut dpiVar, numElements: uint32_t) -> c_int;
}
