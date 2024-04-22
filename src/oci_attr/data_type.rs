// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2021 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

//! The module defines types related to the associate type [`OciAttr::DataType`].
use crate::binding::*;
use crate::chkerr;
#[cfg(doc)]
use crate::oci_attr::OciAttr;
use crate::to_rust_str;
use crate::Connection;
use crate::Context;
use crate::Error;
use crate::Result;
use crate::Statement;
use std::borrow::ToOwned;
use std::convert::TryInto;
use std::mem;
use std::os::raw::c_void;
use std::slice;
use std::time::Duration;

enum Handle {
    Conn(*mut dpiConn, u32),
    Stmt(*mut dpiStmt),
}

/// Attribute value used in [`DataType`]. You have no need to use this except implementing [`DataType`] for your type.
pub struct AttrValue {
    ctxt: Context,
    handle: Handle,
    attr_num: u32,
}

impl AttrValue {
    pub(crate) fn from_conn(conn: &Connection, handle_type: u32, attr_num: u32) -> AttrValue {
        AttrValue {
            ctxt: conn.ctxt().clone(),
            handle: Handle::Conn(conn.handle(), handle_type),
            attr_num,
        }
    }

    pub(crate) fn from_stmt(stmt: &Statement, attr_num: u32) -> AttrValue {
        AttrValue {
            ctxt: stmt.ctxt().clone(),
            handle: Handle::Stmt(stmt.handle()),
            attr_num,
        }
    }

    fn ctxt(&self) -> &Context {
        &self.ctxt
    }

    fn set_ptr_len(&mut self, ptr: *mut c_void, len: u32) -> Result<()> {
        match &self.handle {
            Handle::Conn(handle, handle_type) => {
                chkerr!(
                    self.ctxt(),
                    dpiConn_setOciAttr(*handle, *handle_type, self.attr_num, ptr, len)
                );
            }
            Handle::Stmt(handle) => {
                chkerr!(
                    self.ctxt(),
                    dpiStmt_setOciAttr(*handle, self.attr_num, ptr, len)
                );
            }
        }
        Ok(())
    }

    /// Sets a value to the attribute.
    ///
    /// Note that if incorrect type parameter `T` is specified,
    /// its behavior is undefined. It may cause an access violation.
    pub unsafe fn set<T>(&mut self, val: &T::Type) -> Result<()>
    where
        T: DataType,
    {
        <T>::set(self, val)
    }

    unsafe fn set_ub1(&mut self, val: u8) -> Result<()> {
        self.set_ptr_len(&val as *const u8 as *mut c_void, 1)
    }

    unsafe fn set_ub2(&mut self, val: u16) -> Result<()> {
        self.set_ptr_len(&val as *const u16 as *mut c_void, 2)
    }

    unsafe fn set_ub4(&mut self, val: u32) -> Result<()> {
        self.set_ptr_len(&val as *const u32 as *mut c_void, 4)
    }

    unsafe fn set_ub8(&mut self, val: u64) -> Result<()> {
        self.set_ptr_len(&val as *const u64 as *mut c_void, 8)
    }

    unsafe fn set_bool(&mut self, val: bool) -> Result<()> {
        let val = i32::from(val);
        self.set_ptr_len(&val as *const i32 as *mut c_void, 4)
    }

    unsafe fn set_pointer(&mut self, val: *mut c_void) -> Result<()> {
        self.set_ptr_len(
            &val as *const *mut c_void as *mut c_void,
            mem::size_of::<*mut c_void>() as u32,
        )
    }

    unsafe fn set_string(&mut self, val: &str) -> Result<()> {
        let mut vec = Vec::with_capacity(val.len() + 1);
        vec.extend_from_slice(val.as_bytes());
        vec.push(0);
        self.set_ptr_len(vec.as_ptr() as *mut c_void, val.len() as u32)
    }

    unsafe fn set_binary(&mut self, val: &[u8]) -> Result<()> {
        self.set_ptr_len(val.as_ptr() as *mut c_void, val.len() as u32)
    }

    fn get_data_buffer(&self) -> Result<(dpiDataBuffer, u32)> {
        let mut buf = unsafe { mem::zeroed() };
        let mut len = 0;
        match &self.handle {
            Handle::Conn(handle, handle_type) => {
                chkerr!(
                    self.ctxt(),
                    dpiConn_getOciAttr(*handle, *handle_type, self.attr_num, &mut buf, &mut len)
                );
            }
            Handle::Stmt(handle) => {
                chkerr!(
                    self.ctxt(),
                    dpiStmt_getOciAttr(*handle, self.attr_num, &mut buf, &mut len)
                );
            }
        }
        Ok((buf, len))
    }

    /// Gets a value from the attribute.
    ///
    /// Note that if incorrect type parameter `T` is specified,
    /// its behavior is undefined. It may cause an access violation.
    pub unsafe fn get<T>(self) -> Result<<T::Type as ToOwned>::Owned>
    where
        T: DataType,
    {
        <T>::get(self)
    }

    unsafe fn get_ub1(&self) -> Result<u8> {
        Ok(self.get_data_buffer()?.0.asUint8)
    }

    unsafe fn get_ub2(&self) -> Result<u16> {
        Ok(self.get_data_buffer()?.0.asUint16)
    }

    unsafe fn get_ub4(&self) -> Result<u32> {
        Ok(self.get_data_buffer()?.0.asUint32)
    }

    unsafe fn get_ub8(&self) -> Result<u64> {
        Ok(self.get_data_buffer()?.0.asUint64)
    }

    unsafe fn get_bool(&self) -> Result<bool> {
        Ok(self.get_data_buffer()?.0.asBoolean != 0)
    }

    unsafe fn get_pointer(&self) -> Result<*mut c_void> {
        Ok(self.get_data_buffer()?.0.asRaw)
    }

    unsafe fn get_string(&self) -> Result<String> {
        let val = self.get_data_buffer()?;
        Ok(to_rust_str(val.0.asString, val.1))
    }

    unsafe fn get_binary(&self) -> Result<Vec<u8>> {
        let val = self.get_data_buffer()?;
        let s = slice::from_raw_parts(val.0.asRaw as *const u8, val.1.try_into()?);
        Ok(s.to_vec())
    }
}

/// A trait to get and set OCI attributes as rust types. You have no need to use this except implementing [`OciAttr`] for your type.
pub unsafe trait DataType {
    type Type: ToOwned + ?Sized;

    unsafe fn set(attr: &mut AttrValue, val: &Self::Type) -> Result<()>;
    unsafe fn get(attr: AttrValue) -> Result<<Self::Type as ToOwned>::Owned>;
}

unsafe impl DataType for u8 {
    type Type = u8;

    unsafe fn set(attr: &mut AttrValue, val: &u8) -> Result<()> {
        attr.set_ub1(*val)
    }

    unsafe fn get(attr: AttrValue) -> Result<u8> {
        attr.get_ub1()
    }
}

unsafe impl DataType for u16 {
    type Type = u16;

    unsafe fn set(attr: &mut AttrValue, val: &u16) -> Result<()> {
        attr.set_ub2(*val)
    }

    unsafe fn get(attr: AttrValue) -> Result<u16> {
        attr.get_ub2()
    }
}

unsafe impl DataType for u32 {
    type Type = u32;

    unsafe fn set(attr: &mut AttrValue, val: &u32) -> Result<()> {
        attr.set_ub4(*val)
    }

    unsafe fn get(attr: AttrValue) -> Result<u32> {
        attr.get_ub4()
    }
}

unsafe impl DataType for u64 {
    type Type = u64;

    unsafe fn set(attr: &mut AttrValue, val: &u64) -> Result<()> {
        attr.set_ub8(*val)
    }

    unsafe fn get(attr: AttrValue) -> Result<u64> {
        attr.get_ub8()
    }
}

unsafe impl DataType for bool {
    type Type = bool;

    unsafe fn set(attr: &mut AttrValue, val: &bool) -> Result<()> {
        attr.set_bool(*val)
    }

    unsafe fn get(attr: AttrValue) -> Result<bool> {
        attr.get_bool()
    }
}

unsafe impl DataType for *mut c_void {
    type Type = *mut c_void;

    unsafe fn set(attr: &mut AttrValue, val: &*mut c_void) -> Result<()> {
        attr.set_pointer(*val)
    }

    unsafe fn get(attr: AttrValue) -> Result<*mut c_void> {
        attr.get_pointer()
    }
}

unsafe impl DataType for str {
    type Type = str;

    unsafe fn set(attr: &mut AttrValue, val: &str) -> Result<()> {
        attr.set_string(val)
    }

    unsafe fn get(attr: AttrValue) -> Result<String> {
        attr.get_string()
    }
}

unsafe impl DataType for [u8] {
    type Type = [u8];

    unsafe fn set(attr: &mut AttrValue, val: &[u8]) -> Result<()> {
        attr.set_binary(val)
    }

    unsafe fn get(attr: AttrValue) -> Result<Vec<u8>> {
        attr.get_binary()
    }
}

/// A type to get and set u64 microsecond attribute values as [`Duration`].
///
/// This is a data type for [`CallTime`](super::CallTime`).
pub struct DurationUsecU64 {
    _unused: [usize; 0],
}

unsafe impl DataType for DurationUsecU64 {
    type Type = Duration;

    unsafe fn set(_attr: &mut AttrValue, _val: &Duration) -> Result<()> {
        // OCI_ATTR_CALL_TIME is read-only attribute.
        // No need to implement `set`.
        //
        // Here is an example when `set` is required to be implemented.
        //
        //     let usecs = val.as_micros().try_into()?;
        //     if usecs == 0 && !val.is_zero() {
        //         return Err(Error::OutOfRange(format!("too short duration: {:?}", val)));
        //     }
        //     attr.set::<u64>(&usecs) // set microseconds as u64
        unimplemented!()
    }

    unsafe fn get(attr: AttrValue) -> Result<Duration> {
        let val = attr.get::<u64>()?;
        Ok(Duration::from_micros(val))
    }
}

/// A type corresponding to the `init.ora` parameter [`MAX_STRING_SIZE`]
///
/// This is a data type for [`CallTime`](super::VarTypeMaxLenCompat`).
///
/// [`MAX_STRING_SIZE`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-D424D23B-0933-425F-BC69-9C0E6724693C
#[derive(Debug, Clone)]
pub enum MaxStringSize {
    /// The maximum size is 4000 bytes for `VARCHAR2` and `NVARCHAR`, 2000 bytes for `RAW`.
    Standard,
    /// The maximum size is 32767 bytes for `VARCHAR2`, `NVARCHAR` and `RAW`.
    Extended,
}

unsafe impl DataType for MaxStringSize {
    type Type = MaxStringSize;

    unsafe fn set(_attr: &mut AttrValue, _val: &MaxStringSize) -> Result<()> {
        // OCI_ATTR_VARTYPE_MAXLEN_COMPAT is read-only attribute.
        // No need to implement `set`.
        unimplemented!()
    }

    unsafe fn get(attr: AttrValue) -> Result<MaxStringSize> {
        // Constants defined in `oci.h` included in Oracle Instant Client SDK
        const OCI_ATTR_MAXLEN_COMPAT_STANDARD: u8 = 1;
        const OCI_ATTR_MAXLEN_COMPAT_EXTENDED: u8 = 2;

        match attr.get::<u8>()? {
            OCI_ATTR_MAXLEN_COMPAT_STANDARD => Ok(MaxStringSize::Standard),
            OCI_ATTR_MAXLEN_COMPAT_EXTENDED => Ok(MaxStringSize::Extended),
            x => Err(Error::internal_error(format!(
                "invalid MaxStringSize {}",
                x
            ))),
        }
    }
}
