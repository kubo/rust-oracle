use value::Value;
use Error;
use Result;
use Timestamp;
use IntervalYM;
use IntervalDS;
use error::ConversionError;

pub trait FromSql {
    fn from(val: &Value) -> Result<Self> where Self: Sized;
    /// type name just for information put in error messages.
    fn type_name() -> String;
}

pub trait ToSql {
    fn to(val: &mut Value, newval: Self) -> Result<()>;
    /// type name just for information put in error messages.
    fn type_name() -> String;
}

macro_rules! define_from_sql {
    ($type_:ty, $func:ident) => {
        impl FromSql for $type_ {
            fn from(val: &Value) -> Result<$type_> {
                val.$func()
            }
            fn type_name() -> String {
                stringify!($type_).to_string()
            }
        }
    };
}

macro_rules! define_to_sql {
    ($type_:ty, $func:ident) => {
        impl ToSql for $type_ {
            fn to(val: &mut Value, newval: $type_) -> Result<()> {
                val.$func(newval)
            }
            fn type_name() -> String {
                stringify!($type_).to_string()
            }
        }
    };
}

define_from_sql!(i8, as_i8);
define_from_sql!(i16, as_i16);
define_from_sql!(i32, as_i32);
define_from_sql!(i64, as_i64);
define_from_sql!(u8, as_u8);
define_from_sql!(u16, as_u16);
define_from_sql!(u32, as_u32);
define_from_sql!(u64, as_u64);
define_from_sql!(f64, as_f64);
define_from_sql!(f32, as_f32);
define_from_sql!(bool, as_bool);
define_from_sql!(String, as_string);
define_from_sql!(Vec<u8>, as_bytes);
define_from_sql!(Timestamp, as_timestamp);
define_from_sql!(IntervalDS, as_interval_ds);
define_from_sql!(IntervalYM, as_interval_ym);

define_to_sql!(i64, set_int64);
define_to_sql!(u64, set_uint64);
define_to_sql!(f64, set_double);
define_to_sql!(f32, set_float);

impl<T: FromSql> FromSql for Option<T> {
    fn from(val: &Value) -> Result<Option<T>> {
        match <T>::from(val) {
            Ok(v) => Ok(Some(v)),
            Err(Error::ConversionError(ConversionError::NullValue)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn type_name() -> String {
        format!("Option<{}>", <T>::type_name())
    }
}

impl<T: ToSql> ToSql for Option<T> {
    fn to(val: &mut Value, newval: Option<T>) -> Result<()> {
        match newval {
            Some(v) => <T>::to(val, v),
            None => val.set_null(),
        }
    }

    fn type_name() -> String {
        format!("Option<{}>", <T>::type_name())
    }
}
