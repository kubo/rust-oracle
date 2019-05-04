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

extern crate oracle;

use oracle::sql_type::{OracleType, Timestamp};
use oracle::Connection;

fn main() {
    let conn = Connection::connect("scott", "tiger", "").unwrap();
    let mut stmt = conn
        .prepare(
            "select empno, ename, job, mgr, hiredate, sal, comm, deptno from emp",
            &[],
        )
        .unwrap();
    let rows = stmt.query(&[]).unwrap();

    // stmt.define("HIREDATE", OracleType::Varchar2(60)).unwrap();

    println!(" {:-30} {:-8} {}", "Name", "Null?", "Type");
    println!(
        " {:-30} {:-8} {}",
        "------------------------------", "--------", "----------------------------"
    );
    for info in rows.column_info() {
        println!(
            " {:-30} {:-8} {}",
            info.name(),
            if info.nullable() { "" } else { "NOT NULL" },
            info.oracle_type()
        );
    }
    println!("");

    for row_result in rows {
        let row = row_result.unwrap();
        let empno: i32 = row.get(0).unwrap(); // index by 0-based position
        let ename: String = row.get("ENAME").unwrap(); // index by case-sensitive string
        let job: String = row.get(2).unwrap();
        let mgr: Option<i32> = row.get(3).unwrap(); // nullable column must be get as Option<...> to avoid panic.
        let hiredate: Timestamp = row.get(4).unwrap();
        let sal: f64 = row.get(5).unwrap();
        let comm: Option<f64> = row.get(6).unwrap();
        let deptno: Option<i32> = row.get(7).unwrap();

        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            empno,
            ename,
            job,
            mgr.map_or("".to_string(), |v| v.to_string()), // empty string if None, otherwise content of Some(x).
            hiredate,
            sal,
            comm.map_or("".to_string(), |v| v.to_string()),
            deptno.map_or("".to_string(), |v| v.to_string())
        );
    }

    // Set/Get bind values
    let mut stmt = conn.prepare("begin :1 := :2; end;", &[]).unwrap();
    stmt.bind(1, &OracleType::Varchar2(5)).unwrap();
    stmt.bind(2, &123).unwrap();
    stmt.execute(&[]).unwrap();
    let retval: String = stmt.bind_value(1).unwrap();
    println!(":1 (as String) => {}", retval);
    let retval: i32 = stmt.bind_value(1).unwrap();
    println!(":1 (as i32) => {}", retval);
    stmt.bind(2, &None::<i32>).unwrap();
    stmt.execute(&[]).unwrap();
    let retval: Option<i32> = stmt.bind_value(1).unwrap();
    println!(":1 is null? => {}", retval.is_none());

    if false {
        // This cause panic because 10000 is out of the range of `i8`.
        let _val = conn
            .query_row_as::<i8>("select 100000 from dual", &[])
            .unwrap();
        println!("never reach here!");
    }
}
