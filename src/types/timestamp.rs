// Rust Oracle - Rust binding for Oracle database
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

use std::cmp;
use std::fmt;
use std::str;

use binding::dpiTimestamp;
use util::Scanner;
use OracleType;
use ParseOracleTypeError;

/// Timestamp type corresponding to Oracle datetime types: `DATE`, `TIMESTAMP`,
/// `TIMESTAMP WITH TIME ZONE` and `TIMESTAMP WITH LOCAL TIME ZONE`.
///
/// Don't use this type directly in your applications. This is public
/// for types implementing `FromSql` and `ToSql` traits.
#[derive(Debug, Clone, Copy)]
pub struct Timestamp {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub nanosecond: u32,
    pub tz_hour_offset: i32,
    pub tz_minute_offset: i32,
    pub precision: u8,
    pub with_tz: bool,
}

impl Timestamp {
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
            nanosecond: ts.fsecond as u32,
            tz_hour_offset: ts.tzHourOffset as i32,
            tz_minute_offset: ts.tzMinuteOffset as i32,
            precision: precision,
            with_tz: with_tz,
        }
    }

    pub fn new(year: i32, month: u32, day: u32,
               hour: u32, minute: u32, second: u32, nanosecond: u32) -> Timestamp {
        Timestamp {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            nanosecond: nanosecond,
            tz_hour_offset: 0,
            tz_minute_offset: 0,
            precision: 9,
            with_tz: false,
        }
    }

    #[inline]
    pub fn and_tz_offset(&self, offset: i32) -> Timestamp {
        Timestamp {
            tz_hour_offset: offset / 3600,
            tz_minute_offset: offset % 3600 / 60,
            with_tz: true,
            .. *self
        }
    }

    #[inline]
    pub fn and_tz_hm_offset(&self, hour_offset: i32, minute_offset: i32) -> Timestamp {
        Timestamp {
            tz_hour_offset: hour_offset,
            tz_minute_offset: minute_offset,
            with_tz: true,
            .. *self
        }
    }

    #[inline]
    pub fn with_precision(&self, precision: u8) -> Timestamp {
        Timestamp {
            precision: precision,
            .. *self
        }
    }

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
        write!(f, "{}-{:02}-{:02} {:02}:{:02}:{:02}", self.year, self.month, self.day, self.hour, self.minute, self.second)?;
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
            write!(f, " {}{:02}:{:02}", sign,
                   self.tz_hour_offset.abs(), self.tz_minute_offset.abs())?;
        }
        Ok(())
    }
}

impl str::FromStr for Timestamp {
    type Err = ParseOracleTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseOracleTypeError::new("Timestamp");
        let mut s = Scanner::new(s);
        let minus = if let Some('-') = s.char() {
            s.next();
            true
        } else {
            false
        };
        let mut year = s.read_digits().ok_or(err())?;
        let mut month = 1;
        let mut day = 1;
        match s.char() {
            Some('T') | Some(' ') | None => {
                if year > 10000 {
                    day = year % 100;
                    month = (year / 100) % 100;
                    year /= 10000;
                }
            },
            Some('-') => {
                s.next();
                month = s.read_digits().ok_or(err())?;
                if let Some('-') = s.char() {
                    s.next();
                    day = s.read_digits().ok_or(err())?
                }
            },
            _ => return Err(err()),
        }
        let mut hour = 0;
        let mut min = 0;
        let mut sec = 0;
        let mut nsec = 0;
        let mut tz_hour:i32 = 0;
        let mut tz_min:i32 = 0;
        let mut precision = 0;
        let mut with_tz = false;
        if let Some(c) = s.char() {
            match c {
                'T' | ' ' => {
                    s.next();
                    hour = s.read_digits().ok_or(err())?;
                    if let Some(':') = s.char() {
                        s.next();
                        min = s.read_digits().ok_or(err())?;
                        if let Some(':') = s.char() {
                            s.next();
                            sec = s.read_digits().ok_or(err())?;
                        }
                    } else if s.ndigits() == 6 {
                        // 123456 -> 12:34:56
                        sec = hour % 100;
                        min = (hour / 100) % 100;
                        hour /= 10000;
                    } else {
                        return Err(err())
                    }
                },
                _ => return Err(err())
            }
            if let Some('.') = s.char() {
                s.next();
                nsec = s.read_digits().ok_or(err())?;
                let ndigit = s.ndigits();
                precision = ndigit;
                if ndigit < 9 {
                    nsec *= 10u64.pow(9 - ndigit);
                } else if ndigit > 9 {
                    nsec /= 10u64.pow(ndigit - 9);
                    precision = 9;
                }
            }
            if let Some(' ') = s.char() {
                s.next();
            }
            match s.char() {
                Some('+') => {
                    s.next();
                    tz_hour = s.read_digits().ok_or(err())? as i32;
                    if let Some(':') = s.char() {
                        s.next();
                        tz_min = s.read_digits().ok_or(err())? as i32;
                    } else {
                        tz_min = tz_hour % 100;
                        tz_hour /= 100;
                    }
                    with_tz = true;
                },
                Some('-') => {
                    s.next();
                    tz_hour = s.read_digits().ok_or(err())? as i32;
                    if let Some(':') = s.char() {
                        s.next();
                        tz_min = s.read_digits().ok_or(err())? as i32;
                    } else {
                        tz_min = tz_hour % 100;
                        tz_hour /= 100;
                    }
                    tz_hour = - tz_hour;
                    tz_min = - tz_min;
                    with_tz = true;
                },
                Some('Z') => {
                    s.next();
                    with_tz = true;
                },
                _ => (),
            }
            if s.char().is_some() {
                return Err(err())
            }
        }
        let mut ts = Timestamp::new(if minus { - (year as i32) } else { year as i32},
                                    month as u32, day as u32,
                                    hour as u32, min as u32, sec as u32, nsec as u32);
        ts.precision = precision as u8;
        if with_tz {
            ts = ts.and_tz_hm_offset(tz_hour, tz_min);
        }
        Ok(ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let mut ts = Timestamp::new(2012, 3, 4, 5, 6, 7, 890123456).and_tz_hm_offset(8, 45);
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
        ts.tz_hour_offset = -8; ts.tz_minute_offset = -45;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07.890123456 -08:45");
        ts.precision = 0;
        assert_eq!(ts.to_string(), "2012-03-04 05:06:07 -08:45");
        ts.year = -123;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 -08:45");
        let mut ts = ts.and_tz_offset(-3600 - 1800);
        assert_eq!(ts.tz_hour_offset, -1);
        assert_eq!(ts.tz_minute_offset, -30);
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 -01:30");
        ts.tz_hour_offset = 0;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 -00:30");
        ts.tz_minute_offset = 30;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 +00:30");
        ts.tz_minute_offset = 0;
        assert_eq!(ts.to_string(), "-123-03-04 05:06:07 +00:00");
    }

    #[test]
    fn parse() {
        let mut ts = Timestamp::new(2012, 1, 1, // year, month, day,
                                    0, 0, 0, 0); // hour, minute, second, nanosecond,
        ts.precision = 0;
        assert_eq!("2012".parse(), Ok(ts));
        ts.month = 3; ts.day = 4;
        assert_eq!("20120304".parse(), Ok(ts));
        assert_eq!("2012-03-04".parse(), Ok(ts));
        ts.hour = 5; ts.minute = 6; ts.second = 7;
        assert_eq!("2012-03-04 05:06:07".parse(), Ok(ts));
        assert_eq!("2012-03-04T05:06:07".parse(), Ok(ts));
        assert_eq!("20120304T050607".parse(), Ok(ts));
        ts.nanosecond = 800000000; ts.precision = 1;
        assert_eq!("2012-03-04 05:06:07.8".parse(), Ok(ts));
        ts.nanosecond = 890000000; ts.precision = 2;
        assert_eq!("2012-03-04T05:06:07.89".parse(), Ok(ts));
        ts.nanosecond = 890000000; ts.precision = 3;
        assert_eq!("20120304T050607.890".parse(), Ok(ts));
        ts.nanosecond = 890100000; ts.precision = 4;
        assert_eq!("2012-03-04 05:06:07.8901".parse(), Ok(ts));
        ts.nanosecond = 890120000; ts.precision = 5;
        assert_eq!("2012-03-04 05:06:07.89012".parse(), Ok(ts));
        ts.nanosecond = 890123000; ts.precision = 6;
        assert_eq!("2012-03-04 05:06:07.890123".parse(), Ok(ts));
        ts.nanosecond = 890123400; ts.precision = 7;
        assert_eq!("2012-03-04 05:06:07.8901234".parse(), Ok(ts));
        ts.nanosecond = 890123450; ts.precision = 8;
        assert_eq!("2012-03-04 05:06:07.89012345".parse(), Ok(ts));
        ts.nanosecond = 890123456; ts.precision = 9;
        assert_eq!("2012-03-04 05:06:07.890123456".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07.8901234567".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07.89012345678".parse(), Ok(ts));
        ts.with_tz = true;
        ts.nanosecond = 0; ts.precision = 0;
        assert_eq!("2012-03-04 05:06:07Z".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07+00:00".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +00:00".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07+0000".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +0000".parse(), Ok(ts));
        ts.tz_hour_offset = 8; ts.tz_minute_offset = 45;
        assert_eq!("2012-03-04 05:06:07+08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07+0845".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 +0845".parse(), Ok(ts));
        ts.tz_hour_offset = -8; ts.tz_minute_offset = -45;
        assert_eq!("2012-03-04 05:06:07-08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 -08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07-0845".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07 -0845".parse(), Ok(ts));
        ts.nanosecond = 123000000; ts.precision = 3;
        assert_eq!("2012-03-04 05:06:07.123-08:45".parse(), Ok(ts));
        assert_eq!("2012-03-04 05:06:07.123 -08:45".parse(), Ok(ts));
        ts.year = -123;
        assert_eq!("-123-03-04 05:06:07.123 -08:45".parse(), Ok(ts));
        ts.tz_hour_offset = 0;
        assert_eq!("-123-03-04 05:06:07.123 -00:45".parse(), Ok(ts));
        ts.tz_minute_offset = 45;
        assert_eq!("-123-03-04 05:06:07.123 +00:45".parse(), Ok(ts));
    }
}
