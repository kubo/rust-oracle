use std::fmt;

use Error;
use Result;

use binding::*;

// NativeType corresponds to dpiNativeTypeNum in ODPI
// except Char, Number, Raw, CLOB and BLOB.
#[derive(Debug, Clone, PartialEq)]
pub enum NativeType {
    Int64,      // i64 in rust
    UInt64,     // u64 in rust
    Float,      // f32 in rust
    Double,     // f64 in rust
    Char,       // String or `str in rust
    Number,     // string represention of Oracle number
    Raw,        // Vec<u8> in rust
    Timestamp,  // oracle::Timestamp in rust
    IntervalDS, // oracle::IntervalDS in rust
    IntervalYM, // oracle::IntervalYM in rust
    #[allow(dead_code)]
    CLOB,
    #[allow(dead_code)]
    BLOB,
    #[allow(dead_code)]
    Object,
    #[allow(dead_code)]
    Stmt,
    #[allow(dead_code)]
    Boolean,    // bool in rust
    Rowid,
}

impl NativeType {
    pub fn to_native_type_num(&self) -> dpiNativeTypeNum {
        match *self {
            NativeType::Int64 => DPI_NATIVE_TYPE_INT64,
            NativeType::UInt64 => DPI_NATIVE_TYPE_UINT64,
            NativeType::Float => DPI_NATIVE_TYPE_FLOAT,
            NativeType::Double => DPI_NATIVE_TYPE_DOUBLE,
            NativeType::Char => DPI_NATIVE_TYPE_BYTES,
            NativeType::Number => DPI_NATIVE_TYPE_BYTES,
            NativeType::Raw => DPI_NATIVE_TYPE_BYTES,
            NativeType::Timestamp => DPI_NATIVE_TYPE_TIMESTAMP,
            NativeType::IntervalDS => DPI_NATIVE_TYPE_INTERVAL_DS,
            NativeType::IntervalYM => DPI_NATIVE_TYPE_INTERVAL_YM,
            NativeType::CLOB => DPI_NATIVE_TYPE_LOB,
            NativeType::BLOB => DPI_NATIVE_TYPE_LOB,
            NativeType::Object => DPI_NATIVE_TYPE_OBJECT,
            NativeType::Stmt => DPI_NATIVE_TYPE_STMT,
            NativeType::Boolean => DPI_NATIVE_TYPE_BOOLEAN,
            NativeType::Rowid => DPI_NATIVE_TYPE_ROWID,
        }
    }
}

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

    // Returns parameters to create a dpiVar handle.
    pub(crate) fn var_create_param(&self) -> Result<(u32, NativeType, u32, i32)> {
        // The followings are basically same with dpiAllOracleTypes[] in
        // dpiOracleType.c. If enum OracleType has an attribute corresponding
        // to defaultNativeTypeNum of dpiQueryInfo, this mapping is not needed.
        // However I don't want to do it to hide internal information such
        // as dpiNativeTypeNum.
        match *self {
            OracleType::Varchar2(size) =>
                Ok((DPI_ORACLE_TYPE_VARCHAR, NativeType::Char, size, 1)),
            OracleType::Nvarchar2(size) =>
                Ok((DPI_ORACLE_TYPE_NVARCHAR, NativeType::Char, size, 0)),
            OracleType::Char(size) =>
                Ok((DPI_ORACLE_TYPE_CHAR, NativeType::Char, size, 1)),
            OracleType::NChar(size) =>
                Ok((DPI_ORACLE_TYPE_NCHAR, NativeType::Char, size, 0)),
            OracleType::Rowid =>
                Ok((DPI_ORACLE_TYPE_ROWID, NativeType::Rowid, 0, 0)),
            OracleType::Raw(size) =>
                Ok((DPI_ORACLE_TYPE_RAW, NativeType::Raw, size, 1)),
            OracleType::BinaryFloat =>
                Ok((DPI_ORACLE_TYPE_NATIVE_FLOAT, NativeType::Float, 0, 0)),
            OracleType::BinaryDouble =>
                Ok((DPI_ORACLE_TYPE_NATIVE_DOUBLE, NativeType::Double, 0, 0)),
            OracleType::Number(_, _) =>
                Ok((DPI_ORACLE_TYPE_NUMBER, NativeType::Number, 0, 0)),
            OracleType::Date =>
                Ok((DPI_ORACLE_TYPE_DATE, NativeType::Timestamp, 0, 0)),
            OracleType::Timestamp(_) =>
                Ok((DPI_ORACLE_TYPE_TIMESTAMP, NativeType::Timestamp, 0, 0)),
            OracleType::TimestampTZ(_) =>
                Ok((DPI_ORACLE_TYPE_TIMESTAMP_TZ, NativeType::Timestamp, 0, 0)),
            OracleType::TimestampLTZ(_) =>
                Ok((DPI_ORACLE_TYPE_TIMESTAMP_LTZ, NativeType::Timestamp, 0, 0)),
            OracleType::IntervalDS(_, _) =>
                Ok((DPI_ORACLE_TYPE_INTERVAL_DS, NativeType::IntervalDS, 0, 0)),
            OracleType::IntervalYM(_) =>
                Ok((DPI_ORACLE_TYPE_INTERVAL_YM, NativeType::IntervalYM, 0, 0)),
//            OracleType::CLob =>
//                Ok((DPI_ORACLE_TYPE_CLOB, NativeType::CLOB, 0, 0)),
//            OracleType::NCLob =>
//                Ok((DPI_ORACLE_TYPE_NCLOB, NativeType::CLOB, 0, 0)),
//            OracleType::BLob =>
//                Ok((DPI_ORACLE_TYPE_BLOB, NativeType::BLOB, 0, 0)),
//            OracleType::BFile =>
//                Ok((DPI_ORACLE_TYPE_BFILE, NativeType::BLOB, 0, 0)),
//            OracleType::RefCursor =>
//                Ok((DPI_ORACLE_TYPE_STMT, NativeType::Stmt, 0, 0)),
//            OracleType::Boolean =>
//                Ok((DPI_ORACLE_TYPE_BOOLEAN, NativeType::Boolean, 0, 0)),
//            OracleType::Object =>
//                Ok((DPI_ORACLE_TYPE_OBJECT, NativeType::Object, 0, 0)),
            OracleType::Long =>
                Ok((DPI_ORACLE_TYPE_LONG_VARCHAR, NativeType::Char, 0, 0)),
            OracleType::LongRaw =>
                Ok((DPI_ORACLE_TYPE_LONG_RAW, NativeType::Raw, 0, 0)),
            OracleType::Int64 =>
                Ok((DPI_ORACLE_TYPE_NATIVE_INT, NativeType::Int64, 0, 0)),
            OracleType::UInt64 =>
                Ok((DPI_ORACLE_TYPE_NATIVE_UINT, NativeType::UInt64, 0, 0)),
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
