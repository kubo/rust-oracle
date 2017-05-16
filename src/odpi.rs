use std::cmp::PartialEq;
use std::ffi::CStr;
use std::fmt;
use std::ops::BitOr;
use std::ptr;
use std::slice;
use libc::c_char;
use libc::uint32_t;

use super::ffi::*;
use super::DbError;
use super::Error;
use super::Result;

//
// AuthMode
//

#[derive(Debug, Clone)]
pub enum AuthMode {
    Default,
    /// Connect as SYSDBA
    SysDba,
    /// Connect as SYSOPER
    SysOper,
    Prelim,
    /// Connect as SYSASM
    SysAsm,
    /// bitor-ed value of other values
    BitFlags(i32),
}

impl AuthMode {
    pub fn as_i32(&self) -> i32 {
        match *self {
            AuthMode::Default => DPI_MODE_AUTH_DEFAULT,
            AuthMode::SysDba => DPI_MODE_AUTH_SYSDBA,
            AuthMode::SysOper => DPI_MODE_AUTH_SYSOPER,
            AuthMode::Prelim => DPI_MODE_AUTH_PRELIM,
            AuthMode::SysAsm => DPI_MODE_AUTH_SYSASM,
            AuthMode::BitFlags(n) => n,
        }
    }
}

impl BitOr for AuthMode {
    type Output = AuthMode;
    fn bitor(self, rhs: AuthMode) -> AuthMode {
        AuthMode::BitFlags(self.as_i32() | rhs.as_i32())
    }
}

impl PartialEq for AuthMode {
    fn eq(&self, other: &AuthMode) -> bool {
        self.as_i32() == other.as_i32()
    }

    fn ne(&self, other: &AuthMode) -> bool {
        self.as_i32() != other.as_i32()
    }
}

//
// ShutdownMode
//

#[derive(Debug, Clone, PartialEq)]
pub enum ShutdownMode {
    Default = DPI_MODE_SHUTDOWN_DEFAULT as isize,
    Transactional = DPI_MODE_SHUTDOWN_TRANSACTIONAL as isize,
    TransactionalLocal = DPI_MODE_SHUTDOWN_TRANSACTIONAL_LOCAL as isize,
    Immediate = DPI_MODE_SHUTDOWN_IMMEDIATE as isize,
    Abort = DPI_MODE_SHUTDOWN_ABORT as isize,
    Final = DPI_MODE_SHUTDOWN_FINAL as isize,
}

//
// StartupMode
//

#[derive(Debug, Clone, PartialEq)]
pub enum StartupMode {
    Default = DPI_MODE_STARTUP_DEFAULT as isize,
    Force = DPI_MODE_STARTUP_FORCE as isize,
    Restrict = DPI_MODE_STARTUP_RESTRICT as isize,
}

//
// StatementType
//

#[derive(Debug, Clone, PartialEq)]
pub enum StatementType {
    Unknown = DPI_STMT_TYPE_UNKNOWN as isize,
    Select = DPI_STMT_TYPE_SELECT as isize,
    Update = DPI_STMT_TYPE_UPDATE as isize,
    Delete = DPI_STMT_TYPE_DELETE as isize,
    Insert = DPI_STMT_TYPE_INSERT as isize,
    Create = DPI_STMT_TYPE_CREATE as isize,
    Drop = DPI_STMT_TYPE_DROP as isize,
    Alter = DPI_STMT_TYPE_ALTER as isize,
    Begin = DPI_STMT_TYPE_BEGIN as isize,
    Declare = DPI_STMT_TYPE_DECLARE as isize,
    Call = DPI_STMT_TYPE_CALL as isize,
}

impl StatementType {
    fn from_i32(n: i32) -> StatementType {
        match n {
            DPI_STMT_TYPE_SELECT => StatementType::Select,
            DPI_STMT_TYPE_UPDATE => StatementType::Update,
            DPI_STMT_TYPE_DELETE => StatementType::Delete,
            DPI_STMT_TYPE_INSERT => StatementType::Insert,
            DPI_STMT_TYPE_CREATE => StatementType::Create,
            DPI_STMT_TYPE_DROP => StatementType::Drop,
            DPI_STMT_TYPE_ALTER => StatementType::Alter,
            DPI_STMT_TYPE_BEGIN => StatementType::Begin,
            DPI_STMT_TYPE_DECLARE => StatementType::Declare,
            DPI_STMT_TYPE_CALL => StatementType::Call,
            _ => StatementType::Unknown,
        }
    }
}

//
// OracleType
//

#[derive(Debug, Clone, PartialEq)]
pub enum OracleType {
    #[doc(hidden)]
    None,
    /// VARCHAR2 data type
    Varchar2(u32), // size
    /// NVARCHAR2 data type
    Nvarchar2(u32), // size
    /// CHAR data type
    Char(u32), // size
    /// NCHAR data type
    NChar(u32), // size
    /// ROWID data type
    Rowid,
    /// RAW data type
    Raw(u32), // size
    /// BINARY_FLOAT data type
    BinaryFloat,
    /// BINARY_DOUBLE data type
    BinaryDouble,
    /// NUMBER data type
    Number(i16, i8), // prec, scale
    /// DATE data type
    Date,
    /// TIMESTAMP data type
    Timestamp(u8), // fsprec
    /// TIMESTAMP WITH TIME ZONE data type
    TimestampTZ(u8), // fsprec
    /// TIMESTAMP WITH LOCAL TIME ZONE data type
    TimestampLTZ(u8), // fsprec
    /// INTERVAL DAY TO SECOND data type
    IntervalDS(u8, u8), // lfprec, fsprec
    /// INTERVAL YEAR TO MONTH data type
    IntervalYM(u8), // lfprec
    /// CLOB data type. not supported yet
    CLob,
    /// NCLOB data type. not supported yet
    NCLob,
    /// BLOB data type. not supported yet
    BLob,
    /// BFILE data type. not supported yet
    BFile,
    /// REF CURSOR data type. not supported yet
    RefCursor,
    /// BOOLEAN data type used in PL/SQL, not supported yet
    Boolean,
    /// Object data type. not supported yet
    Object, // fix when object type is supported.
    /// LONG data type
    Long,
    /// LONG RAW data type
    LongRaw,
    /// Not an Oracle type, used only for internally to bind/define values as i64
    Int64,
    /// Not an Oracle type, used only for internally to bind/define values as u64
    UInt64,
}

impl OracleType {

    fn from_query_info(info: &dpiQueryInfo) -> Result<OracleType> {
        match info.oracleTypeNum {
            DPI_ORACLE_TYPE_VARCHAR => Ok(OracleType::Varchar2(info.dbSizeInBytes)),
            DPI_ORACLE_TYPE_NVARCHAR => Ok(OracleType::Nvarchar2(info.sizeInChars)),
            DPI_ORACLE_TYPE_CHAR => Ok(OracleType::Char(info.dbSizeInBytes)),
            DPI_ORACLE_TYPE_NCHAR => Ok(OracleType::NChar(info.sizeInChars)),
            DPI_ORACLE_TYPE_ROWID => Ok(OracleType::Rowid),
            DPI_ORACLE_TYPE_RAW => Ok(OracleType::Raw(info.dbSizeInBytes)),
            DPI_ORACLE_TYPE_NATIVE_FLOAT => Ok(OracleType::BinaryFloat),
            DPI_ORACLE_TYPE_NATIVE_DOUBLE => Ok(OracleType::BinaryDouble),
            DPI_ORACLE_TYPE_NUMBER => Ok(OracleType::Number(info.precision, info.scale)),
            DPI_ORACLE_TYPE_DATE => Ok(OracleType::Date),
            // If dpiQueryInfo provides fsprec information, use the value. Use 6 for a while
            DPI_ORACLE_TYPE_TIMESTAMP => Ok(OracleType::Timestamp(6)),
            // If dpiQueryInfo provides fsprec information, use the value. Use 6 for a while
            DPI_ORACLE_TYPE_TIMESTAMP_TZ => Ok(OracleType::TimestampTZ(6)),
            // If dpiQueryInfo provides fsprec information, use the value. Use 6 for a while
            DPI_ORACLE_TYPE_TIMESTAMP_LTZ => Ok(OracleType::TimestampLTZ(6)),
            // If dpiQueryInfo provides lfprec and fsprec information, use the values. Use 2 and 6 for a while
            DPI_ORACLE_TYPE_INTERVAL_DS => Ok(OracleType::IntervalDS(2, 6)),
            // If dpiQueryInfo provides lfprec information, use the value. Use 2 for a while
            DPI_ORACLE_TYPE_INTERVAL_YM => Ok(OracleType::IntervalYM(2)),
            DPI_ORACLE_TYPE_CLOB => Ok(OracleType::CLob),
            DPI_ORACLE_TYPE_NCLOB => Ok(OracleType::NCLob),
            DPI_ORACLE_TYPE_BLOB => Ok(OracleType::BLob),
            DPI_ORACLE_TYPE_BFILE => Ok(OracleType::BFile),
            DPI_ORACLE_TYPE_STMT => Ok(OracleType::RefCursor),
            DPI_ORACLE_TYPE_BOOLEAN => Ok(OracleType::Boolean),
            DPI_ORACLE_TYPE_OBJECT => Ok(OracleType::Object),
            DPI_ORACLE_TYPE_LONG_VARCHAR => Ok(OracleType::Long),
            DPI_ORACLE_TYPE_LONG_RAW => Ok(OracleType::LongRaw),
            _ => Err(Error::InternalError(format!("Unknown oracle type number: {}", info.oracleTypeNum))),
        }
    }

    // Returns parameters to create a new dpiVar.
    fn var_create_param(&self) -> Result<(i32, i32, u32, i32)> {
        // The followings are basically same with dpiAllOracleTypes[] in
        // dpiOracleType.c. If enum OracleType has an attribute corresponding
        // to defaultNativeTypeNum of dpiQueryInfo, this mapping is not needed.
        // However I don't want to do it to hide internal information such
        // as dpiNativeTypeNum.
        match *self {
            OracleType::Varchar2(size) =>
                Ok((DPI_ORACLE_TYPE_VARCHAR, DPI_NATIVE_TYPE_BYTES, size, 1)),
            OracleType::Nvarchar2(size) =>
                Ok((DPI_ORACLE_TYPE_NVARCHAR, DPI_NATIVE_TYPE_BYTES, size, 0)),
            OracleType::Char(size) =>
                Ok((DPI_ORACLE_TYPE_CHAR, DPI_NATIVE_TYPE_BYTES, size, 1)),
            OracleType::NChar(size) =>
                Ok((DPI_ORACLE_TYPE_NCHAR, DPI_NATIVE_TYPE_BYTES, size, 0)),
            OracleType::Rowid =>
                Ok((DPI_ORACLE_TYPE_ROWID, DPI_NATIVE_TYPE_ROWID, 0, 0)),
            OracleType::Raw(size) =>
                Ok((DPI_ORACLE_TYPE_RAW, DPI_NATIVE_TYPE_BYTES, size, 1)),
            OracleType::BinaryFloat =>
                Ok((DPI_ORACLE_TYPE_NATIVE_FLOAT, DPI_NATIVE_TYPE_FLOAT, 0, 0)),
            OracleType::BinaryDouble =>
                Ok((DPI_ORACLE_TYPE_NATIVE_DOUBLE, DPI_NATIVE_TYPE_DOUBLE, 0, 0)),
            OracleType::Number(_, _) =>
                Ok((DPI_ORACLE_TYPE_NUMBER, DPI_NATIVE_TYPE_BYTES, 0, 0)),
            OracleType::Date =>
                Ok((DPI_ORACLE_TYPE_DATE, DPI_NATIVE_TYPE_TIMESTAMP, 0, 0)),
            OracleType::Timestamp(_) =>
                Ok((DPI_ORACLE_TYPE_TIMESTAMP, DPI_NATIVE_TYPE_TIMESTAMP, 0, 0)),
            OracleType::TimestampTZ(_) =>
                Ok((DPI_ORACLE_TYPE_TIMESTAMP_TZ, DPI_NATIVE_TYPE_TIMESTAMP, 0, 0)),
            OracleType::TimestampLTZ(_) =>
                Ok((DPI_ORACLE_TYPE_TIMESTAMP_LTZ, DPI_NATIVE_TYPE_TIMESTAMP, 0, 0)),
            OracleType::IntervalDS(_, _) =>
                Ok((DPI_ORACLE_TYPE_INTERVAL_DS, DPI_NATIVE_TYPE_INTERVAL_DS, 0, 0)),
            OracleType::IntervalYM(_) =>
                Ok((DPI_ORACLE_TYPE_INTERVAL_YM, DPI_NATIVE_TYPE_INTERVAL_YM, 0, 0)),
//            OracleType::CLob =>
//                Ok((DPI_ORACLE_TYPE_CLOB, DPI_NATIVE_TYPE_LOB, 0, 0)),
//            OracleType::NCLob =>
//                Ok((DPI_ORACLE_TYPE_NCLOB, DPI_NATIVE_TYPE_LOB, 0, 0)),
//            OracleType::BLob =>
//                Ok((DPI_ORACLE_TYPE_BLOB, DPI_NATIVE_TYPE_LOB, 0, 0)),
//            OracleType::BFile =>
//                Ok((DPI_ORACLE_TYPE_BFILE, DPI_NATIVE_TYPE_LOB, 0, 0)),
//            OracleType::RefCursor =>
//                Ok((DPI_ORACLE_TYPE_STMT, DPI_NATIVE_TYPE_STMT, 0, 0)),
//            OracleType::Boolean =>
//                Ok((DPI_ORACLE_TYPE_BOOLEAN, DPI_NATIVE_TYPE_BOOLEAN, 0, 0)),
//            OracleType::Object =>
//                Ok((DPI_ORACLE_TYPE_OBJECT, DPI_NATIVE_TYPE_OBJECT, 0, 0)),
            OracleType::Long =>
                Ok((DPI_ORACLE_TYPE_LONG_VARCHAR, DPI_NATIVE_TYPE_BYTES, 0, 0)),
            OracleType::LongRaw =>
                Ok((DPI_ORACLE_TYPE_LONG_RAW, DPI_NATIVE_TYPE_BYTES, 0, 0)),
            OracleType::Int64 =>
                Ok((DPI_ORACLE_TYPE_NATIVE_INT, DPI_NATIVE_TYPE_INT64, 0, 0)),
            OracleType::UInt64 =>
                Ok((DPI_ORACLE_TYPE_NATIVE_UINT, DPI_NATIVE_TYPE_UINT64, 0, 0)),
            _ =>
                Err(Error::InternalError(format!("Unsupported Oracle type {}", self))),
        }
    }
}

impl fmt::Display for OracleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OracleType::None => write!(f, "?"),
            OracleType::Varchar2(size) => write!(f, "VARCHAR2({})", size),
            OracleType::Nvarchar2(size) => write!(f, "NVARCHAR2({})", size),
            OracleType::Char(size) => write!(f, "CHAR({})", size),
            OracleType::NChar(size) => write!(f, "NCHAR({})", size),
            OracleType::Rowid => write!(f, "ROWID"),
            OracleType::Raw(size) => write!(f, "RAW({})", size),
            OracleType::BinaryFloat => write!(f, "BINARY_FLOAT"),
            OracleType::BinaryDouble => write!(f, "BINARY_DOUBLE"),
            OracleType::Number(prec, scale) =>
                match scale {
                    -127 => {
                        match prec {
                            0 => write!(f, "NUMBER"),
                            126 => write!(f, "FLOAT"),
                            _ => write!(f, "FLOAT({})", prec),
                        }
                    },
                    0 => {
                        match prec {
                            0 => write!(f, "NUMBER"),
                            _ => write!(f, "NUMBER({})", prec),
                        }
                    },
                    _ => write!(f, "NUMBER({},{})", prec, scale),
                },
            OracleType::Date => write!(f, "DATE"),
            OracleType::Timestamp(fsprec) =>
                if fsprec == 6 {
                    write!(f, "TIMESTAMP")
                } else {
                    write!(f, "TIMESTAMP({})", fsprec)
                },
            OracleType::TimestampTZ(fsprec) =>
                if fsprec == 6 {
                    write!(f, "TIMESTAMP WITH TIME ZONE")
                } else {
                    write!(f, "TIMESTAMP({}) WITH TIME ZONE", fsprec)
                },
            OracleType::TimestampLTZ(fsprec) =>
                if fsprec == 6 {
                    write!(f, "TIMESTAMP WITH LOCAL TIME ZONE")
                } else {
                    write!(f, "TIMESTAMP({}) WITH LOCAL TIME ZONE", fsprec)
                },
            OracleType::IntervalDS(lfprec, fsprec) =>
                if lfprec == 2 && fsprec == 6 {
                    write!(f, "INTERVAL DAY TO SECOND")
                } else {
                    write!(f, "INTERVAL DAY({}) TO SECOND({})", lfprec, fsprec)
                },
            OracleType::IntervalYM(lfprec) =>
                if lfprec == 2 {
                    write!(f, "INTERVAL YEAR TO MONTH")
                } else {
                    write!(f, "INTERVAL YEAR({}) TO MONTH", lfprec)
                },
            OracleType::CLob => write!(f, "CLOB"),
            OracleType::NCLob => write!(f, "NCLOB"),
            OracleType::BLob => write!(f, "BLOB"),
            OracleType::BFile => write!(f, "BFILE"),
            OracleType::RefCursor => write!(f, "REF CURSOR"),
            OracleType::Boolean => write!(f, "BOOLEAN"),
            OracleType::Object => write!(f, "OBJECT"),
            OracleType::Long => write!(f, "LONG"),
            OracleType::LongRaw => write!(f, "LONG RAW"),
            OracleType::Int64 => write!(f, "INT64 used internally"),
            OracleType::UInt64 =>write!(f, "UINT64 used internally"),
        }
    }
}

//
// Timestamp
//

#[derive(Debug, Clone, PartialEq)]
pub struct Timestamp {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    nanosecond: u32,
    tz_hour_offset: i32,
    tz_minute_offset: i32,
}

impl Timestamp {
    fn from_dpi_timestamp(ts: &dpiTimestamp) -> Timestamp {
        Timestamp {
            year: ts.year as i32,
            month: ts.month as u32,
            day: ts.day as u32,
            hour: ts.hour as u32,
            minute: ts.minute as u32,
            second: ts.second as u32,
            nanosecond: ts.fsecond as u32,
            tz_hour_offset: ts.tzHourOffset as i32,
            tz_minute_offset: ts.tzMinuteOffset as i32,
        }
    }

    #[allow(dead_code)] // This function will be used to bind timestamp
    fn set_dpi_timestamp(&self, ts: &mut dpiTimestamp) {
        ts.year = self.year as i16;
        ts.month = self.month as u8;
        ts.day = self.day as u8;
        ts.hour = self.hour as u8;
        ts.minute = self.minute as u8;
        ts.second = self.second as u8;
        ts.fsecond = self.nanosecond;
        ts.tzHourOffset = self.tz_hour_offset as i8;
        ts.tzMinuteOffset = self.tz_minute_offset as i8;
    }

    pub fn new(year: i32, month: u32, day: u32,
               hour: u32, minute: u32, second: u32, nanosecond: u32) -> Timestamp {
        Timestamp {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            nanosecond: nanosecond,
            tz_hour_offset: 0,
            tz_minute_offset: 0,
        }
    }

    pub fn and_tz_offset(self: Timestamp, tz_hour_offset: i32, tz_minute_offset: i32) -> Timestamp {
        Timestamp {
            tz_hour_offset: tz_hour_offset,
            tz_minute_offset: tz_minute_offset,
            .. self
        }
    }

    pub fn and_tz_offset_sec(self: Timestamp, tz_offset_sec: i32) -> Timestamp {
        let tz_offset_minute = tz_offset_sec / 60;
        let (hour, minute) = if tz_offset_minute >= 0 {
            (tz_offset_minute / 60, tz_offset_minute % 60)
        } else {
            (- (- tz_offset_minute / 60), - (- tz_offset_minute % 60))
        };
        Timestamp {
            tz_hour_offset: hour,
            tz_minute_offset: minute,
            .. self
        }
    }

    pub fn year(&self) -> i32 {
        self.year
    }
    pub fn month(&self) -> u32 {
        self.month
    }
    pub fn day(&self) -> u32 {
        self.day
    }
    pub fn hour(&self) -> u32 {
        self.hour
    }
    pub fn minute(&self) -> u32 {
        self.minute
    }
    pub fn second(&self) -> u32 {
        self.second
    }
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }
    pub fn tz_hour_offset(&self) -> i32 {
        self.tz_hour_offset
    }
    pub fn tz_minute_offset(&self) -> i32 {
        self.tz_minute_offset
    }
    pub fn tz_offset_sec(&self) -> i32 {
        self.tz_hour_offset * 3600 + self.tz_minute_offset * 60
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{:02}-{:02} {:02}:{:02}:{:02}.{:09} {:+02}:{:02}",
               self.year, self.month, self.day,
               self.hour, self.minute, self.second, self.nanosecond,
               self.tz_hour_offset, self.tz_minute_offset)
    }
}

//
// IntervalDS
//

#[derive(Debug, Clone, PartialEq)]
pub struct IntervalDS {
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    nanoseconds: i32,
}

impl IntervalDS {
    fn from_dpi_interval_ds(it: &dpiIntervalDS) -> IntervalDS {
        IntervalDS {
            days: it.days,
            hours: it.hours,
            minutes: it.minutes,
            seconds: it.seconds,
            nanoseconds: it.fseconds,
        }
    }

    #[allow(dead_code)] // This function will be used to bind timestamp
    fn set_dpi_interval_ds(&self, it: &mut dpiIntervalDS) {
        it.days = self.days;
        it.hours = self.hours;
        it.minutes = self.minutes;
        it.seconds = self.seconds;
        it.fseconds = self.nanoseconds;
    }

    pub fn new(days: i32, hours: i32, minutes: i32, seconds: i32, nanoseconds: i32) -> IntervalDS {
        IntervalDS {
            days: days,
            hours: hours,
            minutes: minutes,
            seconds: seconds,
            nanoseconds: nanoseconds,
        }
    }

    pub fn days(&self) -> i32 {
        self.days
    }
    pub fn hours(&self) -> i32 {
        self.hours
    }
    pub fn minutes(&self) -> i32 {
        self.minutes
    }
    pub fn seconds(&self) -> i32 {
        self.seconds
    }
    pub fn nanoseconds(&self) -> i32 {
        self.nanoseconds
    }
}

impl fmt::Display for IntervalDS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.days < 0 || self.hours < 0 || self.minutes < 0 || self.seconds < 0 || self.nanoseconds < 0 {
            write!(f, "INTERVAL '-{} {:02}:{:02}:{:02}.{:09}' DAY TO SECOND",
                   -self.days, -self.hours, -self.minutes, -self.seconds, -self.nanoseconds)
        } else {
            write!(f, "INTERVAL '+{} {:02}:{:02}:{:02}.{:09}' DAY TO SECOND",
                   self.days, self.hours, self.minutes, self.seconds, self.nanoseconds)
        }
    }
}

//
// IntervalYM
//

#[derive(Debug, Clone, PartialEq)]
pub struct IntervalYM {
    years: i32,
    months: i32,
}

impl IntervalYM {
    fn from_dpi_interval_ym(it: &dpiIntervalYM) -> IntervalYM {
        IntervalYM {
            years: it.years,
            months: it.months,
        }
    }

    #[allow(dead_code)] // This function will be used to bind timestamp
    fn set_dpi_interval_ym(&self, it: &mut dpiIntervalYM) {
        it.years = self.years;
        it.months = self.months;
    }

    pub fn new(years: i32, months: i32) -> IntervalYM {
        IntervalYM {
            years: years,
            months: months,
        }
    }

    pub fn years(&self) -> i32 {
        self.years
    }
    pub fn months(&self) -> i32 {
        self.months
    }
}

impl fmt::Display for IntervalYM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.years < 0 || self.months < 0 {
            write!(f, "INTERVAL '-{}-{}' YEAR TO MONTH",
                   -self.years, -self.months)
        } else {
            write!(f, "INTERVAL '{}-{}' YEAR TO MONTH",
                   self.years, self.months)
        }
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

    fn new_from_dpi_ver(ver: dpiVersionInfo) -> Version {
        Version::new(ver.versionNum, ver.releaseNum, ver.updateNum, ver.portReleaseNum, ver.portUpdateNum)
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

//
// functions to check errors
//

fn error_from_dpi_error(err: &dpiErrorInfo) -> Error {
    let err = DbError::new(err.code, err.offset,
                           String::from_utf8_lossy(unsafe {
                               slice::from_raw_parts(err.message as *mut u8, err.messageLength as usize)
                           }).into_owned(),
                           unsafe { CStr::from_ptr(err.fnName) }.to_string_lossy().into_owned(),
                           unsafe { CStr::from_ptr(err.action) }.to_string_lossy().into_owned());
    if err.message().starts_with("DPI") {
        Error::DpiError(err)
    } else {
        Error::OciError(err)
    }
}

fn error_from_dpi_context(ctxt: &DpiContext) -> Error {
    let mut err: dpiErrorInfo = Default::default();
    unsafe {
        dpiContext_getError(ctxt.context, &mut err);
    };
    error_from_dpi_error(&err)
}

macro_rules! dpi_call {
    ($ctxt:expr, $code:expr) => {{
        if unsafe { $code } == DPI_SUCCESS {
            ()
        } else {
            return Err(error_from_dpi_context($ctxt));
        }
    }};
}

//
// Utility struct to convert Rust strings from/to ODPI-C strings
//

struct OdpiStr {
    ptr: *const c_char,
    len: uint32_t,
}

fn new_odpi_str() -> OdpiStr {
    OdpiStr {
        ptr: ptr::null(),
        len: 0,
    }
}

fn to_odpi_str(s: &str) -> OdpiStr {
    OdpiStr {
        ptr: s.as_ptr() as *const c_char,
        len: s.len() as uint32_t,
    }
}

impl OdpiStr {
    fn new(ptr: *const c_char, len: uint32_t) -> OdpiStr {
        OdpiStr {
            ptr: ptr,
            len: len,
        }
    }
    unsafe fn from_bytes(bytes: *mut dpiBytes) -> OdpiStr {
        let len = (*bytes).length;
        if len != 0 {
            OdpiStr {
                ptr: (*bytes).ptr,
                len: len,
            }
        } else {
            OdpiStr {
                ptr: ptr::null(),
                len: 0,
            }
        }
    }
    fn to_string(&self) -> String {
        let vec = unsafe { slice::from_raw_parts(self.ptr as *mut u8, self.len as usize) };
        String::from_utf8_lossy(vec).into_owned()
    }
}

//
// Default value definitions
//

impl Default for dpiCommonCreateParams {
    fn default() -> dpiCommonCreateParams {
        dpiCommonCreateParams {
            createMode: 0,
            encoding: ptr::null(),
            nencoding: ptr::null(),
            edition: ptr::null(),
            editionLength: 0,
            driverName: ptr::null(),
            driverNameLength: 0,
        }
    }
}

impl Default for dpiConnCreateParams {
    fn default() -> dpiConnCreateParams {
        dpiConnCreateParams {
            authMode: 0,
            connectionClass: ptr::null(),
            connectionClassLength: 0,
            purity: 0,
            newPassword: ptr::null(),
            newPasswordLength: 0,
            appContext: ptr::null_mut(),
            numAppContext: 0,
            externalAuth: 0,
            externalHandle: ptr::null_mut(),
            pool: ptr::null_mut(),
            tag: ptr::null(),
            tagLength: 0,
            matchAnyTag: 0,
            outTag: ptr::null(),
            outTagLength: 0,
            outTagFound: 0,
        }
    }
}

impl Default for dpiPoolCreateParams {
    fn default() -> dpiPoolCreateParams {
        dpiPoolCreateParams {
            minSessions: 0,
            maxSessions: 0,
            sessionIncrement: 0,
            pingInterval: 0,
            pingTimeout: 0,
            homogeneous: 0,
            externalAuth: 0,
            getMode: 0,
            outPoolName: ptr::null(),
            outPoolNameLength: 0,
        }
    }
}

impl Default for dpiSubscrCreateParams {
    fn default() -> dpiSubscrCreateParams {
        dpiSubscrCreateParams {
            subscrNamespace: 0,
            protocol: 0,
            qos: 0,
            operations: 0,
            portNumber: 0,
            timeout: 0,
            name: ptr::null(),
            nameLength: 0,
            callback: None,
            callbackContext: ptr::null_mut(),
            recipientName: ptr::null(),
            recipientNameLength: 0,
        }
    }
}

impl Default for dpiErrorInfo {
    fn default() -> dpiErrorInfo {
        dpiErrorInfo {
            code: 0,
            offset: 0,
            message: ptr::null(),
            messageLength: 0,
            encoding: ptr::null(),
            fnName: ptr::null(),
            action: ptr::null(),
            sqlState: ptr::null(),
            isRecoverable: 0,
        }
    }
}

impl Default for dpiQueryInfo {
    fn default() -> dpiQueryInfo {
        dpiQueryInfo {
            name: ptr::null(),
            nameLength: 0,
            oracleTypeNum: 0,
            defaultNativeTypeNum: 0,
            dbSizeInBytes: 0,
            clientSizeInBytes: 0,
            sizeInChars: 0,
            precision: 0,
            scale: 0,
            nullOk: 0,
            objectType: ptr::null_mut(),
        }
    }
}

//
// DpiContext
//

pub struct DpiContext {
    pub context: *mut dpiContext,
    pub common_create_params: dpiCommonCreateParams,
    pub conn_create_params: dpiConnCreateParams,
    pub pool_create_params: dpiPoolCreateParams,
    pub subscr_create_params: dpiSubscrCreateParams,
}

enum DpiContextResult {
    Ok(DpiContext),
    Err(dpiErrorInfo),
}

unsafe impl Sync for DpiContextResult {}

lazy_static! {
    static ref DPI_CONTEXT: DpiContextResult = {
        let mut ctxt = DpiContext {
            context: ptr::null_mut(),
            common_create_params: Default::default(),
            conn_create_params: Default::default(),
            pool_create_params: Default::default(),
            subscr_create_params: Default::default(),
        };
        let mut err: dpiErrorInfo = Default::default();
        if unsafe {
            dpiContext_create(DPI_MAJOR_VERSION, DPI_MINOR_VERSION, &mut ctxt.context, &mut err)
        } == DPI_SUCCESS {
            unsafe {
                let utf8_ptr = "UTF-8\0".as_ptr() as *const c_char;
                let driver_name = "Rust Oracle : 0.0.1"; // Update this line also when version up.
                let driver_name_ptr = driver_name.as_ptr() as *const c_char;
                let driver_name_len = driver_name.len() as uint32_t;
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
            DpiContextResult::Ok(ctxt)
        } else {
            DpiContextResult::Err(err)
        }
    };
}

impl DpiContext {
    pub fn get() -> Result<&'static DpiContext> {
        match *DPI_CONTEXT {
            DpiContextResult::Ok(ref ctxt) => Ok(ctxt),
            DpiContextResult::Err(ref err) => Err(error_from_dpi_error(err)),
        }
    }
    pub fn client_version(&self) -> Result<Version> {
        let mut dpi_ver = Default::default();
        dpi_call!(self,
                  dpiContext_getClientVersion(self.context, &mut dpi_ver));
        Ok(Version::new_from_dpi_ver(dpi_ver))
    }
}

//
// DpiConnection
//

pub struct DpiConnection {
    ctxt: &'static DpiContext,
    conn: *mut dpiConn,
}

impl DpiConnection {
    pub fn new(ctxt: &'static DpiContext, username: &str, password: &str, connect_string: &str, params: &mut dpiConnCreateParams) -> Result<DpiConnection> {
        let username = to_odpi_str(username);
        let password = to_odpi_str(password);
        let connect_string = to_odpi_str(connect_string);
        let mut conn: *mut dpiConn = ptr::null_mut();
        dpi_call!(ctxt,
                  dpiConn_create(ctxt.context, username.ptr, username.len,
                                 password.ptr, password.len, connect_string.ptr,
                                 connect_string.len, &ctxt.common_create_params,
                                 params, &mut conn));
        Ok(DpiConnection{ ctxt: ctxt, conn: conn })
    }

    pub fn break_execution(&self) -> Result<()> {
        dpi_call!(self.ctxt,
                  dpiConn_breakExecution(self.conn));
        Ok(())
    }

    pub fn change_password(&self, username: &str, old_password: &str, new_password: &str) -> Result<()> {
        let username = to_odpi_str(username);
        let old_password = to_odpi_str(old_password);
        let new_password = to_odpi_str(new_password);
        dpi_call!(self.ctxt,
                  dpiConn_changePassword(self.conn,
                                         username.ptr, username.len,
                                         old_password.ptr, old_password.len,
                                         new_password.ptr, new_password.len));
        Ok(())
    }

    pub fn close(&self, mode: dpiConnCloseMode, tag: &str) -> Result<()> {
        let tag = to_odpi_str(tag);
        dpi_call!(self.ctxt,
                  dpiConn_close(self.conn, mode, tag.ptr, tag.len));
        Ok(())
    }

    pub fn commit(&self) -> Result<()> {
        dpi_call!(self.ctxt,
                  dpiConn_commit(self.conn));
        Ok(())
    }

    //pub fn dpiConn_deqObject
    //pub fn dpiConn_enqObject

    pub fn current_schema(&self) -> Result<String> {
        let mut s = new_odpi_str();
        dpi_call!(self.ctxt,
                  dpiConn_getCurrentSchema(self.conn, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    pub fn edition(&self) -> Result<String> {
        let mut s = new_odpi_str();
        dpi_call!(self.ctxt,
                  dpiConn_getEdition(self.conn, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    pub fn external_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        dpi_call!(self.ctxt,
                  dpiConn_getExternalName(self.conn, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    pub fn internal_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        dpi_call!(self.ctxt,
                  dpiConn_getInternalName(self.conn, &mut s.ptr, &mut s.len));
        Ok(s.to_string())
    }

    //pub fn dpiConn_getLTXID
    //pub fn dpiConn_getObjectType

    pub fn server_version(&self) -> Result<(String, Version)> {
        let mut s = new_odpi_str();
        let mut dpi_ver = Default::default();
        dpi_call!(self.ctxt,
                  dpiConn_getServerVersion(self.conn, &mut s.ptr, &mut s.len,
                                           &mut dpi_ver));
        Ok((s.to_string(), Version::new_from_dpi_ver(dpi_ver)))
    }

    pub fn stmt_cache_size(&self) -> Result<u32> {
        let mut size = 0u32;
        dpi_call!(self.ctxt,
                  dpiConn_getStmtCacheSize(self.conn, &mut size));
        Ok(size)
    }

    //pub fn dpiConn_newDeqOptions
    //pub fn dpiConn_newEnqOptions
    //pub fn dpiConn_newMsgProps
    //pub fn dpiConn_newSubscription
    //pub fn dpiConn_newTempLob
    //pub fn dpiConn_newVar

    pub fn ping(&self) -> Result<()> {
        dpi_call!(self.ctxt,
                  dpiConn_ping(self.conn));
        Ok(())
    }

    //pub fn dpiConn_prepareDistribTrans

    pub fn prepare_statement(&self, scrollable: bool, sql: &str, tag: &str) -> Result<DpiStatement> {
        let scrollable = if scrollable { 1 } else { 0 };
        let sql = to_odpi_str(sql);
        let tag = to_odpi_str(tag);
        let mut stmt: *mut dpiStmt = ptr::null_mut();
        dpi_call!(self.ctxt,
                  dpiConn_prepareStmt(self.conn, scrollable, sql.ptr, sql.len,
                                      tag.ptr, tag.len, &mut stmt));
        let mut info: dpiStmtInfo = Default::default();
        let rc = unsafe { dpiStmt_getInfo(stmt, &mut info) };
        if rc != DPI_SUCCESS {
            let err = error_from_dpi_context(&self.ctxt);
            unsafe {
                let _ = dpiStmt_release(stmt);
            };
            return Err(err);
        }
        Ok(DpiStatement{
            ctxt: self.ctxt,
            conn: self,
            stmt: stmt,
            fetch_array_size: 0,
            is_query: info.isQuery != 0,
            is_plsql: info.isPLSQL != 0,
            is_ddl: info.isDDL != 0,
            is_dml: info.isDML != 0,
            statement_type: StatementType::from_i32(info.statementType),
            is_returning: info.isReturning != 0,
        })
    }

    pub fn rollback(&self) -> Result<()> {
        dpi_call!(self.ctxt,
                  dpiConn_rollback(self.conn));
        Ok(())
    }

    pub fn set_action(&self, action: &str) -> Result<()> {
        let s = to_odpi_str(action);
        dpi_call!(self.ctxt,
                  dpiConn_setAction(self.conn, s.ptr, s.len));
        Ok(())
    }

    pub fn set_client_identifier(&self, client_identifier: &str) -> Result<()> {
        let s = to_odpi_str(client_identifier);
        dpi_call!(self.ctxt,
                  dpiConn_setClientIdentifier(self.conn, s.ptr, s.len));
        Ok(())
    }

    pub fn set_client_info(&self, client_info: &str) -> Result<()> {
        let s = to_odpi_str(client_info);
        dpi_call!(self.ctxt,
                  dpiConn_setClientInfo(self.conn, s.ptr, s.len));
        Ok(())
    }

    pub fn set_current_schema(&self, current_schema: &str) -> Result<()> {
        let s = to_odpi_str(current_schema);
        dpi_call!(self.ctxt,
                  dpiConn_setCurrentSchema(self.conn, s.ptr, s.len));
        Ok(())
    }

    pub fn set_db_op(&self, db_op: &str) -> Result<()> {
        let s = to_odpi_str(db_op);
        dpi_call!(self.ctxt,
                  dpiConn_setDbOp(self.conn, s.ptr, s.len));
        Ok(())
    }
    pub fn set_external_name(&self, external_name: &str) -> Result<()> {
        let s = to_odpi_str(external_name);
        dpi_call!(self.ctxt,
                  dpiConn_setExternalName(self.conn, s.ptr, s.len));
        Ok(())
    }
    pub fn set_internal_name(&self, internal_name: &str) -> Result<()> {
        let s = to_odpi_str(internal_name);
        dpi_call!(self.ctxt,
                  dpiConn_setInternalName(self.conn, s.ptr, s.len));
        Ok(())
    }

    pub fn set_module(&self, module: &str) -> Result<()> {
        let s = to_odpi_str(module);
        dpi_call!(self.ctxt,
                  dpiConn_setModule(self.conn, s.ptr, s.len));
        Ok(())
    }

    pub fn set_stmt_cache_size(&self, size: u32) -> Result<()> {
        dpi_call!(self.ctxt,
                  dpiConn_setStmtCacheSize(self.conn, size));
        Ok(())
    }

    pub fn shutdown_database(&self, mode: ShutdownMode) -> Result<()> {
        dpi_call!(self.ctxt,
                  dpiConn_shutdownDatabase(self.conn, mode as i32));
        Ok(())
    }

    pub fn startup_database(&self, mode: StartupMode) -> Result<()> {
        dpi_call!(self.ctxt,
                  dpiConn_startupDatabase(self.conn, mode as i32));
        Ok(())
    }
}

impl Drop for DpiConnection {
    fn drop(&mut self) {
        let _ = unsafe { dpiConn_release(self.conn) };
    }
}

//
// DpiStatement
//

pub struct DpiStatement<'conn> {
    ctxt: &'static DpiContext,
    conn: &'conn DpiConnection,
    stmt: *mut dpiStmt,
    fetch_array_size: u32,
    pub is_query: bool,
    pub is_plsql: bool,
    pub is_ddl: bool,
    pub is_dml: bool,
    pub statement_type: StatementType,
    pub is_returning: bool,
}

impl<'conn> DpiStatement<'conn> {
    pub fn close(&self, tag: &str) -> Result<()> {
        let tag = to_odpi_str(tag);
        dpi_call!(self.ctxt,
                  dpiStmt_close(self.stmt, tag.ptr, tag.len));
        Ok(())
    }

    pub fn execute(&mut self, mode: dpiExecMode) -> Result<usize> {
        let mut num_query_columns = 0;
        dpi_call!(self.ctxt,
                  dpiStmt_execute(self.stmt, mode, &mut num_query_columns));
        dpi_call!(self.ctxt,
                  dpiStmt_getFetchArraySize(self.stmt, &mut self.fetch_array_size));
        Ok(num_query_columns as usize)
    }

    pub fn define(&self, pos: usize, oratype: &OracleType) -> Result<DpiVar<'conn>> {
        let var = try!(DpiVar::new(self.conn, oratype, DPI_DEFAULT_FETCH_ARRAY_SIZE));
        dpi_call!(self.ctxt,
                  dpiStmt_define(self.stmt, pos as u32, var.var));
        Ok(var)
    }

    pub fn fetch(&self) -> Result<(bool, u32)> {
        let mut found = 0;
        let mut buffer_row_index = 0;
        dpi_call!(self.ctxt,
                  dpiStmt_fetch(self.stmt, &mut found, &mut buffer_row_index));
        Ok((found != 0, buffer_row_index))
    }

    pub fn column_info(&self, pos: usize) -> Result<ColumnInfo> {
        let mut info = Default::default();
        dpi_call!(self.ctxt,
                  dpiStmt_getQueryInfo(self.stmt, pos as u32, &mut info));
        Ok(try!(ColumnInfo::new(&info)))
    }

    pub fn query_value<'stmt>(&'stmt self, pos: usize, oratype: &'stmt OracleType) -> Result<DpiData<'stmt>> {
        let mut native_type = 0;
        let mut data = ptr::null_mut();
        dpi_call!(self.ctxt,
                  dpiStmt_getQueryValue(self.stmt, pos as u32, &mut native_type, &mut data));
        Ok(DpiData::new(self, oratype, native_type, data))
    }
}

impl<'conn> Drop for DpiStatement<'conn> {
    fn drop(&mut self) {
        let _ = unsafe { dpiStmt_release(self.stmt) };
    }
}

//
// ColumnInfo - corresponds to dpiQueryInfo
//

pub struct ColumnInfo {
    name: String,
    oracle_type: OracleType,
    nullable: bool,
}

impl ColumnInfo {
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

impl ColumnInfo {
    fn new(info: &dpiQueryInfo) -> Result<ColumnInfo> {
        Ok(ColumnInfo {
            name: OdpiStr::new(info.name, info.nameLength).to_string(),
            oracle_type: try!(OracleType::from_query_info(&info)),
            nullable: info.nullOk != 0,
        })
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
// DpiVar
//

#[allow(dead_code)]
pub struct DpiVar<'conn> {
    _conn: &'conn DpiConnection,
    var: *mut dpiVar,
    data: *mut dpiData,
}

impl<'conn> DpiVar<'conn> {
    fn new(conn: &'conn DpiConnection, oratype: &OracleType, array_size: u32) -> Result<DpiVar<'conn>> {
        let mut var: *mut dpiVar = ptr::null_mut();
        let mut data: *mut dpiData = ptr::null_mut();
        let (oratype, native_type, size, size_is_byte) = try!(oratype.var_create_param());
        dpi_call!(conn.ctxt,
                  dpiConn_newVar(conn.conn, oratype, native_type, array_size, size, size_is_byte,
                                 0, ptr::null_mut(), &mut var, &mut data));
        Ok(DpiVar {
            _conn: conn,
            var: var,
            data: data,
        })
    }
}

impl<'conn> Drop for DpiVar<'conn> {
    fn drop(&mut self) {
        let _ = unsafe { dpiVar_release(self.var) };
    }
}

//
// DpiData
//

pub struct DpiData<'stmt> {
    _stmt: &'stmt DpiStatement<'stmt>,
    oratype: &'stmt OracleType,
    native_type: i32,
    data: *mut dpiData,
}

macro_rules! check_not_null {
    ($var:ident) => {
        if $var.is_null() {
            return Err(Error::NullConversionError);
        }
    }
}

impl<'stmt> DpiData<'stmt> {
    fn new(stmt: &'stmt DpiStatement, oratype: &'stmt OracleType, native_type: i32, data: *mut dpiData) -> DpiData<'stmt> {
        DpiData {
            _stmt: stmt,
            oratype: oratype,
            native_type: native_type,
            data: data,
        }
    }

    fn invalid_type_conversion<T>(&self, to_type: &str) -> Result<T> {
        Err(Error::InvalidTypeConversion(self.oratype.to_string(), to_type.to_string()))
    }

    pub fn is_null(&self) -> bool {
        unsafe {
            (&*self.data).isNull != 0
        }
    }

    pub fn as_int64(&self) -> Result<i64> {
        if self.native_type == DPI_NATIVE_TYPE_INT64 {
            check_not_null!(self);
            unsafe {
                Ok(dpiData_getInt64(self.data))
            }
        } else {
            self.invalid_type_conversion("i64")
        }
    }

    pub fn as_uint64(&self) -> Result<u64> {
        if self.native_type == DPI_NATIVE_TYPE_UINT64 {
            check_not_null!(self);
            unsafe {
                Ok(dpiData_getUint64(self.data))
            }
        } else {
            self.invalid_type_conversion("uint64")
        }
    }

    pub fn as_double(&self) -> Result<f64> {
        if self.native_type == DPI_NATIVE_TYPE_DOUBLE {
            check_not_null!(self);
            unsafe {
                Ok(dpiData_getDouble(self.data))
            }
        } else {
            self.invalid_type_conversion("f64")
        }
    }

    pub fn as_float(&self) -> Result<f32> {
        if self.native_type == DPI_NATIVE_TYPE_FLOAT {
            check_not_null!(self);
            unsafe {
                Ok(dpiData_getFloat(self.data))
            }
        } else {
            self.invalid_type_conversion("f32")
        }
    }

    pub fn as_string(&self) -> Result<String> {
        if self.native_type == DPI_NATIVE_TYPE_BYTES {
            check_not_null!(self);
            unsafe {
                let bytes = dpiData_getBytes(self.data);
                Ok(OdpiStr::from_bytes(bytes).to_string())
            }
        } else {
            self.invalid_type_conversion("String")
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        if self.native_type == DPI_NATIVE_TYPE_BOOLEAN {
            check_not_null!(self);
            unsafe {
                Ok(dpiData_getBool(self.data) != 0)
            }
        } else {
            self.invalid_type_conversion("bool")
        }
    }

    pub fn as_timestamp(&self) -> Result<Timestamp> {
        if self.native_type == DPI_NATIVE_TYPE_TIMESTAMP {
            check_not_null!(self);
            unsafe {
                let ts = dpiData_getTimestamp(self.data);
                Ok(Timestamp::from_dpi_timestamp(ts))
            }
        } else {
            self.invalid_type_conversion("Timestamp")
        }
    }

    pub fn as_interval_ds(&self) -> Result<IntervalDS> {
        if self.native_type == DPI_NATIVE_TYPE_INTERVAL_DS {
            check_not_null!(self);
            unsafe {
                let it = dpiData_getIntervalDS(self.data);
                Ok(IntervalDS::from_dpi_interval_ds(it))
            }
        } else {
            self.invalid_type_conversion("IntervalDS")
        }
    }

    pub fn as_interval_ym(&self) -> Result<IntervalYM> {
        if self.native_type == DPI_NATIVE_TYPE_INTERVAL_YM {
            check_not_null!(self);
            unsafe {
                let it = dpiData_getIntervalYM(self.data);
                Ok(IntervalYM::from_dpi_interval_ym(it))
            }
        } else {
            self.invalid_type_conversion("IntervalYM")
        }
    }
}
