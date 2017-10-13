use std::fmt;
use std::ptr;
use std::slice;

use binding::*;
use types::FromSql;
use types::ToSql;
use Connection;
use error::error_from_context;
use Context;
use Error;
use Result;
use OracleType;
use Timestamp;
use IntervalDS;
use IntervalYM;

macro_rules! check_not_null {
    ($var:ident) => {
        if $var.is_null()? {
            return Err(Error::NullConversionError);
        }
    }
}

pub struct ValueRef {
    ctxt: &'static Context,
    pub(crate) handle: *mut dpiVar,
    num_data: usize,
    native_type: u32,
    oratype: OracleType,
    pub(crate) buffer_row_index: u32,
}

impl ValueRef {

    pub(crate) fn new(ctxt: &'static Context) -> ValueRef {
        ValueRef {
            ctxt: ctxt,
            handle: ptr::null_mut(),
            num_data: 0,
            native_type: 0,
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
        chkerr!(conn.ctxt,
                dpiConn_newVar(conn.handle, oratype_num, native_type, array_size, size, size_is_byte,
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

    fn invalid_type_conversion<T>(&self, to_type: &str) -> Result<T> {
        Err(Error::InvalidTypeConversion(self.oratype.to_string(), to_type.to_string()))
    }

    fn invalid_to_sql_type_conversion<T>(&self, from_type: &str) -> Result<T> {
        Err(Error::InvalidTypeConversion(from_type.to_string(), self.oratype.to_string()))
    }

    fn out_of_range<T>(&self, from_type: &str, to_type: &str) -> Result<T> {
        Err(Error::OutOfRange(from_type.to_string(), to_type.to_string()))
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

    fn get_int64_unchecked(&self) -> Result<i64> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getInt64(self.data()?)) }
    }

    fn get_uint64_unchecked(&self) -> Result<u64> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getUint64(self.data()?)) }
    }

    fn get_float_unchecked(&self) -> Result<f32> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getFloat(self.data()?)) }
    }

    fn get_double_unchecked(&self) -> Result<f64> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getDouble(self.data()?)) }
    }

    fn get_string_unchecked(&self) -> Result<String> {
        check_not_null!(self);
        unsafe {
            let bytes = dpiData_getBytes(self.data()?);
            let ptr = (*bytes).ptr as *mut u8;
            let len = (*bytes).length as usize;
            Ok(String::from_utf8_lossy(slice::from_raw_parts(ptr, len)).into_owned())
        }
    }

    fn get_bytes_unchecked(&self) -> Result<Vec<u8>> {
        check_not_null!(self);
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
        check_not_null!(self);
        unsafe {
            let ts = dpiData_getTimestamp(self.data()?);
            Ok(Timestamp::from_dpi_timestamp(&*ts))
        }
    }

    fn get_interval_ds_unchecked(&self) -> Result<IntervalDS> {
        check_not_null!(self);
        unsafe {
            let it = dpiData_getIntervalDS(self.data()?);
            Ok(IntervalDS::from_dpi_interval_ds(&*it))
        }
    }

    fn get_interval_ym_unchecked(&self) -> Result<IntervalYM> {
        check_not_null!(self);
        unsafe {
            let it = dpiData_getIntervalYM(self.data()?);
            Ok(IntervalYM::from_dpi_interval_ym(&*it))
        }
    }

    fn get_bool_unchecked(&self) -> Result<bool> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getBool(self.data()?) != 0) }
    }

    fn set_int64_unchecked(&mut self, val: i64) -> Result<()> {
        unsafe { Ok(dpiData_setInt64(self.data()?, val)) }
    }

    fn set_uint64_unchecked(&mut self, val: u64) -> Result<()> {
        unsafe { Ok(dpiData_setUint64(self.data()?, val)) }
    }

    fn set_float_unchecked(&mut self, val: f32) -> Result<()> {
        unsafe { Ok(dpiData_setFloat(self.data()?, val)) }
    }

    fn set_double_unchecked(&mut self, val: f64) -> Result<()> {
        unsafe { Ok(dpiData_setDouble(self.data()?, val)) }
    }

    pub fn as_int64(&self) -> Result<i64> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                self.get_int64_unchecked()
            },
            DPI_NATIVE_TYPE_UINT64 => {
                let n = self.get_uint64_unchecked()?;
                if n <= i64::max_value() as u64 {
                    Ok(n as i64)
                } else {
                    self.out_of_range("u64", "i64")
                }
            },
            DPI_NATIVE_TYPE_FLOAT => {
                let n = self.get_float_unchecked()?;
                if i64::min_value() as f32 <= n && n <= i64::max_value() as f32 {
                    Ok(n as i64)
                } else {
                    self.out_of_range("f32", "i64")
                }
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                let n = self.get_double_unchecked()?;
                if i64::min_value() as f64 <= n && n <= i64::max_value() as f64 {
                    Ok(n as i64)
                } else {
                    self.out_of_range("f64", "i64")
                }
            },
            DPI_NATIVE_TYPE_BYTES => {
                let s = self.get_string_unchecked()?;
                s.parse().or(self.out_of_range("string", "i64")) // TODO: map core::num::ParseIntErrorto error::Error
            },
            _ => self.invalid_type_conversion("i64"),
        }
    }

    pub fn as_uint64(&self) -> Result<u64> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                let n = self.get_int64_unchecked()?;
                if 0 <= n {
                    Ok(n as u64)
                } else {
                    self.out_of_range("i64", "u64")
                }
            },
            DPI_NATIVE_TYPE_UINT64 => {
                self.get_uint64_unchecked()
            },
            DPI_NATIVE_TYPE_FLOAT => {
                let n = self.get_float_unchecked()?;
                if 0.0f32 <= n && n <= u64::max_value() as f32 {
                    Ok(n as u64)
                } else {
                    self.out_of_range("f32", "u64")
                }
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                let n = self.get_double_unchecked()?;
                if 0.0 <= n && n <= u64::max_value() as f64 {
                    Ok(n as u64)
                } else {
                    self.out_of_range("f64", "u64")
                }
            },
            DPI_NATIVE_TYPE_BYTES => {
                let s = self.get_string_unchecked()?;
                s.parse().or(self.out_of_range("string", "u64"))
            },
            _ => self.invalid_type_conversion("u64"),
        }
    }

    pub fn as_double(&self) -> Result<f64> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                Ok(self.get_int64_unchecked()? as f64)
            },
            DPI_NATIVE_TYPE_UINT64 => {
                Ok(self.get_uint64_unchecked()? as f64)
            },
            DPI_NATIVE_TYPE_FLOAT => {
                Ok(self.get_float_unchecked()? as f64)
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                self.get_double_unchecked()
            },
            DPI_NATIVE_TYPE_BYTES => {
                let s = self.get_string_unchecked()?;
                s.parse().or(self.out_of_range("string", "f64"))
            },
            _ => self.invalid_type_conversion("f64"),
        }
    }

    pub fn as_float(&self) -> Result<f32> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                Ok(self.get_int64_unchecked()? as f32)
            },
            DPI_NATIVE_TYPE_UINT64 => {
                Ok(self.get_uint64_unchecked()? as f32)
            },
            DPI_NATIVE_TYPE_FLOAT => {
                self.get_float_unchecked()
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                Ok(self.get_double_unchecked()? as f32)
            },
            DPI_NATIVE_TYPE_BYTES => {
                let s = self.get_string_unchecked()?;
                s.parse().or(self.out_of_range("string", "f32"))
            },
            _ => self.invalid_type_conversion("f32"),
        }
    }

    pub fn as_string(&self) -> Result<String> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                Ok(self.get_int64_unchecked()?.to_string())
            },
            DPI_NATIVE_TYPE_UINT64 => {
                Ok(self.get_uint64_unchecked()?.to_string())
            },
            DPI_NATIVE_TYPE_FLOAT => {
                Ok(self.get_float_unchecked()?.to_string())
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                Ok(self.get_double_unchecked()?.to_string())
            },
            DPI_NATIVE_TYPE_BYTES => {
                self.get_string_unchecked()
            },
            _ => self.invalid_type_conversion("string"),
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        match self.native_type {
            DPI_NATIVE_TYPE_BYTES => {
                self.get_bytes_unchecked()
            },
            _ => self.invalid_type_conversion("bytes"),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self.native_type {
            DPI_NATIVE_TYPE_BOOLEAN => {
                Ok(self.get_bool_unchecked()?)
            },
            _ => {
                self.invalid_type_conversion("bool")
            },
        }
    }

    pub fn as_timestamp(&self) -> Result<Timestamp> {
        if self.native_type == DPI_NATIVE_TYPE_TIMESTAMP {
            self.get_timestamp_unchecked()
        } else {
            self.invalid_type_conversion("Timestamp")
        }
    }

    pub fn as_interval_ds(&self) -> Result<IntervalDS> {
        if self.native_type == DPI_NATIVE_TYPE_INTERVAL_DS {
            self.get_interval_ds_unchecked()
        } else {
            self.invalid_type_conversion("IntervalDS")
        }
    }

    pub fn as_interval_ym(&self) -> Result<IntervalYM> {
        if self.native_type == DPI_NATIVE_TYPE_INTERVAL_YM {
            self.get_interval_ym_unchecked()
        } else {
            self.invalid_type_conversion("IntervalYM")
        }
    }

    pub fn set_int64(&mut self, val: i64) -> Result<()> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                self.set_int64_unchecked(val)
            },
            DPI_NATIVE_TYPE_UINT64 => {
                self.set_uint64_unchecked(val as u64)
            },
            DPI_NATIVE_TYPE_FLOAT => {
                self.set_float_unchecked(val as f32)
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                self.set_double_unchecked(val as f64)
            },
            _ => self.invalid_to_sql_type_conversion("i64"),
        }
    }

    pub fn set_uint64(&mut self, val: u64) -> Result<()> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                self.set_int64_unchecked(val as i64)
            },
            DPI_NATIVE_TYPE_UINT64 => {
                self.set_uint64_unchecked(val)
            },
            DPI_NATIVE_TYPE_FLOAT => {
                self.set_float_unchecked(val as f32)
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                self.set_double_unchecked(val as f64)
            },
            _ => self.invalid_to_sql_type_conversion("u64"),
        }
    }

    pub fn set_float(&mut self, val: f32) -> Result<()> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                self.set_int64_unchecked(val as i64)
            },
            DPI_NATIVE_TYPE_UINT64 => {
                self.set_uint64_unchecked(val as u64)
            },
            DPI_NATIVE_TYPE_FLOAT => {
                self.set_float_unchecked(val)
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                self.set_double_unchecked(val as f64)
            },
            _ => self.invalid_to_sql_type_conversion("f32"),
        }
    }

    pub fn set_double(&mut self, val: f64) -> Result<()> {
        match self.native_type {
            DPI_NATIVE_TYPE_INT64 => {
                self.set_int64_unchecked(val as i64)
            },
            DPI_NATIVE_TYPE_UINT64 => {
                self.set_uint64_unchecked(val as u64)
            },
            DPI_NATIVE_TYPE_FLOAT => {
                self.set_float_unchecked(val as f32)
            },
            DPI_NATIVE_TYPE_DOUBLE => {
                self.set_double_unchecked(val)
            },
            _ => self.invalid_to_sql_type_conversion("f64"),
        }
    }
}

impl fmt::Display for ValueRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueRef({})", self.oratype)
    }
}

impl Clone for ValueRef {
    fn clone(&self) -> ValueRef {
        if !self.handle.is_null() {
            unsafe { dpiVar_addRef(self.handle); }
        }
        ValueRef {
            ctxt: self.ctxt,
            handle: self.handle,
            num_data: self.num_data,
            native_type: self.native_type,
            oratype: self.oratype.clone(),
            buffer_row_index: self.buffer_row_index,
        }
    }
}

impl Drop for ValueRef {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { dpiVar_release(self.handle) };
        }
    }
}
