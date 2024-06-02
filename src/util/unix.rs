use crate::Error;
use crate::Result;
use std::ffi::{CString, OsString};
use std::os::unix::ffi::OsStringExt;

// Converts OsString to CString.
// On Windows it returns string encoded in ANSI code page.
// On unix it returns bytes excluding nul.
pub fn os_string_into_ansi_c_string(s: OsString, name: &str) -> Result<CString> {
    CString::new(s.into_vec()).map_err(|err| {
        Error::invalid_argument(format!("{} cannot contain nul characters", name)).add_source(err)
    })
}
