// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2018 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use std::fmt;
use std::ptr;

use crate::binding::*;
use crate::connection::Conn;
use crate::sql_type::ObjectType;
use crate::DpiObjectType;
use crate::Error;
use crate::Result;

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
    Clob,
    Blob,
    Object(ObjectType),
    Stmt,
    Boolean, // bool in rust
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
            NativeType::Clob => DPI_NATIVE_TYPE_LOB,
            NativeType::Blob => DPI_NATIVE_TYPE_LOB,
            NativeType::Object(_) => DPI_NATIVE_TYPE_OBJECT,
            NativeType::Stmt => DPI_NATIVE_TYPE_STMT,
            NativeType::Boolean => DPI_NATIVE_TYPE_BOOLEAN,
            NativeType::Rowid => DPI_NATIVE_TYPE_ROWID,
        }
    }

    pub(crate) fn to_object_type_handle(&self) -> *mut dpiObjectType {
        match *self {
            NativeType::Object(ref objtype) => objtype.handle().raw(),
            _ => ptr::null_mut(),
        }
    }
}

/// Oracle data type
#[derive(Debug, Clone, PartialEq)]
pub enum OracleType {
    /// VARCHAR2(size)
    Varchar2(u32),

    /// NVARCHAR2(size)
    NVarchar2(u32),

    /// CHAR(size)
    Char(u32),

    /// NCHAR(size)
    NChar(u32),

    /// ROWID
    Rowid,

    /// RAW(size)
    Raw(u32),

    /// BINARY_FLOAT
    ///
    /// IEEE 754 single-precision (32-bit) floating-point number
    BinaryFloat,

    /// BINARY_DOUBLE
    ///
    /// IEEE 754 double-precision (64-bit) floating-point number
    BinaryDouble,

    /// NUMBER(precision, scale)
    ///
    /// `precision` is between 0 and 38. When it is 0, its actual precision is
    /// 38 and `(precision, scale)` is omitted in text represention.
    ///
    /// `scale` is between -87 and 127. When it is 0, this is represented
    /// as `NUMBER(precision)` in text.
    Number(u8, i8),

    /// FLOAT(precision)
    ///
    /// This is a subtype of NUMBER. The internal format is same with NUMBER,
    /// which means that numbers are stored as decimal not as binary.
    /// Use BINARY_DOUBLE or BINARY_FLOAT to store f64 or f32 rust types.
    ///
    /// `precision` is between 0 and 126. When it is 126, `(precision)` is
    /// omitted in text represention.
    Float(u8),

    /// DATE data type
    Date,

    /// TIMESTAMP(fsprec)
    ///
    /// Timestamp data type without time zone.
    ///
    /// `fsprec` is fractional seconds precision between 0 and 9. When it is
    /// 6, `(fsprec)` is omitted in text represention.
    Timestamp(u8),

    /// TIMESTAMP(fsprec) WITH TIME ZONE
    ///
    /// Timestamp data type with time zone.
    ///
    /// `fsprec` is fractional seconds precision between 0 and 9. When it is
    /// 6, `(fsprec)` is omitted in text represention.
    TimestampTZ(u8),

    /// TIMESTAMP(fsprec) WITH LOCAL TIME ZONE
    ///
    /// Timestamp data type in local session time zone. Clients in different
    /// session time zones retrieves different timestamp.
    ///
    /// `fsprec` is fractional seconds precision between 0 and 9. When it is
    /// 6, `(fsprec)` is omitted in text represention.
    TimestampLTZ(u8),

    /// INTERVAL DAY(lfprec) TO SECOND(fsprec)
    ///
    /// `lfprec` is leading field precision between 0 and 9. When it is 2,
    /// `(lfprec)` is omitted in text represention.
    ///
    /// `fsprec` is fractional seconds precision between 0 and 9. When it is
    /// 6, `(fsprec)` is omitted in text represention.
    IntervalDS(u8, u8),

    /// INTERVAL YEAR(lfprec) TO MONTH
    ///
    /// `lfprec` is leading field precision between 0 and 9. When it is 2,
    /// `(lfprec)` is omitted in text represention.
    IntervalYM(u8),

    /// CLOB
    CLOB,

    /// NCLOB
    NCLOB,

    /// BLOB
    BLOB,

    /// BFILE
    BFILE,

    /// REF CURSOR (not supported)
    RefCursor,

    /// BOOLEAN (not supported)
    Boolean,

    /// Object
    Object(ObjectType),

    /// LONG
    Long,

    /// LONG RAW
    LongRaw,

    /// JSON data type introduced in Oracle 21c
    Json,

    /// XML
    Xml,

    /// Integer type in Oracle object type attributes. This will be renamed to Integer in future.
    Int64,

    /// Not an Oracle type, used only internally to bind/define values as u64
    UInt64,
}

impl OracleType {
    pub(crate) fn from_type_info(conn: &Conn, info: &dpiDataTypeInfo) -> Result<OracleType> {
        match info.oracleTypeNum {
            DPI_ORACLE_TYPE_VARCHAR => Ok(OracleType::Varchar2(info.dbSizeInBytes)),
            DPI_ORACLE_TYPE_NVARCHAR => Ok(OracleType::NVarchar2(info.sizeInChars)),
            DPI_ORACLE_TYPE_CHAR => Ok(OracleType::Char(info.dbSizeInBytes)),
            DPI_ORACLE_TYPE_NCHAR => Ok(OracleType::NChar(info.sizeInChars)),
            DPI_ORACLE_TYPE_ROWID => Ok(OracleType::Rowid),
            DPI_ORACLE_TYPE_RAW => Ok(OracleType::Raw(info.dbSizeInBytes)),
            DPI_ORACLE_TYPE_NATIVE_FLOAT => Ok(OracleType::BinaryFloat),
            DPI_ORACLE_TYPE_NATIVE_DOUBLE => Ok(OracleType::BinaryDouble),
            DPI_ORACLE_TYPE_NATIVE_INT => Ok(OracleType::Int64),
            DPI_ORACLE_TYPE_NUMBER => {
                if info.precision != 0 && info.scale == -127 {
                    Ok(OracleType::Float(info.precision as u8))
                } else {
                    Ok(OracleType::Number(info.precision as u8, info.scale))
                }
            }
            DPI_ORACLE_TYPE_DATE => Ok(OracleType::Date),
            DPI_ORACLE_TYPE_TIMESTAMP => Ok(OracleType::Timestamp(info.fsPrecision)),
            DPI_ORACLE_TYPE_TIMESTAMP_TZ => Ok(OracleType::TimestampTZ(info.fsPrecision)),
            DPI_ORACLE_TYPE_TIMESTAMP_LTZ => Ok(OracleType::TimestampLTZ(info.fsPrecision)),
            DPI_ORACLE_TYPE_INTERVAL_DS => Ok(OracleType::IntervalDS(
                info.precision as u8,
                info.fsPrecision,
            )),
            DPI_ORACLE_TYPE_INTERVAL_YM => Ok(OracleType::IntervalYM(info.precision as u8)),
            DPI_ORACLE_TYPE_CLOB => Ok(OracleType::CLOB),
            DPI_ORACLE_TYPE_NCLOB => Ok(OracleType::NCLOB),
            DPI_ORACLE_TYPE_BLOB => Ok(OracleType::BLOB),
            DPI_ORACLE_TYPE_BFILE => Ok(OracleType::BFILE),
            DPI_ORACLE_TYPE_STMT => Ok(OracleType::RefCursor),
            DPI_ORACLE_TYPE_BOOLEAN => Ok(OracleType::Boolean),
            DPI_ORACLE_TYPE_OBJECT => Ok(OracleType::Object(ObjectType::from_dpi_object_type(
                conn.clone(),
                DpiObjectType::with_add_ref(info.objectType),
            )?)),
            DPI_ORACLE_TYPE_LONG_VARCHAR => Ok(OracleType::Long),
            DPI_ORACLE_TYPE_LONG_RAW => Ok(OracleType::LongRaw),
            DPI_ORACLE_TYPE_JSON => Ok(OracleType::Json),
            DPI_ORACLE_TYPE_XMLTYPE => Ok(OracleType::Xml),
            _ => Err(Error::internal_error(format!(
                "unknown Oracle type number {}",
                info.oracleTypeNum
            ))),
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
            OracleType::Varchar2(size) => Ok((DPI_ORACLE_TYPE_VARCHAR, NativeType::Char, size, 0)),
            OracleType::NVarchar2(size) => {
                Ok((DPI_ORACLE_TYPE_NVARCHAR, NativeType::Char, size, 0))
            }
            OracleType::Char(size) => Ok((DPI_ORACLE_TYPE_CHAR, NativeType::Char, size, 0)),
            OracleType::NChar(size) => Ok((DPI_ORACLE_TYPE_NCHAR, NativeType::Char, size, 0)),
            OracleType::Rowid => Ok((DPI_ORACLE_TYPE_ROWID, NativeType::Rowid, 0, 0)),
            OracleType::Raw(size) => Ok((DPI_ORACLE_TYPE_RAW, NativeType::Raw, size, 1)),
            OracleType::BinaryFloat => Ok((DPI_ORACLE_TYPE_NATIVE_FLOAT, NativeType::Float, 0, 0)),
            OracleType::BinaryDouble => {
                Ok((DPI_ORACLE_TYPE_NATIVE_DOUBLE, NativeType::Double, 0, 0))
            }
            OracleType::Number(_, _) | OracleType::Float(_) => {
                Ok((DPI_ORACLE_TYPE_NUMBER, NativeType::Number, 0, 0))
            }
            OracleType::Date => Ok((DPI_ORACLE_TYPE_DATE, NativeType::Timestamp, 0, 0)),
            OracleType::Timestamp(_) => {
                Ok((DPI_ORACLE_TYPE_TIMESTAMP, NativeType::Timestamp, 0, 0))
            }
            OracleType::TimestampTZ(_) => {
                Ok((DPI_ORACLE_TYPE_TIMESTAMP_TZ, NativeType::Timestamp, 0, 0))
            }
            OracleType::TimestampLTZ(_) => {
                Ok((DPI_ORACLE_TYPE_TIMESTAMP_LTZ, NativeType::Timestamp, 0, 0))
            }
            OracleType::IntervalDS(_, _) => {
                Ok((DPI_ORACLE_TYPE_INTERVAL_DS, NativeType::IntervalDS, 0, 0))
            }
            OracleType::IntervalYM(_) => {
                Ok((DPI_ORACLE_TYPE_INTERVAL_YM, NativeType::IntervalYM, 0, 0))
            }
            OracleType::CLOB => Ok((DPI_ORACLE_TYPE_CLOB, NativeType::Clob, 0, 0)),
            OracleType::NCLOB => Ok((DPI_ORACLE_TYPE_NCLOB, NativeType::Clob, 0, 0)),
            OracleType::BLOB => Ok((DPI_ORACLE_TYPE_BLOB, NativeType::Blob, 0, 0)),
            OracleType::BFILE => Ok((DPI_ORACLE_TYPE_BFILE, NativeType::Blob, 0, 0)),
            OracleType::RefCursor => Ok((DPI_ORACLE_TYPE_STMT, NativeType::Stmt, 0, 0)),
            OracleType::Boolean => Ok((DPI_ORACLE_TYPE_BOOLEAN, NativeType::Boolean, 0, 0)),
            OracleType::Object(ref objtype) => Ok((
                DPI_ORACLE_TYPE_OBJECT,
                NativeType::Object(objtype.clone()),
                0,
                0,
            )),
            OracleType::Long => Ok((DPI_ORACLE_TYPE_LONG_VARCHAR, NativeType::Char, 0, 0)),
            OracleType::LongRaw => Ok((DPI_ORACLE_TYPE_LONG_RAW, NativeType::Raw, 0, 0)),
            OracleType::Xml => Ok((DPI_ORACLE_TYPE_XMLTYPE, NativeType::Char, 0, 0)),
            OracleType::Int64 => Ok((DPI_ORACLE_TYPE_NATIVE_INT, NativeType::Int64, 0, 0)),
            OracleType::UInt64 => Ok((DPI_ORACLE_TYPE_NATIVE_UINT, NativeType::UInt64, 0, 0)),
            _ => Err(Error::internal_error(format!(
                "unsupported Oracle type {}",
                self
            ))),
        }
    }
}

impl fmt::Display for OracleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OracleType::Varchar2(size) => write!(f, "VARCHAR2({})", size),
            OracleType::NVarchar2(size) => write!(f, "NVARCHAR2({})", size),
            OracleType::Char(size) => write!(f, "CHAR({})", size),
            OracleType::NChar(size) => write!(f, "NCHAR({})", size),
            OracleType::Rowid => write!(f, "ROWID"),
            OracleType::Raw(size) => write!(f, "RAW({})", size),
            OracleType::BinaryFloat => write!(f, "BINARY_FLOAT"),
            OracleType::BinaryDouble => write!(f, "BINARY_DOUBLE"),
            OracleType::Number(prec, scale) => {
                if prec == 0 {
                    write!(f, "NUMBER")
                } else if scale == 0 {
                    write!(f, "NUMBER({})", prec)
                } else {
                    write!(f, "NUMBER({},{})", prec, scale)
                }
            }
            OracleType::Float(prec) => {
                if prec == 126 {
                    write!(f, "FLOAT")
                } else {
                    write!(f, "FLOAT({})", prec)
                }
            }
            OracleType::Date => write!(f, "DATE"),
            OracleType::Timestamp(fsprec) => {
                if fsprec == 6 {
                    write!(f, "TIMESTAMP")
                } else {
                    write!(f, "TIMESTAMP({})", fsprec)
                }
            }
            OracleType::TimestampTZ(fsprec) => {
                if fsprec == 6 {
                    write!(f, "TIMESTAMP WITH TIME ZONE")
                } else {
                    write!(f, "TIMESTAMP({}) WITH TIME ZONE", fsprec)
                }
            }
            OracleType::TimestampLTZ(fsprec) => {
                if fsprec == 6 {
                    write!(f, "TIMESTAMP WITH LOCAL TIME ZONE")
                } else {
                    write!(f, "TIMESTAMP({}) WITH LOCAL TIME ZONE", fsprec)
                }
            }
            OracleType::IntervalDS(lfprec, fsprec) => {
                if lfprec == 2 && fsprec == 6 {
                    write!(f, "INTERVAL DAY TO SECOND")
                } else {
                    write!(f, "INTERVAL DAY({}) TO SECOND({})", lfprec, fsprec)
                }
            }
            OracleType::IntervalYM(lfprec) => {
                if lfprec == 2 {
                    write!(f, "INTERVAL YEAR TO MONTH")
                } else {
                    write!(f, "INTERVAL YEAR({}) TO MONTH", lfprec)
                }
            }
            OracleType::CLOB => write!(f, "CLOB"),
            OracleType::NCLOB => write!(f, "NCLOB"),
            OracleType::BLOB => write!(f, "BLOB"),
            OracleType::BFILE => write!(f, "BFILE"),
            OracleType::RefCursor => write!(f, "REF CURSOR"),
            OracleType::Boolean => write!(f, "BOOLEAN"),
            OracleType::Object(ref ty) => write!(f, "{}.{}", ty.schema(), ty.name()),
            OracleType::Long => write!(f, "LONG"),
            OracleType::LongRaw => write!(f, "LONG RAW"),
            OracleType::Json => write!(f, "JSON"),
            OracleType::Xml => write!(f, "XML"),
            OracleType::Int64 => write!(f, "INT64 used internally"),
            OracleType::UInt64 => write!(f, "UINT64 used internally"),
        }
    }
}
