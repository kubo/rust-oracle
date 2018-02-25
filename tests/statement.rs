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
fn statement_type() {
    let conn = common::connect().unwrap();

    let stmt_type = conn.prepare("SELECT ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Select);
    assert_eq!(stmt_type.to_string(), "select");

    let stmt_type = conn.prepare("INSERT ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Insert);
    assert_eq!(stmt_type.to_string(), "insert");

    let stmt_type = conn.prepare("UPDATE ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Update);
    assert_eq!(stmt_type.to_string(), "update");

    let stmt_type = conn.prepare("DELETE ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Delete);
    assert_eq!(stmt_type.to_string(), "delete");

    let stmt_type = conn.prepare("MERGE ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Merge);
    assert_eq!(stmt_type.to_string(), "merge");

    let stmt_type = conn.prepare("CREATE ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Create);
    assert_eq!(stmt_type.to_string(), "create");

    let stmt_type = conn.prepare("ALTER ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Alter);
    assert_eq!(stmt_type.to_string(), "alter");

    let stmt_type = conn.prepare("DROP ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Drop);
    assert_eq!(stmt_type.to_string(), "drop");

    let stmt_type = conn.prepare("BEGIN ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Begin);
    assert_eq!(stmt_type.to_string(), "PL/SQL(begin)");

    let stmt_type = conn.prepare("DECLARE ...").unwrap().statement_type();
    assert_eq!(stmt_type, oracle::StatementType::Declare);
    assert_eq!(stmt_type.to_string(), "PL/SQL(declare)");
}

#[test]
fn bind_names() {
    let conn = common::connect().unwrap();

    let stmt = conn.prepare("BEGIN :val1 := :val2 || :val1 || :aàáâãäå; END;").unwrap();
    assert_eq!(stmt.bind_count(), 3);
    let bind_names = stmt.bind_names();
    assert_eq!(bind_names.len(), 3);
    assert_eq!(bind_names[0], "VAL1");
    assert_eq!(bind_names[1], "VAL2");
    assert_eq!(bind_names[2], "aàáâãäå".to_uppercase());

    let stmt = conn.prepare("SELECT :val1, :val2, :val1, :aàáâãäå from dual").unwrap();
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
    let sql_stmt = "select IntCol from TestStrings where IntCol >= :lower order by IntCol";

    let mut stmt = conn.prepare(sql_stmt).unwrap();
    stmt.set_fetch_array_size(3);

    for (idx, row_result) in stmt.query(&[&2]).unwrap().enumerate() {
        let row = row_result.unwrap();
        let int_col: usize = row.get(0).unwrap();
        assert_eq!(int_col, idx + 2);
    }

    for (idx, row_result) in stmt.query_named(&[("lower", &3)]).unwrap().enumerate() {
        let row = row_result.unwrap();
        let int_col: usize = row.get(0).unwrap();
        assert_eq!(int_col, idx + 3);
    }

    let res_vec: Vec<_> = stmt.query(&[&2]).unwrap().collect();
    for (idx, row_result) in res_vec.into_iter().enumerate() {
        let row = row_result.unwrap();
        let int_col: usize = row.get(0).unwrap();
        assert_eq!(int_col, idx + 2);
    }
}

#[test]
fn query_as() {
    let conn = common::connect().unwrap();
    let sql_stmt = "select * from TestStrings where IntCol >= :lower order by IntCol";

    let mut stmt = conn.prepare(sql_stmt).unwrap();
    stmt.set_fetch_array_size(3);

    for (idx, row_result) in stmt.query_as::<usize>(&[&2]).unwrap().enumerate() {
        let int_col = row_result.unwrap();
        assert_eq!(int_col, idx + 2);
    }

    for (idx, row_result) in stmt.query_as_named::<(usize, String)>(&[("lower", &3)]).unwrap().enumerate() {
        let (int_col, string_col) = row_result.unwrap();
        assert_eq!(int_col, idx + 3);
        assert_eq!(string_col, format!("String {}", idx + 3));
    }

    for (idx, row_result) in stmt.query_as_named::<common::TestString>(&[("lower", &3)]).unwrap().enumerate() {
        let test_string = row_result.unwrap();
        let int_col = (idx + 3) as i32;
        assert_eq!(test_string.int_col, int_col);
        assert_eq!(test_string.string_col, format!("String {}", int_col));
        assert_eq!(test_string.raw_col, format!("Raw {}", int_col).as_bytes());
        assert_eq!(test_string.fixed_char_col, format!("Fixed Char {:<29}", int_col));
        if int_col % 2 == 0 {
            assert_eq!(test_string.nullable_col, None);
        } else {
            assert_eq!(test_string.nullable_col, Some(format!("Nullable {}", int_col).to_string()));
        }
    }
}
