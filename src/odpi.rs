use std::fmt;
use std::ptr;
use std::slice;
use libc::c_char;
use libc::uint32_t;

use super::binding::*;
use super::Error;
use super::error::error_from_context;
use super::Result;
use connection::Connection;
use statement::Statement;

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
    IntervalDS(i16, u8), // lfprec, fsprec
    /// INTERVAL YEAR TO MONTH data type
    IntervalYM(i16), // lfprec
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

    pub(crate) fn from_type_info(info: &dpiDataTypeInfo) -> Result<OracleType> {
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
            DPI_ORACLE_TYPE_TIMESTAMP => Ok(OracleType::Timestamp(info.fsPrecision)),
            DPI_ORACLE_TYPE_TIMESTAMP_TZ => Ok(OracleType::TimestampTZ(info.fsPrecision)),
            DPI_ORACLE_TYPE_TIMESTAMP_LTZ => Ok(OracleType::TimestampLTZ(info.fsPrecision)),
            DPI_ORACLE_TYPE_INTERVAL_DS => Ok(OracleType::IntervalDS(info.precision, info.fsPrecision)),
            DPI_ORACLE_TYPE_INTERVAL_YM => Ok(OracleType::IntervalYM(info.precision)),
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
    fn var_create_param(&self) -> Result<(u32, u32, u32, i32)> {
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

    pub fn new_from_dpi_ver(ver: dpiVersionInfo) -> Version {
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
// Utility struct to convert Rust strings from/to ODPI-C strings
//

pub struct OdpiStr {
    pub ptr: *const c_char,
    pub len: uint32_t,
}

pub fn new_odpi_str() -> OdpiStr {
    OdpiStr {
        ptr: ptr::null(),
        len: 0,
    }
}

pub fn to_odpi_str(s: &str) -> OdpiStr {
    OdpiStr {
        ptr: s.as_ptr() as *const c_char,
        len: s.len() as uint32_t,
    }
}

impl OdpiStr {
    pub fn new(ptr: *const c_char, len: uint32_t) -> OdpiStr {
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
    pub fn to_string(&self) -> String {
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
            createMode: DPI_MODE_CREATE_DEFAULT,
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
            authMode: DPI_MODE_AUTH_DEFAULT,
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
            shardingKeyColumns: ptr::null_mut(),
            numShardingKeyColumns: 0,
            superShardingKeyColumns: ptr::null_mut(),
            numSuperShardingKeyColumns: 0,
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
            qos: dpiSubscrQOS(0),
            operations: dpiOpCode(0),
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

impl Default for dpiDataTypeInfo {
    fn default() -> dpiDataTypeInfo {
        dpiDataTypeInfo {
            oracleTypeNum: 0,
            defaultNativeTypeNum: 0,
            ociTypeCode: 0,
            dbSizeInBytes: 0,
            clientSizeInBytes: 0,
            sizeInChars: 0,
            precision: 0,
            scale: 0,
            fsPrecision: 0,
            objectType: ptr::null_mut(),
        }
    }
}

impl Default for dpiQueryInfo {
    fn default() -> dpiQueryInfo {
        dpiQueryInfo {
            name: ptr::null(),
            nameLength: 0,
            typeInfo: Default::default(),
            nullOk: 0,
        }
    }
}

impl Default for dpiVersionInfo {
    fn default() -> dpiVersionInfo {
        dpiVersionInfo {
            versionNum: 0,
            releaseNum: 0,
            updateNum: 0,
            portReleaseNum: 0,
            portUpdateNum: 0,
            fullVersionNum: 0,
        }
    }
}

impl Default for dpiStmtInfo {
    fn default() -> dpiStmtInfo {
        dpiStmtInfo {
            isQuery: 0,
            isPLSQL: 0,
            isDDL: 0,
            isDML: 0,
            statementType: 0,
            isReturning: 0,
        }
    }
}

//
// DpiVar
//

#[allow(dead_code)]
pub struct DpiVar<'conn> {
    _conn: &'conn Connection,
    pub(crate) var: *mut dpiVar,
    data: *mut dpiData,
}

impl<'conn> DpiVar<'conn> {
    pub(crate) fn new(conn: &'conn Connection, oratype: &OracleType, array_size: u32) -> Result<DpiVar<'conn>> {
        let mut var: *mut dpiVar = ptr::null_mut();
        let mut data: *mut dpiData = ptr::null_mut();
        let (oratype, native_type, size, size_is_byte) = try!(oratype.var_create_param());
        chkerr!(conn.ctxt,
                dpiConn_newVar(conn.handle, oratype, native_type, array_size, size, size_is_byte,
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
    _stmt: &'stmt Statement<'stmt>,
    oratype: &'stmt OracleType,
    native_type: u32,
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
    pub(crate) fn new(stmt: &'stmt Statement, oratype: &'stmt OracleType, native_type: u32, data: *mut dpiData) -> DpiData<'stmt> {
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
                Ok(Timestamp::from_dpi_timestamp(&*ts))
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
                Ok(IntervalDS::from_dpi_interval_ds(&*it))
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
                Ok(IntervalYM::from_dpi_interval_ym(&*it))
            }
        } else {
            self.invalid_type_conversion("IntervalYM")
        }
    }
}
