// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017-2018 Kubo Takehiro <kubo@jiubao.org>
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
    let params = [
        oracle::ConnParam::AppContext("CLIENTCONTEXT".into(), "foo".into(), "bar".into()),
    ];
    let conn = oracle::Connection::connect(&common::main_user(), &common::main_password(), &common::connect_string(), &params).unwrap();
    let val = conn.query_row_as::<String>("select sys_context('CLIENTCONTEXT', 'foo') from dual", &[]).unwrap();
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
fn execute() {
    let conn = common::connect().unwrap();

    conn.execute("delete from TestTempTable", &[]).unwrap();
    if cfg!(feature = "restore-deleted") {
        conn.execute("select * from TestTempTable", &[]).expect("error for select statements");
    } else {
        if conn.execute("select * from TestTempTable", &[]).is_ok() {
            panic!("No error for select statements");
        }
    }
}

#[test]
fn query() {
    let conn = common::connect().unwrap();
    let sql = "select * from TestStrings where IntCol >= :icol order by IntCol";

    let rows = conn.query(sql, &[&2]).unwrap();
    for (idx, row_result) in rows.enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_row(idx + 2, &row);
    }

    let rows = conn.query_named(sql, &[("icol", &3)]).unwrap();
    for (idx, row_result) in rows.enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_row(idx + 3, &row);
    }

    let rows = conn.query_as::<common::TestString>(sql, &[&4]).unwrap();
    for (idx, row_result) in rows.enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_type(idx + 4, &row);
    }

    let rows = conn.query_as_named::<common::TestStringTuple>(sql, &[("icol", &5)]).unwrap();
    for (idx, row_result) in rows.enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_tuple(idx + 5, &row);
    }
}

#[test]
fn query_row() {
    let conn = common::connect().unwrap();
    let sql = "select * from TestStrings where IntCol = :icol";

    let row = conn.query_row(sql, &[&2]).unwrap();
    common::assert_test_string_row(2, &row);

    let row = conn.query_row_named(sql, &[("icol", &3)]).unwrap();
    common::assert_test_string_row(3, &row);

    let row = conn.query_row_as::<common::TestStringTuple>(sql, &[&4]).unwrap();
    common::assert_test_string_tuple(4, &row);

    let row = conn.query_row_as_named::<common::TestString>(sql, &[("icol", &5)]).unwrap();
    common::assert_test_string_type(5, &row);
}

