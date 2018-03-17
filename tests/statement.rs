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

use oracle::{StmtParam, StatementType};

#[test]
fn statement_type() {
    let conn = common::connect().unwrap();

    let stmt_type = conn.prepare("SELECT ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Select);
    assert_eq!(stmt_type.to_string(), "select");

    let stmt_type = conn.prepare("INSERT ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Insert);
    assert_eq!(stmt_type.to_string(), "insert");

    let stmt_type = conn.prepare("UPDATE ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Update);
    assert_eq!(stmt_type.to_string(), "update");

    let stmt_type = conn.prepare("DELETE ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Delete);
    assert_eq!(stmt_type.to_string(), "delete");

    let stmt_type = conn.prepare("MERGE ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Merge);
    assert_eq!(stmt_type.to_string(), "merge");

    let stmt_type = conn.prepare("CREATE ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Create);
    assert_eq!(stmt_type.to_string(), "create");

    let stmt_type = conn.prepare("ALTER ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Alter);
    assert_eq!(stmt_type.to_string(), "alter");

    let stmt_type = conn.prepare("DROP ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Drop);
    assert_eq!(stmt_type.to_string(), "drop");

    let stmt_type = conn.prepare("BEGIN ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Begin);
    assert_eq!(stmt_type.to_string(), "PL/SQL(begin)");

    let stmt_type = conn.prepare("DECLARE ...", &[]).unwrap().statement_type();
    assert_eq!(stmt_type, StatementType::Declare);
    assert_eq!(stmt_type.to_string(), "PL/SQL(declare)");
}

#[test]
fn bind_names() {
    let conn = common::connect().unwrap();

    let stmt = conn.prepare("BEGIN :val1 := :val2 || :val1 || :aàáâãäå; END;", &[]).unwrap();
    assert_eq!(stmt.bind_count(), 3);
    let bind_names = stmt.bind_names();
    assert_eq!(bind_names.len(), 3);
    assert_eq!(bind_names[0], "VAL1");
    assert_eq!(bind_names[1], "VAL2");
    assert_eq!(bind_names[2], "aàáâãäå".to_uppercase());

    let stmt = conn.prepare("SELECT :val1, :val2, :val1, :aàáâãäå from dual", &[]).unwrap();
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
    for (idx, row_result) in stmt.query_as_named::<(usize, String)>(&[("icol", &3)]).unwrap().enumerate() {
        let (int_col, string_col) = row_result.unwrap();
        assert_eq!(int_col, idx + 3);
        assert_eq!(string_col, format!("String {}", idx + 3));
    }

    for (idx, row_result) in stmt.query_as::<common::TestString>(&[&3]).unwrap().enumerate() {
        let row = row_result.unwrap();
        common::assert_test_string_type(idx + 3, &row);
    }

    for (idx, row_result) in stmt.query_as_named::<common::TestStringTuple>(&[("icol", &3)]).unwrap().enumerate() {
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

    let row = stmt.query_row_as_named::<common::TestString>(&[("icol", &5)]).unwrap();
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
#[cfg(feature = "restore-deleted")]
#[allow(deprecated)]
fn deprecated_methods() {

    let conn = common::connect().unwrap();
    let sql = "select * from TestStrings where IntCol >= :icol order by IntCol";

    let mut stmt = conn.execute(sql, &[&2]).unwrap();
    let mut idx = 2;
    while let Ok(row) = stmt.fetch() {
        common::assert_test_string_row(idx, row);
        idx += 1;
    }

    stmt.execute_named(&[("icol", &3)]).unwrap();
    let mut idx = 3;
    while let Ok(row) = stmt.fetch() {
        common::assert_test_string_row(idx, row);
        idx += 1;
    }
}
