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

use std::cmp;
use std::fmt;
use std::mem::{self, MaybeUninit};
use std::os::raw::c_char;
use std::sync::Arc;

use crate::binding::*;
use crate::chkerr;
use crate::connection::Conn;
use crate::sql_type::collection::{Indices, Iter, Values};
use crate::sql_type::FromSql;
use crate::sql_type::OracleType;
use crate::sql_type::ToSql;
use crate::to_rust_str;
use crate::util::write_literal;
use crate::AssertSend;
use crate::Connection;
use crate::Context;
use crate::DpiObject;
use crate::DpiObjectAttr;
use crate::DpiObjectType;
use crate::Error;
use crate::Result;
use crate::SqlValue;

unsafe fn release_dpi_data(data: &dpiData, native_type_num: u32) {
    if data.isNull == 0 {
        match native_type_num {
            DPI_NATIVE_TYPE_LOB => {
                dpiLob_release(data.value.asLOB);
            }
            DPI_NATIVE_TYPE_OBJECT => {
                dpiObject_release(data.value.asObject);
            }
            DPI_NATIVE_TYPE_ROWID => {
                dpiRowid_release(data.value.asRowid);
            }
            _ => (),
        }
    }
}

/// Oracle-specific collection data type
///
/// This type corresponds to varray and nested table data types.
/// See [Oracle manual](https://docs.oracle.com/database/122/ADOBJ/collection-data-types.htm).
///
/// ```no_run
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
///
/// // MDSYS.SDO_ELEM_INFO_ARRAY is defined as VARRAY (1048576) of NUMBER.
/// let objtype = conn.object_type("MDSYS.SDO_ELEM_INFO_ARRAY")?;
///
/// // Create a new collection
/// let mut obj = objtype.new_collection()?;
/// obj.push(&1);
/// obj.push(&3);
/// assert_eq!(obj.to_string(), "MDSYS.SDO_ELEM_INFO_ARRAY(1, 3)");
/// # Ok::<(), Error>(())
/// ```
///
/// Note: Methods in the type may be changed in future.
pub struct Collection {
    conn: Conn,
    pub(crate) handle: DpiObject,
    objtype: ObjectType,
}

impl Collection {
    pub(crate) fn new(conn: Conn, handle: DpiObject, objtype: ObjectType) -> Collection {
        Collection {
            conn,
            handle,
            objtype,
        }
    }

    pub(crate) fn ctxt(&self) -> &Context {
        self.conn.ctxt()
    }

    fn handle(&self) -> *mut dpiObject {
        self.handle.raw
    }

    /// Returns type information.
    pub fn object_type(&self) -> &ObjectType {
        &self.objtype
    }

    /// Returns the number of elements.
    ///
    /// This counts also deleted elements. See "Comments" about [OCICollSize()][].
    ///
    /// [OCICollSize()]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-B8F6665F-12F1-43DB-A27E-82A2A655D701
    pub fn size(&self) -> Result<i32> {
        let mut size = 0;
        chkerr!(self.ctxt(), dpiObject_getSize(self.handle(), &mut size));
        Ok(size)
    }

    /// Returns an iterator visiting all values with indices in the collection.
    ///
    /// ```
    /// # use oracle::{Error, Result};
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// // Creates VARRAY type and gets the type information
    /// conn.execute("create or replace type string_varray is varray(20) of varchar2(60)", &[])?;
    /// let objtype = conn.object_type("STRING_VARRAY")?;
    ///
    /// // Creates a VARRAY instance and appends three elements.
    /// let mut coll = objtype.new_collection()?;
    /// coll.push(&"First Element")?;
    /// coll.push(&"Second Element")?;
    /// coll.push(&"Third Element")?;
    ///
    /// let vec = coll
    ///     .iter::<String>() // iterator returning Result<(i32, String)>
    ///     .collect::<Result<Vec<_>>>()?;
    /// assert_eq!(vec[0], (0, "First Element".to_string()));
    /// assert_eq!(vec[1], (1, "Second Element".to_string()));
    /// assert_eq!(vec[2], (2, "Third Element".to_string()));
    ///
    /// // Creates Table type and gets the type information
    /// conn.execute("create or replace type string_table is table of varchar2(60)", &[])?;
    /// let objtype = conn.object_type("STRING_TABLE")?;
    ///
    /// // Creates a TABLE instance, appends four elements and makes a hole.
    /// let mut coll = objtype.new_collection()?;
    /// coll.push(&"First Element")?;
    /// coll.push(&"Second Element")?;
    /// coll.push(&"Third Element")?;
    /// coll.push(&"Fourth Element")?;
    /// coll.remove(2)?; // Remove "Third Element"
    ///
    /// // iterator returning Result<(i32, String)>
    /// let mut iter = coll.iter::<String>();
    /// assert_eq!(iter.next().unwrap()?, (0, "First Element".to_string()));
    /// assert_eq!(iter.next().unwrap()?, (1, "Second Element".to_string()));
    /// assert_eq!(iter.next().unwrap()?, (3, "Fourth Element".to_string()));
    /// assert!(iter.next().is_none());
    /// # // check fused
    /// # assert!(iter.next().is_none());
    /// # // backward
    /// # assert_eq!(iter.next_back().unwrap()?, (3, "Fourth Element".to_string()));
    /// # assert_eq!(iter.next_back().unwrap()?, (1, "Second Element".to_string()));
    /// # assert_eq!(iter.next_back().unwrap()?, (0, "First Element".to_string()));
    /// # assert!(iter.next_back().is_none());
    /// # // check fused
    /// # assert!(iter.next_back().is_none());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn iter<T>(&self) -> Iter<T>
    where
        T: FromSql,
    {
        Iter::new(self)
    }

    /// Returns an iterator visiting all indices in the collection.
    ///
    /// ```
    /// # use oracle::{Error, Result};
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// // Creates Table type and gets the type information
    /// conn.execute("create or replace type string_table is table of varchar2(60)", &[])?;
    /// let objtype = conn.object_type("STRING_TABLE")?;
    ///
    /// // Creates a TABLE instance, appends four elements and makes a hole.
    /// let mut coll = objtype.new_collection()?;
    /// coll.push(&"First Element")?; // index 0
    /// coll.push(&"Second Element")?; // index 1
    /// coll.push(&"Third Element")?; // index 2
    /// coll.push(&"Fourth Element")?; // index 3
    /// coll.remove(2)?; // remote index 2
    /// // coll's indices are 0, 1 and 3.
    ///
    /// let mut indices = coll.indices();
    /// assert_eq!(indices.next().unwrap().unwrap(), 0);
    /// assert_eq!(indices.next().unwrap().unwrap(), 1);
    /// assert_eq!(indices.next().unwrap().unwrap(), 3);
    /// assert!(indices.next().is_none());
    /// # // check fused or not
    /// # assert!(indices.next().is_none());
    /// # // backward
    /// # assert_eq!(indices.next_back().unwrap()?, 3);
    /// # assert_eq!(indices.next_back().unwrap()?, 1);
    /// # assert_eq!(indices.next_back().unwrap()?, 0);
    /// # assert!(indices.next_back().is_none());
    /// # // check fused or not
    /// # assert!(indices.next_back().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn indices(&self) -> Indices {
        Indices::new(self)
    }

    /// Returns an iterator visiting all values in the collection.
    ///
    /// ```
    /// # use oracle::{Error, Result};
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// // Creates VARRAY type and gets the type information
    /// conn.execute("create or replace type string_varray3 is varray(20) of varchar2(60)", &[])?;
    /// let objtype = conn.object_type("STRING_VARRAY3")?;
    ///
    /// // Creates a VARRAY instance and appends three elements.
    /// let mut coll = objtype.new_collection()?;
    /// coll.push(&"First Element");
    /// coll.push(&"Second Element");
    /// coll.push(&"Third Element");
    ///
    /// let mut iter = coll.values::<String>();
    /// assert_eq!(iter.next().unwrap()?, "First Element".to_string());
    /// assert_eq!(iter.next().unwrap()?, "Second Element".to_string());
    /// assert_eq!(iter.next().unwrap()?, "Third Element".to_string());
    /// assert!(iter.next().is_none());
    /// # // check fused
    /// # assert!(iter.next().is_none());
    /// # // backward
    /// # assert_eq!(iter.next_back().unwrap()?, "Third Element".to_string());
    /// # assert_eq!(iter.next_back().unwrap()?, "Second Element".to_string());
    /// # assert_eq!(iter.next_back().unwrap()?, "First Element".to_string());
    /// # assert!(iter.next_back().is_none());
    /// # // check fused
    /// # assert!(iter.next_back().is_none());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn values<T>(&self) -> Values<T>
    where
        T: FromSql,
    {
        Values::new(self)
    }

    /// Returns the first index.
    ///
    /// Use this method if indexes of the collection isn't continuous.
    pub fn first_index(&self) -> Result<i32> {
        let mut index = 0;
        let mut exists = 0;
        chkerr!(
            self.ctxt(),
            dpiObject_getFirstIndex(self.handle(), &mut index, &mut exists)
        );
        if exists != 0 {
            Ok(index)
        } else {
            Err(Error::no_data_found())
        }
    }

    /// Returns the last index.
    ///
    /// Use this method if indexes of the collection isn't continuous.
    pub fn last_index(&self) -> Result<i32> {
        let mut index = 0;
        let mut exists = 0;
        chkerr!(
            self.ctxt(),
            dpiObject_getLastIndex(self.handle(), &mut index, &mut exists)
        );
        if exists != 0 {
            Ok(index)
        } else {
            Err(Error::no_data_found())
        }
    }

    /// Returns the next index following the specified index.
    ///
    /// Use this method if indexes of the collection isn't continuous.
    pub fn next_index(&self, index: i32) -> Result<i32> {
        let mut next = 0;
        let mut exists = 0;
        chkerr!(
            self.ctxt(),
            dpiObject_getNextIndex(self.handle(), index, &mut next, &mut exists)
        );
        if exists != 0 {
            Ok(next)
        } else {
            Err(Error::no_data_found())
        }
    }

    /// Returns the previous index following the specified index.
    ///
    /// Use this method if indexes of the collection isn't continuous.
    pub fn prev_index(&self, index: i32) -> Result<i32> {
        let mut prev = 0;
        let mut exists = 0;
        chkerr!(
            self.ctxt(),
            dpiObject_getPrevIndex(self.handle(), index, &mut prev, &mut exists)
        );
        if exists != 0 {
            Ok(prev)
        } else {
            Err(Error::no_data_found())
        }
    }

    /// Returns whether an element exists at the specified index.
    pub fn exist(&self, index: i32) -> Result<bool> {
        let mut exists = 0;
        chkerr!(
            self.ctxt(),
            dpiObject_getElementExistsByIndex(self.handle(), index, &mut exists)
        );
        Ok(exists != 0)
    }

    /// Returns the value of the element at the specified index.
    pub fn get<T>(&self, index: i32) -> Result<T>
    where
        T: FromSql,
    {
        let oratype = self.objtype.element_oracle_type().unwrap();
        let mut data = unsafe { mem::zeroed() };
        let mut buf = [0 as c_char; DPI_NUMBER_AS_TEXT_CHARS as usize];
        match oratype {
            &OracleType::Number(_, _) | &OracleType::Float(_) => unsafe {
                dpiData_setBytes(&mut data, buf.as_mut_ptr(), buf.len() as u32);
            },
            _ => (),
        }
        let res;
        let native_type_num;
        {
            let sql_value = SqlValue::from_oratype(self.conn.clone(), oratype, &mut data)?;
            native_type_num = sql_value.native_type_num();
            chkerr!(
                self.ctxt(),
                dpiObject_getElementValueByIndex(
                    self.handle(),
                    index,
                    native_type_num,
                    sql_value.data()?
                )
            );
            res = sql_value.get();
        }
        unsafe { release_dpi_data(&data, native_type_num) };
        res
    }

    /// Sets the value to the element at the specified index.
    pub fn set(&mut self, index: i32, value: &dyn ToSql) -> Result<()> {
        let oratype = self.objtype.element_oracle_type().unwrap();
        let mut data = unsafe { mem::zeroed() };
        let mut sql_value = SqlValue::from_oratype(self.conn.clone(), oratype, &mut data)?;
        sql_value.set(value)?;
        chkerr!(
            self.ctxt(),
            dpiObject_setElementValueByIndex(
                self.handle(),
                index,
                sql_value.native_type_num(),
                sql_value.data()?
            )
        );
        Ok(())
    }

    /// Appends an element to the end of the collection.
    pub fn push(&mut self, value: &dyn ToSql) -> Result<()> {
        let oratype = self.objtype.element_oracle_type().unwrap();
        let mut data = unsafe { mem::zeroed() };
        let mut sql_value = SqlValue::from_oratype(self.conn.clone(), oratype, &mut data)?;
        sql_value.set(value)?;
        chkerr!(
            self.ctxt(),
            dpiObject_appendElement(
                self.handle(),
                sql_value.native_type_num(),
                sql_value.data()?
            )
        );
        Ok(())
    }

    /// Remove the element at the specified index.
    /// Note that the position ordinals of the remaining elements are not changed.
    /// The operation creates **holes** in the collection.
    pub fn remove(&mut self, index: i32) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiObject_deleteElementByIndex(self.handle(), index)
        );
        Ok(())
    }

    /// Trims a number of elements from the end of a collection.
    ///
    /// If the number of of elements to trim exceeds the current size
    /// of the collection an error is returned.
    pub fn trim(&mut self, len: usize) -> Result<()> {
        chkerr!(self.ctxt(), dpiObject_trim(self.handle(), len as u32));
        Ok(())
    }
}

impl Clone for Collection {
    fn clone(&self) -> Collection {
        Collection::new(self.conn.clone(), self.handle.clone(), self.objtype.clone())
    }
}

impl FromSql for Collection {
    fn from_sql(val: &SqlValue) -> Result<Collection> {
        val.to_collection()
    }
}

impl ToSql for Collection {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Object(self.object_type().clone()))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_collection(self)
    }
}

impl fmt::Display for Collection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.objtype)?;
        if let Ok(index) = self.first_index() {
            let mut idx = index;
            let oratype = self.objtype.element_oracle_type().unwrap();
            loop {
                write_literal(f, &self.get(idx), oratype)?;
                if let Ok(index) = self.next_index(idx) {
                    idx = index;
                    write!(f, ", ")?;
                } else {
                    break;
                }
            }
        }
        write!(f, ")")
    }
}

impl fmt::Debug for Collection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let oratype = self.objtype.element_oracle_type().unwrap();
        write!(f, "Collection({} collection of {}: ", self.objtype, oratype)?;
        if let Ok(index) = self.first_index() {
            let mut idx = index;
            loop {
                write_literal(f, &self.get(idx), oratype)?;
                if let Ok(index) = self.next_index(idx) {
                    idx = index;
                    write!(f, ", ")?;
                } else {
                    break;
                }
            }
        }
        write!(f, ")")
    }
}

impl AssertSend for Collection {}

/// Oracle-specific object data type
///
/// ```no_run
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
///
/// // MDSYS.SDO_GEOMETRY
/// // https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-683FF8C5-A773-4018-932D-2AF6EC8BC119
/// let geom_type = conn.object_type("MDSYS.SDO_GEOMETRY")?;
/// let point_type = conn.object_type("MDSYS.SDO_POINT_TYPE")?;
///
/// // Create a new object
/// let mut obj = geom_type.new_object()?;
/// let mut point = point_type.new_object()?;
/// point.set("X", &-79)?;
/// point.set("Y", &37)?;
/// obj.set("SDO_GTYPE", &2001)?;
/// obj.set("SDO_POINT", &point)?;
/// assert_eq!(obj.to_string(), "MDSYS.SDO_GEOMETRY(2001, NULL, MDSYS.SDO_POINT_TYPE(-79, 37, NULL), NULL, NULL)");
///
/// // Gets an attribute value.
/// let gtype: i32 = obj.get("SDO_GTYPE")?;
/// assert_eq!(gtype, 2001);
/// # Ok::<(), Error>(())
/// ```
///
/// Note: Methods in the type may be changed in future.
pub struct Object {
    conn: Conn,
    pub(crate) handle: DpiObject,
    objtype: ObjectType,
}

impl Object {
    pub(crate) fn new(conn: Conn, handle: DpiObject, objtype: ObjectType) -> Object {
        Object {
            conn,
            handle,
            objtype,
        }
    }

    pub(crate) fn ctxt(&self) -> &Context {
        self.conn.ctxt()
    }

    pub(crate) fn handle(&self) -> *mut dpiObject {
        self.handle.raw
    }

    /// Returns type information.
    pub fn object_type(&self) -> &ObjectType {
        &self.objtype
    }

    fn type_attr(&self, name: &str) -> Result<&ObjectTypeAttr> {
        for attr in self.objtype.attributes() {
            if attr.name() == name {
                return Ok(attr);
            }
        }
        Err(Error::invalid_attribute_name(name))
    }

    pub(crate) fn get_by_attr<T>(&self, attr: &ObjectTypeAttr) -> Result<T>
    where
        T: FromSql,
    {
        let mut data = unsafe { mem::zeroed() };
        let mut buf = [0 as c_char; DPI_NUMBER_AS_TEXT_CHARS as usize];
        match &attr.oratype {
            &OracleType::Number(_, _) | &OracleType::Float(_) => unsafe {
                dpiData_setBytes(&mut data, buf.as_mut_ptr(), buf.len() as u32);
            },
            _ => (),
        }
        let res;
        let native_type_num;
        {
            let sql_value = SqlValue::from_oratype(self.conn.clone(), &attr.oratype, &mut data)?;
            native_type_num = sql_value.native_type_num();
            chkerr!(
                self.ctxt(),
                dpiObject_getAttributeValue(
                    self.handle(),
                    attr.handle.raw(),
                    native_type_num,
                    sql_value.data()?
                )
            );
            res = sql_value.get();
        }
        unsafe { release_dpi_data(&data, native_type_num) };
        res
    }

    /// Gets an value at the specified attribute.
    pub fn get<T>(&self, name: &str) -> Result<T>
    where
        T: FromSql,
    {
        self.get_by_attr(self.type_attr(name)?)
    }

    /// Sets the value to the specified attribute.
    pub fn set(&mut self, name: &str, value: &dyn ToSql) -> Result<()> {
        let attrtype = self.type_attr(name)?;
        let mut data = unsafe { mem::zeroed() };
        let mut sql_value =
            SqlValue::from_oratype(self.conn.clone(), &attrtype.oratype, &mut data)?;
        sql_value.set(value)?;
        chkerr!(
            self.ctxt(),
            dpiObject_setAttributeValue(
                self.handle(),
                attrtype.handle.raw(),
                sql_value.native_type_num(),
                sql_value.data()?
            )
        );
        Ok(())
    }
}

impl Clone for Object {
    fn clone(&self) -> Object {
        Object::new(self.conn.clone(), self.handle.clone(), self.objtype.clone())
    }
}

impl FromSql for Object {
    fn from_sql(val: &SqlValue) -> Result<Object> {
        val.to_object()
    }
}

impl ToSql for Object {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Object(self.object_type().clone()))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_object(self)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.objtype)?;
        let mut first = true;
        for attr in self.objtype.attributes() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write_literal(f, &self.get_by_attr(attr), &attr.oratype)?;
        }
        write!(f, ")")
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Object({}(", self.objtype)?;
        let mut first = true;
        for attr in self.objtype.attributes() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{}({}): ", attr.name(), attr.oracle_type())?;
            write_literal(f, &self.get_by_attr(attr), &attr.oratype)?;
        }
        write!(f, "))")
    }
}

impl AssertSend for Object {}

/// Type information about Object or Collection data type
///
/// This is for not only Object type information but also
/// collection type information.
///
/// # Examples
///
/// Gets MDSYS.SDO_GEOMETRY object type information.
///
/// ```no_run
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
/// let objtype = conn.object_type("MDSYS.SDO_GEOMETRY");
/// # Ok::<(), Error>(())
/// ```
///
/// Gets object type infomration in query.
///
/// ```no_run
/// # use oracle::*; use oracle::sql_type::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
/// // conn.execute("create table location (name varchar2(60), loc sdo_geometry)", &[]);
/// let mut stmt = conn
///     .statement("select loc from location where name = '...'")
///     .build()?;
/// let rows = stmt.query(&[])?;
/// let objtype = if let OracleType::Object(ref objtype) = *rows.column_info()[0].oracle_type() {
///     objtype
/// } else {
///     panic!("Not an object type")
/// };
/// # Ok::<(), Error>(())
/// ```
#[derive(Clone)]
pub struct ObjectType {
    pub(crate) internal: Arc<ObjectTypeInternal>,
}

impl ObjectType {
    pub(crate) fn from_dpi_object_type(conn: Conn, handle: DpiObjectType) -> Result<ObjectType> {
        Ok(ObjectType {
            internal: Arc::new(ObjectTypeInternal::from_dpi_object_type(conn, handle)?),
        })
    }

    pub(crate) fn handle(&self) -> &DpiObjectType {
        &self.internal.handle
    }

    /// Gets schema name
    pub fn schema(&self) -> &str {
        &self.internal.schema
    }

    /// Gets object name
    pub fn name(&self) -> &str {
        &self.internal.name
    }

    /// Gets package name if it is a PL/SQL type.
    /// Otherwise, `None`.
    pub fn package_name(&self) -> Option<&str> {
        if let Some(ref pkg_name) = self.internal.package_name {
            Some(pkg_name)
        } else {
            None
        }
    }

    /// True when it is a collectoin. Otherwise false.
    pub fn is_collection(&self) -> bool {
        self.internal.elem_oratype.is_some()
    }

    /// Gets the Oracle type of elements if it is a collection.
    /// Otherwise, `None`.
    pub fn element_oracle_type(&self) -> Option<&OracleType> {
        if let Some(ref oratype) = self.internal.elem_oratype {
            Some(oratype)
        } else {
            None
        }
    }

    /// Gets the number of attributes if it isn't a collection.
    /// Otherwise, 0.
    pub fn num_attributes(&self) -> usize {
        self.internal.attrs.len()
    }

    /// Gets a vector of attribute information if it isn't a collection.
    /// Otherwise, a zero-length vector.
    ///
    /// # Examples
    ///
    /// Prints attribute information of `MDSYS.SDO_GEOMETRY`.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    /// let objtype = conn.object_type("MDSYS.SDO_GEOMETRY")?;
    /// for attr in objtype.attributes() {
    ///     println!("{:-20} {}", attr.name(), attr.oracle_type());
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn attributes(&self) -> &[ObjectTypeAttr] {
        &self.internal.attrs
    }

    /// Create a new Oracle object.
    pub fn new_object(&self) -> Result<Object> {
        if self.is_collection() {
            return Err(Error::invalid_operation(format!(
                "{}.{} isn't object type.",
                self.schema(),
                self.name()
            )));
        }
        let conn = &self.internal.conn;
        let mut handle = DpiObject::null();
        chkerr!(
            conn.ctxt(),
            dpiObjectType_createObject(self.internal.handle.raw(), &mut handle.raw)
        );
        Ok(Object::new(conn.clone(), handle, self.clone()))
    }

    /// Create a new collection.
    pub fn new_collection(&self) -> Result<Collection> {
        if !self.is_collection() {
            return Err(Error::invalid_operation(format!(
                "{}.{} isn't collection type.",
                self.schema(),
                self.name()
            )));
        }
        let conn = &self.internal.conn;
        let mut handle = DpiObject::null();
        chkerr!(
            conn.ctxt(),
            dpiObjectType_createObject(self.internal.handle.raw(), &mut handle.raw)
        );
        Ok(Collection::new(conn.clone(), handle, self.clone()))
    }
}

impl cmp::PartialEq for ObjectType {
    fn eq(&self, other: &Self) -> bool {
        self.internal == other.internal
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.internal)
    }
}

impl fmt::Debug for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.internal)
    }
}

/// Object type attribute information
///
/// See [ObjectType.attributes()](struct.ObjectType.html#method.attributes)
pub struct ObjectTypeAttr {
    conn: Conn,
    handle: DpiObjectAttr,
    name: String,
    oratype: OracleType,
}

impl ObjectTypeAttr {
    fn new(conn: Conn, handle: DpiObjectAttr) -> Result<ObjectTypeAttr> {
        let mut info = MaybeUninit::uninit();
        chkerr!(
            conn.ctxt(),
            dpiObjectAttr_getInfo(handle.raw(), info.as_mut_ptr())
        );
        let info = unsafe { info.assume_init() };
        Ok(ObjectTypeAttr {
            oratype: OracleType::from_type_info(&conn, &info.typeInfo)?,
            conn,
            handle,
            name: to_rust_str(info.name, info.nameLength),
        })
    }

    /// Gets the attribute name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the attribute type
    pub fn oracle_type(&self) -> &OracleType {
        &self.oratype
    }
}

impl Clone for ObjectTypeAttr {
    fn clone(&self) -> ObjectTypeAttr {
        ObjectTypeAttr {
            conn: self.conn.clone(),
            handle: self.handle.clone(),
            name: self.name.clone(),
            oratype: self.oratype.clone(),
        }
    }
}

impl fmt::Debug for ObjectTypeAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ObjectTypeAttr {{ handle: {:?}, name: {:?}, oratype: {:?} }}",
            self.handle.raw(),
            self.name,
            self.oratype
        )
    }
}

//
// ObjectTypeInternal
//

pub(crate) struct ObjectTypeInternal {
    conn: Conn,
    handle: DpiObjectType,
    schema: String,
    name: String,
    package_name: Option<String>,
    elem_oratype: Option<OracleType>,
    attrs: Vec<ObjectTypeAttr>,
}

impl ObjectTypeInternal {
    fn from_dpi_object_type(conn: Conn, handle: DpiObjectType) -> Result<ObjectTypeInternal> {
        let mut info = MaybeUninit::uninit();
        chkerr!(
            conn.ctxt(),
            dpiObjectType_getInfo(handle.raw(), info.as_mut_ptr())
        );
        let info = unsafe { info.assume_init() };
        let (elem_oratype, attrs) = if info.isCollection != 0 {
            match OracleType::from_type_info(&conn, &info.elementTypeInfo) {
                Ok(oratype) => (Some(oratype), Vec::new()),
                Err(err) => return Err(err),
            }
        } else {
            let attrnum = info.numAttributes as usize;
            let mut handles = Vec::<DpiObjectAttr>::with_capacity(attrnum);
            chkerr!(
                conn.ctxt(),
                dpiObjectType_getAttributes(
                    handle.raw(),
                    info.numAttributes,
                    // The following code works only when
                    // the size of `*mut dpiObjectAttr` equals to that of `DpiObjectAttr`.
                    handles.as_mut_ptr() as *mut *mut dpiObjectAttr
                )
            );
            unsafe {
                handles.set_len(attrnum);
            }
            let attrs: Result<Vec<_>> = handles
                .into_iter()
                .map(|handle| ObjectTypeAttr::new(conn.clone(), handle))
                .collect();
            (None, attrs?)
        };
        Ok(ObjectTypeInternal {
            conn,
            handle,
            schema: to_rust_str(info.schema, info.schemaLength),
            name: to_rust_str(info.name, info.nameLength),
            package_name: if info.packageNameLength != 0 {
                Some(to_rust_str(info.packageName, info.packageNameLength))
            } else {
                None
            },
            elem_oratype,
            attrs,
        })
    }
}

impl cmp::PartialEq for ObjectTypeInternal {
    fn eq(&self, other: &Self) -> bool {
        self.handle.raw() == other.handle.raw()
    }
}

impl fmt::Display for ObjectTypeInternal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.schema, self.name)
    }
}

impl fmt::Debug for ObjectTypeInternal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.elem_oratype.is_some() {
            write!(
                f,
                "ObjectType({}.{} collection of {})",
                self.schema,
                self.name,
                self.elem_oratype.as_ref().unwrap()
            )
        } else {
            write!(f, "ObjectType({}.{}(", self.schema, self.name)?;
            let mut first = true;
            for attr in &self.attrs {
                if first {
                    first = false;
                } else {
                    write!(f, ", ")?;
                }
                write!(f, "{} {}", attr.name(), attr.oracle_type())?;
            }
            write!(f, "))")
        }
    }
}
