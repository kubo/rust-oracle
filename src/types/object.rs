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
use OracleType;
use Result;

use OdpiStr;

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
}

impl cmp::PartialEq for ObjectType {
    fn eq(&self, other: &Self) -> bool {
        self.internal == other.internal
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

impl fmt::Debug for ObjectTypeInternal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.elem_oratype.is_some() {
            write!(f, "ObjectType({}.{} collection of {:?})", self.schema, self.name,
                   self.elem_oratype.as_ref().unwrap())
        } else {
            write!(f, "ObjectType({}.{} {{", self.schema, self.name)?;
            for attr in &self.attrs {
                write!(f, "{}: {:?}, ", attr.name(), attr.oracle_type())?;
            }
            write!(f, "}})")
        }
    }
}
