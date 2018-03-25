// Rust-oracle - Rust binding for Oracle database
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
#[derive(PartialEq, Eq, PartialOrd, Ord)]
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
