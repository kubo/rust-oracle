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

use crate::binding::dpiIntervalYM;
use crate::sql_type::OracleType;
use crate::util::Scanner;
use crate::Error;
use crate::ParseOracleTypeError;
use crate::Result;
use std::cmp;
use std::fmt;
use std::result;
use std::str;

/// Oracle-specific [Interval Year to Month][INTVL_YM] data type.
///
/// [INTVL_YM]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-517CEB46-C6FA-4B94-9299-5BBB5A58CF7B
///
/// # Examples
///
/// ```
/// # use oracle::*; use oracle::sql_type::*;
/// // Create an interval by new().
/// let intvl1 = IntervalYM::new(2, 3)?;
///
/// // All arguments must be zero or negative to create a negative interval.
/// let intvl2 = IntervalYM::new(-2, -3)?;
///
/// // Convert to string.
/// assert_eq!(intvl1.to_string(), "+000000002-03");
/// assert_eq!(intvl2.to_string(), "-000000002-03");
///
/// // Create an interval with precision.
/// let intvl3 = IntervalYM::new(2, 3)?.and_prec(3)?;
///
/// // The string representation depends on the precisions.
/// assert_eq!(intvl3.to_string(), "+002-03");
///
/// // Precisions are ignored when intervals are compared.
/// assert!(intvl1 == intvl3);
///
/// // Create an interval from string.
/// let intvl4: IntervalYM = "+002-3".parse()?;
///
/// // The precision is determined by number of decimal digits in the string.
/// assert_eq!(intvl4.precision(), 3);
/// # Ok::<(), Error>(())
/// ```
///
/// Fetch and bind interval values.
///
/// ```no_run
/// # use oracle::*; use oracle::sql_type::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
///
/// // Fetch IntervalYM
/// let sql = "select interval '+02-03' year to month from dual";
/// let intvl = conn.query_row_as::<IntervalYM>(sql, &[])?;
/// assert_eq!(intvl.to_string(), "+02-03");
///
/// // Bind IntervalYM
/// let sql = "begin \
///              :outval := to_timestamp('2017-08-09', 'yyyy-mm-dd') + :inval; \
///            end;";
/// let mut stmt = conn.statement(sql).build()?;
/// stmt.execute(&[&OracleType::Date, // bind null as date
///                &intvl, // bind the intvl variable
///               ])?;
/// let outval: Timestamp = stmt.bind_value(1)?; // get the first bind value.
/// // 2017-08-09 + (2 years and 3 months)
/// assert_eq!(outval.to_string(), "2019-11-09 00:00:00");
/// # Ok::<(), Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct IntervalYM {
    years: i32,
    months: i32,
    precision: u8,
}

impl IntervalYM {
    fn check_validity(self) -> Result<Self> {
        if !(-999999999..=999999999).contains(&self.years) {
            Err(Error::out_of_range(format!(
                "years must be between -999999999 and 999999999 but {:?}",
                self
            )))
        } else if !(-11..=11).contains(&self.months) {
            Err(Error::out_of_range(format!(
                "months must be between -11 and 11 but {:?}",
                self
            )))
        } else if (self.years >= 0 && self.months >= 0) || (self.years <= 0 && self.months <= 0) {
            Ok(self)
        } else {
            Err(Error::out_of_range(format!(
                "years and months must be zeor or positive; or zero or negative but {:?}",
                self
            )))
        }
    }

    pub(crate) fn from_dpi_interval_ym(
        it: &dpiIntervalYM,
        oratype: &OracleType,
    ) -> Result<IntervalYM> {
        let prec = match *oratype {
            OracleType::IntervalYM(prec) => prec,
            _ => 2,
        };
        IntervalYM::new(it.years, it.months)?.and_prec(prec)
    }

    /// Creates a new IntervalYM.
    ///
    /// Valid values are:
    ///
    /// | argument | valid values |
    /// |---|---|
    /// | `years` | -999999999 to 999999999 |
    /// | `months` | -11 to 11 |
    ///
    /// All arguments must be zero or positive to create a positive interval.
    /// All arguments must be zero or negative to create a negative interval.
    pub fn new(years: i32, months: i32) -> Result<IntervalYM> {
        IntervalYM {
            years,
            months,
            precision: 9,
        }
        .check_validity()
    }

    /// Creates a new IntervalYM with precision.
    ///
    /// The precision affects text representation of IntervalYM.
    /// It doesn't affect comparison.
    pub fn and_prec(&self, precision: u8) -> Result<IntervalYM> {
        if precision > 9 {
            Err(Error::out_of_range(format!(
                "precision must be 0 to 9 but {}",
                precision
            )))
        } else {
            Ok(IntervalYM { precision, ..*self })
        }
    }

    /// Returns years component.
    pub fn years(&self) -> i32 {
        self.years
    }

    /// Returns months component.
    pub fn months(&self) -> i32 {
        self.months
    }

    /// Returns precision.
    pub fn precision(&self) -> u8 {
        self.precision
    }
}

impl cmp::PartialEq for IntervalYM {
    fn eq(&self, other: &Self) -> bool {
        self.years == other.years && self.months == other.months
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
    type Err = ParseOracleTypeError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let err = || ParseOracleTypeError::new("IntervalYM");
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
        let years = s.read_digits().ok_or_else(err)? as i32;
        let precision = s.ndigits();
        if let Some('-') = s.char() {
            s.next();
        } else {
            return Err(err());
        }
        let months = s.read_digits().ok_or_else(err)? as i32;
        if s.char().is_some() {
            return Err(err());
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
    fn to_string() -> Result<()> {
        let mut it = IntervalYM::new(1, 2)?;
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

        let mut it = IntervalYM::new(-1, -2)?;
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
        Ok(())
    }

    #[test]
    fn parse() -> Result<()> {
        let mut it = IntervalYM::new(1, 2)?;
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

        let it = IntervalYM::new(-1, -2)?;
        assert_eq!("-000000001-02".parse(), Ok(it));
        Ok(())
    }
}
