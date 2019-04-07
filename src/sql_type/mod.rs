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

//! SQL data types

use crate::Connection;
use crate::Error;
use crate::Result;
use crate::SqlValue;

#[cfg(feature = "chrono")]
mod chrono;
mod interval_ds;
mod interval_ym;
mod object;
mod oracle_type;
mod timestamp;

pub use self::interval_ds::IntervalDS;
pub use self::interval_ym::IntervalYM;
pub use self::object::Collection;
pub use self::object::Object;
pub use self::object::ObjectType;
pub use self::object::ObjectTypeAttr;
pub(crate) use self::object::ObjectTypeInternal;
pub(crate) use self::oracle_type::NativeType;
pub use self::oracle_type::OracleType;
pub use self::timestamp::Timestamp;

/// Conversion from Oracle values to rust values.
///
/// Values in Oracle are converted to Rust type as possible as it can.
/// The following table indicates supported conversion.
///
/// | Oracle Type | Rust Type |
/// | --- | --- |
/// | character data types | String |
/// | " | i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f64, f32 by using `String.parse()` |
/// | " | Vec\<u8> (The Oracle value must be in hexadecimal.) |
/// | " | [Timestamp][] by `String.parse()` |
/// | " | [IntervalDS][] by `String.parse()` |
/// | " | [IntervalYM][] by `String.parse()` |
/// | numeric data types | i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f64, f32 |
/// | " | String |
/// | raw | Vec\<u8> |
/// | " | String (The Oracle value is converted to characters in hexadecimal.) |
/// | timestamp data types | [Timestamp][] |
/// | " | String |
/// | interval day to month | [IntervalDS][] |
/// | " | String |
/// | interval year to month | [IntervalYM][] |
/// | " | String |
/// | [Oracle object] except [Oracle collection] | [Object][] |
/// | " | String |
/// | [Oracle collection] | [Collection][] |
/// | " | String |
/// | boolean (PL/SQL only) | bool |
///
/// When `chrono` feature is enabled, the followings are added.
///
/// | Oracle Type | Rust Type |
/// | --- | --- |
/// | timestamp data types | [chrono::DateTime][] |
/// | " | [chrono::Date] |
/// | " | [chrono::naive::NaiveDateTime][] |
/// | " | [chrono::naive::NaiveDate][] |
/// | interval day to second | [chrono::Duration][] |
///
/// This conversion is used also to get values from output parameters.
///
/// [Oracle object]: https://docs.oracle.com/en/database/oracle/oracle-database/12.2/adobj/about-oracle-objects.html
/// [Oracle collection]: https://docs.oracle.com/database/122/ADOBJ/collection-data-types.htm
/// [Timestamp]: struct.Timestamp.html
/// [IntervalDS]: struct.IntervalDS.html
/// [IntervalYM]: struct.IntervalYM.html
/// [chrono::Date]: https://docs.rs/chrono/0.4/chrono/struct.Date.html
/// [chrono::DateTime]: https://docs.rs/chrono/0.4/chrono/struct.DateTime.html
/// [chrono::naive::NaiveDate]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDate.html
/// [chrono::naive::NaiveDateTime]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDateTime.html
/// [chrono::Duration]: https://docs.rs/chrono/0.4/chrono/struct.Duration.html
/// [Collection]: struct.Collection.html
/// [Object]: struct.Object.html
pub trait FromSql {
    fn from_sql(val: &SqlValue) -> Result<Self>
    where
        Self: Sized;
}

/// A trait specifying Oracle type to bind a null value.
///
/// This trait is used only when binding a `None` value of `Option<T>`.
/// The type of the null value is determined by the rust type.
///
/// | Rust Type | Oracle Type |
/// | --- | --- |
/// | str, String | nvarchar2(0) |
/// | i8, i16, i32, i64, u8, u16, u32, u64, f32, f64 | number |
/// | Vec\<u8> | raw(0) |
/// | bool | boolean (PL/SQL only) |
/// | [Timestamp][] | timestamp(9) with time zone |
/// | [IntervalDS][] | interval day(9) to second(9) |
/// | [IntervalYM][] | interval year(9) to month |
///
/// When `chrono` feature is enabled, the followings are added.
///
/// | Rust Type | Oracle Type |
/// | --- | --- |
/// | [chrono::Date][] | timestamp(0) with time zone |
/// | [chrono::DateTime][] | timestamp(9) with time zone |
/// | [chrono::naive::NaiveDate][] | timestamp(0) |
/// | [chrono::naive::NaiveDateTime][] | timestamp(9) |
/// | [chrono::Duration][] | interval day(9) to second(9) |
///
/// [Timestamp]: struct.Timestamp.html
/// [IntervalDS]: struct.IntervalDS.html
/// [IntervalYM]: struct.IntervalYM.html
/// [chrono::Date]: https://docs.rs/chrono/0.4/chrono/struct.Date.html
/// [chrono::DateTime]: https://docs.rs/chrono/0.4/chrono/struct.DateTime.html
/// [chrono::naive::NaiveDate]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDate.html
/// [chrono::naive::NaiveDateTime]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDateTime.html
/// [chrono::Duration]: https://docs.rs/chrono/0.4/chrono/struct.Duration.html
pub trait ToSqlNull {
    fn oratype_for_null(conn: &Connection) -> Result<OracleType>;
}

/// Conversion from rust values to Oracle values.
///
/// The type of the Oracle value is determined by the rust type.
///
/// | Rust Type | Oracle Type | Oracle Value |
/// | --- | --- | --- |
/// | str, String | nvarchar2(length of the rust value) | The specified value |
/// | i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64 | number | The specified value |
/// | Vec\<u8> | raw(length of the rust value) | The specified value |
/// | bool | boolean (PL/SQL only) | The specified value |
/// | [Timestamp][] | timestamp(9) with time zone | The specified value |
/// | [IntervalDS][] | interval day(9) to second(9) | The specified value |
/// | [IntervalYM][] | interval year(9) to month | The specified value |
/// | [Collection][] | type returned by [Collection.oracle_type][] | The specified value |
/// | [Object][] | type returned by [Object.oracle_type] | The specified value |
/// | Option\<T> where T: ToSql + [ToSqlNull][] | When the value is `Some`, the contained value decides the Oracle type. When it is `None`, ToSqlNull decides it. | When the value is `Some`, the contained value. When it is `None`, a null value.
/// | [OracleType][] | type represented by the OracleType. | a null value |
/// | (&ToSql, &[OracleType][]) | type represented by the second element. | The value of the first element |
///
/// When you need to bind output parameters such as varchar2, use `OracleType`
/// or `(&ToSql, &OracleType)` to specify the maximum length of data types.
///
/// When `chrono` feature is enabled, the following conversions are added.
///
/// | Rust Type | Oracle Type |
/// | --- | --- |
/// | [chrono::Date][] | timestamp(0) with time zone |
/// | [chrono::DateTime][] | timestamp(9) with time zone |
/// | [chrono::naive::NaiveDate][] | timestamp(0) |
/// | [chrono::naive::NaiveDateTime][] | timestamp(9) |
/// | [chrono::Duration][] | interval day(9) to second(9) |
///
/// [Timestamp]: struct.Timestamp.html
/// [IntervalDS]: struct.IntervalDS.html
/// [IntervalYM]: struct.IntervalYM.html
/// [Collection]: struct.Collection.html
/// [Collection.oracle_type]: struct.Collection.html#method.oracle_type
/// [Object]: struct.Object.html
/// [Object.oracle_type]: struct.Object.html#method.oracle_type
/// [OracleType]: enum.OracleType.html
/// [ToSqlNull]: trait.ToSqlNull.html
/// [chrono::Date]: https://docs.rs/chrono/0.4/chrono/struct.Date.html
/// [chrono::DateTime]: https://docs.rs/chrono/0.4/chrono/struct.DateTime.html
/// [chrono::naive::NaiveDate]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDate.html
/// [chrono::naive::NaiveDateTime]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDateTime.html
/// [chrono::Duration]: https://docs.rs/chrono/0.4/chrono/struct.Duration.html
///
pub trait ToSql {
    fn oratype(&self, conn: &Connection) -> Result<OracleType>;
    fn to_sql(&self, val: &mut SqlValue) -> Result<()>;
}

macro_rules! impl_from_sql {
    ($type:ty, $func:ident) => {
        impl FromSql for $type {
            fn from_sql(val: &SqlValue) -> Result<$type> {
                val.$func()
            }
        }
    };
}

macro_rules! impl_to_sql {
    ($type:ty, $func:ident, $oratype:expr) => {
        impl ToSqlNull for $type {
            fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
                Ok($oratype)
            }
        }
        impl ToSql for $type {
            fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
                Ok($oratype)
            }
            fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
                val.$func(self)
            }
        }
    };
}

macro_rules! impl_from_and_to_sql {
    ($type:ty, $to_func:ident, $set_func:ident, $oratype:expr) => {
        impl_from_sql!($type, $to_func);
        impl_to_sql!($type, $set_func, $oratype);
    };
    ($to_type:ty, $to_func:ident, $set_type:ty, $set_func:ident, $oratype:expr) => {
        impl_from_sql!($to_type, $to_func);
        impl_to_sql!($set_type, $set_func, $oratype);
    };
}

impl_from_and_to_sql!(i8, to_i8, set_i8, OracleType::Number(0, 0));
impl_from_and_to_sql!(i16, to_i16, set_i16, OracleType::Number(0, 0));
impl_from_and_to_sql!(i32, to_i32, set_i32, OracleType::Number(0, 0));
impl_from_and_to_sql!(i64, to_i64, set_i64, OracleType::Number(0, 0));
impl_from_and_to_sql!(isize, to_isize, set_isize, OracleType::Number(0, 0));
impl_from_and_to_sql!(u8, to_u8, set_u8, OracleType::Number(0, 0));
impl_from_and_to_sql!(u16, to_u16, set_u16, OracleType::Number(0, 0));
impl_from_and_to_sql!(u32, to_u32, set_u32, OracleType::Number(0, 0));
impl_from_and_to_sql!(u64, to_u64, set_u64, OracleType::Number(0, 0));
impl_from_and_to_sql!(usize, to_usize, set_usize, OracleType::Number(0, 0));
impl_from_and_to_sql!(f64, to_f64, set_f64, OracleType::Number(0, 0));
impl_from_and_to_sql!(f32, to_f32, set_f32, OracleType::Number(0, 0));
impl_from_and_to_sql!(bool, to_bool, set_bool, OracleType::Boolean);
impl_from_sql!(String, to_string);
impl_from_sql!(Vec<u8>, to_bytes);
impl_from_and_to_sql!(
    Timestamp,
    to_timestamp,
    Timestamp,
    set_timestamp,
    OracleType::TimestampTZ(9)
);
impl_from_and_to_sql!(
    IntervalDS,
    to_interval_ds,
    IntervalDS,
    set_interval_ds,
    OracleType::IntervalDS(9, 9)
);
impl_from_and_to_sql!(
    IntervalYM,
    to_interval_ym,
    IntervalYM,
    set_interval_ym,
    OracleType::IntervalYM(9)
);

impl ToSqlNull for String {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::NVarchar2(0))
    }
}

impl ToSql for String {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::NVarchar2(self.len() as u32))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_string(self)
    }
}

impl ToSqlNull for Vec<u8> {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Raw(0))
    }
}

impl ToSql for Vec<u8> {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Raw(self.len() as u32))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_bytes(self)
    }
}

impl<'a> ToSqlNull for &'a str {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::NVarchar2(0))
    }
}

impl<'a> ToSql for &'a str {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::NVarchar2(self.len() as u32))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_string(self)
    }
}

impl<'a> ToSqlNull for &'a [u8] {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Raw(0))
    }
}

impl<'a> ToSql for &'a [u8] {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Raw(self.len() as u32))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_bytes(*self)
    }
}

impl<T: FromSql> FromSql for Option<T> {
    fn from_sql(val: &SqlValue) -> Result<Option<T>> {
        match <T>::from_sql(val) {
            Ok(v) => Ok(Some(v)),
            Err(Error::NullValue) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl<T: ToSql + ToSqlNull> ToSql for Option<T> {
    fn oratype(&self, conn: &Connection) -> Result<OracleType> {
        match *self {
            Some(ref t) => t.oratype(conn),
            None => <T>::oratype_for_null(conn),
        }
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        match *self {
            Some(ref t) => t.to_sql(val),
            None => val.set_null(),
        }
    }
}

impl ToSql for OracleType {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(self.clone())
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_null()?;
        Ok(())
    }
}

impl<'a, T: ToSql> ToSql for (&'a T, &'a OracleType) {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(self.1.clone())
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        (*self.0).to_sql(val)
    }
}
