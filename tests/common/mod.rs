// Rust Oracle - Rust binding for Oracle database
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

use oracle::sql_type::{FromSql, OracleType, ToSql};
use oracle::{Connection, Error, Row, RowValue, Version};
use std::env;

fn env_var_or(env_name: &str, default: &str) -> String {
    match env::var_os(env_name) {
        Some(env_var) => env_var.into_string().unwrap(),
        None => String::from(default),
    }
}

pub fn main_user() -> String {
    env_var_or("ODPIC_TEST_MAIN_USER", "odpic")
}

pub fn main_password() -> String {
    env_var_or("ODPIC_TEST_MAIN_PASSWORD", "welcome")
}

#[allow(dead_code)]
pub fn proxy_user() -> String {
    env_var_or("ODPIC_TEST_PROXY_USER", "odpic_proxy")
}

#[allow(dead_code)]
pub fn proxy_password() -> String {
    env_var_or("ODPIC_TEST_PROXY_PASSWORD", "welcome")
}

pub fn connect_string() -> String {
    env_var_or("ODPIC_TEST_CONNECT_STRING", "localhost/orclpdb")
}

#[allow(dead_code)]
pub fn dir_name() -> String {
    env_var_or("ODPIC_TEST_DIR_NAME", "odpic_dir")
}

#[allow(dead_code)]
pub fn connect() -> Result<Connection, Error> {
    Connection::connect(&main_user(), &main_password(), &connect_string())
}

#[allow(dead_code)]
pub fn check_oracle_version(test_name: &str, conn: &Connection, major: i32, minor: i32) -> bool {
    let ver = Version::new(major, minor, 0, 0, 0);
    let client_ver = Version::client().unwrap();
    let server_ver = conn.server_version().unwrap();
    if client_ver >= ver && server_ver.0 >= ver {
        true
    } else {
        println!(
            "Skip {}, which requires an Oracle {}.{} feature.",
            test_name,
            ver.major(),
            ver.minor()
        );
        false
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! test_from_sql {
    ($conn:expr, $column_literal:expr, $column_type:expr, $expected_result:expr) => {
        common::test_from_sql(
            $conn,
            $column_literal,
            $column_type,
            $expected_result,
            file!(),
            line!(),
        );
    };
}

#[allow(dead_code)]
pub fn test_from_sql<T>(
    conn: &Connection,
    column_literal: &str,
    column_type: &OracleType,
    expected_result: &T,
    file: &str,
    line: u32,
) where
    T: FromSql + ::std::fmt::Debug + ::std::cmp::PartialEq,
{
    let mut stmt = conn
        .prepare(&format!("select {} from dual", column_literal), &[])
        .unwrap();
    let mut rows = stmt
        .query_as::<T>(&[])
        .expect(format!("error at {}:{}", file, line).as_str());
    assert_eq!(
        rows.column_info()[0].oracle_type(),
        column_type,
        "called by {}:{}",
        file,
        line
    );
    let result = rows.next().unwrap().unwrap();
    assert_eq!(&result, expected_result, "called by {}:{}", file, line);
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! test_to_sql {
    ($conn:expr, $input_data:expr, $input_literal:expr, $expected_result:expr) => {
        common::test_to_sql(
            $conn,
            $input_data,
            $input_literal,
            $expected_result,
            file!(),
            line!(),
        );
    };
}

#[allow(dead_code)]
pub fn test_to_sql<T>(
    conn: &Connection,
    input_data: &T,
    input_literal: &str,
    expected_result: &str,
    file: &str,
    line: u32,
) where
    T: ToSql,
{
    let mut stmt = conn
        .prepare(&format!("begin :out := {}; end;", input_literal), &[])
        .unwrap();
    stmt.bind(1, &OracleType::Varchar2(4000)).unwrap();
    stmt.bind(2, input_data).unwrap();
    stmt.execute(&[])
        .expect(format!("error at {}:{}", file, line).as_str());
    let result: String = stmt.bind_value(1).unwrap();
    assert_eq!(&result, expected_result, "called by {}:{}", file, line);
}

#[allow(dead_code)]
pub struct TestString {
    pub int_col: i32,
    pub string_col: String,
    pub raw_col: Vec<u8>,
    pub fixed_char_col: String,
    pub nullable_col: Option<String>,
}

impl RowValue for TestString {
    fn get(row: &Row) -> Result<TestString, Error> {
        Ok(TestString {
            int_col: row.get(0)?,
            string_col: row.get(1)?,
            raw_col: row.get(2)?,
            fixed_char_col: row.get(3)?,
            nullable_col: row.get(4)?,
        })
    }
}

#[allow(dead_code)]
const VALUES_IN_TEST_STRINGS: [(i32, &str, &[u8], &str, Option<&str>); 11] = [
    (0, "", b"", "", None),
    (
        1,
        "String 1",
        b"Raw 1",
        "Fixed Char 1                            ",
        Some("Nullable 1"),
    ),
    (
        2,
        "String 2",
        b"Raw 2",
        "Fixed Char 2                            ",
        None,
    ),
    (
        3,
        "String 3",
        b"Raw 3",
        "Fixed Char 3                            ",
        Some("Nullable 3"),
    ),
    (
        4,
        "String 4",
        b"Raw 4",
        "Fixed Char 4                            ",
        None,
    ),
    (
        5,
        "String 5",
        b"Raw 5",
        "Fixed Char 5                            ",
        Some("Nullable 5"),
    ),
    (
        6,
        "String 6",
        b"Raw 6",
        "Fixed Char 6                            ",
        None,
    ),
    (
        7,
        "String 7",
        b"Raw 7",
        "Fixed Char 7                            ",
        Some("Nullable 7"),
    ),
    (
        8,
        "String 8",
        b"Raw 8",
        "Fixed Char 8                            ",
        None,
    ),
    (
        9,
        "String 9",
        b"Raw 9",
        "Fixed Char 9                            ",
        Some("Nullable 9"),
    ),
    (
        10,
        "String 10",
        b"Raw 10",
        "Fixed Char 10                           ",
        None,
    ),
];

#[allow(dead_code)]
pub type TestStringTuple = (i32, String, Vec<u8>, String, Option<String>);

#[allow(dead_code)]
pub fn assert_test_string_type(idx: usize, row: &TestString) {
    let row = (
        row.int_col,
        row.string_col.clone(),
        row.raw_col.clone(),
        row.fixed_char_col.clone(),
        row.nullable_col.clone(),
    );
    assert_test_string_tuple(idx, &row);
}

#[allow(dead_code)]
pub fn assert_test_string_row(idx: usize, row: &Row) {
    let row = row.get_as::<TestStringTuple>().unwrap();
    assert_test_string_tuple(idx, &row);
}

#[allow(dead_code)]
pub fn assert_test_string_tuple(idx: usize, row: &TestStringTuple) {
    assert_eq!(row.0, VALUES_IN_TEST_STRINGS[idx].0);
    assert_eq!(row.1, VALUES_IN_TEST_STRINGS[idx].1);
    assert_eq!(row.2, VALUES_IN_TEST_STRINGS[idx].2);
    assert_eq!(row.3, VALUES_IN_TEST_STRINGS[idx].3);
    assert_eq!(row.4, VALUES_IN_TEST_STRINGS[idx].4.map(|s| s.to_string()));
}
