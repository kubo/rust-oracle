use std::fmt;
use std::str;

use binding::dpiIntervalYM;
use util::Scanner;
use OracleType;
use ParseError;

/// Interval type corresponding to Oracle type: `INTERVAL YEAR TO MONTH`.
///
/// Don't use this type directly in your applications. This is public
/// for types implementing `FromSql` and `ToSql` traits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntervalYM {
    pub years: i32,
    pub months: i32,
    pub precision: u8,
}

impl IntervalYM {
    pub(crate) fn from_dpi_interval_ym(it: &dpiIntervalYM, oratype: &OracleType) -> IntervalYM {
        let prec = match *oratype {
            OracleType::IntervalYM(prec) => prec as u8,
            _ => 2,
        };
        IntervalYM {
            years: it.years,
            months: it.months,
            precision: prec,
        }
    }

    pub fn new(years: i32, months: i32) -> IntervalYM {
        IntervalYM {
            years: years,
            months: months,
            precision: 2,
        }
    }
}

impl fmt::Display for IntervalYM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.years < 0 || self.months < 0 {
            write!(f, "-")?;
        } else {
            write!(f, "+")?;
        }
        let years = self.years.abs();
        match self.precision {
            2 => write!(f, "{:02}", years)?,
            3 => write!(f, "{:03}", years)?,
            4 => write!(f, "{:04}", years)?,
            5 => write!(f, "{:05}", years)?,
            6 => write!(f, "{:06}", years)?,
            7 => write!(f, "{:07}", years)?,
            8 => write!(f, "{:08}", years)?,
            9 => write!(f, "{:09}", years)?,
            _ => write!(f, "{}", years)?,
        };
        write!(f, "-{:02}", self.months.abs())
    }
}

impl str::FromStr for IntervalYM {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseError::new("IntervalYM");
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
        let years = s.read_digits().ok_or(err())? as i32;
        let precision = s.ndigits();
        if let Some('-') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let months = s.read_digits().ok_or(err())? as i32;
        if s.char().is_some() {
            return Err(err())
        }
        Ok(IntervalYM {
            years: if minus { -years } else { years },
            months: if minus { -months } else { months },
            precision: precision as u8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let mut it = IntervalYM::new(1, 2);
        assert_eq!(it.to_string(), "+01-02");
        it.precision = 0;
        assert_eq!(it.to_string(), "+1-02");
        it.precision = 1;
        assert_eq!(it.to_string(), "+1-02");
        it.precision = 2;
        assert_eq!(it.to_string(), "+01-02");
        it.precision = 3;
        assert_eq!(it.to_string(), "+001-02");
        it.precision = 4;
        assert_eq!(it.to_string(), "+0001-02");
        it.precision = 5;
        assert_eq!(it.to_string(), "+00001-02");
        it.precision = 6;
        assert_eq!(it.to_string(), "+000001-02");
        it.precision = 7;
        assert_eq!(it.to_string(), "+0000001-02");
        it.precision = 8;
        assert_eq!(it.to_string(), "+00000001-02");
        it.precision = 9;
        assert_eq!(it.to_string(), "+000000001-02");

        let mut it = IntervalYM::new(-1, -2);
        assert_eq!(it.to_string(), "-01-02");
        it.precision = 0;
        assert_eq!(it.to_string(), "-1-02");
        it.precision = 1;
        assert_eq!(it.to_string(), "-1-02");
        it.precision = 2;
        assert_eq!(it.to_string(), "-01-02");
        it.precision = 3;
        assert_eq!(it.to_string(), "-001-02");
        it.precision = 4;
        assert_eq!(it.to_string(), "-0001-02");
        it.precision = 5;
        assert_eq!(it.to_string(), "-00001-02");
        it.precision = 6;
        assert_eq!(it.to_string(), "-000001-02");
        it.precision = 7;
        assert_eq!(it.to_string(), "-0000001-02");
        it.precision = 8;
        assert_eq!(it.to_string(), "-00000001-02");
        it.precision = 9;
        assert_eq!(it.to_string(), "-000000001-02");
    }

    #[test]
    fn parse() {
        let mut it = IntervalYM::new(1, 2);
        it.precision = 1;
        assert_eq!("1-2".parse(), Ok(it));
        assert_eq!("+1-02".parse(), Ok(it));
        it.precision = 2;
        assert_eq!("+01-02".parse(), Ok(it));
        it.precision = 3;
        assert_eq!("+001-02".parse(), Ok(it));
        it.precision = 4;
        assert_eq!("+0001-02".parse(), Ok(it));
        it.precision = 5;
        assert_eq!("+00001-02".parse(), Ok(it));
        it.precision = 6;
        assert_eq!("+000001-02".parse(), Ok(it));
        it.precision = 7;
        assert_eq!("+0000001-02".parse(), Ok(it));
        it.precision = 8;
        assert_eq!("+00000001-02".parse(), Ok(it));
        it.precision = 9;
        assert_eq!("+000000001-02".parse(), Ok(it));

        let it = IntervalYM::new(-1, -2);
        assert_eq!("-01-02".parse(), Ok(it));
    }
}
