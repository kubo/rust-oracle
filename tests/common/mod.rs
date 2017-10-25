// Rust Oracle - Rust binding for Oracle database
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

#[allow(dead_code)]
pub fn connect() -> Result<oracle::Connection, oracle::Error> {
    oracle::Connection::new(&main_user(), &main_password(), &connect_string())
}

#[allow(unused_macros)]
macro_rules! test_from_sql {
    ($conn:expr, $column_literal:expr, $column_type:expr, $expected_result:expr) => {
        common::test_from_sql($conn, $column_literal, $column_type, $expected_result, file!(), line!());
    };
}

#[allow(dead_code)]
pub fn test_from_sql<T>(conn: &oracle::Connection, column_literal: &str, column_type: &oracle::OracleType, expected_result: &T, file: &str, line: u32) where T: oracle::FromSql + ::std::fmt::Debug + ::std::cmp::PartialEq {
    let mut stmt = conn.prepare(&format!("select {} from dual", column_literal)).unwrap();
    stmt.execute(&()).expect(format!("error at {}:{}", file, line).as_str());
    assert_eq!(stmt.column_info()[0].oracle_type(), column_type, "called by {}:{}", file, line);
    let row = stmt.fetch().unwrap();
    let result: T = row.get(0).unwrap();
    assert_eq!(&result, expected_result, "called by {}:{}", file, line);
}

#[allow(unused_macros)]
macro_rules! test_to_sql {
    ($conn:expr, $input_data:expr, $input_literal:expr, $expected_result:expr) => {
        common::test_to_sql($conn, $input_data, $input_literal, $expected_result, file!(), line!());
    };
}

#[allow(dead_code)]
pub fn test_to_sql<T>(conn: &oracle::Connection, input_data: T, input_literal: &str, expected_result: &str, file: &str, line: u32) where T: oracle::ToSql {
    let mut stmt = conn.prepare(&format!("begin :out := {}; end;", input_literal)).unwrap();
    stmt.bind(1, &oracle::bind_value(&None::<&str>, 4000)).unwrap();
    stmt.bind(2, input_data).unwrap();
    stmt.execute(&()).expect(format!("error at {}:{}", file, line).as_str());
    let result: String = stmt.bind_value(1).unwrap();
    assert_eq!(&result, expected_result, "called by {}:{}", file, line);
}
