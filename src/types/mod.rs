// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017-2018 Kubo Takehiro <kubo@jiubao.org>
//
// Redistribution and use in source and binary forms, with or without modification, are
// permitted provided that the following conditions are met:
//
//    1. Redistributions of source code must retain the above copyright notice, this list of
//       conditions and the following disclaimer.
//
//    2. Redistributions in binary form must reproduce the above copyright notice, this list
//       of conditions and the following disclaimer in the documentation and/or other materials
//       provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHORS ''AS IS'' AND ANY EXPRESS OR IMPLIED
// WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
// ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
// ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// The views and conclusions contained in the software and documentation are those of the
// authors and should not be interpreted as representing official policies, either expressed
// or implied, of the authors.

use Error;
use IntervalDS;
use IntervalYM;
use OracleType;
use Result;
use SqlValue;
use Timestamp;

#[cfg(feature = "chrono")]
pub mod chrono;
pub mod interval_ds;
pub mod interval_ym;
pub mod object;
pub mod oracle_type;
pub mod timestamp;
pub mod version;

/// Conversion from Oracle values to rust values.
///
/// Values in Oracle are converted to Rust type as possible as it can.
/// The following table indicates supported conversion.
///
/// | Oracle Type | Rust Type |
/// | --- | --- |
/// | CHAR, NCHAR, VARCHAR2, NVARCHAR2 | String |
/// | ″ | i8, i16, i32, i64, u8, u16, u32, u64 by `String.parse()` |
/// | ... | ... |
///
/// This conversion is used also to get values from output parameters.
///
pub trait FromSql {
    fn from_sql(val: &SqlValue) -> Result<Self> where Self: Sized;
}

/// A trait specifying Oracle type to bind a null value.
///
/// This trait is used only when binding a `None` value of `Option<T>`.
/// The type of the null value is determined by the rust type.
///
/// | Rust Type | Oracle Type |
/// | --- | --- |
/// | str, String | NVARCHAR2(0) |
/// | i8, i16, i32, i64, u8, u16, u32, u64, f32, f64 | NUMBER |
/// | Vec\<u8> | RAW(0) |
/// | bool | Boolean (PL/SQL only) |
/// | [Timestamp][] | TIMESTAMP(9) WITH TIME ZONE |
/// | [IntervalDS][] | INTERVAL DAY(9) TO SECOND(9) |
/// | [IntervalYM][] | INTERVAL YEAR(9) TO MONTH |
///
/// When `chrono` feature is enabled, the followings are added.
///
/// | Rust Type | Oracle Type |
/// | --- | --- |
/// | [chrono::Date][] | TIMESTAMP(0) WITH TIME ZONE |
/// | [chrono::DateTime][] | TIMESTAMP(9) WITH TIME ZONE |
/// | [chrono::naive::NaiveDate][] | TIMESTAMP(0) |
/// | [chrono::naive::NaiveDateTime][] | TIMESTAMP(9) |
/// | [chrono::Duration][] | INTERVAL DAY(9) TO SECOND(9) |
pub trait ToSqlNull {
    fn oratype_for_null() -> Result<OracleType>;
}

/// Conversion from rust values to Oracle values.
///
/// The type of the Oracle value is determined by the rust type.
///
/// | Rust Type | Oracle Type | Oracle Value |
/// | --- | --- | --- |
/// | str, String | NVARCHAR2(length of the rust value) | The specified value |
/// | i8, i16, i32, i64, u8, u16, u32, u64, f32, f64 | NUMBER | The specified value |
/// | Vec\<u8> | RAW(length of the rust value) | The specified value |
/// | bool | Boolean (PL/SQL only) | The specified value |
/// | [Timestamp][] | TIMESTAMP(9) WITH TIME ZONE | The specified value |
/// | [IntervalDS][] | INTERVAL DAY(9) TO SECOND(9) | The specified value |
/// | [IntervalYM][] | INTERVAL YEAR(9) TO MONTH | The specified value |
/// | [Collection][] | Type returned by [Collection.oracle_type][] | The specified value |
/// | [Object][] | Type returned by [Object.oracle_type] | The specified value |
/// | Option\<T> where T: ToSql + [ToSqlNull][] | When the value is `Some`, the contained value decides the Oracle type. When it is `None`, ToSqlNull decides it. | When the value is `Some`, the contained value. When it is `None`, a null value.
/// | [OracleType][] | Type represented by the OracleType. | a null value |
/// | (&ToSql, &[OracleType[]) | Type represented by the second element. | The value of the first element |
///
/// When you need to bind output parameters such as varchar2, use `OracleType`
/// or `(&ToSql, &OracleType)` to specify the maximum length of data types.
///
/// When `chrono` feature is enabled, the following conversions are added.
///
/// | Rust Type | Oracle Type |
/// | --- | --- |
/// | [chrono::Date][] | TIMESTAMP(0) WITH TIME ZONE |
/// | [chrono::DateTime][] | TIMESTAMP(9) WITH TIME ZONE |
/// | [chrono::naive::NaiveDate][] | TIMESTAMP(0) |
/// | [chrono::naive::NaiveDateTime][] | TIMESTAMP(9) |
/// | [chrono::Duration][] | INTERVAL DAY(9) TO SECOND(9) |
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
    fn oratype(&self) -> Result<OracleType>;
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
            fn oratype_for_null() -> Result<OracleType> {
                Ok($oratype)
            }
        }
        impl ToSql for $type {
            fn oratype(&self) -> Result<OracleType> {
                Ok($oratype)
            }
            fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
                val.$func(self)
            }
        }
    };
}

macro_rules! impl_from_and_to_sql {
    ($type:ty, $as_func:ident, $set_func:ident, $oratype:expr) => {
        impl_from_sql!($type, $as_func);
        impl_to_sql!($type, $set_func, $oratype);
    };
    ($as_type:ty, $as_func:ident, $set_type:ty, $set_func:ident, $oratype:expr) => {
        impl_from_sql!($as_type, $as_func);
        impl_to_sql!($set_type, $set_func, $oratype);
    };
}

impl_from_and_to_sql!(i8, as_i8, set_i8, OracleType::Number(0,0));
impl_from_and_to_sql!(i16, as_i16, set_i16, OracleType::Number(0,0));
impl_from_and_to_sql!(i32, as_i32, set_i32, OracleType::Number(0,0));
impl_from_and_to_sql!(i64, as_i64, set_i64, OracleType::Number(0,0));
impl_from_and_to_sql!(u8, as_u8, set_u8, OracleType::Number(0,0));
impl_from_and_to_sql!(u16, as_u16, set_u16, OracleType::Number(0,0));
impl_from_and_to_sql!(u32, as_u32, set_u32, OracleType::Number(0,0));
impl_from_and_to_sql!(u64, as_u64, set_u64, OracleType::Number(0,0));
impl_from_and_to_sql!(f64, as_f64, set_f64, OracleType::Number(0,0));
impl_from_and_to_sql!(f32, as_f32, set_f32, OracleType::Number(0,0));
impl_from_and_to_sql!(bool, as_bool, set_bool, OracleType::Boolean);
impl_from_sql!(String, as_string);
impl_from_sql!(Vec<u8>, as_bytes);
impl_from_and_to_sql!(Timestamp, as_timestamp, Timestamp, set_timestamp, OracleType::TimestampTZ(9));
impl_from_and_to_sql!(IntervalDS, as_interval_ds, IntervalDS, set_interval_ds, OracleType::IntervalDS(9,9));
impl_from_and_to_sql!(IntervalYM, as_interval_ym, IntervalYM, set_interval_ym, OracleType::IntervalYM(9));

impl ToSqlNull for String {
    fn oratype_for_null() -> Result<OracleType> {
        Ok(OracleType::NVarchar2(0))
    }
}

impl ToSql for String {
    fn oratype(&self) -> Result<OracleType> {
        Ok(OracleType::NVarchar2(self.len() as u32))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_string(self)
    }
}

impl ToSqlNull for Vec<u8> {
    fn oratype_for_null() -> Result<OracleType> {
        Ok(OracleType::Raw(0))
    }
}

impl ToSql for Vec<u8> {
    fn oratype(&self) -> Result<OracleType> {
        Ok(OracleType::Raw(self.len() as u32))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_bytes(self)
    }
}

impl<'a> ToSqlNull for &'a str {
    fn oratype_for_null() -> Result<OracleType> {
        Ok(OracleType::NVarchar2(0))
    }
}

impl<'a> ToSql for &'a str {
    fn oratype(&self) -> Result<OracleType> {
        Ok(OracleType::NVarchar2(self.len() as u32))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_string(self)
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
    fn oratype(&self) -> Result<OracleType> {
        match *self {
            Some(ref t) => t.oratype(),
            None => <T>::oratype_for_null(),
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
    fn oratype(&self) -> Result<OracleType> {
        Ok(self.clone())
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_null()?;
        Ok(())
    }
}

impl<'a, T: ToSql> ToSql for (&'a T, &'a OracleType) {
    fn oratype(&self) -> Result<OracleType> {
        Ok(self.1.clone())
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        (*self.0).to_sql(val)
    }
}
