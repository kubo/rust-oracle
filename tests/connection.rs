// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2019 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

mod common;

use oracle::{ConnStatus, Connector};

#[test]
fn app_context() {
    let conn = Connector::new(
        common::main_user(),
        common::main_password(),
        common::connect_string(),
    )
    .app_context("CLIENTCONTEXT", "foo", "bar")
    .connect()
    .unwrap();
    let val = conn
        .query_row_as::<String>("select sys_context('CLIENTCONTEXT', 'foo') from dual", &[])
        .unwrap();
    assert_eq!(val, "bar");
}

#[test]
fn test_autocommit() {
    let mut conn = common::connect().unwrap();

    conn.execute("truncate table TestTempTable", &[]).unwrap();

    // Autocommit is disabled by default.
    assert_eq!(conn.autocommit(), false);
    conn.execute("insert into TestTempTable values(1, '1')", &[])
        .unwrap();
    conn.rollback().unwrap();
    let (row_count,) = conn
        .query_row_as::<(u32,)>("select count(*) from TestTempTable", &[])
        .unwrap();
    assert_eq!(row_count, 0);

    // Enable autocommit
    conn.set_autocommit(true);
    assert_eq!(conn.autocommit(), true);
    conn.execute("insert into TestTempTable values(1, '1')", &[])
        .unwrap();
    conn.rollback().unwrap();
    let row_count = conn
        .query_row_as::<u32>("select count(*) from TestTempTable", &[])
        .unwrap();
    assert_eq!(row_count, 1);
    conn.execute("delete TestTempTable where IntCol = 1", &[])
        .unwrap();

    // Disable autocommit
    conn.set_autocommit(false);
    assert_eq!(conn.autocommit(), false);
    conn.execute("insert into TestTempTable values(1, '1')", &[])
        .unwrap();
    conn.rollback().unwrap();
    let row_count = conn
        .query_row_as::<u32>("select count(*) from TestTempTable", &[])
        .unwrap();
    assert_eq!(row_count, 0);
}

#[test]
fn execute() {
    let conn = common::connect().unwrap();

    conn.execute("delete from TestTempTable", &[]).unwrap();
    if conn.execute("select * from TestTempTable", &[]).is_ok() {
        panic!("No error for select statements");
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

    let rows = conn
        .query_as_named::<common::TestStringTuple>(sql, &[("icol", &5)])
        .unwrap();
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

    let row = conn
        .query_row_as::<common::TestStringTuple>(sql, &[&4])
        .unwrap();
    common::assert_test_string_tuple(4, &row);

    let row = conn
        .query_row_as_named::<common::TestString>(sql, &[("icol", &5)])
        .unwrap();
    common::assert_test_string_type(5, &row);
}

#[test]
fn status() {
    let conn = common::connect().unwrap();
    assert_eq!(conn.status().unwrap(), ConnStatus::Normal);
    conn.close().unwrap();
    assert_eq!(conn.status().unwrap(), ConnStatus::Closed);
}
