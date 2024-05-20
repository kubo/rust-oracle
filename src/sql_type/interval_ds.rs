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

use crate::binding::dpiIntervalDS;
use crate::sql_type::OracleType;
use crate::util::Scanner;
use crate::Error;
use crate::ParseOracleTypeError;
use crate::Result;
use std::cmp::{self, Ordering};
use std::fmt;
use std::result;
use std::str;

/// Oracle-specific [Interval Day to Second][INTVL_DS] data type.
///
/// [INTVL_DS]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FD8C41B7-8CDC-4D02-8E6B-5250416BC17D
///
/// This struct doesn't have arithmetic methods and they won't be added to avoid
/// reinventing the wheel. If you need methods such as adding an interval to a
/// timestamp, enable `chrono` feature and use [chrono::Duration][] instead.
///
/// [chrono::Duration]: https://docs.rs/chrono/0.4/chrono/struct.Duration.html
///
/// # Examples
///
/// ```
/// # use oracle::*; use oracle::sql_type::*;
///
/// // Create an interval by new().
/// let intvl1 = IntervalDS::new(1, 2, 3, 4, 500000000)?;
///
/// // All arguments must be zero or negative to create a negative interval.
/// let intvl2 = IntervalDS::new(-1, -2, -3, -4, -500000000)?;
///
/// // Convert to string.
/// assert_eq!(intvl1.to_string(), "+000000001 02:03:04.500000000");
/// assert_eq!(intvl2.to_string(), "-000000001 02:03:04.500000000");
///
/// // Create an interval with leading field and fractional second precisions.
/// let intvl3 = IntervalDS::new(1, 2, 3, 4, 500000000)?.and_prec(2, 3)?;
///
/// // The string representation depends on the precisions.
/// assert_eq!(intvl3.to_string(), "+01 02:03:04.500");
///
/// // Precisions are ignored when intervals are compared.
/// assert!(intvl1 == intvl3);
///
/// // Create an interval from string.
/// let intvl4: IntervalDS = "+1 02:03:04.50".parse()?;
///
/// // The precisions are determined by number of decimal digits in the string.
/// assert_eq!(intvl4.lfprec(), 1);
/// assert_eq!(intvl4.fsprec(), 2);
/// # Ok::<(), Error>(())
/// ```
///
/// Fetch and bind interval values.
///
/// ```no_run
/// # use oracle::*; use oracle::sql_type::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
///
/// // Fetch IntervalDS
/// let sql = "select interval '+01 02:03:04.500' day to second(3) from dual";
/// let intvl = conn.query_row_as::<IntervalDS>(sql, &[])?;
/// assert_eq!(intvl.to_string(), "+01 02:03:04.500");
///
/// // Bind IntervalDS
/// let sql = "begin \
///              :outval := to_timestamp('2017-08-09', 'yyyy-mm-dd') + :inval; \
///            end;";
/// let mut stmt = conn.statement(sql).build()?;
/// stmt.execute(&[&OracleType::Timestamp(3), // bind null as timestamp(3)
///                &intvl, // bind the intvl variable
///               ])?;
/// let outval: Timestamp = stmt.bind_value(1)?; // get the first bind value.
/// // 2017-08-09 + (1 day, 2 hours, 3 minutes and 4.5 seconds)
/// assert_eq!(outval.to_string(), "2017-08-10 02:03:04.500");
/// # Ok::<(), Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct IntervalDS {
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    nanoseconds: i32,
    lfprec: u8,
    fsprec: u8,
}

impl IntervalDS {
    fn check_validity(self) -> Result<Self> {
        if !(-999999999..=999999999).contains(&self.days) {
            Err(Error::out_of_range(format!(
                "days must be between -999999999 and 999999999 but {:?}",
                self
            )))
        } else if !(-23..=23).contains(&self.hours) {
            Err(Error::out_of_range(format!(
                "hours must be between -23 and 23 but {:?}",
                self
            )))
        } else if !(-59..=59).contains(&self.minutes) {
            Err(Error::out_of_range(format!(
                "minutes must be between -59 and 59 but {:?}",
                self
            )))
        } else if !(-59..=59).contains(&self.seconds) {
            Err(Error::out_of_range(format!(
                "seconds must be between -59 and 59 but {:?}",
                self
            )))
        } else if !(-999999999..=999999999).contains(&self.nanoseconds) {
            Err(Error::out_of_range(format!(
                "nanoseconds must be between -999999999 and 999999999 but {:?}",
                self
            )))
        } else if self.days >= 0
            && self.hours >= 0
            && self.minutes >= 0
            && self.seconds >= 0
            && self.nanoseconds >= 0
        {
            // all members are zero or positive.
            Ok(self)
        } else if self.days <= 0
            && self.hours <= 0
            && self.minutes <= 0
            && self.seconds <= 0
            && self.nanoseconds <= 0
        {
            // all members are zero or negative.
            Ok(self)
        } else {
            Err(Error::out_of_range(format!("days, hours, minutes, seconds and nanoseconds must be zeor or positive; or zero or negative but {:?}", self)))
        }
    }

    pub(crate) fn from_dpi_interval_ds(
        it: &dpiIntervalDS,
        oratype: &OracleType,
    ) -> Result<IntervalDS> {
        let (lfprec, fsprec) = match *oratype {
            OracleType::IntervalDS(lfprec, fsprec) => (lfprec, fsprec),
            _ => (0, 0),
        };
        IntervalDS::new(it.days, it.hours, it.minutes, it.seconds, it.fseconds)?
            .and_prec(lfprec, fsprec)
    }

    /// Creates a new IntervalDS.
    ///
    /// Valid values are:
    ///
    /// | argument | valid values |
    /// |---|---|
    /// | `days` | -999999999 to 999999999 |
    /// | `hours` | -23 to 23 |
    /// | `minutes` | -59 to 59 |
    /// | `seconds` | -59 to 59 |
    /// | `nanoseconds` | -999999999 to 999999999 |
    ///
    /// All arguments must be zero or positive to create a positive interval.
    /// All arguments must be zero or negative to create a negative interval.
    pub fn new(
        days: i32,
        hours: i32,
        minutes: i32,
        seconds: i32,
        nanoseconds: i32,
    ) -> Result<IntervalDS> {
        IntervalDS {
            days,
            hours,
            minutes,
            seconds,
            nanoseconds,
            lfprec: 9,
            fsprec: 9,
        }
        .check_validity()
    }

    /// Creates a new IntervalDS with precisions.
    ///
    /// `lfprec` and `fsprec` are leading field precision and fractional second
    /// precision respectively.
    /// The precisions affect text representation of IntervalDS.
    /// They don't affect comparison.
    pub fn and_prec(&self, lfprec: u8, fsprec: u8) -> Result<IntervalDS> {
        if lfprec > 9 {
            Err(Error::out_of_range(format!(
                "lfprec must be 0 to 9 but {}",
                lfprec
            )))
        } else if fsprec > 9 {
            Err(Error::out_of_range(format!(
                "fsprec must be 0 to 9 but {}",
                fsprec
            )))
        } else {
            Ok(IntervalDS {
                lfprec,
                fsprec,
                ..*self
            })
        }
    }

    /// Returns days component.
    pub fn days(&self) -> i32 {
        self.days
    }

    /// Returns hours component.
    pub fn hours(&self) -> i32 {
        self.hours
    }

    /// Returns minutes component.
    pub fn minutes(&self) -> i32 {
        self.minutes
    }

    /// Returns seconds component.
    pub fn seconds(&self) -> i32 {
        self.seconds
    }

    /// Returns nanoseconds component.
    pub fn nanoseconds(&self) -> i32 {
        self.nanoseconds
    }

    /// Returns leading field precision.
    pub fn lfprec(&self) -> u8 {
        self.lfprec
    }

    /// Returns fractional second precision.
    pub fn fsprec(&self) -> u8 {
        self.fsprec
    }
}

impl cmp::PartialEq for IntervalDS {
    fn eq(&self, other: &Self) -> bool {
        self.days == other.days
            && self.hours == other.hours
            && self.minutes == other.minutes
            && self.seconds == other.seconds
            && self.nanoseconds == other.nanoseconds
    }
}

impl fmt::Display for IntervalDS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.days < 0
            || self.hours < 0
            || self.minutes < 0
            || self.seconds < 0
            || self.nanoseconds < 0
        {
            write!(f, "-")?;
        } else {
            write!(f, "+")?;
        };
        let days = self.days.abs();
        match self.lfprec {
            2 => write!(f, "{:02}", days)?,
            3 => write!(f, "{:03}", days)?,
            4 => write!(f, "{:04}", days)?,
            5 => write!(f, "{:05}", days)?,
            6 => write!(f, "{:06}", days)?,
            7 => write!(f, "{:07}", days)?,
            8 => write!(f, "{:08}", days)?,
            9 => write!(f, "{:09}", days)?,
            _ => write!(f, "{}", days)?,
        };
        write!(
            f,
            " {:02}:{:02}:{:02}",
            self.hours.abs(),
            self.minutes.abs(),
            self.seconds.abs()
        )?;
        let nsec = self.nanoseconds.abs();
        match self.fsprec {
            1 => write!(f, ".{:01}", nsec / 100000000),
            2 => write!(f, ".{:02}", nsec / 10000000),
            3 => write!(f, ".{:03}", nsec / 1000000),
            4 => write!(f, ".{:04}", nsec / 100000),
            5 => write!(f, ".{:05}", nsec / 10000),
            6 => write!(f, ".{:06}", nsec / 1000),
            7 => write!(f, ".{:07}", nsec / 100),
            8 => write!(f, ".{:08}", nsec / 10),
            9 => write!(f, ".{:09}", nsec),
            _ => Ok(()),
        }
    }
}

impl str::FromStr for IntervalDS {
    type Err = ParseOracleTypeError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let err = || ParseOracleTypeError::new("IntervalDS");
        let mut s = Scanner::new(s);
        let minus = match s.char() {
            Some('+') => {
                s.next();
                false
            }
            Some('-') => {
                s.next();
                true
            }
            _ => false,
        };
        let days = s.read_digits().ok_or_else(err)? as i32;
        let lfprec = s.ndigits();
        if let Some(' ') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let hours = s.read_digits().ok_or_else(err)? as i32;
        if let Some(':') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let minutes = s.read_digits().ok_or_else(err)? as i32;
        if let Some(':') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let seconds = s.read_digits().ok_or_else(err)? as i32;
        let mut nsecs = 0;
        let mut fsprec = 0;
        if let Some('.') = s.char() {
            s.next();
            nsecs = s.read_digits().ok_or_else(err)? as i32;
            let ndigit = s.ndigits();
            fsprec = ndigit;
            match ndigit.cmp(&9) {
                Ordering::Less => nsecs *= 10i32.pow(9 - ndigit),
                Ordering::Equal => (),
                Ordering::Greater => {
                    nsecs /= 10i32.pow(ndigit - 9);
                    fsprec = 9;
                }
            }
        }
        if s.char().is_some() {
            return Err(err());
        }
        Ok(IntervalDS {
            days: if minus { -days } else { days },
            hours: if minus { -hours } else { hours },
            minutes: if minus { -minutes } else { minutes },
            seconds: if minus { -seconds } else { seconds },
            nanoseconds: if minus { -nsecs } else { nsecs },
            lfprec: lfprec as u8,
            fsprec: fsprec as u8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() -> Result<()> {
        let mut it = IntervalDS::new(1, 2, 3, 4, 123456789)?;
        it.lfprec = 0;
        it.fsprec = 0;
        assert_eq!(it.to_string(), "+1 02:03:04");
        it.fsprec = 1;
        assert_eq!(it.to_string(), "+1 02:03:04.1");
        it.fsprec = 2;
        assert_eq!(it.to_string(), "+1 02:03:04.12");
        it.fsprec = 3;
        assert_eq!(it.to_string(), "+1 02:03:04.123");
        it.fsprec = 4;
        assert_eq!(it.to_string(), "+1 02:03:04.1234");
        it.fsprec = 5;
        assert_eq!(it.to_string(), "+1 02:03:04.12345");
        it.fsprec = 6;
        assert_eq!(it.to_string(), "+1 02:03:04.123456");
        it.fsprec = 7;
        assert_eq!(it.to_string(), "+1 02:03:04.1234567");
        it.fsprec = 8;
        assert_eq!(it.to_string(), "+1 02:03:04.12345678");
        it.fsprec = 9;
        assert_eq!(it.to_string(), "+1 02:03:04.123456789");

        let mut it = IntervalDS::new(-1, -2, -3, -4, -123456789)?;
        it.lfprec = 0;
        it.fsprec = 0;
        assert_eq!(it.to_string(), "-1 02:03:04");
        it.fsprec = 1;
        assert_eq!(it.to_string(), "-1 02:03:04.1");
        it.fsprec = 2;
        assert_eq!(it.to_string(), "-1 02:03:04.12");
        it.fsprec = 3;
        assert_eq!(it.to_string(), "-1 02:03:04.123");
        it.fsprec = 4;
        assert_eq!(it.to_string(), "-1 02:03:04.1234");
        it.fsprec = 5;
        assert_eq!(it.to_string(), "-1 02:03:04.12345");
        it.fsprec = 6;
        assert_eq!(it.to_string(), "-1 02:03:04.123456");
        it.fsprec = 7;
        assert_eq!(it.to_string(), "-1 02:03:04.1234567");
        it.fsprec = 8;
        assert_eq!(it.to_string(), "-1 02:03:04.12345678");
        it.fsprec = 9;
        assert_eq!(it.to_string(), "-1 02:03:04.123456789");

        it.lfprec = 1;
        assert_eq!(it.to_string(), "-1 02:03:04.123456789");
        it.lfprec = 2;
        assert_eq!(it.to_string(), "-01 02:03:04.123456789");
        it.lfprec = 3;
        assert_eq!(it.to_string(), "-001 02:03:04.123456789");
        it.lfprec = 4;
        assert_eq!(it.to_string(), "-0001 02:03:04.123456789");
        it.lfprec = 5;
        assert_eq!(it.to_string(), "-00001 02:03:04.123456789");
        it.lfprec = 6;
        assert_eq!(it.to_string(), "-000001 02:03:04.123456789");
        it.lfprec = 7;
        assert_eq!(it.to_string(), "-0000001 02:03:04.123456789");
        it.lfprec = 8;
        assert_eq!(it.to_string(), "-00000001 02:03:04.123456789");
        it.lfprec = 9;
        assert_eq!(it.to_string(), "-000000001 02:03:04.123456789");
        Ok(())
    }

    #[test]
    fn parse() -> Result<()> {
        let mut it = IntervalDS::new(1, 2, 3, 4, 0)?;
        it.lfprec = 1;
        it.fsprec = 0;
        assert_eq!("1 02:03:04".parse(), Ok(it));
        assert_eq!("+1 02:03:04".parse(), Ok(it));
        it.lfprec = 2;
        assert_eq!("01 02:03:04".parse(), Ok(it));
        it.lfprec = 3;
        assert_eq!("001 02:03:04".parse(), Ok(it));
        it.lfprec = 4;
        assert_eq!("0001 02:03:04".parse(), Ok(it));
        it.lfprec = 5;
        assert_eq!("00001 02:03:04".parse(), Ok(it));
        it.lfprec = 6;
        assert_eq!("000001 02:03:04".parse(), Ok(it));
        it.lfprec = 7;
        assert_eq!("0000001 02:03:04".parse(), Ok(it));
        it.lfprec = 8;
        assert_eq!("00000001 02:03:04".parse(), Ok(it));
        it.lfprec = 9;
        assert_eq!("000000001 02:03:04".parse(), Ok(it));

        it.fsprec = 1;
        it.nanoseconds = 100000000;
        assert_eq!("000000001 02:03:04.1".parse(), Ok(it));

        let mut it = IntervalDS::new(-1, -2, -3, -4, 0)?;
        it.lfprec = 1;
        it.fsprec = 0;
        assert_eq!("-1 02:03:04".parse(), Ok(it));

        it.fsprec = 1;
        it.nanoseconds = -100000000;
        assert_eq!("-1 02:03:04.1".parse(), Ok(it));
        it.fsprec = 2;
        it.nanoseconds = -120000000;
        assert_eq!("-1 02:03:04.12".parse(), Ok(it));
        it.fsprec = 3;
        it.nanoseconds = -123000000;
        assert_eq!("-1 02:03:04.123".parse(), Ok(it));
        it.fsprec = 4;
        it.nanoseconds = -123400000;
        assert_eq!("-1 02:03:04.1234".parse(), Ok(it));
        it.fsprec = 5;
        it.nanoseconds = -123450000;
        assert_eq!("-1 02:03:04.12345".parse(), Ok(it));
        it.fsprec = 6;
        it.nanoseconds = -123456000;
        assert_eq!("-1 02:03:04.123456".parse(), Ok(it));
        it.fsprec = 7;
        it.nanoseconds = -123456700;
        assert_eq!("-1 02:03:04.1234567".parse(), Ok(it));
        it.fsprec = 8;
        it.nanoseconds = -123456780;
        assert_eq!("-1 02:03:04.12345678".parse(), Ok(it));
        it.fsprec = 9;
        it.nanoseconds = -123456789;
        assert_eq!("-1 02:03:04.123456789".parse(), Ok(it));
        Ok(())
    }
}
