use std::fmt;

use binding::*;
use error::Error;
use Result;

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
    pub(crate) fn var_create_param(&self) -> Result<(u32, u32, u32, i32)> {
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

    pub(crate) fn native_type(&self) -> Result<u32> {
        let (_, native_type, _, _) = self.var_create_param()?;
        return Ok(native_type);
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
    pub(crate) fn from_dpi_timestamp(ts: &dpiTimestamp) -> Timestamp {
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
    pub(crate) fn from_dpi_interval_ds(it: &dpiIntervalDS) -> IntervalDS {
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
    pub(crate) fn from_dpi_interval_ym(it: &dpiIntervalYM) -> IntervalYM {
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

    pub(crate) fn new_from_dpi_ver(ver: dpiVersionInfo) -> Version {
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
