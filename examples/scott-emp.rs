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

use oracle::{Connection, OracleType, Timestamp};

fn main() {
    let conn = Connection::connect("scott", "tiger", "", &[]).unwrap();
    let mut stmt = conn.prepare("select empno, ename, job, mgr, hiredate, sal, comm, deptno from emp", &[]).unwrap();
    let rows = stmt.query(&[]).unwrap();

    // stmt.define("HIREDATE", OracleType::Varchar2(60)).unwrap();

    println!(" {:-30} {:-8} {}", "Name", "Null?", "Type");
    println!(" {:-30} {:-8} {}", "------------------------------", "--------", "----------------------------");
    for info in rows.column_info() {
        println!(" {:-30} {:-8} {}",
                 info.name(),
                 if info.nullable() {""} else {"NOT NULL"},
                 info.oracle_type());
    }
    println!("");

    for row_result in &rows {
        let row = row_result.unwrap();
        let empno: i32 = row.get(0).unwrap();  // index by 0-based position
        let ename: String = row.get("ENAME").unwrap(); // index by case-sensitive string
        let job: String = row.get(2).unwrap();
        let mgr: Option<i32> = row.get(3).unwrap(); // nullable column must be get as Option<...> to avoid panic.
        let hiredate: Timestamp = row.get(4).unwrap();
        let sal: f64 = row.get(5).unwrap();
        let comm: Option<f64> = row.get(6).unwrap();
        let deptno: Option<i32> = row.get(7).unwrap();

        println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                 empno,
                 ename,
                 job,
                 mgr.map_or("".to_string(), |v| v.to_string()), // empty string if None, otherwise content of Some(x).
                 hiredate,
                 sal,
                 comm.map_or("".to_string(), |v| v.to_string()),
                 deptno.map_or("".to_string(), |v| v.to_string()));
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
        let _val = conn.query_row_as::<i8>("select 100000 from dual", &[]).unwrap();
        println!("never reach here!");
    }
}

