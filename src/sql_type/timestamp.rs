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

use crate::binding::dpiTimestamp;
use crate::sql_type::OracleType;
use crate::util::Scanner;
use crate::Error;
use crate::ParseOracleTypeError;
use crate::Result;
use std::cmp::{self, Ordering};
use std::fmt;
use std::result;
use std::str;

/// Oracle-specific [Datetime][] data type
///
/// [Datetime]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-3A1B7AC6-2EDB-4DDC-9C9D-223D4C72AC74
///
/// This struct doesn't have arithmetic methods and they won't be added to avoid
/// reinventing the wheel. If you need methods such as adding an interval to a
/// timestamp, enable `chrono` feature and use [chrono::Date][], [chrono::DateTime][],
/// [chrono::naive::NaiveDate][] or [chrono::naive::NaiveDateTime][] instead.
///
/// [chrono::Date]: https://docs.rs/chrono/0.4/chrono/struct.Date.html
/// [chrono::DateTime]: https://docs.rs/chrono/0.4/chrono/struct.DateTime.html
/// [chrono::naive::NaiveDate]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDate.html
/// [chrono::naive::NaiveDateTime]: https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDateTime.html
///
/// # Examples
///
/// ```
/// # use oracle::*; use oracle::sql_type::*;
/// // Create a timestamp.
/// let ts1 = Timestamp::new(2017, 8, 9, 11, 22, 33, 500000000)?;
///
/// // Convert to string.
/// assert_eq!(ts1.to_string(), "2017-08-09 11:22:33.500000000");
///
/// // Create a timestamp with time zone (-8:00).
/// let ts2 = Timestamp::new(2017, 8, 9, 11, 22, 33, 500000000)?.and_tz_hm_offset(-8, 0)?;
///
/// // Convert to string.
/// assert_eq!(ts2.to_string(), "2017-08-09 11:22:33.500000000 -08:00");
///
/// // Create a timestamp with precision
/// let ts3 = Timestamp::new(2017, 8, 9, 11, 22, 33, 500000000)?.and_prec(3)?;
///
/// // The string representation depends on the precision.
/// assert_eq!(ts3.to_string(), "2017-08-09 11:22:33.500");
///
/// // Precisions are ignored when intervals are compared.
/// assert_eq!(ts1, ts3);
///
/// // Create a timestamp from string.
/// let ts4: Timestamp = "2017-08-09 11:22:33.500 -08:00".parse()?;
///
/// // The precision is determined by number of decimal digits in the string.
/// assert_eq!(ts4.precision(), 3);
/// # Ok::<(), Error>(())
/// ```
///
/// Fetch and bind interval values.
///
/// ```no_run
/// # use oracle::*; use oracle::sql_type::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
///
/// // Fetch Timestamp
/// let sql = "select TIMESTAMP '2017-08-09 11:22:33.500' from dual";
/// let ts = conn.query_row_as::<Timestamp>(sql, &[])?;
/// assert_eq!(ts.to_string(), "2017-08-09 11:22:33.500000000");
///
/// // Bind Timestamp
/// let sql = "begin \
///              :outval := :inval + interval '+1 02:03:04.5' day to second; \
///            end;";
/// let mut stmt = conn.statement(sql).build()?;
/// stmt.execute(&[&OracleType::Timestamp(3), // bind null as timestamp(3)
///                &ts, // bind the ts variable
///               ])?;
/// let outval: Timestamp = stmt.bind_value(1)?; // get the first bind value.
/// // ts + (1 day, 2 hours, 3 minutes and 4.5 seconds)
/// assert_eq!(outval.to_string(), "2017-08-10 13:25:38.000");
/// # Ok::<(), Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Timestamp {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    nanosecond: u32,
    tz_hour_offset: i32,
    tz_minute_offset: i32,
    precision: u8,
    with_tz: bool,
}

impl Timestamp {
    fn check_validity(self) -> Result<Self> {
        if !(-4713..=9999).contains(&self.year) {
            Err(Error::out_of_range(format!(
                "year must be between -4713 and 9999 but {:?}",
                self
            )))
        } else if !(1..=12).contains(&self.month) {
            Err(Error::out_of_range(format!(
                "month must be between 1 and 12 but {:?}",
                self
            )))
        } else if !(1..=31).contains(&self.day) {
            Err(Error::out_of_range(format!(
                "day must be between 1 and 31 but {:?}",
                self
            )))
        } else if !(0..=23).contains(&self.hour) {
            Err(Error::out_of_range(format!(
                "hour must be between 0 and 23 but {:?}",
                self
            )))
        } else if !(0..=59).contains(&self.minute) {
            Err(Error::out_of_range(format!(
                "minute must be between 0 and 59 but {:?}",
                self
            )))
        } else if !(0..=59).contains(&self.second) {
            Err(Error::out_of_range(format!(
                "second must be between 0 and 59 but {:?}",
                self
            )))
        } else if !(0..=999999999).contains(&self.nanosecond) {
            Err(Error::out_of_range(format!(
                "nanosecond must be between 0 and 999,999,999 but {:?}",
                self
            )))
        } else {
            Ok(self)
        }
    }

    fn check_tz_hm_offset(hour_offset: i32, minute_offset: i32) -> Result<()> {
        if !(-59..=59).contains(&minute_offset) {
            Err(Error::out_of_range(format!(
                "minute_offset must be between -59 and 59 but {}",
                minute_offset
            )))
        } else if hour_offset < 0 && minute_offset > 0 {
            Err(Error::out_of_range(
                "hour_offset is negative but minimum is positive",
            ))
        } else if hour_offset > 0 && minute_offset < 0 {
            Err(Error::out_of_range(
                "hour_offset is positive but minimum is negative",
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn from_dpi_timestamp(ts: &dpiTimestamp, oratype: &OracleType) -> Timestamp {
        let (precision, with_tz) = match *oratype {
            OracleType::Timestamp(prec) => (prec, false),
            OracleType::TimestampTZ(prec) => (prec, true),
            OracleType::TimestampLTZ(prec) => (prec, true),
            _ => (0, false),
        };
        Timestamp {
            year: ts.year as i32,
            month: ts.month as u32,
            day: ts.day as u32,
            hour: ts.hour as u32,
            minute: ts.minute as u32,
            second: ts.second as u32,
            nanosecond: ts.fsecond,
            tz_hour_offset: ts.tzHourOffset as i32,
            tz_minute_offset: ts.tzMinuteOffset as i32,
            precision,
            with_tz,
        }
    }

    /// Creates a timestamp.
    ///
    /// Valid values are:
    ///
    /// | argument | valid values |
    /// |---|---|
    /// | `year` | -4713 to 9999 |
    /// | `month` | 1 to 12 |
    /// | `day` | 1 to 31 |
    /// | `hour` | 0 to 23 |
    /// | `minute` | 0 to 59 |
    /// | `second` | 0 to 59 |
    /// | `nanosecond` | 0 to 999,999,999 |
    ///
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nanosecond: u32,
    ) -> Result<Timestamp> {
        Timestamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
            tz_hour_offset: 0,
            tz_minute_offset: 0,
            precision: 9,
            with_tz: false,
        }
        .check_validity()
    }

    /// Creates a timestamp with time zone.
    ///
    /// `offset` is time zone offset seconds from UTC.
    #[inline]
    pub fn and_tz_offset(&self, offset: i32) -> Result<Timestamp> {
        self.and_tz_hm_offset(offset / 3600, offset % 3600 / 60)
    }

    /// Creates a timestamp with time zone.
    ///
    /// `hour_offset` and `minute_offset` are time zone offset in hours and minutes from UTC.
    /// All arguments must be zero or positive in the eastern hemisphere. They must be
    /// zero or negative in the western hemisphere.
    #[inline]
    pub fn and_tz_hm_offset(&self, hour_offset: i32, minute_offset: i32) -> Result<Timestamp> {
        Self::check_tz_hm_offset(hour_offset, minute_offset)?;
        Ok(Timestamp {
            tz_hour_offset: hour_offset,
            tz_minute_offset: minute_offset,
            with_tz: true,
            ..*self
        })
    }

    /// Creates a timestamp with precision.
    ///
    /// The precision affects text representation of Timestamp.
    /// It doesn't affect comparison.
    #[inline]
    pub fn and_prec(&self, precision: u8) -> Result<Timestamp> {
        if precision > 9 {
            Err(Error::out_of_range(format!(
                "precision must be 0 to 9 but {}",
                precision
            )))
        } else {
            Ok(Timestamp { precision, ..*self })
        }
    }

    /// Returns the year number from -4713 to 9999.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month number from 1 to 12.
    pub fn month(&self) -> u32 {
        self.month
    }

    /// Returns the day number from 1 to 31.
    pub fn day(&self) -> u32 {
        self.day
    }

    /// Returns the hour number from 0 to 23.
    pub fn hour(&self) -> u32 {
        self.hour
    }

    /// Returns the minute number from 0 to 59.
    pub fn minute(&self) -> u32 {
        self.minute
    }

    /// Returns the second number from 0 to 59.
    pub fn second(&self) -> u32 {
        self.second
    }

    /// Returns the nanosecond number from 0 to 999,999,999.
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }

    /// Returns hour component of time zone.
    pub fn tz_hour_offset(&self) -> i32 {
        self.tz_hour_offset
    }

    /// Returns minute component of time zone.
    pub fn tz_minute_offset(&self) -> i32 {
        self.tz_minute_offset
    }

    /// Returns precision
    pub fn precision(&self) -> u8 {
        self.precision
    }

    /// Returns true when the timestamp's text representation includes time zone information.
    /// Otherwise, false.
    pub fn with_tz(&self) -> bool {
        self.with_tz
    }

    /// Returns total time zone offset from UTC in seconds.
    pub fn tz_offset(&self) -> i32 {
        self.tz_hour_offset * 3600 + self.tz_minute_offset * 60
    }
}

impl cmp::PartialEq for Timestamp {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year
            && self.month == other.month
            && self.day == other.day
            && self.hour == other.hour
            && self.minute == other.minute
            && self.second == other.second
            && self.nanosecond == other.nanosecond
            && self.tz_hour_offset == other.tz_hour_offset
            && self.tz_minute_offset == other.tz_minute_offset
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )?;
        match self.precision {
            1 => write!(f, ".{:01}", self.nanosecond / 100000000)?,
            2 => write!(f, ".{:02}", self.nanosecond / 10000000)?,
            3 => write!(f, ".{:03}", self.nanosecond / 1000000)?,
            4 => write!(f, ".{:04}", self.nanosecond / 100000)?,
            5 => write!(f, ".{:05}", self.nanosecond / 10000)?,
            6 => write!(f, ".{:06}", self.nanosecond / 1000)?,
            7 => write!(f, ".{:07}", self.nanosecond / 100)?,
            8 => write!(f, ".{:08}", self.nanosecond / 10)?,
            9 => write!(f, ".{:09}", self.nanosecond)?,
            _ => (),
        }
        if self.with_tz {
            let sign = if self.tz_hour_offset < 0 || self.tz_minute_offset < 0 {
                '-'
            } else {
                '+'
            };
            write!(
                f,
                " {}{:02}:{:02}",
                sign,
                self.tz_hour_offset.abs(),
                self.tz_minute_offset.abs()
            )?;
        }
        Ok(())
    }
}

impl str::FromStr for Timestamp {
    type Err = ParseOracleTypeError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let err = || ParseOracleTypeError::new("Timestamp");
        let mut s = Scanner::new(s);
        let minus = if let Some('-') = s.char() {
            s.next();
            true
        } else {
            false
        };
        let mut year = s.read_digits().ok_or_else(err)?;
        let mut month = 1;
        let mut day = 1;
        match s.char() {
            Some('T') | Some(' ') | None => {
                if year > 10000 {
                    day = year % 100;
                    month = (year / 100) % 100;
                    year /= 10000;
                }
            }
            Some('-') => {
                s.next();
                month = s.read_digits().ok_or_else(err)?;
                if let Some('-') = s.char() {
                    s.next();
                    day = s.read_digits().ok_or_else(err)?
                }
            }
            _ => return Err(err()),
        }
        let mut hour = 0;
        let mut min = 0;
        let mut sec = 0;
        let mut nsec = 0;
        let mut tz_hour: i32 = 0;
        let mut tz_min: i32 = 0;
        let mut precision = 0;
        let mut with_tz = false;
        if let Some(c) = s.char() {
            match c {
                'T' | ' ' => {
                    s.next();
                    hour = s.read_digits().ok_or_else(err)?;
                    if let Some(':') = s.char() {
                        s.next();
                        min = s.read_digits().ok_or_else(err)?;
                        if let Some(':') = s.char() {
                            s.next();
                            sec = s.read_digits().ok_or_else(err)?;
                        }
                    } else if s.ndigits() == 6 {
                        // 123456 -> 12:34:56
                        sec = hour % 100;
                        min = (hour / 100) % 100;
                        hour /= 10000;
                    } else {
                        return Err(err());
                    }
                }
                _ => return Err(err()),
            }
            if let Some('.') = s.char() {
                s.next();
                nsec = s.read_digits().ok_or_else(err)?;
                let ndigit = s.ndigits();
                precision = ndigit;
                match ndigit.cmp(&9) {
                    Ordering::Less => nsec *= 10u64.pow(9 - ndigit),
                    Ordering::Equal => (),
                    Ordering::Greater => {
                        nsec /= 10u64.pow(ndigit - 9);
                        precision = 9;
                    }
                }
            }
            if let Some(' ') = s.char() {
                s.next();
            }
            match s.char() {
                Some('+') => {
                    s.next();
                    tz_hour = s.read_digits().ok_or_else(err)? as i32;
                    if let Some(':') = s.char() {
                        s.next();
                        tz_min = s.read_digits().ok_or_else(err)? as i32;
                    } else {
                        tz_min = tz_hour % 100;
                        tz_hour /= 100;
                    }
                    with_tz = true;
                }
                Some('-') => {
                    s.next();
                    tz_hour = s.read_digits().ok_or_else(err)? as i32;
                    if let Some(':') = s.char() {
                        s.next();
                        tz_min = s.read_digits().ok_or_else(err)? as i32;
                    } else {
                        tz_min = tz_hour % 100;
                        tz_hour /= 100;
                    }
                    tz_hour = -tz_hour;
                    tz_min = -tz_min;
                    with_tz = true;
                }
                Some('Z') => {
                    s.next();
                    with_tz = true;
                }
                _ => (),
            }
            if s.char().is_some() {
                return Err(err());
            }
        }
        let mut ts = Timestamp::new(
            if minus { -(year as i32) } else { year as i32 },
            month as u32,
            day as u32,
            hour as u32,
            min as u32,
            sec as u32,
            nsec as u32,
        )
        .map_err(|_| err())?;
        ts.precision = precision as u8;
        if with_tz {
            ts = ts.and_tz_hm_offset(tz_hour, tz_min).map_err(|_| err())?;
        }
        Ok(ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() -> Result<()> {
        let mut ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 890123456)?.and_tz_hm_offset(8, 45)?;
        ts.with_tz = false;
        ts.precision = 0;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07");
        ts.precision = 1;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.8");
        ts.precision = 2;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.89");
        ts.precision = 3;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.890");
        ts.precision = 4;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.8901");
        ts.precision = 5;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.89012");
        ts.precision = 6;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.890123");
        ts.precision = 7;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.8901234");
        ts.precision = 8;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.89012345");
        ts.precision = 9;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.890123456");
        ts.with_tz = true;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.890123456 +08:45");
        ts.tz_hour_offset = -8;
        ts.tz_minute_offset = -45;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.890123456 -08:45");
        ts.precision = 0;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07 -08:45");
        ts.year = -123;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 -08:45");
        let mut ts = ts.and_tz_offset(-3600 - 1800)?;
        assert_eq!(ts.tz_hour_offset, -1);
        assert_eq!(ts.tz_minute_offset, -30);
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 -01:30");
        ts.tz_hour_offset = 0;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 -00:30");
        ts.tz_minute_offset = 30;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 +00:30");
        ts.tz_minute_offset = 0;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 +00:00");
        Ok(())
    }

    #[test]
    fn parse() -> Result<()> {
        let mut ts = Timestamp::new(
            2012, 1, 1, // year, month, day,
            0, 0, 0, 0,
        )?; // hour, minute, second, nanosecond,
        ts.precision = 0;
        assert_eq!("2012".parse(), Ok(ts));
        ts.month = 3;
        ts.day = 4;
        assert_eq!("20120304".parse(), Ok(ts));
        assert_eq!("2012-03-04".parse(), Ok(ts));
        ts.hour = 5;
        ts.minute = 6;
        ts.second = 7;
        assert_eq!("2012-03-04 05:06:07".parse(), Ok(ts));
        assert_eq!("2012-03-04T05:06:07".parse(), Ok(ts));
        assert_eq!("20120304T050607".parse(), Ok(ts));
        ts.nanosecond = 800000000;
        ts.precision = 1;
        assert_eq!("2012-03-04 05:06:07.8".parse(), Ok(ts));
        ts.nanosecond = 890000000;
        ts.precision = 2;
        assert_eq!("2012-03-04T05:06:07.89".parse(), Ok(ts));
        ts.nanosecond = 890000000;
        ts.precision = 3;
        assert_eq!("20120304T050607.890".parse(), Ok(ts));
        ts.nanosecond = 890100000;
        ts.precision = 4;
        assert_eq!("2012-03-04 05:06:07.8901".parse(), Ok(ts));
        ts.nanosecond = 890120000;
        ts.precision = 5;
        assert_eq!("2012-03-04 05:06:07.89012".parse(), Ok(ts));
        ts.nanosecond = 890123000;
        ts.precision = 6;
        assert_eq!("2012-03-04 05:06:07.890123".parse(), Ok(ts));
        ts.nanosecond = 890123400;
        ts.precision = 7;
        assert_eq!("2012-03-04 05:06:07.8901234".parse(), Ok(ts));
        ts.nanosecond = 890123450;
        ts.precision = 8;
        assert_eq!("2012-03-04 05:06:07.89012345".parse(), Ok(ts));
        ts.nanosecond = 890123456;
        ts.precision = 9;
        assert_eq!("2012-03-04 05:06:07.890123456".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07.8901234567".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07.89012345678".parse(), Ok(ts));
        ts.with_tz = true;
        ts.nanosecond = 0;
        ts.precision = 0;
        assert_eq!("2012-03-04 05:06:07Z".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07+00:00".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +00:00".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07+0000".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +0000".parse(), Ok(ts));
        ts.tz_hour_offset = 8;
        ts.tz_minute_offset = 45;
        assert_eq!("2012-03-04 05:06:07+08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07+0845".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +0845".parse(), Ok(ts));
        ts.tz_hour_offset = -8;
        ts.tz_minute_offset = -45;
        assert_eq!("2012-03-04 05:06:07-08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 -08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07-0845".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 -0845".parse(), Ok(ts));
        ts.nanosecond = 123000000;
        ts.precision = 3;
        assert_eq!("2012-03-04 05:06:07.123-08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07.123 -08:45".parse(), Ok(ts));
        ts.year = -123;
        assert_eq!("-123-03-04 05:06:07.123 -08:45".parse(), Ok(ts));
        ts.tz_hour_offset = 0;
        assert_eq!("-123-03-04 05:06:07.123 -00:45".parse(), Ok(ts));
        ts.tz_minute_offset = 45;
        assert_eq!("-123-03-04 05:06:07.123 +00:45".parse(), Ok(ts));
        Ok(())
    }
}
