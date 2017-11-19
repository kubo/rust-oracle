// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017 Kubo Takehiro <kubo@jiubao.org>
//
// Redistribution and use in source and binary forms, with or without modification, are
// permitted provided that the following conditions are met:
//
//    1. Redistributions of source code must retain the above copyright notice, this list of
//       conditions and the following disclaimer.
//
//    2. Redistributions in binary form must reproduce the above copyright notice, this list
//       of conditions and the following disclaimer in the documentation and/or other materials
//       provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHORS ''AS IS'' AND ANY EXPRESS OR IMPLIED
// WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
// ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
// ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// The views and conclusions contained in the software and documentation are those of the
// authors and should not be interpreted as representing official policies, either expressed
// or implied, of the authors.

use std::cmp;
use std::fmt;
use std::ptr;
use std::rc::Rc;

use binding::*;
use Context;
use Error;
use FromSql;
use OracleType;
use Result;
use SqlValue;
use ToSql;

use OdpiStr;
use util::write_literal;

/// Collection data type of Oracle database
///
/// This type corresponds to varray and nested table data types.
/// See [Oracle manual](https://docs.oracle.com/database/122/ADOBJ/collection-data-types.htm)
///
/// ```no_run
/// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
/// // MDSYS.SDO_ELEM_INFO_ARRAY is defined as VARRAY (1048576) of NUMBER.
/// let objtype = conn.object_type("MDSYS.SDO_ELEM_INFO_ARRAY").unwrap();
/// // Create a new collection
/// let mut obj = objtype.new_collection().unwrap();
/// obj.push(&1);
/// obj.push(&1003);
/// obj.push(&3);
/// assert_eq!(obj.to_string(), "MDSYS.SDO_ELEM_INFO_ARRAY(1, 3)");
/// ```
pub struct Collection {
    ctxt: &'static Context,
    pub(crate) handle: *mut dpiObject,
    objtype: ObjectType,
}

impl Collection {
    pub(crate) fn new(ctxt: &'static Context, handle: *mut dpiObject, objtype: ObjectType) -> Collection {
        Collection {
            ctxt: ctxt,
            handle: handle,
            objtype: objtype,
        }
    }

    pub fn object_type(&self) -> &ObjectType {
        &self.objtype
    }

    pub fn size(&self) -> Result<i32> {
        let mut size = 0;
        chkerr!(self.ctxt,
                dpiObject_getSize(self.handle, &mut size));
        Ok(size)
    }

    pub fn first_index(&self) -> Result<i32> {
        let mut index = 0;
        let mut exists = 0;
        chkerr!(self.ctxt,
                dpiObject_getFirstIndex(self.handle, &mut index, &mut exists));
        if exists != 0 {
            Ok(index)
        } else {
            Err(Error::NoMoreData)
        }
    }

    pub fn last_index(&self) -> Result<i32> {
        let mut index = 0;
        let mut exists = 0;
        chkerr!(self.ctxt,
                dpiObject_getLastIndex(self.handle, &mut index, &mut exists));
        if exists != 0 {
            Ok(index)
        } else {
            Err(Error::NoMoreData)
        }
    }

    pub fn next_index(&self, index: i32) -> Result<i32> {
        let mut next = 0;
        let mut exists = 0;
        chkerr!(self.ctxt,
                dpiObject_getNextIndex(self.handle, index, &mut next, &mut exists));
        if exists != 0 {
            Ok(next)
        } else {
            Err(Error::NoMoreData)
        }
    }

    pub fn prev_index(&self, index: i32) -> Result<i32> {
        let mut prev = 0;
        let mut exists = 0;
        chkerr!(self.ctxt,
                dpiObject_getPrevIndex(self.handle, index, &mut prev, &mut exists));
        if exists != 0 {
            Ok(prev)
        } else {
            Err(Error::NoMoreData)
        }
    }

    pub fn exist(&self, index: i32) -> Result<bool> {
        let mut exists = 0;
        chkerr!(self.ctxt,
                dpiObject_getElementExistsByIndex(self.handle, index, &mut exists));
        Ok(exists != 0)
    }

    pub fn get<T>(&self, index: i32) -> Result<T> where T: FromSql {
        let oratype = self.objtype.element_oracle_type().unwrap();
        let mut data = Default::default();
        let mut buf = [0i8; 172]; // DPI_NUMBER_AS_TEXT_CHARS in odpi/src/dpiImpl.h
        if let OracleType::Number(_, _) = *oratype {
            unsafe {
                dpiData_setBytes(&mut data, buf.as_mut_ptr(), buf.len() as u32);
            }
        }
        let sql_value = SqlValue::from_oratype(self.ctxt, oratype, &mut data)?;
        chkerr!(self.ctxt,
                dpiObject_getElementValueByIndex(self.handle, index, sql_value.native_type_num(), &mut data));
        sql_value.get()
    }

    pub fn set(&mut self, index: i32, value: &ToSql) -> Result<()> {
        let oratype = self.objtype.element_oracle_type().unwrap();
        let mut data = Default::default();
        let mut sql_value = SqlValue::from_oratype(self.ctxt, oratype, &mut data)?;
        sql_value.set(value)?;
        chkerr!(self.ctxt,
                dpiObject_setElementValueByIndex(self.handle, index, sql_value.native_type_num(), &mut data));
        Ok(())
    }

    pub fn push(&mut self, value: &ToSql) -> Result<()> {
        let oratype = self.objtype.element_oracle_type().unwrap();
        let mut data = Default::default();
        let mut sql_value = SqlValue::from_oratype(self.ctxt, oratype, &mut data)?;
        sql_value.set(value)?;
        chkerr!(self.ctxt,
                dpiObject_appendElement(self.handle, sql_value.native_type_num(), &mut data));
        Ok(())
    }

    pub fn remove(&mut self, index: i32) -> Result<()> {
        chkerr!(self.ctxt,
                dpiObject_deleteElementByIndex(self.handle, index));
        Ok(())
    }

    pub fn trim(&mut self, len: usize) -> Result<()> {
        chkerr!(self.ctxt,
                dpiObject_trim(self.handle, len as u32));
        Ok(())
    }
}

impl Clone for Collection {
    fn clone(&self) -> Collection {
        unsafe { dpiObject_addRef(self.handle) };
        Collection::new(self.ctxt, self.handle, self.objtype.clone())
    }
}

impl Drop for Collection {
    fn drop(&mut self) {
        let _ = unsafe { dpiObject_release(self.handle) };
    }
}

impl FromSql for Collection {
    fn from_sql(val: &SqlValue) -> Result<Collection> {
        val.as_collection()
    }
}

impl ToSql for Collection {
    fn oratype(&self) -> Result<OracleType> {
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

//
// Object
//

pub struct Object {
    ctxt: &'static Context,
    pub(crate) handle: *mut dpiObject,
    objtype: ObjectType,
}

impl Object {
    pub(crate) fn new(ctxt: &'static Context, handle: *mut dpiObject, objtype: ObjectType) -> Object {
        Object {
            ctxt: ctxt,
            handle: handle,
            objtype: objtype,
        }
    }

    pub fn object_type(&self) -> &ObjectType {
        &self.objtype
    }

    fn type_attr(&self, name: &str) -> Result<&ObjectTypeAttr> {
        for attr in self.objtype.attributes() {
            if attr.name() == name {
                return Ok(attr);
            }
        }
        Err(Error::InvalidAttributeName(name.to_string()))
    }

    pub fn get_by_attr<T>(&self, attr: &ObjectTypeAttr) -> Result<T> where T: FromSql {
        let mut data = Default::default();
        let mut buf = [0i8; 172]; // DPI_NUMBER_AS_TEXT_CHARS in odpi/src/dpiImpl.h
        if let OracleType::Number(_, _) = attr.oratype {
            unsafe {
                dpiData_setBytes(&mut data, buf.as_mut_ptr(), buf.len() as u32);
            }
        }
        let sql_value = SqlValue::from_oratype(self.ctxt, &attr.oratype, &mut data)?;
        chkerr!(self.ctxt,
                dpiObject_getAttributeValue(self.handle, attr.handle,
                                            sql_value.native_type_num(), &mut data));
        sql_value.get()
    }

    pub fn get<T>(&self, name: &str) -> Result<T> where T: FromSql {
        self.get_by_attr(self.type_attr(name)?)
    }

    pub fn set(&mut self, name: &str, value: &ToSql) -> Result<()> {
        let attrtype = self.type_attr(name)?;
        let mut data = Default::default();
        let mut sql_value = SqlValue::from_oratype(self.ctxt, &attrtype.oratype, &mut data)?;
        sql_value.set(value)?;
        chkerr!(self.ctxt,
                dpiObject_setAttributeValue(self.handle, attrtype.handle,
                                            sql_value.native_type_num(), &mut data));
        Ok(())
    }
}

impl Clone for Object {
    fn clone(&self) -> Object {
        unsafe { dpiObject_addRef(self.handle) };
        Object::new(self.ctxt, self.handle, self.objtype.clone())
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        let _ = unsafe { dpiObject_release(self.handle) };
    }
}

impl FromSql for Object {
    fn from_sql(val: &SqlValue) -> Result<Object> {
        val.as_object()
    }
}

impl ToSql for Object {
    fn oratype(&self) -> Result<OracleType> {
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

//
// ObjectType
//

/// Object type information
///
/// # Examples
///
/// Gets MDSYS.SDO_GEOMETRY object type information.
///
/// ```no_run
/// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
/// let objtype = conn.object_type("MDSYS.SDO_GEOMETRY");
/// ```
///
/// Gets object type infomration in query.
///
/// ```no_run
/// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
/// // conn.execute("create table location (name varchar2(60), loc sdo_geometry)", &[]);
/// let mut stmt = conn.execute("select loc from location where name = '...'", &[]).unwrap();
/// let objtype = if let oracle::OracleType::Object(ref objtype) = *stmt.column_info()[0].oracle_type() {
///     objtype
/// } else {
///     panic!("Not an object type")
/// };
/// ```
#[derive(Clone)]
pub struct ObjectType {
    internal: Rc<ObjectTypeInternal>,
}

impl ObjectType {
    #[allow(non_snake_case)]
    pub(crate) fn from_dpiObjectType(ctxt: &'static Context, handle: *mut dpiObjectType) -> Result<ObjectType> {
        Ok(ObjectType {
            internal: Rc::new(ObjectTypeInternal::from_dpiObjectType(ctxt, handle)?)
        })
    }

    pub(crate) fn handle(&self) -> *mut dpiObjectType {
        self.internal.handle
    }

    /// Gets schema name
    pub fn schema(&self) -> &String {
        &self.internal.schema
    }

    /// Gets object name
    pub fn name(&self) -> &String {
        &self.internal.name
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
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    /// let objtype = conn.object_type("MDSYS.SDO_GEOMETRY").unwrap();
    /// for attr in objtype.attributes() {
    ///     println!("{:-20} {}", attr.name(), attr.oracle_type());
    /// }
    /// ```
    pub fn attributes(&self) -> &Vec<ObjectTypeAttr> {
        &self.internal.attrs
    }

    pub fn new_object(&self) -> Option<Object> {
        if self.is_collection() {
            return None
        }
        let ctxt = self.internal.ctxt;
        let mut handle = ptr::null_mut();
        if unsafe {dpiObjectType_createObject(self.internal.handle, &mut handle)} != DPI_SUCCESS as i32 {
            return None;
        }
        Some(Object::new(ctxt, handle, self.clone()))
    }

    pub fn new_collection(&self) -> Option<Collection> {
        if !self.is_collection() {
            return None
        }
        let ctxt = self.internal.ctxt;
        let mut handle = ptr::null_mut();
        if unsafe {dpiObjectType_createObject(self.internal.handle, &mut handle)} != DPI_SUCCESS as i32 {
            return None;
        }
        Some(Collection::new(ctxt, handle, self.clone()))
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

//
// ObjectTypeAttr
//

/// Object type attribute information
///
/// See [ObjectType.attributes()](struct.ObjectType.html#method.attributes)
pub struct ObjectTypeAttr {
    ctxt: &'static Context,
    handle: *mut dpiObjectAttr,
    name: String,
    oratype: OracleType,
}

impl ObjectTypeAttr {
    fn new(ctxt: &'static Context, handle: *mut dpiObjectAttr) -> Result<ObjectTypeAttr> {
        let mut info = Default::default();
        chkerr!(ctxt,
                dpiObjectAttr_getInfo(handle, &mut info));
        Ok(ObjectTypeAttr {
            ctxt: ctxt,
            handle: handle,
            name: OdpiStr::new(info.name, info.nameLength).to_string(),
            oratype: OracleType::from_type_info(ctxt, &info.typeInfo)?,
        })
    }

    /// Gets the attribute name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Gets the attribute type
    pub fn oracle_type(&self) -> &OracleType {
        &self.oratype
    }
}

impl Clone for ObjectTypeAttr {
    fn clone(&self) -> ObjectTypeAttr {
        unsafe { dpiObjectAttr_addRef(self.handle) };
        ObjectTypeAttr {
            ctxt: self.ctxt,
            handle: self.handle,
            name: self.name.clone(),
            oratype: self.oratype.clone(),
        }
    }
}

impl Drop for ObjectTypeAttr {
    fn drop(&mut self) {
        unsafe { dpiObjectAttr_release(self.handle); };
    }
}

//
// ObjectTypeInternal
//

struct ObjectTypeInternal {
    ctxt: &'static Context,
    handle: *mut dpiObjectType,
    schema: String,
    name: String,
    elem_oratype: Option<OracleType>,
    attrs: Vec<ObjectTypeAttr>,
}

impl ObjectTypeInternal {
    #[allow(non_snake_case)]
    fn from_dpiObjectType(ctxt: &'static Context, handle: *mut dpiObjectType) -> Result<ObjectTypeInternal> {
        let mut info = Default::default();
        chkerr!(ctxt,
                dpiObjectType_getInfo(handle, &mut info));
        let (elem_oratype, attrs) = if info.isCollection != 0 {
            match OracleType::from_type_info(ctxt, &info.elementTypeInfo) {
                Ok(oratype) => (Some(oratype), Vec::new()),
                Err(err) => return Err(err),
            }
        } else {
            let attrnum = info.numAttributes as usize;
            let mut attr_handles = vec![ptr::null_mut(); attrnum];
            chkerr!(ctxt,
                    dpiObjectType_getAttributes(handle, info.numAttributes,
                                                attr_handles.as_mut_ptr()));
            let mut attrs = Vec::with_capacity(attrnum);
            for i in 0..attrnum {
                match ObjectTypeAttr::new(ctxt, attr_handles[i]) {
                    Ok(attr) => attrs.push(attr),
                    Err(err) => {
                        for j in i..attrnum {
                            unsafe { dpiObjectAttr_release(attr_handles[j]); }
                        }
                        return Err(err);
                    },
                }
            }
            (None, attrs)
        };
        unsafe { dpiObjectType_addRef(handle); }
        Ok(ObjectTypeInternal {
            ctxt: ctxt,
            handle: handle,
            schema: OdpiStr::new(info.schema, info.schemaLength).to_string(),
            name: OdpiStr::new(info.name, info.nameLength).to_string(),
            elem_oratype: elem_oratype,
            attrs: attrs,
        })
    }
}

impl Drop for ObjectTypeInternal {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { dpiObjectType_release(self.handle); };
        }
    }
}

impl cmp::PartialEq for ObjectTypeInternal {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
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
            write!(f, "ObjectType({}.{} collection of {})", self.schema, self.name,
                   self.elem_oratype.as_ref().unwrap())
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
