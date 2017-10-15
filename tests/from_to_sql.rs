extern crate oracle;
#[macro_use]
mod common;

use oracle::*;

#[test]
fn timestamp_from_sql() {
    let conn = common::connect().unwrap();
    let mut ts = Timestamp::new(2012, 3, 4, 0, 0, 0, 0);

    test_from_sql!(&conn,
                   "DATE '2012-03-04'",
                   &OracleType::Date, &ts);
    ts.hour = 5; ts.minute = 6; ts.second = 7;
    test_from_sql!(&conn,
                   "TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
                   &OracleType::Date, &ts);

    test_from_sql!(&conn,
                   "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP(0))",
                   &OracleType::Timestamp(0), &ts);
    ts.precision = 1;
    test_from_sql!(&conn,
                   "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP(1))",
                   &OracleType::Timestamp(1), &ts);
    ts.precision = 6;
    test_from_sql!(&conn,
                   "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP)",
                   &OracleType::Timestamp(6), &ts);
    ts.precision = 9;
    test_from_sql!(&conn,
                   "CAST(TO_DATE('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS') AS TIMESTAMP(9))",
                   &OracleType::Timestamp(9), &ts);
    test_from_sql!(&conn,
                   "TO_TIMESTAMP('2012-03-04 05:06:07', 'YYYY-MM-DD HH24:MI:SS')",
                   &OracleType::Timestamp(9), &ts);
    ts.nanosecond = 123456789;
    test_from_sql!(&conn,
                   "TO_TIMESTAMP('2012-03-04 05:06:07.123456789', 'YYYY-MM-DD HH24:MI:SS.FF')",
                   &OracleType::Timestamp(9), &ts);
    ts.nanosecond = 123456000;
    test_from_sql!(&conn,
                   "TO_TIMESTAMP('2012-03-04 05:06:07.123456', 'YYYY-MM-DD HH24:MI:SS.FF')",
                   &OracleType::Timestamp(9), &ts);
    ts.nanosecond = 123000000;
    test_from_sql!(&conn,
                   "TO_TIMESTAMP('2012-03-04 05:06:07.123', 'YYYY-MM-DD HH24:MI:SS.FF')",
                   &OracleType::Timestamp(9), &ts);

    ts.with_tz = true;
    ts.nanosecond = 0;
    test_from_sql!(&conn,
                   "TO_TIMESTAMP_TZ('2012-03-04 05:06:07 +00:00', 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
                   &OracleType::TimestampTZ(9), &ts);
    ts.tz_hour_offset = 8; ts.tz_minute_offset = 45;
    test_from_sql!(&conn,
                   "TO_TIMESTAMP_TZ('2012-03-04 05:06:07 +08:45', 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
                   &OracleType::TimestampTZ(9), &ts);
    ts.tz_hour_offset = -8; ts.tz_minute_offset = -45;
    test_from_sql!(&conn,
                   "TO_TIMESTAMP_TZ('2012-03-04 05:06:07 -08:45', 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
                   &OracleType::TimestampTZ(9), &ts);
}

#[test]
fn timestamp_to_sql() {
    let conn = common::connect().unwrap();
    let mut ts = Timestamp::new(2012, 3, 4, 0, 0, 0, 0);

    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
                 &OracleType::Date,
                 "2012-03-04 00:00:00");

    ts.hour = 5; ts.minute = 6; ts.second = 7;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
                 &OracleType::Date,
                 "2012-03-04 05:06:07");

    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
                 &OracleType::Timestamp(0),
                 "2012-03-04 05:06:07");
    ts.precision = 1;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
                 &OracleType::Timestamp(1),
                 "2012-03-04 05:06:07");
    ts.precision = 6;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
                 &OracleType::Timestamp(6),
                 "2012-03-04 05:06:07");
    ts.precision = 9;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
                 &OracleType::Timestamp(9),
                 "2012-03-04 05:06:07");
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS')",
                 &OracleType::Timestamp(9),
                 "2012-03-04 05:06:07");
    ts.nanosecond = 123456789;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF')",
                 &OracleType::Timestamp(9),
                 "2012-03-04 05:06:07.123456789");
    ts.nanosecond = 123456000;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF6')",
                 &OracleType::Timestamp(9),
                 "2012-03-04 05:06:07.123456");
    ts.nanosecond = 123000000;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS.FF3')",
                 &OracleType::Timestamp(9),
                 "2012-03-04 05:06:07.123");

    ts.with_tz = true;
    ts.nanosecond = 0;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
                 &OracleType::TimestampTZ(9),
                 "2012-03-04 05:06:07 +00:00");
    ts.tz_hour_offset = 8; ts.tz_minute_offset = 45;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
                 &OracleType::TimestampTZ(9),
                 "2012-03-04 05:06:07 +08:45");
    ts.tz_hour_offset = -8; ts.tz_minute_offset = -45;
    test_to_sql!(&conn, &ts,
                 "TO_CHAR(:1, 'YYYY-MM-DD HH24:MI:SS TZH:TZM')",
                 &OracleType::TimestampTZ(9),
                 "2012-03-04 05:06:07 -08:45");
}

#[test]
fn interval_ds_from_sql() {
    let conn = common::connect().unwrap();
    let mut it = IntervalDS::new(1, 2, 3, 4, 0);

    test_from_sql!(&conn,
                   "INTERVAL '1 02:03:04' DAY TO SECOND",
                   &OracleType::IntervalDS(2, 6), &it);
    it.nanoseconds = 123456789;
    it.fsprecision = 9;
    test_from_sql!(&conn,
                   "INTERVAL '+1 02:03:04.123456789' DAY TO SECOND(9)",
                   &OracleType::IntervalDS(2, 9), &it);
    it.days = 123456789;
    it.lfprecision = 9;
    test_from_sql!(&conn,
                   "INTERVAL '+123456789 02:03:04.123456789' DAY(9) TO SECOND(9)",
                   &OracleType::IntervalDS(9, 9), &it);

    let mut it = IntervalDS::new(-1, -2, -3, -4, 0);

    test_from_sql!(&conn,
                   "INTERVAL '-1 02:03:04' DAY TO SECOND",
                   &OracleType::IntervalDS(2, 6), &it);
    it.nanoseconds = -123456789;
    it.fsprecision = 9;
    test_from_sql!(&conn,
                   "INTERVAL '-1 02:03:04.123456789' DAY TO SECOND(9)",
                   &OracleType::IntervalDS(2, 9), &it);
    it.days = -123456789;
    it.lfprecision = 9;
    test_from_sql!(&conn,
                   "INTERVAL '-123456789 02:03:04.123456789' DAY(9) TO SECOND(9)",
                   &OracleType::IntervalDS(9, 9), &it);
}

#[test]
fn interval_ds_to_sql() {
    let conn = common::connect().unwrap();
    let mut it = IntervalDS::new(1, 2, 3, 4, 0);

    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalDS(1,0),
                 "+000000001 02:03:04.000000000");
    it.nanoseconds = 123456789;
    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalDS(1,0),
                 "+000000001 02:03:04.123456789");
    it.days = 123456789;
    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalDS(1,0),
                 "+123456789 02:03:04.123456789");

    let mut it = IntervalDS::new(-1, -2, -3, -4, 0);

    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalDS(1,0),
                 "-000000001 02:03:04.000000000");
    it.nanoseconds = -123456789;
    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalDS(1,0),
                 "-000000001 02:03:04.123456789");
    it.days = -123456789;
    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalDS(1,0),
                 "-123456789 02:03:04.123456789");
}

#[test]
fn interval_ym_from_sql() {
    let conn = common::connect().unwrap();
    let mut it = IntervalYM::new(1, 2);

    test_from_sql!(&conn,
                   "INTERVAL '1-2' YEAR TO MONTH",
                   &OracleType::IntervalYM(2), &it);
    it.years = 123456789;
    it.precision = 9;
    test_from_sql!(&conn,
                   "INTERVAL '123456789-2' YEAR(9) TO MONTH",
                   &OracleType::IntervalYM(9), &it);

    let mut it = IntervalYM::new(-1, -2);

    test_from_sql!(&conn,
                   "INTERVAL '-1-2' YEAR TO MONTH",
                   &OracleType::IntervalYM(2), &it);
    it.years = -123456789;
    it.precision = 9;
    test_from_sql!(&conn,
                   "INTERVAL '-123456789-2' YEAR(9) TO MONTH",
                   &OracleType::IntervalYM(9), &it);
}

#[test]
fn interval_ym_to_sql() {
    let conn = common::connect().unwrap();
    let mut it = IntervalYM::new(1, 2);

    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalYM(2),
                 "+000000001-02");
    it.years = 123456789;
    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalYM(9),
                 "+123456789-02");

    let mut it = IntervalYM::new(-1, -2);

    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalYM(2),
                 "-000000001-02");
    it.years = -123456789;
    test_to_sql!(&conn, &it,
                 "TO_CHAR(:1)",
                 &OracleType::IntervalYM(9),
                 "-123456789-02");
}
