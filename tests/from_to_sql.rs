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

use oracle::sql_type::{IntervalDS, IntervalYM, OracleType, Timestamp};
use oracle::{ErrorKind, Result};

macro_rules! chk_num_from {
    ($conn:ident, $val_from:expr, $val_to:expr, $(($T:ident, $success:tt)),+) => {
        let row = $conn.query_row(&format!("select {} from dual", $val_from), &[])?;
        $(
            chk_num_from!(row, $val_from, $T, $success);
        )+

        let row = $conn.query_row(&format!("select {} from dual", $val_to), &[])?;
        $(
            chk_num_from!(row, $val_to, $T, $success);
        )+
    };
    ($row:ident, $val:expr, $T:ident, true) => {
        assert_eq!($val as $T, $row.get_as::<$T>()?);
    };
    ($row:ident, $val:expr, $T:ident, false) => {
        let err = $row.get_as::<$T>().unwrap_err();
        if err.kind() != ErrorKind::ParseError {
            panic!("Unexpected error: {}", err);
        }
    };
}

#[test]
fn string_from_sql() -> Result<()> {
    let conn = common::connect()?;

    // Get number as string
    test_from_sql!(&conn, "10", &OracleType::Number(0, -127), &"10".to_string());

    // Get binary_float as string
    test_from_sql!(
        &conn,
        "10.5f",
        &OracleType::BinaryFloat,
        &"10.5".to_string()
    );

    // Get binary_double as string
    test_from_sql!(
        &conn,
        "10.5d",
        &OracleType::BinaryDouble,
        &"10.5".to_string()
    );

    // Get raw as string
    test_from_sql!(
        &conn,
        "hextoraw('48656c6c6f20576f726c64')",
        &OracleType::Raw(11),
        &"48656C6C6F20576F726C64".to_string()
    );

    // Get date as string
    test_from_sql!(
        &conn,
        "DATE '2012-03-04'",
        &OracleType::Date,
        &"2012-03-04 00:00:00".to_string()
    );

    // Get timestamp as string
    test_from_sql!(
        &conn,
        "TIMESTAMP '1997-01-31 09:26:50.124'",
        &OracleType::Timestamp(9),
        &"1997-01-31 09:26:50.124000000".to_string()
    );

    // Get timestamp with time zone as string
    test_from_sql!(
        &conn,
        "TIMESTAMP '1997-01-31 09:26:56.66 +02:00'",
        &OracleType::TimestampTZ(9),
        &"1997-01-31 09:26:56.660000000 +02:00".to_string()
    );

    // Get interval day to second as string
    test_from_sql!(
        &conn,
        "INTERVAL '4 5:12:10.222' DAY TO SECOND(3)",
        &OracleType::IntervalDS(2, 3),
        &"+04 05:12:10.222".to_string()
    );

    // Get interval year to month as string
    test_from_sql!(
        &conn,
        "INTERVAL '123-2' YEAR(3) TO MONTH",
        &OracleType::IntervalYM(3),
        &"+123-02".to_string()
    );

    // Get CLOB as string
    conn.execute("insert into TestCLOBs values (1, 'CLOB DATA')", &[])?;
    assert_eq!(
        conn.query_row_as::<String>("select CLOBCol from TestCLOBs where IntCol = 1", &[])?,
        "CLOB DATA".to_string()
    );
    conn.rollback()?;

    // Get BLOB as string
    conn.execute(
        "insert into TestBLOBs values (1, '424C4F422044415441')",
        &[],
    )?;
    assert_eq!(
        conn.query_row_as::<String>("select BLOBCol from TestBLOBs where IntCol = 1", &[])?,
        "424C4F422044415441".to_string()
    );
    conn.rollback()?;

    // Get object type as string
    assert_eq!(
        conn.query_row_as::<String>("select udt_SubObject(1, '10') from dual", &[])?,
        format!(
            r#"{}.UDT_SUBOBJECT(1, "10")"#, //  TODO: fix "10" -> '10'
            common::main_user().to_uppercase()
        )
    );

    // Get rowid as string
    assert_eq!(
        conn.query_row_as::<String>("select rowid from dual", &[])?,
        conn.query_row_as::<String>("select rowidtochar(rowid) from dual", &[])?
    );

    // Get boolean as bool and string
    if common::check_oracle_version("bind boolean", &conn, 12, 1)? {
        let mut stmt = conn
            .statement("declare b boolean; begin :1 := TRUE; end;")
            .build()?;
        stmt.execute(&[&None::<bool>])?;
        let val: bool = stmt.bind_value(1)?;
        assert_eq!(val, true);
        let val: String = stmt.bind_value(1)?;
        assert_eq!(val, "TRUE".to_string());

        let mut stmt = conn
            .statement("declare b boolean; begin :1 := FALSE; end;")
            .build()?;
        stmt.execute(&[&None::<bool>])?;
        let val: bool = stmt.bind_value(1)?;
        assert_eq!(val, false);
        let val: String = stmt.bind_value(1)?;
        assert_eq!(val, "FALSE".to_string());
    }

    // Get XMLTYPE
    let xmldata = "<data>ABCDEFGHIJKLMNOP</data>\n";
    conn.execute("insert into TestXML values (1, :1)", &[&xmldata])?;
    assert_eq!(
        &conn.query_row_as::<String>("select XMLCol from TestXML where IntCol = 1", &[])?,
        xmldata,
    );
    conn.rollback()?;
    Ok(())
}

#[test]
fn numeric_from_sql() -> Result<()> {
    let conn = common::connect()?;

    chk_num_from!(
        conn,
        0,
        0x7f,
        (i8, true),
        (i16, true),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, true),
        (u16, true),
        (u32, true),
        (u64, true),
        (usize, true)
    );
    chk_num_from!(
        conn,
        0x80,
        0xff,
        (i8, false),
        (i16, true),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, true),
        (u16, true),
        (u32, true),
        (u64, true),
        (usize, true)
    );
    chk_num_from!(
        conn,
        0x100,
        0x7fff,
        (i8, false),
        (i16, true),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, false),
        (u16, true),
        (u32, true),
        (u64, true),
        (usize, true)
    );
    chk_num_from!(
        conn,
        0x8000,
        0xffff,
        (i8, false),
        (i16, false),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, false),
        (u16, true),
        (u32, true),
        (u64, true),
        (usize, true)
    );
    chk_num_from!(
        conn,
        0x10000,
        0x7fffffff,
        (i8, false),
        (i16, false),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, false),
        (u16, false),
        (u32, true),
        (u64, true),
        (usize, true)
    );
    if cfg!(target_pointer_width = "64") {
        chk_num_from!(
            conn,
            0x80000000u32,
            0xffffffffu32,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, true),
            (isize, true),
            (u8, false),
            (u16, false),
            (u32, true),
            (u64, true),
            (usize, true)
        );
        chk_num_from!(
            conn,
            0x100000000u64,
            0x7fffffffffffffffu64,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, true),
            (isize, true),
            (u8, false),
            (u16, false),
            (u32, false),
            (u64, true),
            (usize, true)
        );
        chk_num_from!(
            conn,
            0x8000000000000000u64,
            0xffffffffffffffffu64,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, false),
            (isize, false),
            (u8, false),
            (u16, false),
            (u32, false),
            (u64, true),
            (usize, true)
        );
    } else {
        chk_num_from!(
            conn,
            0x80000000u32,
            0xffffffffu32,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, true),
            (isize, false),
            (u8, false),
            (u16, false),
            (u32, true),
            (u64, true),
            (usize, true)
        );
        chk_num_from!(
            conn,
            0x100000000u64,
            0x7fffffffffffffffu64,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, true),
            (isize, false),
            (u8, false),
            (u16, false),
            (u32, false),
            (u64, true),
            (usize, false)
        );
        chk_num_from!(
            conn,
            0x8000000000000000u64,
            0xffffffffffffffffu64,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, false),
            (isize, false),
            (u8, false),
            (u16, false),
            (u32, false),
            (u64, true),
            (usize, false)
        );
    }

    chk_num_from!(
        conn,
        -1,
        -0x80,
        (i8, true),
        (i16, true),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, false),
        (u16, false),
        (u32, false),
        (u64, false),
        (usize, false)
    );
    chk_num_from!(
        conn,
        -0x81,
        -0x8000,
        (i8, false),
        (i16, true),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, false),
        (u16, false),
        (u32, false),
        (u64, false),
        (usize, false)
    );
    chk_num_from!(
        conn,
        -0x8001,
        -0x80000000,
        (i8, false),
        (i16, false),
        (i32, true),
        (i64, true),
        (isize, true),
        (u8, false),
        (u16, false),
        (u32, false),
        (u64, false),
        (usize, false)
    );
    if cfg!(target_pointer_width = "64") {
        chk_num_from!(
            conn,
            -0x80000001i64,
            -0x8000000000000000i64,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, true),
            (isize, true),
            (u8, false),
            (u16, false),
            (u32, false),
            (u64, false),
            (usize, false)
        );
    } else {
        chk_num_from!(
            conn,
            -0x80000001i64,
            -0x8000000000000000i64,
            (i8, false),
            (i16, false),
            (i32, false),
            (i64, true),
            (isize, false),
            (u8, false),
            (u16, false),
            (u32, false),
            (u64, false),
            (usize, false)
        );
    }
    Ok(())
}

macro_rules! chk_num_to {
    ($stmt:expr, $typ:ident) => {
        let min_val = $typ::min_value();
        $stmt.execute(&[&OracleType::Varchar2(20), &min_val])?;
        let v: String = $stmt.bind_value(1)?;
        assert_eq!(v, min_val.to_string());

        let max_val = $typ::min_value();
        $stmt.execute(&[&OracleType::Varchar2(20), &max_val])?;
        let v: String = $stmt.bind_value(1)?;
        assert_eq!(v, max_val.to_string());
    };
}

#[test]
fn numeric_to_sql() -> Result<()> {
    let conn = common::connect()?;
    let mut stmt = conn.statement("begin :out := to_char(:in); end;").build()?;
    chk_num_to!(stmt, i8);
    chk_num_to!(stmt, i16);
    chk_num_to!(stmt, i32);
    chk_num_to!(stmt, i64);
    chk_num_to!(stmt, isize);
    chk_num_to!(stmt, u8);
    chk_num_to!(stmt, u16);
    chk_num_to!(stmt, u32);
    chk_num_to!(stmt, u64);
    chk_num_to!(stmt, usize);
    Ok(())
}

#[test]
fn raw_from_to_sql() -> Result<()> {
    let conn = common::connect()?;
    let raw = b"0123456789".to_vec();
    let hex = "30313233343536373839";

    test_from_sql!(
        &conn,
        &format!("hextoraw('{}')", hex),
        &OracleType::Raw(10),
        &raw
    );

    test_to_sql!(&conn, &raw, "rawtohex(:1)", hex);
    Ok(())
}

//
// Timestamp
//

#[test]
fn timestamp_from_sql() -> Result<()> {
    let conn = common::connect()?;
    let ts = Timestamp::new(2012, 3, 4, 0, 0, 0, 0)?;

    test_from_sql!(&conn, "DATE '2012-03-04'", &OracleType::Date, &ts);
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?;
    test_from_sql!(
        &conn,
        "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
        &OracleType::Date,
        &ts
    );

    test_from_sql!(
        &conn,
        "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP(0))",
        &OracleType::Timestamp(0),
        &ts
    );
    let ts = ts.and_prec(1)?;
    test_from_sql!(
        &conn,
        "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP(1))",
        &OracleType::Timestamp(1),
        &ts
    );
    let ts = ts.and_prec(6)?;
    test_from_sql!(
        &conn,
        "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP)",
        &OracleType::Timestamp(6),
        &ts
    );
    let ts = ts.and_prec(9)?;
    test_from_sql!(
        &conn,
        "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP(9))",
        &OracleType::Timestamp(9),
        &ts
    );
    test_from_sql!(
        &conn,
        "TO_TIMESTAMP('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
        &OracleType::Timestamp(9),
        &ts
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 123456789)?;
    test_from_sql!(
        &conn,
        "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF')",
        &OracleType::Timestamp(9),
        &ts
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 123456000)?;
    test_from_sql!(
        &conn,
        "TO_TIMESTAMP('2012-03-04 05:06:07.123456', 'YYYY-MM-DD HH24:MI:SS.FF')",
        &OracleType::Timestamp(9),
        &ts
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 123000000)?;
    test_from_sql!(
        &conn,
        "TO_TIMESTAMP('2012-03-04 05:06:07.123', 'YYYY-MM-DD HH24:MI:SS.FF')",
        &OracleType::Timestamp(9),
        &ts
    );

    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?.and_tz_offset(0)?;
    test_from_sql!(
        &conn,
        "TO_TIMESTAMP_TZ('2012-03-04 05:06:07 +00:00', 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
        &OracleType::TimestampTZ(9),
        &ts
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?.and_tz_hm_offset(8, 45)?;
    test_from_sql!(
        &conn,
        "TO_TIMESTAMP_TZ('2012-03-04 05:06:07 +08:45', 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
        &OracleType::TimestampTZ(9),
        &ts
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?.and_tz_hm_offset(-8, -45)?;
    test_from_sql!(
        &conn,
        "TO_TIMESTAMP_TZ('2012-03-04 05:06:07 -08:45', 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
        &OracleType::TimestampTZ(9),
        &ts
    );
    Ok(())
}

#[test]
fn timestamp_to_sql() -> Result<()> {
    let conn = common::connect()?;
    let ts = Timestamp::new(2012, 3, 4, 0, 0, 0, 0)?;

    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
        "2012-03-04 00:00:00"
    );

    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
        "2012-03-04 05:06:07"
    );

    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
        "2012-03-04 05:06:07"
    );
    let ts = ts.and_prec(1)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
        "2012-03-04 05:06:07"
    );
    let ts = ts.and_prec(6)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
        "2012-03-04 05:06:07"
    );
    let ts = ts.and_prec(9)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
        "2012-03-04 05:06:07"
    );
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
        "2012-03-04 05:06:07"
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 123456789)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF')",
        "2012-03-04 05:06:07.123456789"
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 123456000)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF6')",
        "2012-03-04 05:06:07.123456"
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 123000000)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF3')",
        "2012-03-04 05:06:07.123"
    );

    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?.and_tz_offset(0)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
        "2012-03-04 05:06:07 +00:00"
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?.and_tz_hm_offset(8, 45)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
        "2012-03-04 05:06:07 +08:45"
    );
    let ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 0)?.and_tz_hm_offset(-8, -45)?;
    test_to_sql!(
        &conn,
        &ts,
        "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
        "2012-03-04 05:06:07 -08:45"
    );
    Ok(())
}

//
// IntervalDS
//

#[test]
fn interval_ds_from_sql() -> Result<()> {
    let conn = common::connect()?;

    let it = IntervalDS::new(1, 2, 3, 4, 0)?;
    test_from_sql!(
        &conn,
        "INTERVAL '1 02:03:04' DAY TO SECOND",
        &OracleType::IntervalDS(2, 6),
        &it
    );

    let it = IntervalDS::new(1, 2, 3, 4, 123456789)?;
    test_from_sql!(
        &conn,
        "INTERVAL '+1 02:03:04.123456789' DAY TO SECOND(9)",
        &OracleType::IntervalDS(2, 9),
        &it
    );

    let it = IntervalDS::new(123456789, 2, 3, 4, 123456789)?;
    test_from_sql!(
        &conn,
        "INTERVAL '+123456789 02:03:04.123456789' DAY(9) TO SECOND(9)",
        &OracleType::IntervalDS(9, 9),
        &it
    );

    let it = IntervalDS::new(-1, -2, -3, -4, 0)?;
    test_from_sql!(
        &conn,
        "INTERVAL '-1 02:03:04' DAY TO SECOND",
        &OracleType::IntervalDS(2, 6),
        &it
    );

    let it = IntervalDS::new(-1, -2, -3, -4, -123456789)?;
    test_from_sql!(
        &conn,
        "INTERVAL '-1 02:03:04.123456789' DAY TO SECOND(9)",
        &OracleType::IntervalDS(2, 9),
        &it
    );

    let it = IntervalDS::new(-123456789, -2, -3, -4, -123456789)?;
    test_from_sql!(
        &conn,
        "INTERVAL '-123456789 02:03:04.123456789' DAY(9) TO SECOND(9)",
        &OracleType::IntervalDS(9, 9),
        &it
    );
    Ok(())
}

#[test]
fn interval_ds_to_sql() -> Result<()> {
    let conn = common::connect()?;

    let it = IntervalDS::new(1, 2, 3, 4, 0)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "+000000001 02:03:04.000000000");

    let it = IntervalDS::new(1, 2, 3, 4, 123456789)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "+000000001 02:03:04.123456789");

    let it = IntervalDS::new(123456789, 2, 3, 4, 123456789)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "+123456789 02:03:04.123456789");

    let it = IntervalDS::new(-1, -2, -3, -4, 0)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "-000000001 02:03:04.000000000");

    let it = IntervalDS::new(-1, -2, -3, -4, -123456789)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "-000000001 02:03:04.123456789");

    let it = IntervalDS::new(-123456789, -2, -3, -4, -123456789)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "-123456789 02:03:04.123456789");
    Ok(())
}

//
// IntervalYM
//

#[test]
fn interval_ym_from_sql() -> Result<()> {
    let conn = common::connect()?;

    let it = IntervalYM::new(1, 2)?;
    test_from_sql!(
        &conn,
        "INTERVAL '1-2' YEAR TO MONTH",
        &OracleType::IntervalYM(2),
        &it
    );

    let it = IntervalYM::new(123456789, 2)?;
    test_from_sql!(
        &conn,
        "INTERVAL '123456789-2' YEAR(9) TO MONTH",
        &OracleType::IntervalYM(9),
        &it
    );

    let it = IntervalYM::new(-1, -2)?;
    test_from_sql!(
        &conn,
        "INTERVAL '-1-2' YEAR TO MONTH",
        &OracleType::IntervalYM(2),
        &it
    );

    let it = IntervalYM::new(-123456789, -2)?;
    test_from_sql!(
        &conn,
        "INTERVAL '-123456789-2' YEAR(9) TO MONTH",
        &OracleType::IntervalYM(9),
        &it
    );
    Ok(())
}

#[test]
fn interval_ym_to_sql() -> Result<()> {
    let conn = common::connect()?;

    let it = IntervalYM::new(1, 2)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "+000000001-02");
    let it = IntervalYM::new(123456789, 2)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "+123456789-02");

    let it = IntervalYM::new(-1, -2)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "-000000001-02");

    let it = IntervalYM::new(-123456789, -2)?;
    test_to_sql!(&conn, &it, "TO_CHAR(:1)", "-123456789-02");
    Ok(())
}

#[cfg(feature = "chrono")]
#[allow(deprecated)]
mod chrono {
    use super::common;
    use super::test_from_sql;
    use super::test_to_sql;
    use chrono::naive::NaiveDate;
    use chrono::prelude::*;
    use chrono::Duration;
    use oracle::sql_type::OracleType;
    use oracle::{ErrorKind, Result};

    //
    // chrono::DateTime<Utc>
    // chrono::DateTime<Local>
    // chrono::DateTime<FixedOffset>
    //

    #[test]
    fn datetime_from_sql() -> Result<()> {
        let conn = common::connect()?;
        let fixed_utc = FixedOffset::east(0);
        let fixed_cet = FixedOffset::east(3600);

        // DATE -> DateTime<Utc>
        let dttm = Utc.ymd(2012, 3, 4).and_hms(5, 6, 7);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // DATE -> DateTime<Local>
        let dttm = Local.ymd(2012, 3, 4).and_hms(5, 6, 7);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // DATE -> DateTime<FixedOffset>  TZ is '+00:00'.
        let dttm = fixed_utc.ymd(2012, 3, 4).and_hms(5, 6, 7);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // TIMESTAMP -> DateTime<Utc>
        let dttm = Utc.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP -> DateTime<Local>
        let dttm = Local.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP -> DateTime<Fixed_Utc>  TZ is '+00:00'.
        let dttm = fixed_utc.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP WITH TIME ZONE -> DateTime<Utc>  TZ is ignored.
        let dttm = Utc.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);

        // TIMESTAMP WITH TIME ZONE -> DateTime<Local>  TZ is ignored.
        let dttm = Local.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);

        // TIMESTAMP WITH TIME ZONE -> DateTime<Fixed_Utc> TZ is set.
        let dttm = fixed_cet.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);
        Ok(())
    }

    #[test]
    fn datetime_to_sql() -> Result<()> {
        let conn = common::connect()?;
        let dttm_utc = Utc.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        let dttm_local = Local.ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        let dttm_fixed_cet = FixedOffset::east(3600)
            .ymd(2012, 3, 4)
            .and_hms_nano(5, 6, 7, 123456789);

        // DateTime<Utc> -> TIMESTAMP WITH TIME ZONE
        test_to_sql!(
            &conn,
            &dttm_utc,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
            "2012-03-04 05:06:07.123456789 +00:00"
        );

        // DateTime<Local> -> TIMESTAMP WITH TIME ZONE
        let tz_offset = dttm_local.offset().fix().local_minus_utc();
        let tz_sign = if tz_offset >= 0 { '+' } else { '-' };
        let tz_hour = tz_offset.abs() / 3600;
        let tz_min = tz_offset.abs() % 3600 / 60;
        test_to_sql!(
            &conn,
            &dttm_local,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
            &format!(
                "2012-03-04 05:06:07.123456789 {}{:02}:{:02}",
                tz_sign, tz_hour, tz_min
            )
        );

        // DateTime<FixedOffset> -> TIMESTAMP WITH TIME ZONE
        test_to_sql!(
            &conn,
            &dttm_fixed_cet,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
            "2012-03-04 05:06:07.123456789 +01:00"
        );
        Ok(())
    }

    //
    // chrono::Date<Utc>
    // chrono::Date<Local>
    // chrono::Date<FixedOffset>
    //

    #[test]
    fn date_from_sql() -> Result<()> {
        let conn = common::connect()?;
        let fixed_utc = FixedOffset::east(0);
        let fixed_cet = FixedOffset::east(3600);

        // DATE -> Date<Utc>
        let dttm = Utc.ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // DATE -> Date<Local>
        let dttm = Local.ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // DATE -> Date<FixedOffset>  TZ is '+00:00'.
        let dttm = fixed_utc.ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // TIMESTAMP -> Date<Utc>
        let dttm = Utc.ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP -> Date<Local>
        let dttm = Local.ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP -> Date<Fixed_Utc>  TZ is '+00:00'.
        let dttm = fixed_utc.ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP WITH TIME ZONE -> Date<Utc>  TZ is ignored.
        let dttm = Utc.ymd(2012, 3, 4);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);

        // TIMESTAMP WITH TIME ZONE -> Date<Local>  TZ is ignored.
        let dttm = Local.ymd(2012, 3, 4);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);

        // TIMESTAMP WITH TIME ZONE -> Date<Fixed_Utc> TZ is set.
        let dttm = fixed_cet.ymd(2012, 3, 4);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);
        Ok(())
    }

    #[test]
    fn date_to_sql() -> Result<()> {
        let conn = common::connect()?;
        let dttm_utc = Utc.ymd(2012, 3, 4);
        let dttm_local = Local.ymd(2012, 3, 4);
        let dttm_fixed_cet = FixedOffset::east(3600).ymd(2012, 3, 4);

        // Date<Utc> -> TIMESTAMP WITH TIME ZONE
        test_to_sql!(
            &conn,
            &dttm_utc,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
            "2012-03-04 00:00:00.000000000 +00:00"
        );

        // Date<Local> -> TIMESTAMP WITH TIME ZONE
        let tz_offset = dttm_local.offset().fix().local_minus_utc();
        let tz_sign = if tz_offset >= 0 { '+' } else { '-' };
        let tz_hour = tz_offset.abs() / 3600;
        let tz_min = tz_offset.abs() % 3600 / 60;
        test_to_sql!(
            &conn,
            &dttm_local,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
            &format!(
                "2012-03-04 00:00:00.000000000 {}{:02}:{:02}",
                tz_sign, tz_hour, tz_min
            )
        );

        // Date<FixedOffset> -> TIMESTAMP WITH TIME ZONE
        test_to_sql!(
            &conn,
            &dttm_fixed_cet,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
            "2012-03-04 00:00:00.000000000 +01:00"
        );
        Ok(())
    }

    //
    // chrono::naive::NaiveDateTime
    //

    #[test]
    fn naive_datetime_from_sql() -> Result<()> {
        let conn = common::connect()?;

        // DATE -> NaiveDateTime
        let dttm = NaiveDate::from_ymd(2012, 3, 4).and_hms(5, 6, 7);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // TIMESTAMP -> NaiveDateTime
        let dttm = NaiveDate::from_ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP WITH TIME ZONE -> NaiveDateTime (TZ is ignored.)
        let dttm = NaiveDate::from_ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);
        Ok(())
    }

    #[test]
    fn naive_datetime_to_sql() -> Result<()> {
        let conn = common::connect()?;

        // NaiveDateTime -> TIMESTAMP
        let dttm = NaiveDate::from_ymd(2012, 3, 4).and_hms_nano(5, 6, 7, 123456789);
        test_to_sql!(
            &conn,
            &dttm,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9')",
            "2012-03-04 05:06:07.123456789"
        );
        Ok(())
    }

    //
    // chrono::NaiveDate
    //

    #[test]
    fn naive_date_from_sql() -> Result<()> {
        let conn = common::connect()?;

        // DATE -> NaiveDate
        let dttm = NaiveDate::from_ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
            &OracleType::Date,
            &dttm
        );

        // TIMESTAMP -> NaiveDate
        let dttm = NaiveDate::from_ymd(2012, 3, 4);
        test_from_sql!(
            &conn,
            "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF9')",
            &OracleType::Timestamp(9),
            &dttm
        );

        // TIMESTAMP WITH TIME ZONE -> NaiveDate (TZ is ignored.)
        let dttm = NaiveDate::from_ymd(2012, 3, 4);
        test_from_sql!(&conn,
                       "TO_TIMESTAMP_TZ('2012-03-04 05:06:07.123456789 +01:00', 'YYYY-MM-DD HH24:MI:SS.FF9 TZH:TZM')",
                       &OracleType::TimestampTZ(9), &dttm);
        Ok(())
    }

    #[test]
    fn naive_date_to_sql() -> Result<()> {
        let conn = common::connect()?;

        // NaiveDate -> TIMESTAMP
        let dttm = NaiveDate::from_ymd(2012, 3, 4);
        test_to_sql!(
            &conn,
            &dttm,
            "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF9')",
            "2012-03-04 00:00:00.000000000"
        );
        Ok(())
    }

    //
    // chrono::Duration
    //

    #[test]
    fn duration_from_sql() -> Result<()> {
        let conn = common::connect()?;

        // INTERVAL DAY TO SECOND -> Duration
        let d = Duration::days(1)
            + Duration::hours(2)
            + Duration::minutes(3)
            + Duration::seconds(4)
            + Duration::nanoseconds(123456789);
        test_from_sql!(
            &conn,
            "INTERVAL '+1 02:03:04.123456789' DAY TO SECOND(9)",
            &OracleType::IntervalDS(2, 9),
            &d
        );
        let d = -d;
        test_from_sql!(
            &conn,
            "INTERVAL '-1 02:03:04.123456789' DAY TO SECOND(9)",
            &OracleType::IntervalDS(2, 9),
            &d
        );

        let d = Duration::days(999999999)
            + Duration::hours(23)
            + Duration::minutes(59)
            + Duration::seconds(59)
            + Duration::nanoseconds(999999999);
        test_from_sql!(
            &conn,
            "INTERVAL '+999999999 23:59:59.999999999' DAY(9) TO SECOND(9)",
            &OracleType::IntervalDS(9, 9),
            &d
        );

        let d = -d;
        test_from_sql!(
            &conn,
            "INTERVAL '-999999999 23:59:59.999999999' DAY(9) TO SECOND(9)",
            &OracleType::IntervalDS(9, 9),
            &d
        );
        Ok(())
    }

    #[test]
    fn duration_to_sql() -> Result<()> {
        let conn = common::connect()?;

        // Duration -> INTERVAL DAY TO SECOND
        let d = Duration::days(1)
            + Duration::hours(2)
            + Duration::minutes(3)
            + Duration::seconds(4)
            + Duration::nanoseconds(123456789);
        test_to_sql!(&conn, &d, "TO_CHAR(:1)", "+000000001 02:03:04.123456789");

        let d = -d;
        test_to_sql!(&conn, &d, "TO_CHAR(:1)", "-000000001 02:03:04.123456789");

        let d = Duration::days(999999999)
            + Duration::hours(23)
            + Duration::minutes(59)
            + Duration::seconds(59)
            + Duration::nanoseconds(999999999);
        test_to_sql!(&conn, &d, "TO_CHAR(:1)", "+999999999 23:59:59.999999999");

        let d = -d;
        test_to_sql!(&conn, &d, "TO_CHAR(:1)", "-999999999 23:59:59.999999999");

        // Overflow
        let d = Duration::days(1000000000);
        let mut stmt = conn.statement("begin :out := TO_CHAR(:1); end;").build()?;
        let err = stmt.bind(2, &d).expect_err("expect out of range error");
        assert_eq!(err.kind(), ErrorKind::OutOfRange);
        Ok(())
    }
}
