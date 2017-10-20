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

pub fn check_number_format(s: &str) -> bool {
    let mut s = Scanner::new(s);

    // optional negative sign
    if let Some('-') = s.char() {
        s.next();
    }

    // decimal part
    if s.read_digits().is_none() {
        return false;
    }
    // optional fractional part
    if let Some('.') = s.char() {
        s.next();
        if s.read_digits().is_none() {
            return false;
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
                return false;
            }
        },
        _ => (),
    }
    if let Some(_) = s.char() {
        return false;
    }
    return true;
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
        assert_eq!(check_number_format("123"), true);
        assert_eq!(check_number_format("-123"), true);
        assert_eq!(check_number_format("-123."), false);
        assert_eq!(check_number_format("-123.5"), true);
        assert_eq!(check_number_format("-123e"), false);
        assert_eq!(check_number_format("-123e1"), true);
        assert_eq!(check_number_format("-123e+1"), true);
        assert_eq!(check_number_format("-123e-1"), true);
        assert_eq!(check_number_format("-123e-10"), true);
        assert_eq!(check_number_format(".123"), false);
        assert_eq!(check_number_format("0.123"), true);
        assert_eq!(check_number_format(" 123"), false);
        assert_eq!(check_number_format(""), false);
        assert_eq!(check_number_format("a"), false);
        assert_eq!(check_number_format("0.0"), true);
        assert_eq!(check_number_format("9.9"), true);
    }
}

