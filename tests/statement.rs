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

use oracle::sql_type::{IntervalDS, Timestamp};
use oracle::{StatementType, StmtParam};

#[test]
fn statement_type() {
    let conn = common::connect().unwrap();

    let stmt = conn.prepare("SELECT ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Select);
    assert_eq!(stmt_type.to_string(), "select");
    assert_eq!(stmt.is_query(), true);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("INSERT ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Insert);
    assert_eq!(stmt_type.to_string(), "insert");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), true);

    let stmt = conn.prepare("UPDATE ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Update);
    assert_eq!(stmt_type.to_string(), "update");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), true);

    let stmt = conn.prepare("DELETE ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Delete);
    assert_eq!(stmt_type.to_string(), "delete");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), true);

    let stmt = conn.prepare("MERGE ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Merge);
    assert_eq!(stmt_type.to_string(), "merge");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), true);

    let stmt = conn.prepare("CREATE ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Create);
    assert_eq!(stmt_type.to_string(), "create");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), true);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("ALTER ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Alter);
    assert_eq!(stmt_type.to_string(), "alter");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), true);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("DROP ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Drop);
    assert_eq!(stmt_type.to_string(), "drop");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), true);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("BEGIN ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Begin);
    assert_eq!(stmt_type.to_string(), "PL/SQL(begin)");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), true);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("DECLARE ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Declare);
    assert_eq!(stmt_type.to_string(), "PL/SQL(declare)");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), true);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("COMMIT ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Commit);
    assert_eq!(stmt_type.to_string(), "commit");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("ROLLBACK ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::Rollback);
    assert_eq!(stmt_type.to_string(), "rollback");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), false);

    let stmt = conn.prepare("EXPLAIN PLAN FOR ...", &[]).unwrap();
    let stmt_type = stmt.statement_type();
    assert_eq!(stmt_type, StatementType::ExplainPlan);
    assert_eq!(stmt_type.to_string(), "explain plan");
    assert_eq!(stmt.is_query(), false);
    assert_eq!(stmt.is_plsql(), false);
    assert_eq!(stmt.is_ddl(), false);
    assert_eq!(stmt.is_dml(), false);
}

#[test]
fn bind_names() {
    let conn = common::connect().unwrap();

    let stmt = conn
        .prepare("BEGIN :val1 := :val2 || :val1 || :aàáâãäå; END;", &[])
        .unwrap();
    assert_eq!(stmt.bind_count(), 3);
    let bind_names = stmt.bind_names();
    assert_eq!(bind_names.len(), 3);
    assert_eq!(bind_names[0], "VAL1");
    assert_eq!(bind_names[1], "VAL2");
    assert_eq!(bind_names[2], "aàáâãäå".to_uppercase());

    let stmt = conn
        .prepare("SELECT :val1, :val2, :val1, :aàáâãäå from dual", &[])
        .unwrap();
    assert_eq!(stmt.bind_count(), 4);
    let bind_names = stmt.bind_names();
    assert_eq!(bind_names.len(), 3);
    assert_eq!(bind_names[0], "VAL1");
    assert_eq!(bind_names[1], "VAL2");
    assert_eq!(bind_names[2], "aàáâãäå".to_uppercase());
}

#[test]
fn query() {
    let conn = common::connect().unwrap();
    let sql = "select * from TestStrings where IntCol >= :icol order by IntCol";

    let mut stmt = conn.prepare(sql, &[StmtParam::FetchArraySize(3)]).unwrap();

    for (idx, row_result) in stmt.query(&[&2]).unwrap().enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_row(idx + 2, &row);
    }

    for (idx, row_result) in stmt.query_named(&[("icol", &3)]).unwrap().enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_row(idx + 3, &row);
    }

    let res_vec: Vec<_> = stmt.query(&[&2]).unwrap().collect();
    for (idx, row_result) in res_vec.into_iter().enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_row(idx + 2, &row);
    }

    // fetch the first column
    for (idx, row_result) in stmt.query_as::<usize>(&[&2]).unwrap().enumerate() {
        let int_col = row_result.unwrap();
        assert_eq!(int_col, idx + 2);
    }

    // fetch the first two columns
    for (idx, row_result) in stmt
        .query_as_named::<(usize, String)>(&[("icol", &3)])
        .unwrap()
        .enumerate()
    {
        let (int_col, string_col) = row_result.unwrap();
        assert_eq!(int_col, idx + 3);
        assert_eq!(string_col, format!("String {}", idx + 3));
    }

    for (idx, row_result) in stmt
        .query_as::<common::TestString>(&[&3])
        .unwrap()
        .enumerate()
    {
        let row = row_result.unwrap();
        common::assert_test_string_type(idx + 3, &row);
    }

    for (idx, row_result) in stmt
        .query_as_named::<common::TestStringTuple>(&[("icol", &3)])
        .unwrap()
        .enumerate()
    {
        let row = row_result.unwrap();
        common::assert_test_string_tuple(idx + 3, &row);
    }
}

#[test]
fn query_row() {
    let conn = common::connect().unwrap();
    let sql = "select * from TestStrings where IntCol = :icol";

    let mut stmt = conn.prepare(sql, &[StmtParam::FetchArraySize(1)]).unwrap();

    let row = stmt.query_row(&[&2]).unwrap();
    common::assert_test_string_row(2, &row);

    let row = stmt.query_row_named(&[("icol", &3)]).unwrap();
    common::assert_test_string_row(3, &row);

    let row = stmt.query_row_as::<common::TestStringTuple>(&[&4]).unwrap();
    common::assert_test_string_tuple(4, &row);

    let row = stmt
        .query_row_as_named::<common::TestString>(&[("icol", &5)])
        .unwrap();
    common::assert_test_string_type(5, &row);
}

#[test]
fn dml_returning() {
    let conn = common::connect().unwrap();
    let sql = "update TestStrings set StringCol = StringCol where IntCol >= :1 returning IntCol into :icol";

    let mut stmt = conn.prepare(sql, &[]).unwrap();
    assert_eq!(stmt.is_returning(), true);

    stmt.bind(2, &None::<i32>).unwrap();

    // update no rows
    stmt.execute(&[&11]).unwrap();
    let updated_int_col: Vec<i32> = stmt.returned_values(2).unwrap();
    assert_eq!(updated_int_col, vec![]);

    // update one row
    stmt.execute(&[&10]).unwrap();
    let updated_int_col: Vec<i32> = stmt.returned_values(2).unwrap();
    assert_eq!(updated_int_col, vec![10]);

    // update 10 rows
    stmt.execute(&[&1]).unwrap();
    let mut updated_int_col: Vec<i32> = stmt.returned_values(2).unwrap();
    updated_int_col.sort();
    assert_eq!(updated_int_col, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // update no rows
    stmt.execute(&[&11]).unwrap();
    let updated_int_col: Vec<i32> = stmt.returned_values(2).unwrap();
    assert_eq!(updated_int_col, vec![]);
}

#[test]
fn insert_and_fetch() {
    let conn = common::connect().unwrap();
    let char_data = "Hello, Guten Tag";
    let nchar_data = "Hello, こんにちは, 你好";
    let raw_data = b"\x7fELF, PE\0\0";
    let timestamp_data: Timestamp = "2017-08-09 10:11:13".parse().unwrap();
    let interval_ds_data: IntervalDS = "+12 03:04:05.6789".parse().unwrap();

    conn.execute(
        "insert into TestNumbers values (:1, :2, :3, :4, :5)",
        &[&100, &9.2, &10.14, &7.14, &None::<i32>],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, f64, f64, f64, Option<i32>)>(
            "select * from TestNumbers where IntCol = :1",
            &[&100],
        )
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, 9.2);
    assert_eq!(row.2, 10.14);
    assert_eq!(row.3, 7.14);
    assert_eq!(row.4, None);

    conn.execute(
        "insert into TestStrings values (:1, :2, :3, :4, :5)",
        &[
            &100,
            &char_data,
            &raw_data.as_ref(),
            &char_data,
            &None::<String>,
        ],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, String, Vec<u8>, String, Option<String>)>(
            "select * from TestStrings where IntCol = :1",
            &[&100],
        )
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, char_data);
    assert_eq!(row.2, raw_data);
    assert_eq!(row.3, format!("{:40}", char_data));
    assert_eq!(row.4, None);

    conn.execute(
        "insert into TestUnicodes values (:1, :2, :3, :4)",
        &[&100, &nchar_data, &nchar_data, &None::<String>],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, String, String, Option<String>)>(
            "select * from TestUnicodes where IntCol = :1",
            &[&100],
        )
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, nchar_data);
    assert_eq!(row.2, format!("{:40}", nchar_data));
    assert_eq!(row.3, None);

    conn.execute(
        "insert into TestDates values (:1, :2, :3)",
        &[&100, &timestamp_data, &None::<Timestamp>],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, Timestamp, Option<Timestamp>)>(
            "select * from TestDates where IntCol = :1",
            &[&100],
        )
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, timestamp_data);
    assert_eq!(row.2, None);

    conn.execute("insert into TestCLOBs values (:1, :2)", &[&100, &char_data])
        .unwrap();
    let row = conn
        .query_row_as::<(i32, String)>("select * from TestCLOBs where IntCol = :1", &[&100])
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, char_data);

    conn.execute(
        "insert into TestNCLOBs values (:1, :2)",
        &[&100, &nchar_data],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, String)>("select * from TestNCLOBs where IntCol = :1", &[&100])
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, nchar_data);

    conn.execute(
        "insert into TestBLOBs values (:1, :2)",
        &[&100, &raw_data.as_ref()],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, Vec<u8>)>("select * from TestBLOBs where IntCol = :1", &[&100])
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, raw_data);

    // conn.execute("insert into TestBFILEs values (:1, :2)",
    //              &[&100, ...]).unwrap();
    // let row = conn.query_row_as::<(i32, ...)>
    //     ("select * from TestBFILEs where IntCol = :1", &[&100]).unwrap();
    // assert_eq!(row.0, 100);
    // assert_eq!(row.1, ...);

    conn.execute("insert into TestLongs values (:1, :2)", &[&100, &char_data])
        .unwrap();
    let row = conn
        .query_row_as::<(i32, String)>("select * from TestLongs where IntCol = :1", &[&100])
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, char_data);

    conn.execute(
        "insert into TestLongRaws values (:1, :2)",
        &[&100, &raw_data.to_vec()],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, Vec<u8>)>("select * from TestLongRaws where IntCol = :1", &[&100])
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, raw_data);

    conn.execute(
        "insert into TestIntervals values (:1, :2, :3)",
        &[&100, &interval_ds_data, &None::<IntervalDS>],
    )
    .unwrap();
    let row = conn
        .query_row_as::<(i32, IntervalDS, Option<IntervalDS>)>(
            "select * from TestIntervals where IntCol = :1",
            &[&100],
        )
        .unwrap();
    assert_eq!(row.0, 100);
    assert_eq!(row.1, interval_ds_data);
    assert_eq!(row.2, None);
}

#[test]
fn row_count() {
    let conn = common::connect().unwrap();

    // rows affected
    let stmt = conn
        .execute(
            "update TestStrings set StringCol = StringCol where IntCol >= :1",
            &[&6],
        )
        .unwrap();
    assert_eq!(stmt.row_count().unwrap(), 5);

    // rows fetched
    let mut stmt = conn
        .prepare("select * from TestStrings where IntCol >= :1", &[])
        .unwrap();
    assert_eq!(stmt.row_count().unwrap(), 0); // before fetch
    for _row in stmt.query(&[&6]).unwrap() {}
    assert_eq!(stmt.row_count().unwrap(), 5); // after fetch
}

#[test]
fn iterate_rows_by_ref_and_check_fused() {
    let conn = common::connect().unwrap();
    let mut rows = conn
        .query_as::<usize>("select IntCol from TestNumbers order by IntCol", &[])
        .unwrap();

    let mut idx = 0;
    // fetch 5 rows
    for row_result in rows.by_ref().take(5) {
        idx += 1;
        assert_eq!(row_result.unwrap(), idx);
    }
    // fetch all rest rows
    for row_result in rows.by_ref() {
        idx += 1;
        assert_eq!(row_result.unwrap(), idx);
    }
    assert_eq!(idx, 10);

    // Ensure that the iterator is fused.
    assert!(rows.next().is_none());
    assert!(rows.next().is_none());
}
