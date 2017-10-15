use std::fmt;
use std::ptr;
use std::slice;
use try_from::TryInto;

use binding::*;
use types::FromSql;
use types::ToSql;
use Connection;
use Context;
use Error;
use Result;
use OracleType;
use Timestamp;
use IntervalDS;
use IntervalYM;
use error::ConversionError;
use ParseError;
use to_odpi_str;
use NativeType;
use util::check_number_format;

macro_rules! flt_to_int {
    ($expr:expr, $src_type:ident, $dest_type:ident) => {
        {
            let src_val = $expr;
            if $dest_type::min_value() as $src_type <= src_val && src_val <= $dest_type::max_value() as $src_type {
                Ok(src_val as $dest_type)
            } else {
                Err(Error::ConversionError(ConversionError::Overflow(format!("{}", src_val), stringify!($dest_type))))
            }
        }
    }
}

macro_rules! define_fn_as_int {
    ($func_name:ident, $type:ident) => {
        pub fn $func_name(&self) -> Result<$type> {
            match self.native_type {
                NativeType::Int64 =>
                    Ok(self.get_i64_unchecked()?.try_into()?),
                NativeType::UInt64 =>
                    Ok(self.get_u64_unchecked()?.try_into()?),
                NativeType::Float =>
                    flt_to_int!(self.get_f32_unchecked()?, f32, $type),
                NativeType::Double =>
                    flt_to_int!(self.get_f64_unchecked()?, f64, $type),
                NativeType::Char |
                NativeType::Number =>
                    Ok(self.get_string_unchecked()?.parse()?),
                _ =>
                    self.unsupported_as_type_conversion(stringify!($type))
            }
        }
    }
}

macro_rules! define_fn_set_int {
    ($func_name:ident, $type:ident) => {
        pub fn $func_name(&mut self, val: $type) -> Result<()> {
            match self.native_type {
                NativeType::Int64 =>
                    self.set_i64_unchecked(val as i64),
                NativeType::UInt64 =>
                    self.set_u64_unchecked(val as u64),
                NativeType::Float =>
                    self.set_f32_unchecked(val as f32),
                NativeType::Double =>
                    self.set_f64_unchecked(val as f64),
                NativeType::Char |
                NativeType::Number => {
                    let s = format!("{}", val);
                    self.set_string_unchecked(&s)
                },
                _ =>
                    self.unsupported_set_type_conversion(stringify!($type))
            }
        }
    }
}


pub struct Value {
    ctxt: &'static Context,
    pub(crate) handle: *mut dpiVar,
    native_type: NativeType,
    oratype: OracleType,
    pub(crate) buffer_row_index: u32,
}

impl Value {

    pub(crate) fn new(ctxt: &'static Context) -> Value {
        Value {
            ctxt: ctxt,
            handle: ptr::null_mut(),
            native_type: NativeType::Int64,
            oratype: OracleType::None,
            buffer_row_index: 0,
        }
    }

    pub(crate) fn init_handle(&mut self, conn: &Connection, oratype: &OracleType, array_size: u32) -> Result<()> {
        if !self.handle.is_null() {
            unsafe { dpiVar_release(self.handle) };
        }
        self.handle = ptr::null_mut();
        let mut handle: *mut dpiVar = ptr::null_mut();
        let mut data: *mut dpiData = ptr::null_mut();
        let (oratype_num, native_type, size, size_is_byte) = oratype.var_create_param()?;
        let native_type_num = native_type.to_native_type_num();
        chkerr!(conn.ctxt,
                dpiConn_newVar(conn.handle, oratype_num, native_type_num, array_size, size, size_is_byte,
                               0, ptr::null_mut(), &mut handle, &mut data));
        self.handle = handle;
        self.native_type = native_type;
        self.oratype = oratype.clone();
        Ok(())
    }

    pub(crate) fn initialized(&self) -> bool {
        self.oratype != OracleType::None
    }

    fn data(&self) -> Result<*mut dpiData> {
        if self.oratype == OracleType::None {
            return Err(Error::UninitializedBindValue);
        }
        let mut num = 0;
        let mut data = ptr::null_mut();
        chkerr!(self.ctxt,
                dpiVar_getData(self.handle, &mut num, &mut data));
        if self.buffer_row_index < num {
            Ok(unsafe{data.offset(self.buffer_row_index as isize)})
        } else {
            Err(Error::InternalError(format!("Invalid buffer row index {} (num: {})", self.buffer_row_index, num)))
        }
    }

    pub fn get<T>(&self) -> Result<T> where T: FromSql {
        <T>::from(self)
    }

    pub fn set<T>(&mut self, val: T) -> Result<()> where T: ToSql {
        <T>::to(self, val)
    }

    fn unsupported_as_type_conversion<T>(&self, to_type: &str) -> Result<T> {
        Err(Error::ConversionError(ConversionError::UnsupportedType(self.oratype.to_string(), to_type.to_string())))
    }

    fn unsupported_set_type_conversion<T>(&self, from_type: &str) -> Result<T> {
        Err(Error::ConversionError(ConversionError::UnsupportedType(from_type.to_string(), self.oratype.to_string())))
    }

    fn check_not_null(&self) -> Result<()> {
        if self.is_null()? {
            Err(Error::ConversionError(ConversionError::NullValue))
        } else {
            Ok(())
        }
    }

    pub fn is_null(&self) -> Result<bool> {
        unsafe {
            Ok((*self.data()?).isNull != 0)
        }
    }

    pub fn set_null(&mut self) -> Result<()> {
        unsafe {
            (*self.data()?).isNull = 1;
        }
        Ok(())
    }

    pub fn oracle_type(&self) -> &OracleType {
        &self.oratype
    }

    //
    // get_TYPE_unchecked methods
    //

    fn get_i64_unchecked(&self) -> Result<i64> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getInt64(self.data()?)) }
    }

    fn get_u64_unchecked(&self) -> Result<u64> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getUint64(self.data()?)) }
    }

    fn get_f32_unchecked(&self) -> Result<f32> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getFloat(self.data()?)) }
    }

    fn get_f64_unchecked(&self) -> Result<f64> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getDouble(self.data()?)) }
    }

    fn get_string_unchecked(&self) -> Result<String> {
        self.check_not_null()?;
        unsafe {
            let bytes = dpiData_getBytes(self.data()?);
            let ptr = (*bytes).ptr as *mut u8;
            let len = (*bytes).length as usize;
            Ok(String::from_utf8_lossy(slice::from_raw_parts(ptr, len)).into_owned())
        }
    }

    fn get_bytes_unchecked(&self) -> Result<Vec<u8>> {
        self.check_not_null()?;
        unsafe {
            let bytes = dpiData_getBytes(self.data()?);
            let ptr = (*bytes).ptr as *mut u8;
            let len = (*bytes).length as usize;
            let mut vec = Vec::with_capacity(len);
            vec.extend_from_slice(slice::from_raw_parts(ptr, len));
            Ok(vec)
        }
    }

    fn get_timestamp_unchecked(&self) -> Result<Timestamp> {
        self.check_not_null()?;
        unsafe {
            let ts = dpiData_getTimestamp(self.data()?);
            Ok(Timestamp::from_dpi_timestamp(&*ts, &self.oratype))
        }
    }

    fn get_interval_ds_unchecked(&self) -> Result<IntervalDS> {
        self.check_not_null()?;
        unsafe {
            let it = dpiData_getIntervalDS(self.data()?);
            Ok(IntervalDS::from_dpi_interval_ds(&*it, &self.oratype))
        }
    }

    fn get_interval_ym_unchecked(&self) -> Result<IntervalYM> {
        self.check_not_null()?;
        unsafe {
            let it = dpiData_getIntervalYM(self.data()?);
            Ok(IntervalYM::from_dpi_interval_ym(&*it, &self.oratype))
        }
    }

    fn get_bool_unchecked(&self) -> Result<bool> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getBool(self.data()?) != 0) }
    }

    //
    // set_TYPE_unchecked methods
    //

    fn set_i64_unchecked(&mut self, val: i64) -> Result<()> {
        unsafe { dpiData_setInt64(self.data()?, val) }
        Ok(())
    }

    fn set_u64_unchecked(&mut self, val: u64) -> Result<()> {
        unsafe { dpiData_setUint64(self.data()?, val) }
        Ok(())
    }

    fn set_f32_unchecked(&mut self, val: f32) -> Result<()> {
        unsafe { dpiData_setFloat(self.data()?, val) }
        Ok(())
    }

    fn set_f64_unchecked(&mut self, val: f64) -> Result<()> {
        unsafe { dpiData_setDouble(self.data()?, val) }
        Ok(())
    }

    fn set_string_unchecked(&self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(self.ctxt,
                dpiVar_setFromBytes(self.handle, self.buffer_row_index, val.ptr, val.len));
        Ok(())
    }

    fn set_bytes_unchecked(&self, val: &Vec<u8>) -> Result<()> {
        chkerr!(self.ctxt,
                dpiVar_setFromBytes(self.handle, self.buffer_row_index,
                                    val.as_ptr() as *const i8, val.len() as u32));
        Ok(())
    }

    fn set_timestamp_unchecked(&self, val: &Timestamp) -> Result<()> {
        unsafe { dpiData_setTimestamp(self.data()?, val.year as i16,
                                      val.month as u8, val.day as u8,
                                      val.hour as u8, val.minute as u8, val.second as u8,
                                      val.nanosecond, val.tz_hour_offset as i8,
                                      val.tz_minute_offset.abs() as i8) }
        Ok(())
    }

    fn set_interval_ds_unchecked(&self, val: &IntervalDS) -> Result<()> {
        unsafe { dpiData_setIntervalDS(self.data()?, val.days, val.hours,
                                       val.minutes, val.seconds, val.nanoseconds) }
        Ok(())
    }

    fn set_interval_ym_unchecked(&self, val: &IntervalYM) -> Result<()> {
        unsafe { dpiData_setIntervalYM(self.data()?, val.years, val.months) }
        Ok(())
    }

    fn set_bool_unchecked(&self, val: bool) -> Result<()> {
        unsafe { dpiData_setBool(self.data()?, if val { 1 } else { 0 }) }
        Ok(())
    }

    //
    // as_TYPE methods
    //

    pub fn as_i64(&self) -> Result<i64> {
        match self.native_type {
            NativeType::Int64 =>
                self.get_i64_unchecked(),
            NativeType::UInt64 =>
                Ok(self.get_u64_unchecked()?.try_into()?),
            NativeType::Float =>
                flt_to_int!(self.get_f32_unchecked()?, f32, i64),
            NativeType::Double =>
                flt_to_int!(self.get_f64_unchecked()?, f64, i64),
            NativeType::Char |
            NativeType::Number =>
                Ok(self.get_string_unchecked()?.parse()?),
            _ =>
                self.unsupported_as_type_conversion("i64"),
        }
    }

    pub fn as_u64(&self) -> Result<u64> {
        match self.native_type {
            NativeType::Int64 =>
                Ok(self.get_i64_unchecked()?.try_into()?),
            NativeType::UInt64 =>
                self.get_u64_unchecked(),
            NativeType::Float =>
                flt_to_int!(self.get_f32_unchecked()?, f32, u64),
            NativeType::Double =>
                flt_to_int!(self.get_f64_unchecked()?, f64, u64),
            NativeType::Char |
            NativeType::Number =>
                Ok(self.get_string_unchecked()?.parse()?),
            _ =>
                self.unsupported_as_type_conversion("u64"),
        }
    }

    define_fn_as_int!(as_i8, i8);
    define_fn_as_int!(as_i16, i16);
    define_fn_as_int!(as_i32, i32);
    define_fn_as_int!(as_u8, u8);
    define_fn_as_int!(as_u16, u16);
    define_fn_as_int!(as_u32, u32);

    pub fn as_f64(&self) -> Result<f64> {
        match self.native_type {
            NativeType::Int64 =>
                Ok(self.get_i64_unchecked()? as f64),
            NativeType::UInt64 =>
                Ok(self.get_u64_unchecked()? as f64),
            NativeType::Float =>
                Ok(self.get_f32_unchecked()? as f64),
            NativeType::Double =>
                self.get_f64_unchecked(),
            NativeType::Char |
            NativeType::Number =>
                Ok(self.get_string_unchecked()?.parse()?),
            _ =>
                self.unsupported_as_type_conversion("f64"),
        }
    }

    pub fn as_f32(&self) -> Result<f32> {
        match self.native_type {
            NativeType::Int64 =>
                Ok(self.get_i64_unchecked()? as f32),
            NativeType::UInt64 =>
                Ok(self.get_u64_unchecked()? as f32),
            NativeType::Float =>
                self.get_f32_unchecked(),
            NativeType::Double =>
                Ok(self.get_f64_unchecked()? as f32),
            NativeType::Char |
            NativeType::Number =>
                Ok(self.get_string_unchecked()?.parse()?),
            _ =>
                self.unsupported_as_type_conversion("f32"),
        }
    }

    pub fn as_string(&self) -> Result<String> {
        match self.native_type {
            NativeType::Int64 =>
                Ok(self.get_i64_unchecked()?.to_string()),
            NativeType::UInt64 =>
                Ok(self.get_u64_unchecked()?.to_string()),
            NativeType::Float =>
                Ok(self.get_f32_unchecked()?.to_string()),
            NativeType::Double =>
                Ok(self.get_f64_unchecked()?.to_string()),
            NativeType::Char |
            NativeType::Number =>
                self.get_string_unchecked(),
            NativeType::Timestamp =>
                Ok(self.get_timestamp_unchecked()?.to_string()),
            NativeType::IntervalDS =>
                Ok(self.get_interval_ds_unchecked()?.to_string()),
            NativeType::IntervalYM =>
                Ok(self.get_interval_ym_unchecked()?.to_string()),
            _ =>
                self.unsupported_as_type_conversion("string"),
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        match self.native_type {
            NativeType::Raw =>
                self.get_bytes_unchecked(),
            _ =>
                self.unsupported_as_type_conversion("bytes"),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self.native_type {
            NativeType::Boolean =>
                Ok(self.get_bool_unchecked()?),
            _ =>
                self.unsupported_as_type_conversion("bool"),
        }
    }

    pub fn as_timestamp(&self) -> Result<Timestamp> {
        if self.native_type == NativeType::Timestamp {
            self.get_timestamp_unchecked()
        } else {
            self.unsupported_as_type_conversion("Timestamp")
        }
    }

    pub fn as_interval_ds(&self) -> Result<IntervalDS> {
        if self.native_type == NativeType::IntervalDS {
            self.get_interval_ds_unchecked()
        } else {
            self.unsupported_as_type_conversion("IntervalDS")
        }
    }

    pub fn as_interval_ym(&self) -> Result<IntervalYM> {
        if self.native_type == NativeType::IntervalYM {
            self.get_interval_ym_unchecked()
        } else {
            self.unsupported_as_type_conversion("IntervalYM")
        }
    }

    //
    // set_TYPE methods
    //

    define_fn_set_int!(set_i8, i8);
    define_fn_set_int!(set_i16, i16);
    define_fn_set_int!(set_i32, i32);
    define_fn_set_int!(set_i64, i64);
    define_fn_set_int!(set_u8, u8);
    define_fn_set_int!(set_u16, u16);
    define_fn_set_int!(set_u32, u32);
    define_fn_set_int!(set_u64, u64);
    define_fn_set_int!(set_f64, f64);
    define_fn_set_int!(set_f32, f32);

    pub fn set_string(&mut self, val: &str) -> Result<()> {
        match self.native_type {
            NativeType::Int64 =>
                self.set_i64_unchecked(val.parse()?),
            NativeType::UInt64 =>
                self.set_u64_unchecked(val.parse()?),
            NativeType::Float =>
                self.set_f32_unchecked(val.parse()?),
            NativeType::Double =>
                self.set_f64_unchecked(val.parse()?),
            NativeType::Char =>
                Ok(self.set_string_unchecked(val)?),
            NativeType::Number => {
                if !check_number_format(val) {
                    return Err(Error::ConversionError(ConversionError::ParseError(Box::new(ParseError::new("number")))));
                }
                Ok(self.set_string_unchecked(val)?)
            },
            NativeType::Timestamp =>
                self.set_timestamp_unchecked(&val.parse()?),
            NativeType::IntervalDS =>
                self.set_interval_ds_unchecked(&val.parse()?),
            NativeType::IntervalYM =>
                self.set_interval_ym_unchecked(&val.parse()?),
            _ =>
                self.unsupported_set_type_conversion("&str"),
        }
    }

    pub fn set_bytes(&self, val: &Vec<u8>) -> Result<()> {
        match self.native_type {
            NativeType::Raw =>
                Ok(self.set_bytes_unchecked(val)?),
            _ =>
                self.unsupported_set_type_conversion("Vec<u8>"),
        }
    }

    pub fn set_timestamp(&self, val: &Timestamp) -> Result<()> {
        match self.native_type {
            NativeType::Timestamp =>
                Ok(self.set_timestamp_unchecked(val)?),
            _ =>
                self.unsupported_set_type_conversion("Timestamp"),
        }
    }

    pub fn set_interval_ds(&self, val: &IntervalDS) -> Result<()> {
        match self.native_type {
            NativeType::IntervalDS =>
                Ok(self.set_interval_ds_unchecked(val)?),
            _ =>
                self.unsupported_set_type_conversion("IntervalDS"),
        }
    }

    pub fn set_interval_ym(&self, val: &IntervalYM) -> Result<()> {
        match self.native_type {
            NativeType::IntervalYM =>
                Ok(self.set_interval_ym_unchecked(val)?),
            _ =>
                self.unsupported_set_type_conversion("IntervalYM"),
        }
    }

    pub fn set_bool(&mut self, val: bool) -> Result<()> {
        match self.native_type {
            NativeType::Boolean =>
                Ok(self.set_bool_unchecked(val)?),
            _ =>
                self.unsupported_set_type_conversion("bool"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Value({})", self.oratype)
    }
}

impl Clone for Value {
    fn clone(&self) -> Value {
        if !self.handle.is_null() {
            unsafe { dpiVar_addRef(self.handle); }
        }
        Value {
            ctxt: self.ctxt,
            handle: self.handle,
            native_type: self.native_type.clone(),
            oratype: self.oratype.clone(),
            buffer_row_index: self.buffer_row_index,
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { dpiVar_release(self.handle) };
        }
    }
}
