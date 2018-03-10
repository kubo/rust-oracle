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

extern crate oracle;
mod common;
use oracle::*;

#[test]
fn collection_udt_nestedarray() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("UDT_NESTEDARRAY").unwrap();
    let subobjtype = conn.object_type("UDT_SUBOBJECT").unwrap();
    let mut obj = objtype.new_collection().unwrap();
    let mut subobj1 = subobjtype.new_object().unwrap();
    let subobj2 = subobjtype.new_object().unwrap();
    let username = common::main_user().to_uppercase();

    subobj1.set("SUBNUMBERVALUE", &1).unwrap();
    subobj1.set("SUBSTRINGVALUE", &"STRVAL:1").unwrap();

    assert_eq!(obj.exist(0).unwrap(), false);
    assert_eq!(obj.exist(1).unwrap(), false);
    assert_eq!(obj.size().unwrap(), 0);
    let err = obj.trim(1).unwrap_err();
    assert_eq!(err.to_string(), "OCI Error: OCI-22167: given trim size [1] must be less than or equal to [0]");
    let err = obj.remove(0).unwrap_err();
    assert_eq!(err.to_string(), "OCI Error: OCI-22160: element at index [0] does not exist");

    obj.push(&subobj1).unwrap();
    assert_eq!(obj.exist(0).unwrap(), true);
    assert_eq!(obj.exist(1).unwrap(), false);
    assert_eq!(obj.size().unwrap(), 1);

    obj.push(&subobj2).unwrap();
    assert_eq!(obj.exist(0).unwrap(), true);
    assert_eq!(obj.exist(1).unwrap(), true);
    assert_eq!(obj.size().unwrap(), 2);

    let subobj: Object = obj.get(0).unwrap();
    assert_eq!(subobj.get::<i32>("SUBNUMBERVALUE").unwrap(), 1);
    assert_eq!(subobj.get::<String>("SUBSTRINGVALUE").unwrap(), "STRVAL:1");

    let subobj: Object = obj.get(1).unwrap();
    assert_eq!(subobj.get::<Option<i32>>("SUBNUMBERVALUE").unwrap(), None);
    assert_eq!(subobj.get::<Option<String>>("SUBSTRINGVALUE").unwrap(), None);

    assert_eq!(objtype.to_string(),
               format!("{}.UDT_NESTEDARRAY", username));
    assert_eq!(subobjtype.to_string(),
               format!("{}.UDT_SUBOBJECT", username));
    assert_eq!(obj.to_string(),
               format!("{}.UDT_NESTEDARRAY({}.UDT_SUBOBJECT(1, \"STRVAL:1\"), {}.UDT_SUBOBJECT(NULL, NULL))",
                       username, username, username));
    assert_eq!(subobj1.to_string(),
               format!("{}.UDT_SUBOBJECT(1, \"STRVAL:1\")", username));
    assert_eq!(subobj2.to_string(),
               format!("{}.UDT_SUBOBJECT(NULL, NULL)", username));

    assert_eq!(format!("{:?}", objtype),
               format!("ObjectType({}.UDT_NESTEDARRAY collection of ODPIC.UDT_SUBOBJECT)", username));
    assert_eq!(format!("{:?}", subobjtype),
               format!("ObjectType({}.UDT_SUBOBJECT(SUBNUMBERVALUE NUMBER, SUBSTRINGVALUE VARCHAR2(60)))", username));
    assert_eq!(format!("{:?}", obj),
               format!("Collection({}.UDT_NESTEDARRAY collection of {}.UDT_SUBOBJECT: {}.UDT_SUBOBJECT(1, \"STRVAL:1\"), {}.UDT_SUBOBJECT(NULL, NULL))", username, username, username, username));
    assert_eq!(format!("{:?}", subobj1),
               format!("Object({}.UDT_SUBOBJECT(SUBNUMBERVALUE(NUMBER): 1, SUBSTRINGVALUE(VARCHAR2(60)): \"STRVAL:1\"))", username));
    assert_eq!(format!("{:?}", subobj2),
               format!("Object({}.UDT_SUBOBJECT(SUBNUMBERVALUE(NUMBER): NULL, SUBSTRINGVALUE(VARCHAR2(60)): NULL))", username));

    obj.remove(0).unwrap();
    assert_eq!(obj.exist(0).unwrap(), false);
    assert_eq!(obj.exist(1).unwrap(), true);
    assert_eq!(obj.size().unwrap(), 2); // This counts also deleted elements. See "Comments" about OCICollSize() in OCI manual.

    obj.trim(1).unwrap();
    assert_eq!(obj.exist(0).unwrap(), false);
    assert_eq!(obj.exist(1).unwrap(), false);
    assert_eq!(obj.size().unwrap(), 1);

    obj.trim(1).unwrap();
    assert_eq!(obj.exist(0).unwrap(), false);
    assert_eq!(obj.exist(1).unwrap(), false);
    assert_eq!(obj.size().unwrap(), 0);

    let mut obj = objtype.new_collection().unwrap();
    let mut subobj = subobjtype.new_object().unwrap();
    subobj.set("SUBNUMBERVALUE", &1234.5679999f64).unwrap();
    subobj.set("SUBSTRINGVALUE", &"Test String").unwrap();
    obj.push(&subobj).unwrap();
    assert_eq!(obj.size().unwrap(), 1);
    let mut obj2 = obj.clone(); // shallow copy
    obj2.push(&subobj).unwrap(); // When obj2 is changed,
    assert_eq!(obj.size().unwrap(), 2); // obj is also changed.
    assert_eq!(obj2.size().unwrap(), 2);
}

#[test]
fn udt_array() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("UDT_ARRAY").unwrap();
    let mut obj = objtype.new_collection().unwrap();

    assert_eq!(obj.size().unwrap(), 0);
    obj.push(&10).unwrap();
    assert_eq!(obj.get::<i32>(0).unwrap(), 10);
    obj.push(&11).unwrap();
    assert_eq!(obj.exist(0).unwrap(), true);
    assert_eq!(obj.exist(1).unwrap(), true);
    assert_eq!(obj.exist(2).unwrap(), false);
    obj.set(0, &12).unwrap();
    assert_eq!(obj.get::<i32>(0).unwrap(), 12);
}

#[test]
fn udt_object() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("UDT_OBJECT").unwrap();
    let subobjtype = conn.object_type("UDT_SUBOBJECT").unwrap();
    let objarytype = conn.object_type("UDT_OBJECTARRAY").unwrap();
    let mut obj = objtype.new_object().unwrap();
    let mut subobj = subobjtype.new_object().unwrap();
    let mut objary = objarytype.new_collection().unwrap();

    subobj.set("SUBNUMBERVALUE", &10).unwrap();
    subobj.set("SUBSTRINGVALUE", &"SUBSTRVAL:10").unwrap();
    objary.push(&subobj).unwrap();
    subobj.set("SUBNUMBERVALUE", &11).unwrap();
    subobj.set("SUBSTRINGVALUE", &"SUBSTRVAL:11").unwrap();
    objary.push(&subobj).unwrap();
    subobj.set("SUBNUMBERVALUE", &12).unwrap();
    subobj.set("SUBSTRINGVALUE", &"SUBSTRVAL:12").unwrap();

    obj.set("NUMBERVALUE", &1).unwrap();
    obj.set("STRINGVALUE", &"STRVAL:1").unwrap();
    obj.set("FIXEDCHARVALUE", &"CHARVAL:1").unwrap();
    obj.set("DATEVALUE", &Timestamp::new(2012, 3, 4, 5, 6, 7, 0)).unwrap();
    obj.set("TIMESTAMPVALUE", &Timestamp::new(2017, 2, 3, 4, 5, 6, 123456789)).unwrap();
    obj.set("SUBOBJECTVALUE", &subobj).unwrap();
    obj.set("SUBOBJECTARRAY", &objary).unwrap();

    assert_eq!(obj.get::<i32>("NUMBERVALUE").unwrap(), 1);
    assert_eq!(obj.get::<String>("STRINGVALUE").unwrap(), "STRVAL:1");
    assert_eq!(obj.get::<String>("FIXEDCHARVALUE").unwrap(), "CHARVAL:1");
    assert_eq!(obj.get::<Timestamp>("DATEVALUE").unwrap(), Timestamp::new(2012, 3, 4, 5, 6, 7, 0));
    assert_eq!(obj.get::<Timestamp>("TIMESTAMPVALUE").unwrap(), Timestamp::new(2017, 2, 3, 4, 5, 6, 123456789));
    assert_eq!(obj.get::<Object>("SUBOBJECTVALUE").unwrap().get::<i32>("SUBNUMBERVALUE").unwrap(), 12);
    assert_eq!(obj.get::<Collection>("SUBOBJECTARRAY").unwrap().get::<Object>(0).unwrap().get::<i32>("SUBNUMBERVALUE").unwrap(), 10);
    assert_eq!(obj.get::<Collection>("SUBOBJECTARRAY").unwrap().get::<Object>(1).unwrap().get::<i32>("SUBNUMBERVALUE").unwrap(), 11);

    let err = subobj.get::<Object>("SUBNUMBERVALUE").unwrap_err();
    assert_eq!(err.to_string(), "invalid type conversion from NUMBER to Object");
    let err = subobj.get::<Collection>("SUBNUMBERVALUE").unwrap_err();
    assert_eq!(err.to_string(), "invalid type conversion from NUMBER to Collection");
}

#[test]
fn udt_stringlist() {
    if client_version().unwrap().major() < 12 {
        return;
    }
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("PKG_TESTSTRINGARRAYS.UDT_STRINGLIST").unwrap();

    let mut stmt = conn.prepare("begin pkg_TestStringArrays.TestIndexBy(:1); end;").unwrap();
    stmt.execute(&[&OracleType::Object(objtype)]).unwrap();
    let obj: Collection = stmt.bind_value(1).unwrap();

    // first index
    let idx = obj.first_index().unwrap();
    assert_eq!(idx, -1048576);
    let val: String = obj.get(idx).unwrap();
    assert_eq!(val, "First element");

    // second index
    let idx = obj.next_index(idx).unwrap();
    assert_eq!(idx, -576);
    let val: String = obj.get(idx).unwrap();
    assert_eq!(val, "Second element");

    // third index
    let idx = obj.next_index(idx).unwrap();
    assert_eq!(idx, 284);
    let val: String = obj.get(idx).unwrap();
    assert_eq!(val, "Third element");

    // fourth index
    let idx = obj.next_index(idx).unwrap();
    assert_eq!(idx, 8388608);
    let val: String = obj.get(idx).unwrap();
    assert_eq!(val, "Fourth element");

    // out of index
    let err = obj.next_index(idx).unwrap_err();
    assert_eq!(err.to_string(), "No data found");

    // previous indexes from last
    let idx = obj.last_index().unwrap();
    assert_eq!(idx, 8388608);
    let idx = obj.prev_index(idx).unwrap();
    assert_eq!(idx, 284);
    let idx = obj.prev_index(idx).unwrap();
    assert_eq!(idx, -576);
    let idx = obj.prev_index(idx).unwrap();
    assert_eq!(idx, -1048576);
    let err = obj.prev_index(idx).unwrap_err();
    assert_eq!(err.to_string(), "No data found");
}

#[test]
fn sdo_geometry() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("MDSYS.SDO_GEOMETRY").unwrap();
    let expectec_attrs = [
        ["SDO_GTYPE", "NUMBER"],
        ["SDO_SRID", "NUMBER"],
        ["SDO_POINT", "MDSYS.SDO_POINT_TYPE"],
        ["SDO_ELEM_INFO", "MDSYS.SDO_ELEM_INFO_ARRAY"],
        ["SDO_ORDINATES", "MDSYS.SDO_ORDINATE_ARRAY"],
    ];
    for (idx, attr) in objtype.attributes().iter().enumerate() {
        assert_eq!([attr.name(), &attr.oracle_type().to_string()],
                   [expectec_attrs[idx][0], expectec_attrs[idx][1]],
                   "attrs[{}]", idx);
    }
    let oratype = OracleType::Object(objtype);

    // 2.7.1 Rectangle
    // https://docs.oracle.com/database/122/SPATL/spatial-datatypes-metadata.htm#GUID-9354E585-2B45-43EC-95B3-87A3EAA4BB2E
    let text = "MDSYS.SDO_GEOMETRY(2003, NULL, NULL, MDSYS.SDO_ELEM_INFO_ARRAY(1, 1003, 3), MDSYS.SDO_ORDINATE_ARRAY(1, 1, 5, 7))";
    let mut stmt = conn.prepare(&format!("begin :1 := {}; end;", text)).unwrap();
    stmt.execute(&[&oratype]).unwrap();
    let obj: Object = stmt.bind_value(1).unwrap();
    assert_eq!(obj.to_string(), text);

    // 2.7.5 Point
    // https://docs.oracle.com/database/122/SPATL/spatial-datatypes-metadata.htm#GUID-990FC1F2-5EA2-468A-82AC-CDA7B6BEA17D
    let text = "MDSYS.SDO_GEOMETRY(2001, NULL, MDSYS.SDO_POINT_TYPE(12, 14, NULL), NULL, NULL)";
    let mut stmt = conn.prepare(&format!("begin :1 := {}; end;", text)).unwrap();
    stmt.execute(&[&oratype]).unwrap();
    let obj: Object = stmt.bind_value(1).unwrap();
    assert_eq!(obj.to_string(), text);
}
