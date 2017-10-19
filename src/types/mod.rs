use Error;
use error::ConversionError;
use IntervalDS;
use IntervalYM;
use Result;
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
    fn to(val: &mut Value, newval: Self) -> Result<()>;
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
    (ref $type:ty, $func:ident) => {
        impl<'a> ToSql for &'a $type {
            fn to(val: &mut Value, newval: &'a $type) -> Result<()> {
                val.$func(newval)
            }
        }
    };
    ($type:ty, $func:ident) => {
        impl ToSql for $type {
            fn to(val: &mut Value, newval: $type) -> Result<()> {
                val.$func(newval)
            }
        }
    };
}

macro_rules! impl_from_and_to_sql {
    ($type:ty, $as_func:ident, $set_func:ident) => {
        impl_from_sql!($type, $as_func);
        impl_to_sql!($type, $set_func);
    };
    ($as_type:ty, $as_func:ident, ref $set_type:ty, $set_func:ident) => {
        impl_from_sql!($as_type, $as_func);
        impl_to_sql!(ref $set_type, $set_func);
    };
    ($as_type:ty, $as_func:ident, $set_type:ty, $set_func:ident) => {
        impl_from_sql!($as_type, $as_func);
        impl_to_sql!($set_type, $set_func);
    };
}

impl_from_and_to_sql!(i8, as_i8, set_i8);
impl_from_and_to_sql!(i16, as_i16, set_i16);
impl_from_and_to_sql!(i32, as_i32, set_i32);
impl_from_and_to_sql!(i64, as_i64, set_i64);
impl_from_and_to_sql!(u8, as_u8, set_u8);
impl_from_and_to_sql!(u16, as_u16, set_u16);
impl_from_and_to_sql!(u32, as_u32, set_u32);
impl_from_and_to_sql!(u64, as_u64, set_u64);
impl_from_and_to_sql!(f64, as_f64, set_f64);
impl_from_and_to_sql!(f32, as_f32, set_f32);
impl_from_and_to_sql!(bool, as_bool, set_bool);
impl_from_and_to_sql!(String, as_string, ref str, set_string);
impl_from_and_to_sql!(Vec<u8>, as_bytes, ref Vec<u8>, set_bytes);
impl_from_and_to_sql!(Timestamp, as_timestamp, ref Timestamp, set_timestamp);
impl_from_and_to_sql!(IntervalDS, as_interval_ds, ref IntervalDS, set_interval_ds);
impl_from_and_to_sql!(IntervalYM, as_interval_ym, ref IntervalYM, set_interval_ym);

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
    fn to(val: &mut Value, newval: Option<T>) -> Result<()> {
        match newval {
            Some(v) => <T>::to(val, v),
            None => val.set_null(),
        }
    }
}
