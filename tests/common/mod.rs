use std::env;
use oracle;

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

pub fn connect() -> Result<oracle::Connection, oracle::Error> {
    oracle::Connection::new(&main_user(), &main_password(), &connect_string())
}

macro_rules! test_from_sql {
    ($conn:expr, $column_literal:expr, $column_type:expr, $expected_result:expr) => {
        common::test_from_sql($conn, $column_literal, $column_type, $expected_result, file!(), line!());
    };
}

#[allow(dead_code)]
pub fn test_from_sql<T>(conn: &oracle::Connection, column_literal: &str, column_type: &oracle::OracleType, expected_result: &T, file: &str, line: u32) where T: oracle::FromSql + ::std::fmt::Debug + ::std::cmp::PartialEq {
    let mut stmt = conn.prepare(&format!("select {} from dual", column_literal)).unwrap();
    stmt.execute().unwrap();
    assert_eq!(stmt.column_info()[0].oracle_type(), column_type, "called by {}:{}", file, line);
    let row = stmt.fetch().unwrap();
    let result = row.get::<usize,T>(0).unwrap();
    assert_eq!(&result, expected_result, "called by {}:{}", file, line);
}

macro_rules! test_to_sql {
    ($conn:expr, $input_data:expr, $input_literal:expr, $input_type:expr, $expected_result:expr) => {
        common::test_to_sql($conn, $input_data, $input_literal, $input_type, $expected_result, file!(), line!());
    };
}

#[allow(dead_code)]
pub fn test_to_sql<T>(conn: &oracle::Connection, input_data: T, input_literal: &str, input_type: &oracle::OracleType, expected_result: &str, file: &str, line: u32) where T: oracle::ToSql {
    let mut stmt = conn.prepare(&format!("begin :out := {}; end;", input_literal)).unwrap();
    stmt.bind(1, &oracle::OracleType::Varchar2(1000)).unwrap();
    stmt.bind(2, input_type).unwrap();
    stmt.set_bind_value(2, input_data).unwrap();
    stmt.execute().unwrap();
    let result: String = stmt.bind_value(1).unwrap();
    assert_eq!(&result, expected_result, "called by {}:{}", file, line);
}
