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

use crate::binding::*;
use crate::chkerr;
use crate::Context;
use crate::Result;
use std::fmt;
use std::mem::MaybeUninit;
use std::num::ParseIntError;
use std::result;
use std::str::FromStr;

/// Oracle version information
///
/// # Examples
///
/// ```no_run
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
/// let client_version = Version::client()?;
/// let (server_version, _) = conn.server_version()?;
///
/// println!("Client version:");
/// println!("  1st part: {}", client_version.major());
/// println!("  2nd part: {}", client_version.minor());
/// println!("  3rd part: {}", client_version.update());
/// println!("  4th part: {}", client_version.patch());
/// println!("  5th part: {}", client_version.port_update());
///
/// println!("Server version: {}", server_version);
/// # Ok::<(), Error>(())
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: i32,
    minor: i32,
    update: i32,
    patch: i32,
    port_update: i32,
}

impl Version {
    /// Creates a new version information
    pub const fn new(major: i32, minor: i32, update: i32, patch: i32, port_update: i32) -> Version {
        Version {
            major,
            minor,
            update,
            patch,
            port_update,
        }
    }

    /// Returns the version of Oracle client in use.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::*;
    /// let client_ver = Version::client()?;
    /// println!("Oracle Client Version: {}", client_ver);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn client() -> Result<Version> {
        let ctx = Context::new0()?;
        let mut ver = MaybeUninit::uninit();
        chkerr!(
            &ctx,
            dpiContext_getClientVersion(ctx.context, ver.as_mut_ptr())
        );
        Ok(Version::new_from_dpi_ver(unsafe { ver.assume_init() }))
    }

    pub(crate) fn new_from_dpi_ver(ver: dpiVersionInfo) -> Version {
        Version::new(
            ver.versionNum,
            ver.releaseNum,
            ver.updateNum,
            ver.portReleaseNum,
            ver.portUpdateNum,
        )
    }

    /// Gets 1st part of Oracle version number
    pub fn major(&self) -> i32 {
        self.major
    }

    /// Gets 2nd part of Oracle version number
    pub fn minor(&self) -> i32 {
        self.minor
    }

    /// Gets 3rd part of Oracle version number
    pub fn update(&self) -> i32 {
        self.update
    }

    /// Gets 4th part of Oracle version number
    pub fn patch(&self) -> i32 {
        self.patch
    }

    /// Gets 5th part of Oracle version number
    pub fn port_update(&self) -> i32 {
        self.port_update
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}.{}",
            self.major, self.minor, self.update, self.patch, self.port_update
        )
    }
}

impl FromStr for Version {
    type Err = ParseIntError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut iter = s.split('.').fuse();
        let major = iter.next().map_or(Ok(0), |s| s.parse::<i32>())?;
        let minor = iter.next().map_or(Ok(0), |s| s.parse::<i32>())?;
        let update = iter.next().map_or(Ok(0), |s| s.parse::<i32>())?;
        let patch = iter.next().map_or(Ok(0), |s| s.parse::<i32>())?;
        let port_update = iter.next().map_or(Ok(0), |s| s.parse::<i32>())?;
        Ok(Version::new(major, minor, update, patch, port_update))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn to_string() {
        assert_eq!(Version::new(12, 1, 2, 3, 4).to_string(), "12.1.2.3.4");
    }

    #[test]
    fn from_str() {
        assert_eq!(
            "12".parse::<Version>().unwrap(),
            Version::new(12, 0, 0, 0, 0)
        );
        assert_eq!(
            "12.1".parse::<Version>().unwrap(),
            Version::new(12, 1, 0, 0, 0)
        );
        assert_eq!(
            "12.1.2".parse::<Version>().unwrap(),
            Version::new(12, 1, 2, 0, 0)
        );
        assert_eq!(
            "12.1.2.3".parse::<Version>().unwrap(),
            Version::new(12, 1, 2, 3, 0)
        );
        assert_eq!(
            "12.1.2.3.4".parse::<Version>().unwrap(),
            Version::new(12, 1, 2, 3, 4)
        );
    }

    #[test]
    fn client_version() {
        let ver = Version::client().unwrap();
        let conn = test_util::connect().unwrap();
        let mut ver_from_query = conn.query_row_as::<String>("SELECT client_version FROM v$session_connect_info WHERE sid = SYS_CONTEXT('USERENV', 'SID')", &[]).unwrap();
        // The fifth numeral of client_version may be "01" through "12" such as "23.4.0.24.05".
        // Replace it with "23.4.0.24.5" to pass this test.
        if let Some(pos) = ver_from_query.len().checked_sub(2) {
            if ver_from_query.as_bytes()[pos] == b'0' {
                ver_from_query.remove(pos);
            }
        }
        assert_eq!(ver.to_string(), ver_from_query);
    }
}
