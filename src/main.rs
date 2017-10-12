extern crate oracle;

fn main() {
    let ver = oracle::client_version().unwrap();
    println!("Oracle Client Version: {}", ver);
    let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    let mut stmt = conn.prepare("select empno, ename, job, mgr, hiredate, sal, comm, deptno from emp").unwrap();
    stmt.execute().unwrap();

    // stmt.define("HIREDATE", oracle::OracleType::Varchar2(60)).unwrap();

    println!(" {:-30} {:-8} {}", "Name", "Null?", "Type");
    println!(" {:-30} {:-8} {}", "------------------------------", "--------", "----------------------------");
    for info in stmt.column_info() {
        println!(" {:-30} {:-8} {}",
                 info.name(),
                 if info.nullable() {""} else {"NOT NULL"},
                 info.oracle_type());
    }
    println!("");

    while let Ok(row) = stmt.fetch() {
        let empno: i32 = row.get(0).unwrap();  // index by 0-based position
        let ename: String = row.get("ENAME").unwrap(); // index by case-sensitive string
        let job: String = row.get(2).unwrap();
        let mgr: Option<i32> = row.get(3).unwrap(); // nullable column must be get as Option<...> to avoid panic.
        let hiredate: oracle::Timestamp = row.get(4).unwrap();
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
    let mut stmt = conn.prepare("begin :1 := :2; end;").unwrap();
    stmt.bind(1, &oracle::OracleType::Varchar2(30)).unwrap();
    stmt.bind(2, &oracle::OracleType::Int64).unwrap();
    stmt.set_bind_value(2, 123.0).unwrap();
    stmt.execute().unwrap();
    let retval: String = stmt.bind_value(1).unwrap();
    println!(":1 (as String) => {}", retval);
    let retval: i32 = stmt.bind_value(1).unwrap();
    println!(":1 (as i32) => {}", retval);
    println!(":1 is null? => {}", stmt.is_null_value(1).unwrap());
    stmt.set_null_value(2).unwrap();
    stmt.execute().unwrap();
    println!(":1 is null? => {}", stmt.is_null_value(1).unwrap());

    if false {
        let mut stmt = conn.prepare("select 100000 from dual").unwrap();
        stmt.execute().unwrap();
        let row = stmt.fetch().unwrap();
        // This cause panic because 10000 is out of the range of `i8`.
        let _val: i8 = row.get(0).unwrap();
        println!("never reach here!");
    }
}
