// Rust Oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017 Kubo Takehiro <kubo@jiubao.org>
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

use std::marker::PhantomData;

use Error;
use error::ConversionError;
use IntervalDS;
use IntervalYM;
use OracleType;
use Result;
use Statement;
use Timestamp;
use Value;

pub mod chrono;
pub mod interval_ds;
pub mod interval_ym;
pub mod oracle_type;
pub mod timestamp;
pub mod version;

pub trait FromSql {
    fn from(val: &Value) -> Result<Self> where Self: Sized;
}

pub trait ToSql {
    fn oratype() -> OracleType;
    fn to(&self, val: &mut Value) -> Result<()>;
}

macro_rules! impl_from_sql {
    ($type:ty, $func:ident) => {
        impl FromSql for $type {
            fn from(val: &Value) -> Result<$type> {
                val.$func()
            }
        }
    };
}

macro_rules! impl_to_sql {
    (ref $type:ty, $func:ident, $oratype:expr) => {
        impl<'a> ToSql for &'a $type {
            fn oratype() -> OracleType {
                $oratype
            }
            fn to(&self, val: &mut Value) -> Result<()> {
                val.$func(self)
            }
        }
    };
    ($type:ty, $func:ident, $oratype:expr) => {
        impl ToSql for $type {
            fn oratype() -> OracleType {
                $oratype
            }
            fn to(&self, val: &mut Value) -> Result<()> {
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
    ($as_type:ty, $as_func:ident, ref $set_type:ty, $set_func:ident, $oratype:expr) => {
        impl_from_sql!($as_type, $as_func);
        impl_to_sql!(ref $set_type, $set_func, $oratype);
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
impl_from_and_to_sql!(f64, as_f64, set_f64, OracleType::BinaryDouble);
impl_from_and_to_sql!(f32, as_f32, set_f32, OracleType::BinaryDouble);
impl_from_and_to_sql!(bool, as_bool, set_bool, OracleType::Boolean);
impl_from_and_to_sql!(String, as_string, set_string, OracleType::Long);
impl_from_and_to_sql!(Vec<u8>, as_bytes, Vec<u8>, set_bytes, OracleType::LongRaw);
impl_from_and_to_sql!(Timestamp, as_timestamp, Timestamp, set_timestamp, OracleType::Timestamp(9));
impl_from_and_to_sql!(IntervalDS, as_interval_ds, IntervalDS, set_interval_ds, OracleType::IntervalDS(9,9));
impl_from_and_to_sql!(IntervalYM, as_interval_ym, IntervalYM, set_interval_ym, OracleType::IntervalYM(9));

impl<'a> ToSql for &'a str {
    fn oratype() -> OracleType {
        OracleType::Long
    }

    fn to(&self, val: &mut Value) -> Result<()> {
        val.set_string(*self)
    }
}


impl<T: FromSql> FromSql for Option<T> {
    fn from(val: &Value) -> Result<Option<T>> {
        match <T>::from(val) {
            Ok(v) => Ok(Some(v)),
            Err(Error::ConversionError(ConversionError::NullValue)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl<T: ToSql> ToSql for Option<T> {
    fn oratype() -> OracleType {
        <T>::oratype()
    }

    fn to(&self, val: &mut Value) -> Result<()> {
        match *self {
            Some(ref t) => t.to(val),
            None => val.set_null(),
        }
    }
}

impl<'a, T: ToSql> ToSql for &'a T {
    fn oratype() -> OracleType {
        <T>::oratype()
    }

    fn to(&self, val: &mut Value) -> Result<()> {
        (*self).to(val)
    }
}

pub struct Null<T> where T: ToSql {
    dummy: PhantomData<T>,
}

impl<T> Null<T> where T: ToSql {
    pub fn new() -> Null<T> {
        Null {
            dummy: PhantomData,
        }
    }
}

impl<T: ToSql> ToSql for Null<T> {
    fn oratype() -> OracleType {
        <T>::oratype()
    }

    fn to(&self, val: &mut Value) -> Result<()> {
        val.set_null()
    }
}

//
// ToSqlInTuple
//

pub trait ToSqlInTuple<T> {
    fn bind(&self, stmt: &mut Statement) -> Result<()>;
}

impl ToSqlInTuple<()> for () {
    #[allow(unused_variables)]
    fn bind(&self, stmt: &mut Statement) -> Result<()> {
        Ok(())
    }
}

macro_rules! to_sql_in_tuple_impl {
    ($(
        [$(($idx:tt, $T:ident))+],
    )+) => {
        $(
            impl<$($T:ToSql,)+> ToSqlInTuple<($($T,)+)> for ($($T,)+) {
                fn bind(&self, stmt: &mut Statement) -> Result<()> {
                    $(
                        stmt.bind($idx + 1, &<$T>::oratype())?;
                        stmt.set_bind_value($idx + 1, &self.$idx)?;
                    )+
                    Ok(())
                }
            }
        )+
    }
}
to_sql_in_tuple_impl!{
    [(0,T0)],
    [(0,T0)(1,T1)],
    [(0,T0)(1,T1)(2,T2)],
    [(0,T0)(1,T1)(2,T2)(3,T3)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)(47,T47)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)(47,T47)(48,T48)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)(47,T47)(48,T48)(49,T49)],
}
