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
fn app_context() {
    let mut connector = oracle::Connector::new(&common::main_user(), &common::main_password(), &common::connect_string());
    connector.app_context("CLIENTCONTEXT", "foo", "bar");
    let conn = connector.connect().unwrap();
    let mut stmt = conn.execute("select sys_context('CLIENTCONTEXT', 'foo') from dual", &[]).unwrap();
    let row = stmt.fetch().unwrap();
    let val: String = row.get(0).unwrap();
    assert_eq!(val, "bar");
}

#[test]
fn test_autocommit() {
    let mut conn = common::connect().unwrap();

    conn.execute("truncate table TestTempTable", &[]).unwrap();

    // Autocommit is disabled by default.
    assert_eq!(conn.autocommit(), false);
    conn.execute("insert into TestTempTable values(1, '1')", &[]).unwrap();
    conn.rollback().unwrap();
    let (row_count,) = conn.query_row_as::<(u32,)>("select count(*) from TestTempTable", &[]).unwrap();
    assert_eq!(row_count, 0);

    // Enable autocommit
    conn.set_autocommit(true);
    assert_eq!(conn.autocommit(), true);
    conn.execute("insert into TestTempTable values(1, '1')", &[]).unwrap();
    conn.rollback().unwrap();
    let row_count = conn.query_row_as::<(u32)>("select count(*) from TestTempTable", &[]).unwrap();
    assert_eq!(row_count, 1);
    conn.execute("delete TestTempTable where IntCol = 1", &[]).unwrap();

    // Disable autocommit
    conn.set_autocommit(false);
    assert_eq!(conn.autocommit(), false);
    conn.execute("insert into TestTempTable values(1, '1')", &[]).unwrap();
    conn.rollback().unwrap();
    let row_count = conn.query_row_as::<u32>("select count(*) from TestTempTable", &[]).unwrap();
    assert_eq!(row_count, 0);
}

#[test]
fn query_row() {
    let conn = common::connect().unwrap();
    let sql_stmt = "select IntCol from TestStrings where IntCol = :val";

    let row = conn.query_row(sql_stmt, &[&2]).unwrap();
    let int_col: i32 = row.get(0).unwrap();
    assert_eq!(int_col, 2);

    let row = conn.query_row_named(sql_stmt, &[("val", &3)]).unwrap();
    let int_col: i32 = row.get(0).unwrap();
    assert_eq!(int_col, 3);
}

#[test]
fn query_row_as() {
    let conn = common::connect().unwrap();

    let result = conn.query_row_as::<(String, i32, String)>("select '0', 1, '2' from dual", &[]).unwrap();
    assert_eq!(result.0, "0");
    assert_eq!(result.1, 1);
    assert_eq!(result.2, "2");

    let result = conn.query_row_as::<common::TestString>("select * from TestStrings where IntCol = 1", &[]).unwrap();
    assert_eq!(result.int_col, 1);
    assert_eq!(result.string_col, "String 1");
    assert_eq!(result.raw_col, b"Raw 1");
    assert_eq!(result.fixed_char_col, "Fixed Char 1                            ");
    assert_eq!(result.nullable_col, Some("Nullable 1".to_string()));

    let result = conn.query_row_as_named::<common::TestString>("select * from TestStrings where IntCol = :intcol", &[("intcol", &2)]).unwrap();
    assert_eq!(result.int_col, 2);
    assert_eq!(result.string_col, "String 2");
    assert_eq!(result.raw_col, b"Raw 2");
    assert_eq!(result.fixed_char_col, "Fixed Char 2                            ");
    assert_eq!(result.nullable_col, None);
}

