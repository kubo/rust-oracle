extern crate oracle;
extern crate chrono;
#[macro_use]
mod common;

use oracle::*;

macro_rules! test_in_out {
    ($stmt:expr, $type:ty, $val:expr) => {
        $stmt.execute(&(&None::<$type>, $val)).expect(format!("error at {}:{}", file!(), line!()).as_str());
        let outval: $type = $stmt.bind_value(1).unwrap();
        assert_eq!(outval, $val);
    };
    ($stmt:expr, $type:ty, $val:expr, $outtype:expr) => {
        $stmt.execute(&($outtype, $val)).expect(format!("error at {}:{}", file!(), line!()).as_str());
        let outval: $type = $stmt.bind_value(1).unwrap();
        assert_eq!(outval, $val);
    };
}

#[test]
fn in_out_same_values() {
    let conn = common::connect().unwrap();
    let mut stmt = conn.prepare("begin :1 := :2; end;").unwrap();

    test_in_out!(stmt, i8, -123i8);
    test_in_out!(stmt, i16, -12345i16);
    test_in_out!(stmt, i32, -1234567i32);
    test_in_out!(stmt, i64, -123456789i64);
    test_in_out!(stmt, u8, 123u8);
    test_in_out!(stmt, u16, 12345u16);
    test_in_out!(stmt, u32, 1234567u32);
    test_in_out!(stmt, u64, 123456789u64);
    test_in_out!(stmt, f32, -123.5f32);
    test_in_out!(stmt, f64, 123456789123.5f64);
    //test_in_out!(stmt, bool, true);
    //test_in_out!(stmt, bool, false);
    test_in_out!(stmt, String, "123456789", &oracle::bind_value(&None::<&str>, 9));
    test_in_out!(stmt, Vec<u8>, vec![1u8,2u8,3u8,4u8,5u8], &oracle::bind_value(&None::<Vec<u8>>, 5));
    test_in_out!(stmt, Timestamp, Timestamp::new(2012, 3, 4, 5, 6, 7, 123456789));
    test_in_out!(stmt, IntervalDS, IntervalDS::new(1, 2, 3, 4, 123456789));
    test_in_out!(stmt, IntervalYM, IntervalYM::new(10, 2));
}

macro_rules! test_to_string {
    ($stmt:expr, $val:expr) => {
        $stmt.bind(1, &oracle::bind_value(&None::<&str>, 4000)).unwrap();
        $stmt.bind(2, $val).unwrap();
        $stmt.execute(&()).expect(format!("error at {}:{}", file!(), line!()).as_str());
        let v1: String = $stmt.bind_value(1).unwrap(); // convert $val to string in Oracle
        let v2: String = $stmt.bind_value(2).unwrap(); // convert $val to string in rust oracle
        assert_eq!(v1, v2);
    };
    ($stmt:expr, $val:expr, $expected_str:expr) => {
        $stmt.bind(2, $val).unwrap();
        let v2: String = $stmt.bind_value(2).unwrap(); // convert $val to string in rust oracle
        assert_eq!(v2, $expected_str);
    };
}

#[test]
fn to_string_in_rust_oracle() {
    let conn = common::connect().unwrap();
    let mut stmt = conn.prepare("begin :1 := :2; end;").unwrap();

    conn.execute("alter session set nls_timestamp_format = 'yyyy-mm-dd hh24:mi:ss.ff9'", &()).unwrap();
    conn.execute("alter session set nls_timestamp_tz_format = 'yyyy-mm-dd hh24:mi:ss.ff9 tzh:tzm'", &()).unwrap();

    test_to_string!(stmt, -123i8);
    test_to_string!(stmt, -12345i16);
    test_to_string!(stmt, -1234567i32);
    test_to_string!(stmt, -123456789i64);
    test_to_string!(stmt, 123i8);
    test_to_string!(stmt, 12345i16);
    test_to_string!(stmt, 1234567i32);
    test_to_string!(stmt, 123456789i64);
    test_to_string!(stmt, -123.5f32);
    test_to_string!(stmt, 123456789123.5f64);
    test_to_string!(stmt, "12345");
    test_to_string!(stmt, vec![0x01u8, 0x19u8, 0x9au8, 0xafu8, 0xf0u8]);
    test_to_string!(stmt, Timestamp::new(2012, 3, 4, 5, 6, 7, 123456789));
    test_to_string!(stmt, IntervalDS::new(1, 2, 3, 4, 123456789));
    test_to_string!(stmt, IntervalYM::new(10, 2));

    test_to_string!(stmt, &bind_value(&-123.5f32, 0), "-123.5");
    test_to_string!(stmt, &bind_value(&123456789123.5f64, 0), "123456789123.5");
}
