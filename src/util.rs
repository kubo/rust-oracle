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

use std::str;
use std::result;
use ParseOracleTypeError;

pub struct Scanner<'a> {
    chars: str::Chars<'a>,
    char: Option<char>,
    ndigits: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(s: &'a str) -> Scanner<'a> {
        let mut chars = s.chars();
        let char = chars.next();
        Scanner {
            chars: chars,
            char: char,
            ndigits: 0,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        self.char = self.chars.next();
        self.char
    }

    pub fn char(&self) -> Option<char> {
        self.char
    }

    pub fn read_digits(&mut self) -> Option<u64> {
        let mut num = 0;
        self.ndigits = 0;
        loop {
            num = num * 10 + match self.char {
                Some('0') =>  0,
                Some('1') =>  1,
                Some('2') =>  2,
                Some('3') =>  3,
                Some('4') =>  4,
                Some('5') =>  5,
                Some('6') =>  6,
                Some('7') =>  7,
                Some('8') =>  8,
                Some('9') =>  9,
                _ => {
                    if self.ndigits > 0 {
                        return Some(num);
                    } else {
                        return None;
                    }
                }
            };
            self.char = self.chars.next();
            self.ndigits += 1;
        }
    }

    pub fn ndigits(&self) -> u32 {
        self.ndigits
    }
}

pub fn check_number_format(s: &str) -> result::Result<(), ParseOracleTypeError> {
    let err = || ParseOracleTypeError::new("Oracle number");
    let mut s = Scanner::new(s);

    // optional negative sign
    if let Some('-') = s.char() {
        s.next();
    }

    // decimal part
    if s.read_digits().is_none() {
        return Err(err());
    }
    // optional fractional part
    if let Some('.') = s.char() {
        s.next();
        if s.read_digits().is_none() {
            return Err(err());
        }
    }
    // an optional exponent
    match s.char() {
        Some('e') | Some('E') => {
            s.next();
            match s.char() {
                Some('+') | Some('-') => {
                    s.next();
                },
                _ => (),
            }
            if s.read_digits().is_none() {
                return Err(err());
            }
        },
        _ => (),
    }
    if let Some(_) = s.char() {
        return Err(err());
    }
    Ok(())
}

pub fn parse_str_into_raw(s: &str) -> result::Result<Vec<u8>, ParseOracleTypeError> {
    let mut vec: Vec<u8> = Vec::with_capacity((s.len() + 1) / 2);
    let mut upper = s.len() % 2 == 0; // set upper half
    let mut upper_half = 0u8;
    for chr in s.bytes() {
        let half_byte = match chr {
            b'0' ... b'9' => chr - b'0',
            b'A' ... b'F' => chr - b'A' + 10,
            b'a' ... b'f' => chr - b'a' + 10,
            _ => return Err(ParseOracleTypeError::new("raw")),
        };
        if upper {
            upper_half = half_byte << 4;
        } else {
            vec.push(upper_half + half_byte);
        }
        upper = !upper;
    }
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        let mut s = Scanner::new("123.4567890");
        assert_eq!(s.read_digits(), Some(123));
        assert_eq!(s.read_digits(), None);
        assert_eq!(s.char(), Some('.'));
        s.next();
        assert_eq!(s.read_digits(), Some(4567890));
        assert_eq!(s.char(), None);
    }

    #[test]
    fn test_check_number_format() {
        let ok = Ok(());
        let err = Err(ParseOracleTypeError::new("Oracle number"));
        assert_eq!(check_number_format("123"), ok);
        assert_eq!(check_number_format("-123"), ok);
        assert_eq!(check_number_format("-123."), err);
        assert_eq!(check_number_format("-123.5"), ok);
        assert_eq!(check_number_format("-123e"), err);
        assert_eq!(check_number_format("-123e1"), ok);
        assert_eq!(check_number_format("-123e+1"), ok);
        assert_eq!(check_number_format("-123e-1"), ok);
        assert_eq!(check_number_format("-123e-10"), ok);
        assert_eq!(check_number_format(".123"), err);
        assert_eq!(check_number_format("0.123"), ok);
        assert_eq!(check_number_format(" 123"), err);
        assert_eq!(check_number_format(""), err);
        assert_eq!(check_number_format("a"), err);
        assert_eq!(check_number_format("0.0"), ok);
        assert_eq!(check_number_format("9.9"), ok);
    }

    #[test]
    fn test_parse_str_into_raw() {
        let err = Err(ParseOracleTypeError::new("raw"));
        assert_eq!(parse_str_into_raw(""), Ok(vec![]));
        assert_eq!(parse_str_into_raw("010203"), Ok(vec![1, 2, 3]));
        assert_eq!(parse_str_into_raw("10203"), Ok(vec![1, 2, 3]));
        assert_eq!(parse_str_into_raw("090a0A0f0F"), Ok(vec![9, 10, 10, 15, 15]));
        assert_eq!(parse_str_into_raw("G"), err);
        assert_eq!(parse_str_into_raw("g"), err);
        assert_eq!(parse_str_into_raw("1223344556677889"), Ok(vec![0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, 0x89]));
        assert_eq!(parse_str_into_raw("9aabbccddeeff0"), Ok(vec![0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0]));
        assert_eq!(parse_str_into_raw("9AABBCCDDEEFF0"), Ok(vec![0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0]));
    }
}

