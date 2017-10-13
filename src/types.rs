use value_ref::ValueRef;
use Error;
use Result;
use Timestamp;

pub trait FromSql {
    fn from(valref: &ValueRef) -> Result<Self> where Self: Sized;
    /// type name just for information put in error messages.
    fn type_name() -> String;
}

pub trait ToSql {
    fn to(valref: &mut ValueRef, val: Self) -> Result<()>;
    /// type name just for information put in error messages.
    fn type_name() -> String;
}

macro_rules! define_from_sql {
    ($type_:ident, $func:ident) => {
        impl FromSql for $type_ {
            fn from(valref: &ValueRef) -> Result<$type_> {
                //println!("Converting {} to {}", value, Self::type_name());
                valref.$func()
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

macro_rules! define_to_sql {
    ($type_:ident, $func:ident) => {
        impl ToSql for $type_ {
            fn to(valref: &mut ValueRef, val: $type_) -> Result<()> {
                valref.$func(val)
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

define_to_sql!(i64, set_int64);
define_to_sql!(u64, set_uint64);
define_to_sql!(f64, set_double);
define_to_sql!(f32, set_float);

impl<T: FromSql> FromSql for Option<T> {
    fn from(valref: &ValueRef) -> Result<Option<T>> {
        match <T>::from(valref) {
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
    fn from(valref: &ValueRef) -> Result<Timestamp> {
        valref.as_timestamp()
    }

    fn type_name() -> String {
        "Timestamp".to_string()
    }
}

impl<T: ToSql> ToSql for Option<T> {
    fn to(valref: &mut ValueRef, val: Option<T>) -> Result<()> {
        match val {
            Some(v) => <T>::to(valref, v),
            None => valref.set_null(),
        }
    }

    fn type_name() -> String {
        format!("Option<{}>", <T>::type_name())
    }
}
