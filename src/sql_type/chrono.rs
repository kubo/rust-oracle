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

use chrono::prelude::*;

use crate::sql_type::FromSql;
use crate::sql_type::IntervalDS;
use crate::sql_type::OracleType;
use crate::sql_type::Timestamp;
use crate::sql_type::ToSql;
use crate::sql_type::ToSqlNull;
use crate::Connection;
use crate::Error;
use crate::Result;
use crate::SqlValue;
use chrono::naive::NaiveDate;
use chrono::naive::NaiveDateTime;
use chrono::offset::LocalResult;
use chrono::Duration;

fn fixed_offset_from_sql(ts: &Timestamp) -> Result<FixedOffset> {
    FixedOffset::east_opt(ts.tz_offset())
        .ok_or_else(|| Error::out_of_range(format!("invalid time zone offset: {}", ts.tz_offset())))
}

//
// chrono::DateTime<Utc>
// chrono::DateTime<Local>
// chrono::DateTime<FixedOffset>
//

fn datetime_from_sql<Tz>(tz: &Tz, ts: &Timestamp) -> Result<DateTime<Tz>>
where
    Tz: TimeZone,
{
    date_from_sql(tz, ts)?
        .and_hms_nano_opt(ts.hour(), ts.minute(), ts.second(), ts.nanosecond())
        .ok_or_else(|| {
            Error::out_of_range(format!(
                "invalid year-month-day: {}-{}-{} {}:{}:{}.{:09}",
                ts.year(),
                ts.month(),
                ts.day(),
                ts.hour(),
                ts.minute(),
                ts.second(),
                ts.nanosecond()
            ))
        })
}

impl FromSql for DateTime<Utc> {
    fn from_sql(val: &SqlValue) -> Result<DateTime<Utc>> {
        let ts = val.to_timestamp()?;
        datetime_from_sql(&Utc, &ts)
    }
}

impl FromSql for DateTime<Local> {
    fn from_sql(val: &SqlValue) -> Result<DateTime<Local>> {
        let ts = val.to_timestamp()?;
        datetime_from_sql(&Local, &ts)
    }
}

impl FromSql for DateTime<FixedOffset> {
    fn from_sql(val: &SqlValue) -> Result<DateTime<FixedOffset>> {
        let ts = val.to_timestamp()?;
        datetime_from_sql(&fixed_offset_from_sql(&ts)?, &ts)
    }
}

impl<Tz> ToSqlNull for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::TimestampTZ(9))
    }
}

impl<Tz> ToSql for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::TimestampTZ(9))
    }

    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        let ts = Timestamp::new(
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second(),
            self.nanosecond(),
        )?;
        let ts = ts.and_tz_offset(self.offset().fix().local_minus_utc())?;
        val.set_timestamp(&ts)
    }
}

//
// chrono::Date<Utc>
// chrono::Date<Local>
// chrono::Date<FixedOffset>
//
// chrono::Date has been deprecated since chrono 0.4.23.
// However it cannot be removed without a breaking change.
// So `#[allow(deprecated)]` was added for each trait implementation
// to suppress warnings.

#[allow(deprecated)]
fn date_from_sql<Tz>(tz: &Tz, ts: &Timestamp) -> Result<Date<Tz>>
where
    Tz: TimeZone,
{
    match tz.ymd_opt(ts.year(), ts.month(), ts.day()) {
        LocalResult::Single(date) => Ok(date),
        _ => Err(Error::out_of_range(format!(
            "invalid month and/or day: {}-{}-{}",
            ts.year(),
            ts.month(),
            ts.day()
        ))),
    }
}

#[allow(deprecated)]
impl FromSql for Date<Utc> {
    fn from_sql(val: &SqlValue) -> Result<Date<Utc>> {
        let ts = val.to_timestamp()?;
        date_from_sql(&Utc, &ts)
    }
}

#[allow(deprecated)]
impl FromSql for Date<Local> {
    fn from_sql(val: &SqlValue) -> Result<Date<Local>> {
        let ts = val.to_timestamp()?;
        date_from_sql(&Local, &ts)
    }
}

#[allow(deprecated)]
impl FromSql for Date<FixedOffset> {
    fn from_sql(val: &SqlValue) -> Result<Date<FixedOffset>> {
        let ts = val.to_timestamp()?;
        date_from_sql(&fixed_offset_from_sql(&ts)?, &ts)
    }
}

#[allow(deprecated)]
impl<Tz> ToSqlNull for Date<Tz>
where
    Tz: TimeZone,
{
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::TimestampTZ(0))
    }
}

#[allow(deprecated)]
impl<Tz> ToSql for Date<Tz>
where
    Tz: TimeZone,
{
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::TimestampTZ(0))
    }

    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        let ts = Timestamp::new(self.year(), self.month(), self.day(), 0, 0, 0, 0)?;
        let ts = ts.and_tz_offset(self.offset().fix().local_minus_utc())?;
        val.set_timestamp(&ts)
    }
}

//
// chrono::naive::NaiveDateTime
//

fn naive_date_time_from_sql(ts: &Timestamp) -> Result<NaiveDateTime> {
    naive_date_from_sql(ts)?
        .and_hms_nano_opt(ts.hour(), ts.minute(), ts.second(), ts.nanosecond())
        .ok_or_else(|| {
            Error::out_of_range(format!(
                "invalid year-month-day: {}-{}-{} {}:{}:{}.{:09}",
                ts.year(),
                ts.month(),
                ts.day(),
                ts.hour(),
                ts.minute(),
                ts.second(),
                ts.nanosecond()
            ))
        })
}

impl FromSql for NaiveDateTime {
    fn from_sql(val: &SqlValue) -> Result<NaiveDateTime> {
        let ts = val.to_timestamp()?;
        naive_date_time_from_sql(&ts)
    }
}

impl ToSqlNull for NaiveDateTime {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Timestamp(9))
    }
}

impl ToSql for NaiveDateTime {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Timestamp(9))
    }

    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        let ts = Timestamp::new(
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second(),
            self.nanosecond(),
        )?;
        val.set_timestamp(&ts)
    }
}

//
// chrono::naive::NaiveDate
//

fn naive_date_from_sql(ts: &Timestamp) -> Result<NaiveDate> {
    NaiveDate::from_ymd_opt(ts.year(), ts.month(), ts.day()).ok_or_else(|| {
        Error::out_of_range(format!(
            "invalid year-month-day: {}-{}-{}",
            ts.year(),
            ts.month(),
            ts.day()
        ))
    })
}

impl FromSql for NaiveDate {
    fn from_sql(val: &SqlValue) -> Result<NaiveDate> {
        let ts = val.to_timestamp()?;
        naive_date_from_sql(&ts)
    }
}

impl ToSqlNull for NaiveDate {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Timestamp(0))
    }
}

impl ToSql for NaiveDate {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Timestamp(0))
    }

    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        let ts = Timestamp::new(self.year(), self.month(), self.day(), 0, 0, 0, 0)?;
        val.set_timestamp(&ts)
    }
}

//
// chrono::Duration
//

impl FromSql for Duration {
    fn from_sql(val: &SqlValue) -> Result<Duration> {
        let err = |it: IntervalDS| {
            Error::out_of_range(format!(
                "unable to convert interval day to second {} to chrono::Duration",
                it
            ))
        };
        let it = val.to_interval_ds()?;
        let d = Duration::milliseconds(0);
        let d = d
            .checked_add(&Duration::days(it.days() as i64))
            .ok_or_else(|| err(it))?;
        let d = d
            .checked_add(&Duration::hours(it.hours() as i64))
            .ok_or_else(|| err(it))?;
        let d = d
            .checked_add(&Duration::minutes(it.minutes() as i64))
            .ok_or_else(|| err(it))?;
        let d = d
            .checked_add(&Duration::seconds(it.seconds() as i64))
            .ok_or_else(|| err(it))?;
        let d = d
            .checked_add(&Duration::nanoseconds(it.nanoseconds() as i64))
            .ok_or_else(|| err(it))?;
        Ok(d)
    }
}

impl ToSqlNull for Duration {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::IntervalDS(9, 9))
    }
}

impl ToSql for Duration {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::IntervalDS(9, 9))
    }

    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        let secs = self.num_seconds();
        let nsecs = (*self - Duration::seconds(secs)).num_nanoseconds().unwrap();
        let days = secs / (24 * 60 * 60);
        let secs = secs % (24 * 60 * 60);
        let hours = secs / (60 * 60);
        let secs = secs % (60 * 60);
        let minutes = secs / 60;
        let secs = secs % 60;
        if days.abs() >= 1000000000 {
            return Err(Error::out_of_range(format!("too large days: {}", self)));
        }
        let it = IntervalDS::new(
            days as i32,
            hours as i32,
            minutes as i32,
            secs as i32,
            nsecs as i32,
        )?;
        val.set_interval_ds(&it)
    }
}
