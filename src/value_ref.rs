use std::fmt;
use std::slice;

use binding::*;
use types::FromSql;
use Error;
use Result;
use OracleType;
use Timestamp;
use IntervalDS;
use IntervalYM;

macro_rules! check_not_null {
    ($var:ident) => {
        if $var.is_null() {
            return Err(Error::NullConversionError);
        }
    }
}

pub struct ValueRef<'stmt> {
    data: *mut dpiData,
    native_type: u32,
    oratype: &'stmt OracleType,
}

impl<'stmt> ValueRef<'stmt> {
    pub(crate) fn new(data: *mut dpiData, native_type: u32, oratype: &'stmt OracleType) -> Result<ValueRef<'stmt>> {
        Ok(ValueRef {
            data: data,
            native_type: native_type,
            oratype: oratype,
        })
    }

    pub fn get<T>(&self) -> Result<T> where T: FromSql {
        <T>::from(self)
    }

    fn invalid_type_conversion<T>(&self, to_type: &str) -> Result<T> {
        Err(Error::InvalidTypeConversion(self.oratype.to_string(), to_type.to_string()))
    }

    fn out_of_range<T>(&self, from_type: &str, to_type: &str) -> Result<T> {
        Err(Error::OutOfRange(from_type.to_string(), to_type.to_string()))
    }

    pub fn is_null(&self) -> bool {
        unsafe {
            (&*self.data).isNull != 0
        }
    }

    pub fn oracle_type(&self) -> &OracleType {
        self.oratype
    }

    fn get_int64_unchecked(&self) -> Result<i64> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getInt64(self.data)) }
    }

    fn get_uint64_unchecked(&self) -> Result<u64> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getUint64(self.data)) }
    }

    fn get_float_unchecked(&self) -> Result<f32> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getFloat(self.data)) }
    }

    fn get_double_unchecked(&self) -> Result<f64> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getDouble(self.data)) }
    }

    fn get_string_unchecked(&self) -> Result<String> {
        check_not_null!(self);
        unsafe {
            let bytes = dpiData_getBytes(self.data);
            let ptr = (*bytes).ptr as *mut u8;
            let len = (*bytes).length as usize;
            Ok(String::from_utf8_lossy(slice::from_raw_parts(ptr, len)).into_owned())
        }
    }

    fn get_bytes_unchecked(&self) -> Result<Vec<u8>> {
        check_not_null!(self);
        unsafe {
            let bytes = dpiData_getBytes(self.data);
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
            let ts = dpiData_getTimestamp(self.data);
            Ok(Timestamp::from_dpi_timestamp(&*ts))
        }
    }

    fn get_interval_ds_unchecked(&self) -> Result<IntervalDS> {
        check_not_null!(self);
        unsafe {
            let it = dpiData_getIntervalDS(self.data);
            Ok(IntervalDS::from_dpi_interval_ds(&*it))
        }
    }

    fn get_interval_ym_unchecked(&self) -> Result<IntervalYM> {
        check_not_null!(self);
        unsafe {
            let it = dpiData_getIntervalYM(self.data);
            Ok(IntervalYM::from_dpi_interval_ym(&*it))
        }
    }

    fn get_bool_unchecked(&self) -> Result<bool> {
        check_not_null!(self);
        unsafe { Ok(dpiData_getBool(self.data) != 0) }
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
                s.parse().or(self.out_of_range("string", "i64"))
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
}

impl<'stmt> fmt::Display for ValueRef<'stmt> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueRef({})", self.oratype)
    }
}
