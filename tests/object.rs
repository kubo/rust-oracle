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

mod common;
use oracle::sql_type::{Collection, FromSql, Object, OracleType, Timestamp};
use oracle::{Result, SqlValue};
use std::iter::Iterator;

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
    assert_eq!(
        err.to_string(),
        "OCI Error: OCI-22167: given trim size [1] must be less than or equal to [0]"
    );
    let err = obj.remove(0).unwrap_err();
    assert_eq!(
        err.to_string(),
        "OCI Error: OCI-22160: element at index [0] does not exist"
    );

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
    assert_eq!(
        subobj.get::<Option<String>>("SUBSTRINGVALUE").unwrap(),
        None
    );

    assert_eq!(objtype.to_string(), format!("{}.UDT_NESTEDARRAY", username));
    assert_eq!(
        subobjtype.to_string(),
        format!("{}.UDT_SUBOBJECT", username)
    );
    assert_eq!(
        obj.to_string(),
        format!(
            "{}.UDT_NESTEDARRAY({}.UDT_SUBOBJECT(1, \"STRVAL:1\"), {}.UDT_SUBOBJECT(NULL, NULL))",
            username, username, username
        )
    );
    assert_eq!(
        subobj1.to_string(),
        format!("{}.UDT_SUBOBJECT(1, \"STRVAL:1\")", username)
    );
    assert_eq!(
        subobj2.to_string(),
        format!("{}.UDT_SUBOBJECT(NULL, NULL)", username)
    );

    assert_eq!(
        format!("{:?}", objtype),
        format!(
            "ObjectType({}.UDT_NESTEDARRAY collection of ODPIC.UDT_SUBOBJECT)",
            username
        )
    );
    assert_eq!(
        format!("{:?}", subobjtype),
        format!(
            "ObjectType({}.UDT_SUBOBJECT(SUBNUMBERVALUE NUMBER, SUBSTRINGVALUE VARCHAR2(60)))",
            username
        )
    );
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
    obj.set("DATEVALUE", &Timestamp::new(2012, 3, 4, 5, 6, 7, 0))
        .unwrap();
    obj.set(
        "TIMESTAMPVALUE",
        &Timestamp::new(2017, 2, 3, 4, 5, 6, 123456789),
    )
    .unwrap();
    obj.set("SUBOBJECTVALUE", &subobj).unwrap();
    obj.set("SUBOBJECTARRAY", &objary).unwrap();

    assert_eq!(obj.get::<i32>("NUMBERVALUE").unwrap(), 1);
    assert_eq!(obj.get::<String>("STRINGVALUE").unwrap(), "STRVAL:1");
    assert_eq!(obj.get::<String>("FIXEDCHARVALUE").unwrap(), "CHARVAL:1");
    assert_eq!(
        obj.get::<Timestamp>("DATEVALUE").unwrap(),
        Timestamp::new(2012, 3, 4, 5, 6, 7, 0)
    );
    assert_eq!(
        obj.get::<Timestamp>("TIMESTAMPVALUE").unwrap(),
        Timestamp::new(2017, 2, 3, 4, 5, 6, 123456789)
    );
    assert_eq!(
        obj.get::<Object>("SUBOBJECTVALUE")
            .unwrap()
            .get::<i32>("SUBNUMBERVALUE")
            .unwrap(),
        12
    );
    assert_eq!(
        obj.get::<Collection>("SUBOBJECTARRAY")
            .unwrap()
            .get::<Object>(0)
            .unwrap()
            .get::<i32>("SUBNUMBERVALUE")
            .unwrap(),
        10
    );
    assert_eq!(
        obj.get::<Collection>("SUBOBJECTARRAY")
            .unwrap()
            .get::<Object>(1)
            .unwrap()
            .get::<i32>("SUBNUMBERVALUE")
            .unwrap(),
        11
    );

    let err = subobj.get::<Object>("SUBNUMBERVALUE").unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid type conversion from NUMBER to Object"
    );
    let err = subobj.get::<Collection>("SUBNUMBERVALUE").unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid type conversion from NUMBER to Collection"
    );
}

#[test]
fn udt_stringlist() {
    let conn = common::connect().unwrap();
    if !common::check_oracle_version("udt_stringlist", &conn, 12, 1) {
        return;
    }
    let objtype = conn
        .object_type("PKG_TESTSTRINGARRAYS.UDT_STRINGLIST")
        .unwrap();

    let mut stmt = conn
        .prepare("begin pkg_TestStringArrays.TestIndexBy(:1); end;", &[])
        .unwrap();
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
        assert_eq!(
            [attr.name(), &attr.oracle_type().to_string()],
            [expectec_attrs[idx][0], expectec_attrs[idx][1]],
            "attrs[{}]",
            idx
        );
    }
    let oratype = OracleType::Object(objtype);

    // 2.7.1 Rectangle
    // https://docs.oracle.com/database/122/SPATL/spatial-datatypes-metadata.htm#GUID-9354E585-2B45-43EC-95B3-87A3EAA4BB2E
    let text = "MDSYS.SDO_GEOMETRY(2003, NULL, NULL, MDSYS.SDO_ELEM_INFO_ARRAY(1, 1003, 3), MDSYS.SDO_ORDINATE_ARRAY(1, 1, 5, 7))";
    let mut stmt = conn
        .prepare(&format!("begin :1 := {}; end;", text), &[])
        .unwrap();
    stmt.execute(&[&oratype]).unwrap();
    let obj: Object = stmt.bind_value(1).unwrap();
    assert_eq!(obj.to_string(), text);

    // 2.7.5 Point
    // https://docs.oracle.com/database/122/SPATL/spatial-datatypes-metadata.htm#GUID-990FC1F2-5EA2-468A-82AC-CDA7B6BEA17D
    let text = "MDSYS.SDO_GEOMETRY(2001, NULL, MDSYS.SDO_POINT_TYPE(12, 14, NULL), NULL, NULL)";
    let mut stmt = conn
        .prepare(&format!("begin :1 := {}; end;", text), &[])
        .unwrap();
    stmt.execute(&[&oratype]).unwrap();
    let obj: Object = stmt.bind_value(1).unwrap();
    assert_eq!(obj.to_string(), text);
}

#[derive(Debug, PartialEq)]
struct UdtSubObject {
    sub_number_value: i32,
    sub_string_value: String,
}

impl UdtSubObject {
    fn new(sub_number_value: i32, sub_string_value: String) -> UdtSubObject {
        UdtSubObject {
            sub_number_value: sub_number_value,
            sub_string_value: sub_string_value,
        }
    }

    fn from_oracle_object(obj: Object) -> Result<UdtSubObject> {
        Ok(UdtSubObject {
            sub_number_value: obj.get("SUBNUMBERVALUE")?,
            sub_string_value: obj.get("SUBSTRINGVALUE")?,
        })
    }
}

#[derive(Debug, PartialEq)]
struct UdtObject {
    number_value: i32,
    string_value: String,
    fixed_char_value: String,
    date_value: Timestamp,
    timestamp_value: Timestamp,
    sub_object_value: UdtSubObject,
    sub_object_array: Vec<UdtSubObject>,
}

impl UdtObject {
    fn new(
        number_value: i32,
        string_value: String,
        fixed_char_value: String,
        date_value: Timestamp,
        timestamp_value: Timestamp,
        sub_object_value: UdtSubObject,
        sub_object_array: Vec<UdtSubObject>,
    ) -> UdtObject {
        UdtObject {
            number_value: number_value,
            string_value: string_value,
            fixed_char_value: fixed_char_value,
            date_value: date_value,
            timestamp_value: timestamp_value,
            sub_object_value: sub_object_value,
            sub_object_array: sub_object_array,
        }
    }

    fn from_oracle_object(obj: Object) -> Result<UdtObject> {
        let coll: Collection = obj.get("SUBOBJECTARRAY")?;
        let mut idx_result = coll.first_index();
        let mut vec = Vec::new();
        while let Ok(idx) = idx_result {
            vec.push(UdtSubObject::from_oracle_object(coll.get(idx)?)?);
            idx_result = coll.next_index(idx);
        }
        Ok(UdtObject {
            number_value: obj.get("NUMBERVALUE")?,
            string_value: obj.get("STRINGVALUE")?,
            fixed_char_value: obj.get("FIXEDCHARVALUE")?,
            date_value: obj.get("DATEVALUE")?,
            timestamp_value: obj.get("TIMESTAMPVALUE")?,
            sub_object_value: UdtSubObject::from_oracle_object(obj.get("SUBOBJECTVALUE")?)?,
            sub_object_array: vec,
        })
    }
}

impl FromSql for UdtObject {
    fn from_sql(val: &SqlValue) -> Result<UdtObject> {
        UdtObject::from_oracle_object(val.get()?)
    }
}

#[derive(Debug, PartialEq)]
struct UdtArray {
    val: Vec<Option<i32>>,
}

impl UdtArray {
    fn new(val: Vec<Option<i32>>) -> UdtArray {
        UdtArray { val: val }
    }

    fn from_oracle_object(coll: Collection) -> Result<UdtArray> {
        let mut idx_result = coll.first_index();
        let mut vec = Vec::new();
        while let Ok(idx) = idx_result {
            vec.push(coll.get(idx)?);
            idx_result = coll.next_index(idx);
        }
        Ok(UdtArray::new(vec))
    }
}

impl FromSql for UdtArray {
    fn from_sql(val: &SqlValue) -> Result<UdtArray> {
        UdtArray::from_oracle_object(val.get()?)
    }
}

#[test]
fn select_objects() {
    let conn = common::connect().unwrap();
    let sql = "select * from TestObjects order by IntCol";
    let mut stmt = conn.prepare(sql, &[]).unwrap();
    for (idx, row_result) in stmt
        .query_as::<(usize, Option<UdtObject>, Option<UdtArray>)>(&[])
        .unwrap()
        .enumerate()
    {
        let row = row_result.unwrap();
        assert_eq!(row.0, idx + 1);
        match row.0 {
            1 => {
                assert_eq!(
                    row.1.unwrap(),
                    UdtObject::new(
                        1,
                        "First row".to_string(),
                        "First     ".to_string(),
                        Timestamp::new(2007, 3, 6, 0, 0, 0, 0),
                        Timestamp::new(2008, 9, 12, 16, 40, 0, 0),
                        UdtSubObject::new(11, "Sub object 1".to_string()),
                        vec![
                            UdtSubObject::new(5, "first element".to_string()),
                            UdtSubObject::new(6, "second element".to_string())
                        ]
                    )
                );
                assert_eq!(
                    row.2.unwrap(),
                    UdtArray::new(vec![Some(5), Some(10), None, Some(20)])
                );
            }
            2 => {
                assert!(row.1.is_none());
                assert_eq!(
                    row.2.unwrap(),
                    UdtArray::new(vec![Some(3), None, Some(9), Some(12), Some(15)])
                );
            }
            3 => {
                assert_eq!(
                    row.1.unwrap(),
                    UdtObject::new(
                        3,
                        "Third row".to_string(),
                        "Third     ".to_string(),
                        Timestamp::new(2007, 6, 21, 0, 0, 0, 0),
                        Timestamp::new(2007, 12, 13, 7, 30, 45, 0),
                        UdtSubObject::new(13, "Sub object 3".to_string()),
                        vec![
                            UdtSubObject::new(10, "element #1".to_string()),
                            UdtSubObject::new(20, "element #2".to_string()),
                            UdtSubObject::new(30, "element #3".to_string()),
                            UdtSubObject::new(40, "element #4".to_string())
                        ]
                    )
                );
                assert!(row.2.is_none());
            }
            _ => panic!("Unexpected IntCol value: {}", row.0),
        }
    }
}
