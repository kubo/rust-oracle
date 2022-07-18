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

use oracle::sql_type::{OracleType, Timestamp};
use oracle::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::connect("scott", "tiger", "")?;
    let mut stmt = conn
        .statement("select empno, ename, job, mgr, hiredate, sal, comm, deptno from emp")
        .build()?;
    let rows = stmt.query(&[])?;

    // stmt.define("HIREDATE", OracleType::Varchar2(60))?;

    println!(" {:-30} {:-8} {:0}", "Name", "Null?", "Type");
    println!(
        " {:-30} {:-8} {:0}",
        "------------------------------", "--------", "----------------------------"
    );
    for info in rows.column_info() {
        println!(
            " {:-30} {:-8} {:0}",
            info.name(),
            if info.nullable() { "" } else { "NOT NULL" },
            info.oracle_type()
        );
    }
    println!();

    for row_result in rows {
        let row = row_result?;
        let empno: i32 = row.get(0)?; // index by 0-based position
        let ename: String = row.get("ENAME")?; // index by case-sensitive string
        let job: String = row.get(2)?;
        let mgr: Option<i32> = row.get(3)?; // nullable column must be get as Option<...> to avoid panic.
        let hiredate: Timestamp = row.get(4)?;
        let sal: f64 = row.get(5)?;
        let comm: Option<f64> = row.get(6)?;
        let deptno: Option<i32> = row.get(7)?;

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
    let mut stmt = conn.statement("begin :1 := :2; end;").build()?;
    stmt.bind(1, &OracleType::Varchar2(5))?;
    stmt.bind(2, &123)?;
    stmt.execute(&[])?;
    let retval: String = stmt.bind_value(1)?;
    println!(":1 (as String) => {}", retval);
    let retval: i32 = stmt.bind_value(1)?;
    println!(":1 (as i32) => {}", retval);
    stmt.bind(2, &None::<i32>)?;
    stmt.execute(&[])?;
    let retval: Option<i32> = stmt.bind_value(1)?;
    println!(":1 is null? => {}", retval.is_none());

    if false {
        // This cause panic because 10000 is out of the range of `i8`.
        let _val = conn.query_row_as::<i8>("select 100000 from dual", &[])?;
        println!("never reach here!");
    }
    Ok(())
}
