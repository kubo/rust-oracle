// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2018 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

extern crate oracle;

use oracle::Connection;

// Select a table and print column types and values as CSV.
// The CSV format isn't valid if data include double quotation
// marks, commas or return codes.
fn main() {
    let username = "scott";
    let password = "tiger";
    let database = "";
    let sql = "select * from emp";

    let conn = Connection::connect(username, password, database).unwrap();
    let mut stmt = conn.prepare(sql, &[]).unwrap();
    let rows = stmt.query(&[]).unwrap();

    // print column types
    for (idx, info) in rows.column_info().iter().enumerate() {
        if idx != 0 {
            print!(",");
        }
        print!("{}", info);
    }
    println!("");

    for row_result in rows {
        // print column values
        for (idx, val) in row_result.unwrap().sql_values().iter().enumerate() {
            if idx != 0 {
                print!(",");
            }
            print!("{}", val);
        }
        println!("");
    }
}
