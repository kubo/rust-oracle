// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2022 Christoph Heiss <contact@christoph-heiss.at>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use crate::binding::*;
use std::ffi::CStr;

/// Oracle [encoding Information](https://oracle.github.io/odpi/doc/structs/dpiEncodingInfo.html)
///
/// # Examples
///
/// ```ignore
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
/// let encoding = conn.encoding_info()?;
///
/// println!("Encoding Info:");
/// println!("  Encoding used for CHAR data: {}, {} bytes per character",
///          encoding.char_encoding(), encoding.char_size());
/// println!("  Encoding used for NCHAR data: {}, {} bytes per character",
///          encoding.nchar_encoding(), encoding.nchar_size());
/// ```
#[derive(Debug, PartialEq)]
pub struct EncodingInfo {
    char_encoding: String,
    char_size: u32,
    nchar_encoding: String,
    nchar_size: u32,
}

impl EncodingInfo {
    pub(crate) fn new_from_dpi_info(info: dpiEncodingInfo) -> Self {
        let char_encoding = unsafe { CStr::from_ptr(info.encoding) }
            .to_string_lossy()
            .into_owned();

        let nchar_encoding = unsafe { CStr::from_ptr(info.nencoding) }
            .to_string_lossy()
            .into_owned();

        Self {
            char_encoding,
            char_size: info.maxBytesPerCharacter as u32,
            nchar_encoding,
            nchar_size: info.nmaxBytesPerCharacter as u32,
        }
    }

    pub fn char_encoding(&self) -> &str {
        &self.char_encoding
    }

    pub fn char_size(&self) -> u32 {
        self.char_size
    }

    pub fn nchar_encoding(&self) -> &str {
        &self.nchar_encoding
    }

    pub fn nchar_size(&self) -> u32 {
        self.nchar_size
    }
}
