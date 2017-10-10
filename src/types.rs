use value_ref::ValueRef;
use super::Error;
use super::Result;
use super::Timestamp;

pub trait FromSql {
    fn from(value: &ValueRef) -> Result<Self> where Self: Sized;
    /// type name just for information put in error messages.
    fn type_name() -> String;
}

macro_rules! define_from_sql {
    ($type_:ident, $func:ident) => {
        impl FromSql for $type_ {
            fn from(value: &ValueRef) -> Result<$type_> {
                //println!("Converting {} to {}", value, Self::type_name());
                value.$func()
            }
            fn type_name() -> String {
                stringify!($type_).to_string()
            }
        }
    };
}

macro_rules! define_from_sql_with_range_check {
    ($type_:ident, $func:ident, $func_ret_type:ident) => {
        impl FromSql for $type_ {
            fn from(value: &ValueRef) -> Result<$type_> {
                //println!("Converting {} to {}", value, Self::type_name());
                let n = try!(value.$func());
                if $type_::min_value() as $func_ret_type <= n && n <= $type_::max_value() as $func_ret_type{
                    Ok(n as $type_)
                } else {
                    Err(Error::OutOfRange(value.oracle_type().to_string(), $type_::type_name()))
                }
            }
            fn type_name() -> String {
                stringify!($type_).to_string()
            }
        }
    };
}

define_from_sql_with_range_check!(i8, as_int64, i64);
define_from_sql_with_range_check!(i16, as_int64, i64);
define_from_sql_with_range_check!(i32, as_int64, i64);
define_from_sql!(i64, as_int64);
define_from_sql_with_range_check!(u8, as_uint64, u64);
define_from_sql_with_range_check!(u16, as_uint64, u64);
define_from_sql_with_range_check!(u32, as_uint64, u64);
define_from_sql!(u64, as_uint64);
define_from_sql!(f64, as_double);
define_from_sql!(f32, as_float);
define_from_sql!(bool, as_bool);
define_from_sql!(String, as_string);

impl<T: FromSql> FromSql for Option<T> {
    fn from(value: &ValueRef) -> Result<Option<T>> {
        match <T>::from(value) {
            Ok(val) => Ok(Some(val)),
            Err(Error::NullConversionError) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn type_name() -> String {
        format!("Option<{}>", <T>::type_name())
    }
}

impl FromSql for Timestamp {
    fn from(value: &ValueRef) -> Result<Timestamp> {
        value.as_timestamp()
    }

    fn type_name() -> String {
        "Timestamp".to_string()
    }
}
