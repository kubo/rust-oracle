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

use binding::dpiIntervalDS;
use util::Scanner;
use OracleType;
use ParseError;

/// Interval type corresponding to Oracle type: `INTERVAL DAY TO SECOND`.
///
/// Don't use this type directly in your applications. This is public
/// for types implementing `FromSql` and `ToSql` traits.
#[derive(Debug, Clone, Copy)]
pub struct IntervalDS {
    pub days: i32,
    pub hours: i32,
    pub minutes: i32,
    pub seconds: i32,
    pub nanoseconds: i32,
    pub lfprecision: u8,
    pub fsprecision: u8,
}

impl IntervalDS {
    pub(crate) fn from_dpi_interval_ds(it: &dpiIntervalDS, oratype: &OracleType) -> IntervalDS {
        let (lfprec, fsprec) = match *oratype {
            OracleType::IntervalDS(lfprec, fsprec) => (lfprec, fsprec),
            _ => (0, 0),
        };
        IntervalDS {
            days: it.days,
            hours: it.hours,
            minutes: it.minutes,
            seconds: it.seconds,
            nanoseconds: it.fseconds,
            lfprecision: lfprec as u8,
            fsprecision: fsprec,
        }
    }

    pub fn new(days: i32, hours: i32, minutes: i32, seconds: i32, nanoseconds: i32) -> IntervalDS {
        IntervalDS {
            days: days,
            hours: hours,
            minutes: minutes,
            seconds: seconds,
            nanoseconds: nanoseconds,
            lfprecision: 9,
            fsprecision: 9,
        }
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
        if self.days < 0 || self.hours < 0 || self.minutes < 0 || self.seconds < 0 || self.nanoseconds < 0 {
            write!(f, "-")?;
        } else {
            write!(f, "+")?;
        };
        let days = self.days.abs();
        match self.lfprecision {
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
        write!(f, " {:02}:{:02}:{:02}", self.hours.abs(), self.minutes.abs(), self.seconds.abs())?;
        let nsec = self.nanoseconds.abs();
        match self.fsprecision {
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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseError::new("IntervalDS");
        let mut s = Scanner::new(s);
        let minus = match s.char() {
            Some('+') => {
                s.next();
                false
            },
            Some('-') => {
                s.next();
                true
            },
            _ => false,
        };
        let days = s.read_digits().ok_or(err())? as i32;
        let lfprecision = s.ndigits();
        if let Some(' ') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let hours = s.read_digits().ok_or(err())? as i32;
        if let Some(':') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let minutes = s.read_digits().ok_or(err())? as i32;
        if let Some(':') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let seconds = s.read_digits().ok_or(err())? as i32;
        let mut nsecs = 0;
        let mut fsprecision = 0;
        if let Some('.') = s.char() {
            s.next();
            nsecs = s.read_digits().ok_or(err())? as i32;
            let ndigit = s.ndigits();
            fsprecision = ndigit;
            if ndigit < 9 {
                nsecs *= 10i32.pow(9 - ndigit);
            } else if ndigit > 9 {
                nsecs /= 10i32.pow(ndigit - 9);
                fsprecision = 9;
            }
        }
        if s.char().is_some() {
            return Err(err())
        }
        Ok(IntervalDS {
            days: if minus { -days } else { days },
            hours: if minus { -hours } else { hours },
            minutes: if minus { -minutes } else { minutes },
            seconds: if minus { -seconds } else { seconds },
            nanoseconds: if minus { -nsecs } else { nsecs },
            lfprecision: lfprecision as u8,
            fsprecision: fsprecision as u8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let mut it = IntervalDS::new(1, 2, 3, 4, 123456789);
        it.lfprecision = 0; it.fsprecision = 0;
        assert_eq!(it.to_string(), "+1 02:03:04");
        it.fsprecision = 1;
        assert_eq!(it.to_string(), "+1 02:03:04.1");
        it.fsprecision = 2;
        assert_eq!(it.to_string(), "+1 02:03:04.12");
        it.fsprecision = 3;
        assert_eq!(it.to_string(), "+1 02:03:04.123");
        it.fsprecision = 4;
        assert_eq!(it.to_string(), "+1 02:03:04.1234");
        it.fsprecision = 5;
        assert_eq!(it.to_string(), "+1 02:03:04.12345");
        it.fsprecision = 6;
        assert_eq!(it.to_string(), "+1 02:03:04.123456");
        it.fsprecision = 7;
        assert_eq!(it.to_string(), "+1 02:03:04.1234567");
        it.fsprecision = 8;
        assert_eq!(it.to_string(), "+1 02:03:04.12345678");
        it.fsprecision = 9;
        assert_eq!(it.to_string(), "+1 02:03:04.123456789");

        let mut it = IntervalDS::new(-1, -2, -3, -4, -123456789);
        it.lfprecision = 0; it.fsprecision = 0;
        assert_eq!(it.to_string(), "-1 02:03:04");
        it.fsprecision = 1;
        assert_eq!(it.to_string(), "-1 02:03:04.1");
        it.fsprecision = 2;
        assert_eq!(it.to_string(), "-1 02:03:04.12");
        it.fsprecision = 3;
        assert_eq!(it.to_string(), "-1 02:03:04.123");
        it.fsprecision = 4;
        assert_eq!(it.to_string(), "-1 02:03:04.1234");
        it.fsprecision = 5;
        assert_eq!(it.to_string(), "-1 02:03:04.12345");
        it.fsprecision = 6;
        assert_eq!(it.to_string(), "-1 02:03:04.123456");
        it.fsprecision = 7;
        assert_eq!(it.to_string(), "-1 02:03:04.1234567");
        it.fsprecision = 8;
        assert_eq!(it.to_string(), "-1 02:03:04.12345678");
        it.fsprecision = 9;
        assert_eq!(it.to_string(), "-1 02:03:04.123456789");

        it.lfprecision = 1;
        assert_eq!(it.to_string(), "-1 02:03:04.123456789");
        it.lfprecision = 2;
        assert_eq!(it.to_string(), "-01 02:03:04.123456789");
        it.lfprecision = 3;
        assert_eq!(it.to_string(), "-001 02:03:04.123456789");
        it.lfprecision = 4;
        assert_eq!(it.to_string(), "-0001 02:03:04.123456789");
        it.lfprecision = 5;
        assert_eq!(it.to_string(), "-00001 02:03:04.123456789");
        it.lfprecision = 6;
        assert_eq!(it.to_string(), "-000001 02:03:04.123456789");
        it.lfprecision = 7;
        assert_eq!(it.to_string(), "-0000001 02:03:04.123456789");
        it.lfprecision = 8;
        assert_eq!(it.to_string(), "-00000001 02:03:04.123456789");
        it.lfprecision = 9;
        assert_eq!(it.to_string(), "-000000001 02:03:04.123456789");
    }

    #[test]
    fn parse() {
        let mut it = IntervalDS::new(1, 2, 3, 4, 0);
        it.lfprecision = 1; it.fsprecision = 0;
        assert_eq!("1 02:03:04".parse(), Ok(it));
        assert_eq!("+1 02:03:04".parse(), Ok(it));
        it.lfprecision = 2;
        assert_eq!("01 02:03:04".parse(), Ok(it));
        it.lfprecision = 3;
        assert_eq!("001 02:03:04".parse(), Ok(it));
        it.lfprecision = 4;
        assert_eq!("0001 02:03:04".parse(), Ok(it));
        it.lfprecision = 5;
        assert_eq!("00001 02:03:04".parse(), Ok(it));
        it.lfprecision = 6;
        assert_eq!("000001 02:03:04".parse(), Ok(it));
        it.lfprecision = 7;
        assert_eq!("0000001 02:03:04".parse(), Ok(it));
        it.lfprecision = 8;
        assert_eq!("00000001 02:03:04".parse(), Ok(it));
        it.lfprecision = 9;
        assert_eq!("000000001 02:03:04".parse(), Ok(it));

        it.fsprecision = 1; it.nanoseconds = 100000000;
        assert_eq!("000000001 02:03:04.1".parse(), Ok(it));

        let mut it = IntervalDS::new(-1, -2, -3, -4, 0);
        it.lfprecision = 1; it.fsprecision = 0;
        assert_eq!("-1 02:03:04".parse(), Ok(it));

        it.fsprecision = 1; it.nanoseconds = -100000000;
        assert_eq!("-1 02:03:04.1".parse(), Ok(it));
        it.fsprecision = 2; it.nanoseconds = -120000000;
        assert_eq!("-1 02:03:04.12".parse(), Ok(it));
        it.fsprecision = 3; it.nanoseconds = -123000000;
        assert_eq!("-1 02:03:04.123".parse(), Ok(it));
        it.fsprecision = 4; it.nanoseconds = -123400000;
        assert_eq!("-1 02:03:04.1234".parse(), Ok(it));
        it.fsprecision = 5; it.nanoseconds = -123450000;
        assert_eq!("-1 02:03:04.12345".parse(), Ok(it));
        it.fsprecision = 6; it.nanoseconds = -123456000;
        assert_eq!("-1 02:03:04.123456".parse(), Ok(it));
        it.fsprecision = 7; it.nanoseconds = -123456700;
        assert_eq!("-1 02:03:04.1234567".parse(), Ok(it));
        it.fsprecision = 8; it.nanoseconds = -123456780;
        assert_eq!("-1 02:03:04.12345678".parse(), Ok(it));
        it.fsprecision = 9; it.nanoseconds = -123456789;
        assert_eq!("-1 02:03:04.123456789".parse(), Ok(it));
    }
}
