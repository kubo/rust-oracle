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

#[test]
fn invalid_obj() {
    let conn = common::connect().unwrap();
    let err = conn.object_type("DUMMY_OBJECT").unwrap_err();
    if oracle::client_version().unwrap().major() >= 12 {
        assert_eq!(err.to_string(), "OCI Error: OCI-22303: type \"\".\"DUMMY_OBJECT\" not found");
    } else {
        assert_eq!(err.to_string(), "OCI Error: ORA-04043: object DUMMY_OBJECT does not exist");
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
    let stmt = conn.execute("select ObjectCol from TestObjectDataTypes where 1 = 0", &[]).unwrap();
    match *stmt.column_info()[0].oracle_type() {
        oracle::OracleType::Object(ref objtype) =>
            assert_udt_objectdatatypes(objtype),
        _ => assert!(false),
    }
}

fn assert_udt_objectdatatypes(objtype: &oracle::ObjectType) {
    let username = common::main_user().to_uppercase();

    assert_eq!(*objtype.schema(), username);
    assert_eq!(*objtype.name(), "UDT_OBJECTDATATYPES");
    assert_eq!(objtype.is_collection(), false);
    assert_eq!(objtype.element_oracle_type(), None);
    assert_eq!(objtype.num_attributes(), 12);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 12);

    assert_eq!(*attrs[0].name(), "STRINGCOL");
    assert_eq!(*attrs[0].oracle_type(), oracle::OracleType::Varchar2(60));

    assert_eq!(*attrs[1].name(), "UNICODECOL");
    assert_eq!(*attrs[1].oracle_type(), oracle::OracleType::NVarchar2(60));

    assert_eq!(*attrs[2].name(), "FIXEDCHARCOL");
    assert_eq!(*attrs[2].oracle_type(), oracle::OracleType::Char(30));

    assert_eq!(*attrs[3].name(), "FIXEDUNICODECOL");
    assert_eq!(*attrs[3].oracle_type(), oracle::OracleType::NChar(30));

    assert_eq!(*attrs[4].name(), "INTCOL");
    assert_eq!(*attrs[4].oracle_type(), oracle::OracleType::Number(0, -127));

    assert_eq!(*attrs[5].name(), "NUMBERCOL");
    assert_eq!(*attrs[5].oracle_type(), oracle::OracleType::Number(9, 2));

    assert_eq!(*attrs[6].name(), "DATECOL");
    assert_eq!(*attrs[6].oracle_type(), oracle::OracleType::Date);

    assert_eq!(*attrs[7].name(), "TIMESTAMPCOL");
    assert_eq!(*attrs[7].oracle_type(), oracle::OracleType::Timestamp(6));

    assert_eq!(*attrs[8].name(), "TIMESTAMPTZCOL");
    assert_eq!(*attrs[8].oracle_type(), oracle::OracleType::TimestampTZ(6));

    assert_eq!(*attrs[9].name(), "TIMESTAMPLTZCOL");
    assert_eq!(*attrs[9].oracle_type(), oracle::OracleType::TimestampLTZ(6));

    assert_eq!(*attrs[10].name(), "BINARYFLTCOL");
    assert_eq!(*attrs[10].oracle_type(), oracle::OracleType::BinaryFloat);

    assert_eq!(*attrs[11].name(), "BINARYDOUBLECOL");
    assert_eq!(*attrs[11].oracle_type(), oracle::OracleType::BinaryDouble);
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
    let stmt = conn.execute("select ObjectCol from TestObjects where 1 = 0", &[]).unwrap();
    match *stmt.column_info()[0].oracle_type() {
        oracle::OracleType::Object(ref objtype) =>
            assert_udt_object(objtype),
        _ => assert!(false),
    }
}

fn assert_udt_object(objtype: &oracle::ObjectType) {
    let username = common::main_user().to_uppercase();

    assert_eq!(*objtype.schema(), username);
    assert_eq!(*objtype.name(), "UDT_OBJECT");
    assert_eq!(objtype.is_collection(), false);
    assert_eq!(objtype.element_oracle_type(), None);
    assert_eq!(objtype.num_attributes(), 7);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 7);

    assert_eq!(*attrs[0].name(), "NUMBERVALUE");
    assert_eq!(*attrs[0].oracle_type(), oracle::OracleType::Number(0, -127));

    assert_eq!(*attrs[1].name(), "STRINGVALUE");
    assert_eq!(*attrs[1].oracle_type(), oracle::OracleType::Varchar2(60));

    assert_eq!(*attrs[2].name(), "FIXEDCHARVALUE");
    assert_eq!(*attrs[2].oracle_type(), oracle::OracleType::Char(10));

    assert_eq!(*attrs[3].name(), "DATEVALUE");
    assert_eq!(*attrs[3].oracle_type(), oracle::OracleType::Date);

    assert_eq!(*attrs[4].name(), "TIMESTAMPVALUE");
    assert_eq!(*attrs[4].oracle_type(), oracle::OracleType::Timestamp(6));

    assert_eq!(*attrs[5].name(), "SUBOBJECTVALUE");
    assert_udt_subobject(attrs[5].oracle_type());

    assert_eq!(*attrs[6].name(), "SUBOBJECTARRAY");
    match *attrs[6].oracle_type() {
        oracle::OracleType::Object(ref attrtype) => {
            assert_eq!(*attrtype.schema(), username);
            assert_eq!(*attrtype.name(), "UDT_OBJECTARRAY");
            assert_eq!(attrtype.is_collection(), true);
            match attrtype.element_oracle_type() {
                Some(elem_type) => assert_udt_subobject(elem_type),
                None => assert!(false),
            }
            assert_eq!(attrtype.num_attributes(), 0);
            assert_eq!(attrtype.attributes().len(), 0);
        },
        _ => assert!(false),
    }
}

fn assert_udt_subobject(oratype: &oracle::OracleType) {
    let username = common::main_user().to_uppercase();

    match *oratype {
        oracle::OracleType::Object(ref attrtype) => {
            assert_eq!(*attrtype.schema(), username);
            assert_eq!(*attrtype.name(), "UDT_SUBOBJECT");
            assert_eq!(attrtype.is_collection(), false);
            assert_eq!(attrtype.element_oracle_type(), None);
            assert_eq!(attrtype.num_attributes(), 2);
            let attrs_in_attr = attrtype.attributes();
            assert_eq!(attrs_in_attr.len(), 2);
            assert_eq!(*attrs_in_attr[0].name(), "SUBNUMBERVALUE");
            assert_eq!(*attrs_in_attr[0].oracle_type(), oracle::OracleType::Number(0, -127));
            assert_eq!(*attrs_in_attr[1].name(), "SUBSTRINGVALUE");
            assert_eq!(*attrs_in_attr[1].oracle_type(), oracle::OracleType::Varchar2(60));
        },
        _ => assert!(false),
    }
}

#[test]
fn udt_array() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("UDT_ARRAY").unwrap();
    let username = common::main_user().to_uppercase();

    assert_eq!(*objtype.schema(), username);
    assert_eq!(*objtype.name(), "UDT_ARRAY");
    assert_eq!(objtype.is_collection(), true);
    assert_eq!(objtype.element_oracle_type(), Some(&oracle::OracleType::Number(0, -127)));
    assert_eq!(objtype.num_attributes(), 0);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 0);
}


#[test]
fn pkg_testnumberarrays_udt_numberlist() {
    if oracle::client_version().unwrap().major() < 12 {
        return;
    }
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("PKG_TESTNUMBERARRAYS.UDT_NUMBERLIST").unwrap();
    let username = common::main_user().to_uppercase();

    assert_eq!(*objtype.schema(), username);
    assert_eq!(*objtype.name(), "UDT_NUMBERLIST");
    assert_eq!(objtype.is_collection(), true);
    assert_eq!(objtype.element_oracle_type(), Some(&oracle::OracleType::Number(0, -127)));
    assert_eq!(objtype.num_attributes(), 0);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 0);
}

#[test]
fn pkg_testrecords_udt_record() {
    let conn = common::connect().unwrap();
    let objtype = conn.object_type("PKG_TESTRECORDS.UDT_RECORD").unwrap();
    let username = common::main_user().to_uppercase();

    assert_eq!(*objtype.schema(), username);
    assert_eq!(*objtype.name(), "UDT_RECORD");
    assert_eq!(objtype.is_collection(), false);
    assert_eq!(objtype.element_oracle_type(), None);
    assert_eq!(objtype.num_attributes(), 5);
    let attrs = objtype.attributes();
    assert_eq!(attrs.len(), 5);

    assert_eq!(*attrs[0].name(), "NUMBERVALUE");
    assert_eq!(*attrs[0].oracle_type(), oracle::OracleType::Number(0, -127));

    assert_eq!(*attrs[1].name(), "STRINGVALUE");
    assert_eq!(*attrs[1].oracle_type(), oracle::OracleType::Varchar2(30));

    assert_eq!(*attrs[2].name(), "DATEVALUE");
    assert_eq!(*attrs[2].oracle_type(), oracle::OracleType::Date);

    assert_eq!(*attrs[3].name(), "TIMESTAMPVALUE");
    assert_eq!(*attrs[3].oracle_type(), oracle::OracleType::Timestamp(6));

    assert_eq!(*attrs[4].name(), "BOOLEANVALUE");
    assert_eq!(*attrs[4].oracle_type(), oracle::OracleType::Boolean);
}
