// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2018 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use std::cell::RefCell;
use std::convert::TryInto;
use std::fmt;
use std::ptr;
use std::rc::Rc;
use std::str;

use crate::binding::*;
use crate::chkerr;
use crate::sql_type::Collection;
use crate::sql_type::FromSql;
use crate::sql_type::IntervalDS;
use crate::sql_type::IntervalYM;
use crate::sql_type::NativeType;
use crate::sql_type::Object;
use crate::sql_type::ObjectType;
use crate::sql_type::OracleType;
use crate::sql_type::Timestamp;
use crate::sql_type::ToSql;
use crate::to_rust_slice;
use crate::to_rust_str;
use crate::util::check_number_format;
use crate::util::parse_str_into_raw;
use crate::util::set_hex_string;
use crate::Connection;
use crate::Context;
use crate::DpiConn;
use crate::Error;
use crate::Result;

macro_rules! flt_to_int {
    ($expr:expr, $src_type:ident, $dest_type:ident) => {{
        let src_val = $expr;
        if $dest_type::min_value() as $src_type <= src_val
            && src_val <= $dest_type::max_value() as $src_type
        {
            Ok(src_val as $dest_type)
        } else {
            Err(Error::OutOfRange(format!(
                "{} overflow: {}",
                stringify!($dest_type),
                src_val.to_string(),
            )))
        }
    }};
}

macro_rules! define_fn_to_int {
    ($(#[$attr:meta])* : $func_name:ident, $type:ident) => {
        $(#[$attr])*
        pub (crate)fn $func_name(&self) -> Result<$type> {
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
                NativeType::CLOB |
                NativeType::Number =>
                    Ok(self.get_string()?.parse()?),
                _ =>
                    self.invalid_conversion_to_rust_type(stringify!($type))
            }
        }
    }
}

macro_rules! define_fn_set_int {
    ($(#[$attr:meta])* : $func_name:ident, $type:ident) => {
        $(#[$attr])*
        pub(crate) fn $func_name(&mut self, val: &$type) -> Result<()> {
            match self.native_type {
                NativeType::Int64 =>
                    self.set_i64_unchecked(*val as i64),
                NativeType::UInt64 =>
                    self.set_u64_unchecked(*val as u64),
                NativeType::Float =>
                    self.set_f32_unchecked(*val as f32),
                NativeType::Double =>
                    self.set_f64_unchecked(*val as f64),
                NativeType::Char |
                NativeType::Number => {
                    let s = val.to_string();
                    self.set_string_unchecked(&s)
                },
                _ =>
                    self.invalid_conversion_from_rust_type(stringify!($type))
            }
        }
    }
}

pub enum BufferRowIndex {
    Shared(Rc<RefCell<u32>>),
    Owned(u32),
}

/// A type containing an Oracle value
///
/// When this is a column value in a select statement, the Oracle type is
/// determined by the column type.
///
/// When this is a bind value in a SQL statement, the Oracle type is determined
/// by [ToSql.oratype][].
///
/// [FromSql]: trait.FromSql.html
/// [ToSql]: trait.ToSql.html
/// [ToSql.oratype]: trait.ToSql.html#method.oratype
/// [parse]: https://doc.rust-lang.org/std/primitive.str.html#method.parse
/// [get]: #method.get
/// [set]: #method.set
/// [is_null]: #method.is_null
/// [set_null]: #method.set_null
pub struct SqlValue {
    ctxt: &'static Context,
    pub(crate) handle: *mut dpiVar,
    data: *mut dpiData,
    native_type: NativeType,
    oratype: Option<OracleType>,
    pub(crate) array_size: u32,
    pub(crate) buffer_row_index: BufferRowIndex,
    keep_bytes: Vec<u8>,
    keep_dpiobj: *mut dpiObject,
}

impl SqlValue {
    // for column and bind values
    pub(crate) fn new(ctxt: &'static Context) -> SqlValue {
        SqlValue {
            ctxt: ctxt,
            handle: ptr::null_mut(),
            data: ptr::null_mut(),
            native_type: NativeType::Int64,
            oratype: None,
            array_size: 0,
            buffer_row_index: BufferRowIndex::Owned(0),
            keep_bytes: Vec::new(),
            keep_dpiobj: ptr::null_mut(),
        }
    }

    // for object type
    pub(crate) fn from_oratype(
        ctxt: &'static Context,
        oratype: &OracleType,
        data: &mut dpiData,
    ) -> Result<SqlValue> {
        let (_, native_type, _, _) = oratype.var_create_param()?;
        Ok(SqlValue {
            ctxt: ctxt,
            handle: ptr::null_mut(),
            data: data as *mut dpiData,
            native_type: native_type,
            oratype: Some(oratype.clone()),
            array_size: 0,
            buffer_row_index: BufferRowIndex::Owned(0),
            keep_bytes: Vec::new(),
            keep_dpiobj: ptr::null_mut(),
        })
    }

    fn handle_is_reusable(&self, oratype: &OracleType, array_size: u32) -> Result<bool> {
        if self.handle.is_null() {
            return Ok(false);
        }
        if self.array_size != array_size {
            return Ok(false);
        }
        let current_oratype = match self.oratype {
            Some(ref oratype) => oratype,
            None => return Ok(false),
        };
        let (current_oratype_num, current_native_type, current_size, _) =
            current_oratype.var_create_param()?;
        let (new_oratype_num, new_native_type, new_size, _) = oratype.var_create_param()?;
        if current_oratype_num != new_oratype_num {
            return Ok(false);
        }
        match current_oratype_num {
            DPI_ORACLE_TYPE_VARCHAR
            | DPI_ORACLE_TYPE_NVARCHAR
            | DPI_ORACLE_TYPE_CHAR
            | DPI_ORACLE_TYPE_NCHAR
            | DPI_ORACLE_TYPE_RAW => Ok(current_size >= new_size),
            DPI_ORACLE_TYPE_OBJECT => Ok(current_native_type == new_native_type),
            _ => Ok(true),
        }
    }

    pub(crate) fn init_handle(
        &mut self,
        conn_handle: &DpiConn,
        oratype: &OracleType,
        array_size: u32,
    ) -> Result<bool> {
        if self.handle_is_reusable(oratype, array_size)? {
            return Ok(false);
        }
        if !self.handle.is_null() {
            unsafe { dpiVar_release(self.handle) };
        }
        self.handle = ptr::null_mut();
        let mut handle: *mut dpiVar = ptr::null_mut();
        let mut data: *mut dpiData = ptr::null_mut();
        let (oratype_num, native_type, size, size_is_byte) = oratype.var_create_param()?;
        let native_type_num = native_type.to_native_type_num();
        let object_type_handle = native_type.to_object_type_handle();
        chkerr!(
            self.ctxt,
            dpiConn_newVar(
                conn_handle.raw(),
                oratype_num,
                native_type_num,
                array_size,
                size,
                size_is_byte,
                0,
                object_type_handle,
                &mut handle,
                &mut data
            )
        );
        self.handle = handle;
        self.data = data;
        self.native_type = native_type;
        self.oratype = Some(oratype.clone());
        self.array_size = array_size;
        Ok(true)
    }

    pub(crate) fn fix_internal_data(&mut self) -> Result<()> {
        let mut num = 0;
        let mut data = ptr::null_mut();
        chkerr!(
            self.ctxt,
            dpiVar_getReturnedData(self.handle, 0, &mut num, &mut data)
        );
        if num != 0 {
            self.array_size = num;
            self.data = data;
        }
        Ok(())
    }

    fn buffer_row_index(&self) -> u32 {
        match self.buffer_row_index {
            BufferRowIndex::Shared(ref idx) => *idx.borrow(),
            BufferRowIndex::Owned(idx) => idx,
        }
    }

    fn data(&self) -> *mut dpiData {
        unsafe { self.data.offset(self.buffer_row_index() as isize) }
    }

    pub(crate) fn native_type_num(&self) -> dpiNativeTypeNum {
        self.native_type.to_native_type_num()
    }

    /// Gets the Oracle value. It internally does the followings:
    ///
    /// 1. Checks whether the conversion from the Oracle type to the target rust type
    ///    is allowed. It returns `Err(Error::InvalidTypeConversion(...))` when it
    ///    isn't allowed.
    /// 2. Checks whether the Oracle value is null. When it is null and the return
    ///    type is `Option<FromSql>`, it returns `Ok(None)`. When it is null and it
    ///    isn't `Option<FromSql>`, it returns `Err(Error::NullValue)`.
    /// 3. Converts the Oracle value to the rust value. The data type is converted
    ///    implicitly if required. For example string is converted to i64 by
    ///    [parse][] if `get::<i64>()` is called for `VARCHAR2` columns.
    ///    If the conversion fails, various errors are returned.
    pub fn get<T>(&self) -> Result<T>
    where
        T: FromSql,
    {
        <T>::from_sql(self)
    }

    /// Sets a rust value to the Oracle value. It internally does the followings:
    ///
    /// 1. Checks whether the conversion from the rust type to the target Oracle type
    ///    is allowed. It returns `Err(Error::InvalidTypeConversion(...))` when it
    ///    isn't allowed.
    /// 2. When the argument type is `None::<ToSql>`, null is set.
    /// 3. Otherwise, converts the rust value to the Oracle value. The data type
    ///    is converted implicitly if required. For example i64 is converted to
    ///    string by `to_string()` if `set(100i64)` is called for `VARCHAR2` columns.
    ///    When the argument is `None::<ToSql>`
    ///    If the conversion fails, various errors are returned.
    pub fn set(&mut self, val: &dyn ToSql) -> Result<()> {
        val.to_sql(self)
    }

    fn invalid_conversion_to_rust_type<T>(&self, to_type: &str) -> Result<T> {
        match self.oratype {
            Some(ref oratype) => Err(Error::InvalidTypeConversion(
                oratype.to_string(),
                to_type.to_string(),
            )),
            None => Err(Error::UninitializedBindValue),
        }
    }

    fn invalid_conversion_from_rust_type<T>(&self, from_type: &str) -> Result<T> {
        match self.oratype {
            Some(ref oratype) => Err(Error::InvalidTypeConversion(
                from_type.to_string(),
                oratype.to_string(),
            )),
            None => Err(Error::UninitializedBindValue),
        }
    }

    fn check_not_null(&self) -> Result<()> {
        if self.is_null()? {
            Err(Error::NullValue)
        } else {
            Ok(())
        }
    }

    /// Returns `Ok(true)` when the SQL value is null. `Ok(false)` when it isn't null.
    pub fn is_null(&self) -> Result<bool> {
        unsafe { Ok((*self.data()).isNull != 0) }
    }

    /// Sets null to the SQL value.
    pub fn set_null(&mut self) -> Result<()> {
        unsafe {
            (*self.data()).isNull = 1;
        }
        Ok(())
    }

    /// Gets the Oracle type of the SQL value.
    pub fn oracle_type(&self) -> Result<&OracleType> {
        match self.oratype {
            Some(ref oratype) => Ok(&oratype),
            None => Err(Error::UninitializedBindValue),
        }
    }

    fn get_string(&self) -> Result<String> {
        match self.native_type {
            NativeType::Char | NativeType::Number => self.get_string_unchecked(),
            NativeType::CLOB => self.get_clob_as_string_unchecked(),
            _ => self.invalid_conversion_to_rust_type("String"),
        }
    }

    //
    // get_TYPE_unchecked methods
    //

    /// Gets the SQL value as i64. The native_type must be
    /// NativeType::Int64. Otherwise, this returns unexpected value.
    fn get_i64_unchecked(&self) -> Result<i64> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getInt64(self.data())) }
    }

    /// Gets the SQL value as u64. The native_type must be
    /// NativeType::UInt64. Otherwise, this returns unexpected value.
    fn get_u64_unchecked(&self) -> Result<u64> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getUint64(self.data())) }
    }

    /// Gets the SQL value as f32. The native_type must be
    /// NativeType::Float. Otherwise, this returns unexpected value.
    fn get_f32_unchecked(&self) -> Result<f32> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getFloat(self.data())) }
    }

    /// Gets the SQL value as f64. The native_type must be
    /// NativeType::Double. Otherwise, this returns unexpected value.
    fn get_f64_unchecked(&self) -> Result<f64> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getDouble(self.data())) }
    }

    /// Gets the SQL value as utf8 string. The native_type must be
    /// NativeType::Char or NativeType::Number. Otherwise, this may cause access
    /// violation.
    fn get_string_unchecked(&self) -> Result<String> {
        self.check_not_null()?;
        unsafe {
            let bytes = dpiData_getBytes(self.data());
            Ok(to_rust_str((*bytes).ptr, (*bytes).length))
        }
    }

    /// Gets the SQL value as Vec<u8>. The native_type must be
    /// NativeType::Raw. Otherwise, this may cause access violation.
    fn get_raw_unchecked(&self) -> Result<Vec<u8>> {
        self.check_not_null()?;
        unsafe {
            let bytes = dpiData_getBytes(self.data());
            let mut vec = Vec::with_capacity((*bytes).length as usize);
            vec.extend_from_slice(to_rust_slice((*bytes).ptr, (*bytes).length));
            Ok(vec)
        }
    }

    /// Gets the SQL value as hexadecimal string. The native_type must be
    /// NativeType::Raw. Otherwise, this may cause access violation.
    fn get_raw_as_hex_string_unchecked(&self) -> Result<String> {
        self.check_not_null()?;
        unsafe {
            let bytes = dpiData_getBytes(self.data());
            let mut str = String::with_capacity(((*bytes).length * 2) as usize);
            set_hex_string(&mut str, to_rust_slice((*bytes).ptr, (*bytes).length));
            Ok(str)
        }
    }

    /// Gets the SQL value as Timestamp. The native_type must be
    /// NativeType::Timestamp. Otherwise, this returns unexpected value.
    fn get_timestamp_unchecked(&self) -> Result<Timestamp> {
        self.check_not_null()?;
        unsafe {
            let ts = dpiData_getTimestamp(self.data());
            Ok(Timestamp::from_dpi_timestamp(&*ts, self.oracle_type()?))
        }
    }

    /// Gets the SQL value as IntervalDS. The native_type must be
    /// NativeType::IntervalDS. Otherwise, this returns unexpected value.
    fn get_interval_ds_unchecked(&self) -> Result<IntervalDS> {
        self.check_not_null()?;
        unsafe {
            let it = dpiData_getIntervalDS(self.data());
            Ok(IntervalDS::from_dpi_interval_ds(&*it, self.oracle_type()?))
        }
    }

    /// Gets the SQL value as IntervalYM. The native_type must be
    /// NativeType::IntervalYM. Otherwise, this returns unexpected value.
    fn get_interval_ym_unchecked(&self) -> Result<IntervalYM> {
        self.check_not_null()?;
        unsafe {
            let it = dpiData_getIntervalYM(self.data());
            Ok(IntervalYM::from_dpi_interval_ym(&*it, self.oracle_type()?))
        }
    }

    fn get_clob_as_string_unchecked(&self) -> Result<String> {
        self.check_not_null()?;
        const READ_CHAR_SIZE: u64 = 8192;
        let lob = unsafe { dpiData_getLOB(self.data()) };
        let mut total_char_size = 0;
        let mut total_byte_size = 0;
        let mut bufsiz = 0;
        unsafe {
            dpiLob_getSize(lob, &mut total_char_size);
            dpiLob_getBufferSize(lob, total_char_size, &mut total_byte_size);
            dpiLob_getBufferSize(lob, READ_CHAR_SIZE, &mut bufsiz);
        }
        let mut result = String::with_capacity(total_byte_size as usize);
        let mut buf = vec![0u8; bufsiz as usize];
        let bufptr = buf.as_mut_ptr() as *mut i8;

        let mut offset = 1;
        while offset <= total_char_size {
            let mut read_len = bufsiz;
            chkerr!(
                self.ctxt,
                dpiLob_readBytes(lob, offset, READ_CHAR_SIZE, bufptr, &mut read_len)
            );
            result.push_str(str::from_utf8(&buf[..(read_len as usize)])?);
            offset += READ_CHAR_SIZE;
        }
        Ok(result)
    }

    fn get_blob_unchecked(&self) -> Result<Vec<u8>> {
        self.check_not_null()?;
        let lob = unsafe { dpiData_getLOB(self.data()) };
        let mut total_size = 0;
        unsafe {
            dpiLob_getSize(lob, &mut total_size);
        }
        let mut result: Vec<u8> = Vec::with_capacity(total_size as usize);
        let mut read_len = total_size;
        chkerr!(
            self.ctxt,
            dpiLob_readBytes(
                lob,
                1,
                total_size,
                result.as_mut_ptr() as *mut i8,
                &mut read_len
            )
        );
        unsafe {
            result.set_len(read_len as usize);
        }
        Ok(result)
    }

    fn get_blob_as_hex_string_unchecked(&self) -> Result<String> {
        self.check_not_null()?;
        const READ_SIZE: u64 = 8192;
        let lob = unsafe { dpiData_getLOB(self.data()) };
        let mut total_size = 0;
        unsafe {
            dpiLob_getSize(lob, &mut total_size);
        }
        let mut result = String::with_capacity((total_size * 2) as usize);
        let mut buf = vec![0u8; READ_SIZE as usize];
        let bufptr = buf.as_mut_ptr() as *mut i8;

        let mut offset = 1;
        while offset <= total_size {
            let mut read_len = READ_SIZE;
            chkerr!(
                self.ctxt,
                dpiLob_readBytes(lob, offset, READ_SIZE, bufptr, &mut read_len)
            );
            set_hex_string(&mut result, &buf[..(read_len as usize)]);
            offset += READ_SIZE;
        }
        Ok(result)
    }

    fn get_collection_unchecked(&self, objtype: &ObjectType) -> Result<Collection> {
        self.check_not_null()?;
        let dpiobj = unsafe { dpiData_getObject(self.data()) };
        chkerr!(self.ctxt, dpiObject_addRef(dpiobj));
        Ok(Collection::new(self.ctxt, dpiobj, objtype.clone()))
    }

    fn get_object_unchecked(&self, objtype: &ObjectType) -> Result<Object> {
        self.check_not_null()?;
        let dpiobj = unsafe { dpiData_getObject(self.data()) };
        chkerr!(self.ctxt, dpiObject_addRef(dpiobj));
        Ok(Object::new(self.ctxt, dpiobj, objtype.clone()))
    }

    /// Gets the SQL value as bool. The native_type must be
    /// NativeType::Boolean. Otherwise, this returns unexpected value.
    fn get_bool_unchecked(&self) -> Result<bool> {
        self.check_not_null()?;
        unsafe { Ok(dpiData_getBool(self.data()) != 0) }
    }

    //
    // set_TYPE_unchecked methods
    //

    /// Sets i64 to the SQL value. The native_type must be
    /// NativeType::Int64. Otherwise, this may cause access violation.
    fn set_i64_unchecked(&mut self, val: i64) -> Result<()> {
        unsafe { dpiData_setInt64(self.data(), val) }
        Ok(())
    }

    /// Sets u64 to the SQL value. The native_type must be
    /// NativeType::UInt64. Otherwise, this may cause access violation.
    fn set_u64_unchecked(&mut self, val: u64) -> Result<()> {
        unsafe { dpiData_setUint64(self.data(), val) }
        Ok(())
    }

    /// Sets f32 to the SQL value. The native_type must be
    /// NativeType::Float. Otherwise, this may cause access violation.
    fn set_f32_unchecked(&mut self, val: f32) -> Result<()> {
        unsafe { dpiData_setFloat(self.data(), val) }
        Ok(())
    }

    /// Sets f64 to the SQL value. The native_type must be
    /// NativeType::Double. Otherwise, this may cause access violation.
    fn set_f64_unchecked(&mut self, val: f64) -> Result<()> {
        unsafe { dpiData_setDouble(self.data(), val) }
        Ok(())
    }

    fn set_bytes_unchecked(&mut self, val: &[u8]) -> Result<()> {
        if self.handle.is_null() {
            self.keep_bytes = Vec::with_capacity(val.len());
            self.keep_bytes.extend_from_slice(val);
            unsafe {
                dpiData_setBytes(
                    self.data(),
                    self.keep_bytes.as_mut_ptr() as *mut i8,
                    val.len() as u32,
                );
            }
        } else {
            chkerr!(
                self.ctxt,
                dpiVar_setFromBytes(
                    self.handle,
                    self.buffer_row_index(),
                    val.as_ptr() as *const i8,
                    val.len() as u32
                )
            );
        }
        Ok(())
    }

    /// Sets utf8 string to the SQL value. The native_type must be
    /// NativeType::Char or NativeType::Number. Otherwise, this may cause access
    /// violation.
    fn set_string_unchecked(&mut self, val: &str) -> Result<()> {
        self.set_bytes_unchecked(val.as_bytes())
    }

    /// Sets &[u8] to the SQL value. The native_type must be
    /// NativeType::Raw. Otherwise, this may cause access violation.
    fn set_raw_unchecked(&mut self, val: &[u8]) -> Result<()> {
        self.set_bytes_unchecked(val)
    }

    /// Sets Timestamp to the SQL value. The native_type must be
    /// NativeType::Timestamp. Otherwise, this may cause access violation.
    fn set_timestamp_unchecked(&mut self, val: &Timestamp) -> Result<()> {
        unsafe {
            dpiData_setTimestamp(
                self.data(),
                val.year() as i16,
                val.month() as u8,
                val.day() as u8,
                val.hour() as u8,
                val.minute() as u8,
                val.second() as u8,
                val.nanosecond(),
                val.tz_hour_offset() as i8,
                val.tz_minute_offset() as i8,
            )
        }
        Ok(())
    }

    /// Sets IntervalDS to the SQL value. The native_type must be
    /// NativeType::IntervalDS. Otherwise, this may cause access violation.
    fn set_interval_ds_unchecked(&mut self, val: &IntervalDS) -> Result<()> {
        unsafe {
            dpiData_setIntervalDS(
                self.data(),
                val.days(),
                val.hours(),
                val.minutes(),
                val.seconds(),
                val.nanoseconds(),
            )
        }
        Ok(())
    }

    /// Sets IntervalYM to the SQL value. The native_type must be
    /// NativeType::IntervalYM. Otherwise, this may cause access violation.
    fn set_interval_ym_unchecked(&mut self, val: &IntervalYM) -> Result<()> {
        unsafe { dpiData_setIntervalYM(self.data(), val.years(), val.months()) }
        Ok(())
    }

    fn set_string_to_clob_unchecked(&mut self, val: &str) -> Result<()> {
        let ptr = val.as_ptr() as *const i8;
        let len = val.len() as u64;
        let lob = unsafe { dpiData_getLOB(self.data()) };
        chkerr!(self.ctxt, dpiLob_trim(lob, 0));
        chkerr!(self.ctxt, dpiLob_writeBytes(lob, 1, ptr, len));
        unsafe {
            (*self.data()).isNull = 0;
        }
        Ok(())
    }

    fn set_raw_to_blob_unchecked(&mut self, val: &[u8]) -> Result<()> {
        let ptr = val.as_ptr() as *const i8;
        let len = val.len() as u64;
        let lob = unsafe { dpiData_getLOB(self.data()) };
        chkerr!(self.ctxt, dpiLob_trim(lob, 0));
        chkerr!(self.ctxt, dpiLob_writeBytes(lob, 1, ptr, len));
        unsafe {
            (*self.data()).isNull = 0;
        }
        Ok(())
    }

    fn set_object_unchecked(&mut self, obj: *mut dpiObject) -> Result<()> {
        if self.handle.is_null() {
            if !self.keep_dpiobj.is_null() {
                unsafe { dpiObject_release(self.keep_dpiobj) };
            }
            unsafe {
                dpiObject_addRef(obj);
                dpiData_setObject(self.data(), obj)
            }
            self.keep_dpiobj = obj;
        } else {
            chkerr!(
                self.ctxt,
                dpiVar_setFromObject(self.handle, self.buffer_row_index(), obj)
            );
        }
        Ok(())
    }

    /// Sets bool to the SQL value. The native_type must be
    /// NativeType::Boolean. Otherwise, this may cause access violation.
    fn set_bool_unchecked(&mut self, val: bool) -> Result<()> {
        unsafe { dpiData_setBool(self.data(), if val { 1 } else { 0 }) }
        Ok(())
    }

    /// Returns a duplicated value of self.
    pub fn dup(&self, conn: &Connection) -> Result<SqlValue> {
        self.dup_by_handle(&conn.handle)
    }

    pub(crate) fn dup_by_handle(&self, conn_handle: &DpiConn) -> Result<SqlValue> {
        let mut val = SqlValue::new(self.ctxt);
        if let Some(ref oratype) = self.oratype {
            val.init_handle(conn_handle, oratype, 1)?;
            chkerr!(
                self.ctxt,
                dpiVar_copyData(val.handle, 0, self.handle, self.buffer_row_index()),
                unsafe {
                    dpiVar_release(val.handle);
                }
            );
        }
        Ok(val)
    }

    //
    // as_TYPE methods
    //

    define_fn_to_int!(
        /// Gets the SQL value as i8. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_i8, i8);
    define_fn_to_int!(
        /// Gets the SQL value as i16. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_i16, i16);
    define_fn_to_int!(
        /// Gets the SQL value as i32. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_i32, i32);
    define_fn_to_int!(
        /// Gets the SQL value as isize. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_isize, isize);

    /// Gets the SQL value as i64. The Oracle type must be
    /// numeric or string (excluding LOB) types.
    pub(crate) fn to_i64(&self) -> Result<i64> {
        match self.native_type {
            NativeType::Int64 => self.get_i64_unchecked(),
            NativeType::UInt64 => Ok(self.get_u64_unchecked()?.try_into()?),
            NativeType::Float => flt_to_int!(self.get_f32_unchecked()?, f32, i64),
            NativeType::Double => flt_to_int!(self.get_f64_unchecked()?, f64, i64),
            NativeType::Char | NativeType::CLOB | NativeType::Number => {
                Ok(self.get_string()?.parse()?)
            }
            _ => self.invalid_conversion_to_rust_type("i64"),
        }
    }

    define_fn_to_int!(
        /// Gets the SQL value as u8. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_u8, u8);
    define_fn_to_int!(
        /// Gets the SQL value as u16. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_u16, u16);
    define_fn_to_int!(
        /// Gets the SQL value as u32. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_u32, u32);
    define_fn_to_int!(
        /// Gets the SQL value as usize. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : to_usize, usize);

    /// Gets the SQL value as u64. The Oracle type must be
    /// numeric or string (excluding LOB) types.
    pub(crate) fn to_u64(&self) -> Result<u64> {
        match self.native_type {
            NativeType::Int64 => Ok(self.get_i64_unchecked()?.try_into()?),
            NativeType::UInt64 => self.get_u64_unchecked(),
            NativeType::Float => flt_to_int!(self.get_f32_unchecked()?, f32, u64),
            NativeType::Double => flt_to_int!(self.get_f64_unchecked()?, f64, u64),
            NativeType::Char | NativeType::CLOB | NativeType::Number => {
                Ok(self.get_string()?.parse()?)
            }
            _ => self.invalid_conversion_to_rust_type("u64"),
        }
    }

    /// Gets the SQL value as f32. The Oracle type must be
    /// numeric or string (excluding LOB) types.
    pub(crate) fn to_f32(&self) -> Result<f32> {
        match self.native_type {
            NativeType::Int64 => Ok(self.get_i64_unchecked()? as f32),
            NativeType::UInt64 => Ok(self.get_u64_unchecked()? as f32),
            NativeType::Float => self.get_f32_unchecked(),
            NativeType::Double => Ok(self.get_f64_unchecked()? as f32),
            NativeType::Char | NativeType::CLOB | NativeType::Number => {
                Ok(self.get_string()?.parse()?)
            }
            _ => self.invalid_conversion_to_rust_type("f32"),
        }
    }

    /// Gets the SQL value as f64. The Oracle type must be
    /// numeric or string (excluding LOB) types.
    pub(crate) fn to_f64(&self) -> Result<f64> {
        match self.native_type {
            NativeType::Int64 => Ok(self.get_i64_unchecked()? as f64),
            NativeType::UInt64 => Ok(self.get_u64_unchecked()? as f64),
            NativeType::Float => Ok(self.get_f32_unchecked()? as f64),
            NativeType::Double => self.get_f64_unchecked(),
            NativeType::Char | NativeType::CLOB | NativeType::Number => {
                Ok(self.get_string()?.parse()?)
            }
            _ => self.invalid_conversion_to_rust_type("f64"),
        }
    }

    /// Gets the SQL value as string. ...
    pub(crate) fn to_string(&self) -> Result<String> {
        match self.native_type {
            NativeType::Int64 => Ok(self.get_i64_unchecked()?.to_string()),
            NativeType::UInt64 => Ok(self.get_u64_unchecked()?.to_string()),
            NativeType::Float => Ok(self.get_f32_unchecked()?.to_string()),
            NativeType::Double => Ok(self.get_f64_unchecked()?.to_string()),
            NativeType::Char | NativeType::Number => self.get_string_unchecked(),
            NativeType::Raw => self.get_raw_as_hex_string_unchecked(),
            NativeType::Timestamp => Ok(self.get_timestamp_unchecked()?.to_string()),
            NativeType::IntervalDS => Ok(self.get_interval_ds_unchecked()?.to_string()),
            NativeType::IntervalYM => Ok(self.get_interval_ym_unchecked()?.to_string()),
            NativeType::CLOB => self.get_clob_as_string_unchecked(),
            NativeType::BLOB => self.get_blob_as_hex_string_unchecked(),
            NativeType::Object(ref objtype) => {
                if objtype.is_collection() {
                    Ok(self.get_collection_unchecked(objtype)?.to_string())
                } else {
                    Ok(self.get_object_unchecked(objtype)?.to_string())
                }
            }
            _ => self.invalid_conversion_to_rust_type("string"),
        }
    }

    /// Gets the SQL value as Vec\<u8>. ...
    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>> {
        match self.native_type {
            NativeType::Raw => self.get_raw_unchecked(),
            NativeType::BLOB => self.get_blob_unchecked(),
            NativeType::Char | NativeType::CLOB => Ok(parse_str_into_raw(&self.get_string()?)?),
            _ => self.invalid_conversion_to_rust_type("raw"),
        }
    }

    /// Gets the SQL value as Timestamp. The Oracle type must be
    /// `DATE`, `TIMESTAMP`, or `TIMESTAMP WITH TIME ZONE`.
    pub(crate) fn to_timestamp(&self) -> Result<Timestamp> {
        match self.native_type {
            NativeType::Timestamp => self.get_timestamp_unchecked(),
            NativeType::Char | NativeType::CLOB => Ok(self.get_string()?.parse()?),
            _ => self.invalid_conversion_to_rust_type("Timestamp"),
        }
    }

    /// Gets the SQL value as IntervalDS. The Oracle type must be
    /// `INTERVAL DAY TO SECOND`.
    pub(crate) fn to_interval_ds(&self) -> Result<IntervalDS> {
        match self.native_type {
            NativeType::IntervalDS => self.get_interval_ds_unchecked(),
            NativeType::Char | NativeType::CLOB => Ok(self.get_string()?.parse()?),
            _ => self.invalid_conversion_to_rust_type("IntervalDS"),
        }
    }

    /// Gets the SQL value as IntervalYM. The Oracle type must be
    /// `INTERVAL YEAR TO MONTH`.
    pub(crate) fn to_interval_ym(&self) -> Result<IntervalYM> {
        match self.native_type {
            NativeType::IntervalYM => self.get_interval_ym_unchecked(),
            NativeType::Char | NativeType::CLOB => Ok(self.get_string()?.parse()?),
            _ => self.invalid_conversion_to_rust_type("IntervalYM"),
        }
    }

    pub(crate) fn to_collection(&self) -> Result<Collection> {
        match self.native_type {
            NativeType::Object(ref objtype) => {
                if objtype.is_collection() {
                    self.get_collection_unchecked(objtype)
                } else {
                    self.invalid_conversion_to_rust_type("Collection")
                }
            }
            _ => self.invalid_conversion_to_rust_type("Collection"),
        }
    }

    pub(crate) fn to_object(&self) -> Result<Object> {
        match self.native_type {
            NativeType::Object(ref objtype) => {
                if !objtype.is_collection() {
                    self.get_object_unchecked(objtype)
                } else {
                    self.invalid_conversion_to_rust_type("Object")
                }
            }
            _ => self.invalid_conversion_to_rust_type("Object"),
        }
    }

    /// Gets the SQL value as bool. The Oracle type must be
    /// `BOOLEAN`(PL/SQL only).
    pub(crate) fn to_bool(&self) -> Result<bool> {
        match self.native_type {
            NativeType::Boolean => self.get_bool_unchecked(),
            _ => self.invalid_conversion_to_rust_type("bool"),
        }
    }

    //
    // set_TYPE methods
    //

    define_fn_set_int!(
        /// Sets i8 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_i8, i8);
    define_fn_set_int!(
        /// Sets i16 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_i16, i16);
    define_fn_set_int!(
        /// Sets i32 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_i32, i32);
    define_fn_set_int!(
        /// Sets i64 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_i64, i64);
    define_fn_set_int!(
        /// Sets isize to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_isize, isize);
    define_fn_set_int!(
        /// Sets u8 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_u8, u8);
    define_fn_set_int!(
        /// Sets u16 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_u16, u16);
    define_fn_set_int!(
        /// Sets u32 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_u32, u32);
    define_fn_set_int!(
        /// Sets u64 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_u64, u64);
    define_fn_set_int!(
        /// Sets usize to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_usize, usize);
    define_fn_set_int!(
        /// Sets f32 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_f32, f32);
    define_fn_set_int!(
        /// Sets f64 to the SQL value. The Oracle type must be
        /// numeric or string (excluding LOB) types.
        : set_f64, f64);

    /// Sets &str to the SQL value. ...
    pub(crate) fn set_string(&mut self, val: &str) -> Result<()> {
        match self.native_type {
            NativeType::Int64 => self.set_i64_unchecked(val.parse()?),
            NativeType::UInt64 => self.set_u64_unchecked(val.parse()?),
            NativeType::Float => self.set_f32_unchecked(val.parse()?),
            NativeType::Double => self.set_f64_unchecked(val.parse()?),
            NativeType::Char => self.set_string_unchecked(val),
            NativeType::Number => {
                check_number_format(val)?;
                self.set_string_unchecked(val)
            }
            NativeType::Raw => self.set_raw_unchecked(&parse_str_into_raw(val)?),
            NativeType::Timestamp => self.set_timestamp_unchecked(&val.parse()?),
            NativeType::IntervalDS => self.set_interval_ds_unchecked(&val.parse()?),
            NativeType::IntervalYM => self.set_interval_ym_unchecked(&val.parse()?),
            NativeType::CLOB => self.set_string_to_clob_unchecked(val),
            NativeType::BLOB => self.set_raw_to_blob_unchecked(&parse_str_into_raw(val)?),
            _ => self.invalid_conversion_from_rust_type("&str"),
        }
    }

    /// Sets &[u8] to the SQL value. ...
    pub(crate) fn set_bytes(&mut self, val: &[u8]) -> Result<()> {
        match self.native_type {
            NativeType::Raw => self.set_raw_unchecked(val),
            NativeType::BLOB => self.set_raw_to_blob_unchecked(val),
            _ => self.invalid_conversion_from_rust_type("&[u8]"),
        }
    }

    /// Sets Timestamp to the SQL value. The Oracle type must be
    /// `DATE`, `TIMESTAMP`, or `TIMESTAMP WITH TIME ZONE`.
    pub(crate) fn set_timestamp(&mut self, val: &Timestamp) -> Result<()> {
        match self.native_type {
            NativeType::Timestamp => self.set_timestamp_unchecked(val),
            _ => self.invalid_conversion_from_rust_type("Timestamp"),
        }
    }

    /// Sets IntervalDS to the SQL value. The Oracle type must be
    /// `INTERVAL DAY TO SECOND`.
    pub(crate) fn set_interval_ds(&mut self, val: &IntervalDS) -> Result<()> {
        match self.native_type {
            NativeType::IntervalDS => self.set_interval_ds_unchecked(val),
            _ => self.invalid_conversion_from_rust_type("IntervalDS"),
        }
    }

    /// Sets IntervalYM to the SQL value. The Oracle type must be
    /// `INTERVAL YEAR TO MONTH`.
    pub(crate) fn set_interval_ym(&mut self, val: &IntervalYM) -> Result<()> {
        match self.native_type {
            NativeType::IntervalYM => self.set_interval_ym_unchecked(val),
            _ => self.invalid_conversion_from_rust_type("IntervalYM"),
        }
    }

    /// Sets Object to the Sql Value
    pub(crate) fn set_object(&mut self, val: &Object) -> Result<()> {
        match self.native_type {
            NativeType::Object(_) => return self.set_object_unchecked(val.handle),
            _ => self.invalid_conversion_from_rust_type("Object"),
        }
    }

    /// Sets Collection to the Sql Value
    pub(crate) fn set_collection(&mut self, val: &Collection) -> Result<()> {
        match self.native_type {
            NativeType::Object(_) => return self.set_object_unchecked(val.handle),
            _ => self.invalid_conversion_from_rust_type("Collection"),
        }
    }

    /// Sets boolean to the SQL value. The Oracle type must be
    /// `BOOLEAN`(PL/SQL only).
    pub(crate) fn set_bool(&mut self, val: &bool) -> Result<()> {
        match self.native_type {
            NativeType::Boolean => self.set_bool_unchecked(*val),
            _ => self.invalid_conversion_from_rust_type("bool"),
        }
    }

    /// The cloned value must not live longer than self.
    /// Otherwise it may cause access violation.
    pub(crate) fn unsafely_clone(&self) -> SqlValue {
        SqlValue {
            ctxt: self.ctxt,
            handle: ptr::null_mut(),
            data: self.data,
            native_type: self.native_type.clone(),
            oratype: self.oratype.clone(),
            array_size: self.array_size,
            buffer_row_index: BufferRowIndex::Owned(0),
            keep_bytes: Vec::new(),
            keep_dpiobj: ptr::null_mut(),
        }
    }
}

impl fmt::Display for SqlValue {
    /// Formats any SQL value to string using the given formatter.
    /// Note that both a null value and a string `NULL` are formatted
    /// as `NULL`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.oratype.is_some() {
            match self.to_string() {
                Ok(s) => write!(f, "{}", s),
                Err(Error::NullValue) => write!(f, "NULL"),
                Err(err) => write!(f, "{}", err),
            }
        } else {
            write!(f, "uninitialized SQL value")
        }
    }
}

impl fmt::Debug for SqlValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref oratype) = self.oratype {
            write!(f, "SqlValue {{ val: ")?;
            match self.to_string() {
                Ok(s) => match self.native_type {
                    NativeType::Char | NativeType::Raw | NativeType::CLOB | NativeType::BLOB => {
                        write!(f, "{:?}", s)
                    }
                    _ => write!(f, "{}", s),
                },
                Err(Error::NullValue) => write!(f, "NULL"),
                Err(err) => write!(f, "{}", err),
            }?;
            write!(
                f,
                ", type: {}, idx/size: {}/{})",
                oratype,
                self.buffer_row_index(),
                self.array_size
            )
        } else {
            write!(f, "SqlValue {{ uninitialized }}")
        }
    }
}

impl Drop for SqlValue {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { dpiVar_release(self.handle) };
        }
        if !self.keep_dpiobj.is_null() {
            unsafe { dpiObject_release(self.keep_dpiobj) };
        }
    }
}
