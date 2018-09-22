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

use std::fmt;

use binding::dpiVersionInfo;

/// Oracle version information
///
/// # Examples
///
/// ```no_run
/// # use oracle::*; fn try_main() -> Result<()> {
/// let conn = Connection::connect("scott", "tiger", "", &[])?;
/// let client_version = oracle::client_version()?;
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
/// # Ok(())} fn main() { try_main().unwrap(); }
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
    pub fn new(major: i32, minor: i32, update: i32, patch: i32, port_update: i32) -> Version {
        Version { major: major, minor: minor, update: update,
                  patch: patch, port_update: port_update }
    }

    pub(crate) fn new_from_dpi_ver(ver: dpiVersionInfo) -> Version {
        Version::new(ver.versionNum, ver.releaseNum, ver.updateNum, ver.portReleaseNum, ver.portUpdateNum)
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
        write!(f, "{}.{}.{}.{}.{}", self.major, self.minor, self.update, self.patch, self.port_update)
    }
}
