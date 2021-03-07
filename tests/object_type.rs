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

use oracle::sql_type::{ObjectType, OracleType};
use oracle::Version;

#[test]
fn invalid_obj() {
    let conn = common::connect().unwrap();
    let err = conn.object_type("DUMMY_OBJECT").unwrap_err();
    if Version::client().unwrap().major() >= 12 {
        assert_eq!(
            err.to_string(),
            "OCI Error: OCI-22303: type \"\".\"DUMMY_OBJECT\" not found"
        );
    } else {
        assert_eq!(
            err.to_string(),
            "OCI Error: ORA-04043: object DUMMY_OBJECT does not exist"
        );
    }
}

#[test]
fn udt_objectdatatypes() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("UDT_OBJECTDATATYPES").unwrap();

    assert_udt_objectdatatypes(&objtype);
}

#[test]
fn udt_objectdatatypes_in_query() {
    let conn = common::connect().unwrap();
    let mut stmt = conn
        .prepare("select ObjectCol from TestObjectDataTypes where 1 = 0", &[])
        .unwrap();
    let rows = stmt.query(&[]).unwrap();
    match rows.column_info()[0].oracle_type() {
        &OracleType::Object(ref objtype) => assert_udt_objectdatatypes(objtype),
        _ => assert!(false),
    }
}

fn assert_udt_objectdatatypes(objtype: &ObjectType) {
    let username = common::main_user().to_uppercase();
    let expected_attrs = [
        ("STRINGCOL", OracleType::Varchar2(60)),
        ("UNICODECOL", OracleType::NVarchar2(60)),
        ("FIXEDCHARCOL", OracleType::Char(30)),
        ("FIXEDUNICODECOL", OracleType::NChar(30)),
        ("RAWCOL", OracleType::Raw(30)),
        ("INTCOL", OracleType::Number(0, -127)),
        ("NUMBERCOL", OracleType::Number(9, 2)),
        ("DATECOL", OracleType::Date),
        ("TIMESTAMPCOL", OracleType::Timestamp(6)),
        ("TIMESTAMPTZCOL", OracleType::TimestampTZ(6)),
        ("TIMESTAMPLTZCOL", OracleType::TimestampLTZ(6)),
        ("BINARYFLTCOL", OracleType::BinaryFloat),
        ("BINARYDOUBLECOL", OracleType::BinaryDouble),
        ("SIGNEDINTCOL", OracleType::Number(38, 0)),
    ];

    assert_eq!(objtype.schema(), username);
    assert_eq!(objtype.name(), "UDT_OBJECTDATATYPES");
    assert_eq!(objtype.is_collection(), false);
    assert_eq!(objtype.element_oracle_type(), None);
    assert_eq!(objtype.num_attributes(), expected_attrs.len());
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), expected_attrs.len());
    for (attr, expected_attr) in attrs.iter().zip(expected_attrs.iter()) {
        assert_eq!(attr.name(), expected_attr.0);
        assert_eq!(attr.oracle_type(), &expected_attr.1);
    }
}

#[test]
fn udt_object() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("UDT_OBJECT").unwrap();
    assert_udt_object(&objtype);
}

#[test]
fn udt_object_in_query() {
    let conn = common::connect().unwrap();
    let mut stmt = conn
        .prepare("select ObjectCol from TestObjects where 1 = 0", &[])
        .unwrap();
    let rows = stmt.query(&[]).unwrap();
    match rows.column_info()[0].oracle_type() {
        &OracleType::Object(ref objtype) => assert_udt_object(objtype),
        _ => assert!(false),
    }
}

fn assert_udt_object(objtype: &ObjectType) {
    let username = common::main_user().to_uppercase();

    assert_eq!(objtype.schema(), username);
    assert_eq!(objtype.name(), "UDT_OBJECT");
    assert_eq!(objtype.is_collection(), false);
    assert_eq!(objtype.element_oracle_type(), None);
    assert_eq!(objtype.num_attributes(), 7);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 7);

    assert_eq!(attrs[0].name(), "NUMBERVALUE");
    assert_eq!(attrs[0].oracle_type(), &OracleType::Number(0, -127));

    assert_eq!(attrs[1].name(), "STRINGVALUE");
    assert_eq!(attrs[1].oracle_type(), &OracleType::Varchar2(60));

    assert_eq!(attrs[2].name(), "FIXEDCHARVALUE");
    assert_eq!(attrs[2].oracle_type(), &OracleType::Char(10));

    assert_eq!(attrs[3].name(), "DATEVALUE");
    assert_eq!(attrs[3].oracle_type(), &OracleType::Date);

    assert_eq!(attrs[4].name(), "TIMESTAMPVALUE");
    assert_eq!(attrs[4].oracle_type(), &OracleType::Timestamp(6));

    assert_eq!(attrs[5].name(), "SUBOBJECTVALUE");
    assert_udt_subobject(attrs[5].oracle_type());

    assert_eq!(attrs[6].name(), "SUBOBJECTARRAY");
    match *attrs[6].oracle_type() {
        OracleType::Object(ref attrtype) => {
            assert_eq!(attrtype.schema(), username);
            assert_eq!(attrtype.name(), "UDT_OBJECTARRAY");
            assert_eq!(attrtype.is_collection(), true);
            match attrtype.element_oracle_type() {
                Some(elem_type) => assert_udt_subobject(elem_type),
                None => assert!(false),
            }
            assert_eq!(attrtype.num_attributes(), 0);
            assert_eq!(attrtype.attributes().len(), 0);
        }
        _ => assert!(false),
    }
}

fn assert_udt_subobject(oratype: &OracleType) {
    let username = common::main_user().to_uppercase();

    match *oratype {
        OracleType::Object(ref attrtype) => {
            assert_eq!(attrtype.schema(), username);
            assert_eq!(attrtype.name(), "UDT_SUBOBJECT");
            assert_eq!(attrtype.is_collection(), false);
            assert_eq!(attrtype.element_oracle_type(), None);
            assert_eq!(attrtype.num_attributes(), 2);
            let attrs_in_attr = attrtype.attributes();
            assert_eq!(attrs_in_attr.len(), 2);
            assert_eq!(attrs_in_attr[0].name(), "SUBNUMBERVALUE");
            assert_eq!(attrs_in_attr[0].oracle_type(), &OracleType::Number(0, -127));
            assert_eq!(attrs_in_attr[1].name(), "SUBSTRINGVALUE");
            assert_eq!(attrs_in_attr[1].oracle_type(), &OracleType::Varchar2(60));
        }
        _ => assert!(false),
    }
}

#[test]
fn udt_array() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("UDT_ARRAY").unwrap();
    let username = common::main_user().to_uppercase();

    assert_eq!(objtype.schema(), username);
    assert_eq!(objtype.name(), "UDT_ARRAY");
    assert_eq!(objtype.is_collection(), true);
    assert_eq!(
        objtype.element_oracle_type(),
        Some(&OracleType::Number(0, -127))
    );
    assert_eq!(objtype.num_attributes(), 0);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 0);
}

#[test]
fn pkg_testnumberarrays_udt_numberlist() {
    let conn = common::connect().unwrap();
    if !common::check_oracle_version("pkg_testnumberarrays_udt_numberlist", &conn, 12, 1) {
        return;
    }
    let objtype = conn
        .object_type("PKG_TESTNUMBERARRAYS.UDT_NUMBERLIST")
        .unwrap();
    let username = common::main_user().to_uppercase();

    assert_eq!(objtype.schema(), username);
    assert_eq!(objtype.name(), "UDT_NUMBERLIST");
    assert_eq!(objtype.is_collection(), true);
    assert_eq!(
        objtype.element_oracle_type(),
        Some(&OracleType::Number(0, -127))
    );
    assert_eq!(objtype.num_attributes(), 0);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 0);
}

#[test]
fn pkg_testrecords_udt_record() {
    let conn = common::connect().unwrap();
    if !common::check_oracle_version("pkg_testrecords_udt_record", &conn, 12, 1) {
        return;
    }
    let objtype = conn.object_type("PKG_TESTRECORDS.UDT_RECORD").unwrap();
    let username = common::main_user().to_uppercase();

    assert_eq!(objtype.schema(), username);
    assert_eq!(objtype.name(), "UDT_RECORD");
    assert_eq!(objtype.is_collection(), false);
    assert_eq!(objtype.element_oracle_type(), None);
    assert_eq!(objtype.num_attributes(), 7);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 7);

    assert_eq!(attrs[0].name(), "NUMBERVALUE");
    assert_eq!(attrs[0].oracle_type(), &OracleType::Number(0, -127));

    assert_eq!(attrs[1].name(), "STRINGVALUE");
    assert_eq!(attrs[1].oracle_type(), &OracleType::Varchar2(30));

    assert_eq!(attrs[2].name(), "DATEVALUE");
    assert_eq!(attrs[2].oracle_type(), &OracleType::Date);

    assert_eq!(attrs[3].name(), "TIMESTAMPVALUE");
    assert_eq!(attrs[3].oracle_type(), &OracleType::Timestamp(6));

    assert_eq!(attrs[4].name(), "BOOLEANVALUE");
    assert_eq!(attrs[4].oracle_type(), &OracleType::Boolean);

    assert_eq!(attrs[5].name(), "PLSINTEGERVALUE");
    assert_eq!(attrs[5].oracle_type(), &OracleType::Int64);

    assert_eq!(attrs[6].name(), "BINARYINTEGERVALUE");
    assert_eq!(attrs[6].oracle_type(), &OracleType::Int64);
}

#[test]
fn object_type_cache() {
    let conn = common::connect().unwrap();

    conn.object_type("UDT_OBJECTDATATYPES").unwrap();
    assert_eq!(conn.object_type_cache_len(), 1);

    conn.object_type("UDT_SUBOBJECT").unwrap();
    assert_eq!(conn.object_type_cache_len(), 2);

    conn.object_type("UDT_SUBOBJECT").unwrap();
    assert_eq!(conn.object_type_cache_len(), 2);

    // explicitly clear the cache.
    conn.clear_object_type_cache().unwrap();
    assert_eq!(conn.object_type_cache_len(), 0);

    conn.object_type("UDT_SUBOBJECT").unwrap();
    assert_eq!(conn.object_type_cache_len(), 1);

    // "CREATE TYPE" clears the cache.
    conn.execute(
        "create type rust_oracle_test as object (intval number);",
        &[],
    )
    .unwrap();
    assert_eq!(conn.object_type_cache_len(), 0);

    conn.object_type("RUST_ORACLE_TEST").unwrap();
    assert_eq!(conn.object_type_cache_len(), 1);

    if common::check_oracle_version("object_type_cache", &conn, 12, 1) {
        // "ALTER TYPE" clears the cache.
        conn.execute(
            "alter type rust_oracle_test add attribute (strval varchar2(100));",
            &[],
        )
        .unwrap();
        assert_eq!(conn.object_type_cache_len(), 0);

        // The next line fails with 'ORA-22337: the type of accessed
        // object has been evolved' when the Oracle client version is
        // 11.2.
        conn.object_type("RUST_ORACLE_TEST").unwrap();
        assert_eq!(conn.object_type_cache_len(), 1);
    }

    // "DROP TYPE" clears the cache.
    conn.execute("drop type rust_oracle_test", &[]).unwrap();
    assert_eq!(conn.object_type_cache_len(), 0);
}

#[test]
fn udt_issue19() {
    let conn = common::connect().unwrap();
    let float_val: f64 = 1.25;

    let objtype = conn.object_type("UDT_ISSUE19_OBJ").unwrap();
    let mut obj = objtype.new_object().unwrap();
    obj.set("FLOATCOL", &float_val).unwrap();
    assert_eq!(float_val, obj.get::<f64>("FLOATCOL").unwrap());

    let objtype = conn.object_type("UDT_ISSUE19_COL").unwrap();
    let mut coll = objtype.new_collection().unwrap();
    coll.push(&float_val).unwrap();
    assert_eq!(float_val, coll.get::<f64>(0).unwrap());
}
