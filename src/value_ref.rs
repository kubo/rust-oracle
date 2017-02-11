use super::odpi::DpiData;
use super::types::FromSql;
use super::Error;
use super::Result;
use super::OracleType;
use super::Timestamp;
use super::IntervalDS;
use super::IntervalYM;

use std::fmt;

pub struct ValueRef<'stmt> {
    data: DpiData<'stmt>,
    oratype: &'stmt OracleType,
}

impl<'stmt> ValueRef<'stmt> {
    pub fn new(data: DpiData<'stmt>, oratype: &'stmt OracleType) -> ValueRef<'stmt> {
        ValueRef {
            data: data,
            oratype: oratype,
        }
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
        self.data.is_null()
    }

    pub fn oracle_type(&self) -> &OracleType {
        self.oratype
    }

    pub fn as_int64(&self) -> Result<i64> {
        match *self.oratype {
            OracleType::BinaryFloat => {
                let n = try!(self.data.as_float());
                if i64::min_value() as f32 <= n && n <= i64::max_value() as f32 {
                    Ok(n as i64)
                } else {
                    self.out_of_range("f32", "i64")
                }
            },
            OracleType::BinaryDouble => {
                let n = try!(self.data.as_double());
                if i64::min_value() as f64 <= n && n <= i64::max_value() as f64 {
                    Ok(n as i64)
                } else {
                    self.out_of_range("f64", "i64")
                }
            },
            OracleType::Number(_,_) => {
                let s = try!(self.data.as_string());
                s.parse().or(self.out_of_range("number", "i64"))
            }
            OracleType::Int64 => {
                self.data.as_int64()
            },
            OracleType::UInt64 => {
                let n = try!(self.data.as_uint64());
                if n <= i64::max_value() as u64 {
                    Ok(n as i64)
                } else {
                    self.out_of_range("u64", "i64")
                }
            },
            _ => self.invalid_type_conversion("i64"),
        }
    }

    pub fn as_uint64(&self) -> Result<u64> {
        match *self.oratype {
            OracleType::BinaryFloat => {
                let n = try!(self.data.as_float());
                if 0.0f32 <= n && n <= u64::max_value() as f32 {
                    Ok(n as u64)
                } else {
                    self.out_of_range("f32", "u64")
                }
            },
            OracleType::BinaryDouble => {
                let n = try!(self.data.as_double());
                if 0.0 <= n && n <= u64::max_value() as f64 {
                    Ok(n as u64)
                } else {
                    self.out_of_range("f64", "u64")
                }
            },
            OracleType::Number(_,_) => {
                let s = try!(self.data.as_string());
                s.parse().or(self.out_of_range("number", "i64"))
            },
            OracleType::Int64 => {
                let n = try!(self.data.as_int64());
                if 0 <= n {
                    Ok(n as u64)
                } else {
                    self.out_of_range("i64", "u64")
                }
            },
            OracleType::UInt64 => {
                self.data.as_uint64()
            },
            _ => self.invalid_type_conversion("u64"),
        }
    }

    pub fn as_double(&self) -> Result<f64> {
        match *self.oratype {
            OracleType::BinaryFloat => {
                self.data.as_float().map(|n| n as f64)
            },
            OracleType::BinaryDouble => {
                self.data.as_double()
            },
            OracleType::Number(_,_) => {
                let s = try!(self.data.as_string());
                s.parse().or(self.out_of_range("number", "i64"))
            },
            OracleType::Int64 => {
                self.data.as_int64().map(|n| n as f64)
            },
            OracleType::UInt64 => {
                self.data.as_int64().map(|n| n as f64)
            },
            _ => self.invalid_type_conversion("f64"),
        }
    }

    pub fn as_float(&self) -> Result<f32> {
        match *self.oratype {
            OracleType::BinaryFloat => {
                self.data.as_float()
            },
            OracleType::BinaryDouble => {
                self.data.as_double().map(|n| n as f32)
            },
            OracleType::Number(_,_) => {
                let s = try!(self.data.as_string());
                s.parse().or(self.out_of_range("number", "i64"))
            },
            OracleType::Int64 => {
                self.data.as_int64().map(|n| n as f32)
            },
            OracleType::UInt64 => {
                self.data.as_int64().map(|n| n as f32)
            },
            _ => self.invalid_type_conversion("f32"),
        }
    }

    pub fn as_string(&self) -> Result<String> {
        match *self.oratype {
            OracleType::Varchar2(_) |
            OracleType::Nvarchar2(_) |
            OracleType::Char(_) |
            OracleType::NChar(_) |
            OracleType::Raw(_) |
            OracleType::Long |
            OracleType::LongRaw |
            OracleType::Number(_,_) => {
                self.data.as_string()
            },
            _ => {
                self.invalid_type_conversion("String")
            },
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match *self.oratype {
            OracleType::Boolean => {
                self.data.as_bool()
            },
            _ => {
                self.invalid_type_conversion("bool")
            },
        }
    }

    pub fn as_timestamp(&self) -> Result<Timestamp> {
        match *self.oratype {
            OracleType::Date |
            OracleType::Timestamp(_) |
            OracleType::TimestampTZ(_) |
            OracleType::TimestampLTZ(_) => {
                self.data.as_timestamp()
            },
            _ => {
                self.invalid_type_conversion("Timestamp")
            },
        }
    }

    pub fn as_interval_ds(&self) -> Result<IntervalDS> {
        match *self.oratype {
            OracleType::IntervalDS(_,_) => {
                self.data.as_interval_ds()
            },
            _ => {
                self.invalid_type_conversion("intervalDS")
            },
        }
    }

    pub fn as_interval_ym(&self) -> Result<IntervalYM> {
        match *self.oratype {
            OracleType::IntervalYM(_) => {
                self.data.as_interval_ym()
            },
            _ => {
                self.invalid_type_conversion("intervalYM")
            },
        }
    }
}

impl<'stmt> fmt::Display for ValueRef<'stmt> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueRef({})", self.oratype)
    }
}
