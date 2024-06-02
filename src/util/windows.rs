use crate::{Error, Result};
use std::ffi::{CString, OsString};
use std::os::windows::ffi::OsStrExt;
use std::ptr;

const CP_ACP: u32 = 0;

#[link(name = "kernel32")]
extern "system" {
    fn WideCharToMultiByte(
        CodePage: u32,
        dwFlags: u32,
        lpWideCharStr: *const u16,
        cchWideChar: i32,
        lpMultiByteStr: *mut u8,
        cbMultiByte: i32,
        lpDefaultChar: *mut u8,
        lpUsedDefaultChar: *mut i32,
    ) -> i32;

    fn GetACP() -> u32;
}

#[rustversion::since(1.74)]
mod util {
    use std::ffi::{CString, OsStr, OsString};
    use std::iter::Map;
    use std::slice;

    // Returns u8 iterator over OsStr
    //
    // This works only when strings in OsString are encoded in a less-strict variant of UTF-8
    // as documented https://doc.rust-lang.org/std/ffi/struct.OsString.html.
    pub fn os_str_iter<F>(s: &OsStr, f: F) -> Map<slice::Iter<u8>, F>
    where
        F: Fn(&u8) -> u8,
    {
        s.as_encoded_bytes().iter().map(f)
    }

    // `s` must contain only ASCII characters except nul.
    //
    // This works only when strings in OsString are encoded in a less-strict variant of UTF-8
    // as documented https://doc.rust-lang.org/std/ffi/struct.OsString.html.
    pub unsafe fn ascii_os_string_into_c_string(s: OsString) -> CString {
        CString::from_vec_unchecked(s.into_encoded_bytes())
    }

    // Reuses internal Vec in OsString to create a new Vec with capacity
    pub fn vec_from_os_string(capacity: usize, s: OsString) -> Vec<u8> {
        let mut v = s.into_encoded_bytes();
        if v.capacity() < capacity {
            v.reserve_exact(capacity - v.capacity());
        }
        v
    }
}

#[rustversion::before(1.74)]
mod util {
    use std::ffi::{CString, OsStr, OsString};
    use std::os::windows::ffi::OsStrExt;

    // Returns u16 iterator over OsStr
    pub fn os_str_iter<F>(s: &OsStr, _: F) -> std::os::windows::ffi::EncodeWide
    where
        F: Fn(&u8) -> u8,
    {
        s.encode_wide()
    }

    // `s` must contain only ASCII characters except nul.
    pub unsafe fn ascii_os_string_into_c_string(s: OsString) -> CString {
        let mut vec = Vec::with_capacity(s.len() + 1);
        for c in s.encode_wide() {
            vec.push(c as u8);
        }
        CString::from_vec_unchecked(vec)
    }

    pub fn vec_from_os_string(capacity: usize, _: OsString) -> Vec<u8> {
        Vec::with_capacity(capacity)
    }
}

// Converts OsString to CString encoded in ANSI code page, which is, for example,
// CP1252 in English Windows, CP932 in Japanese Windows.
pub fn os_string_into_ansi_c_string(s: OsString, name: &str) -> Result<CString> {
    let mut contains_nul = false;
    let mut ascii_only = true;
    for c in util::os_str_iter(&s, |b| *b) {
        if c == 0 {
            contains_nul = true;
        } else if c > 127 {
            ascii_only = false;
        }
    }
    if contains_nul {
        return Err(Error::invalid_argument(format!(
            "{} cannot contain nul characters",
            name
        )));
    }
    if ascii_only {
        return Ok(unsafe { util::ascii_os_string_into_c_string(s) });
    }
    let wide_chars: Vec<u16> = s.as_os_str().encode_wide().collect();
    let mut used_default_char = 0;
    let len = unsafe {
        WideCharToMultiByte(
            CP_ACP,
            0,
            wide_chars.as_ptr(),
            wide_chars.len() as i32,
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            &mut used_default_char,
        )
    };
    if used_default_char != 0 {
        return Err(Error::invalid_argument(format!(
            "{} cannot contain characters incompatible with the Windows ANSI code page {}",
            name,
            unsafe { GetACP() }
        )));
    }
    let mut vec = util::vec_from_os_string(len as usize + 1, s);
    unsafe {
        let len = WideCharToMultiByte(
            CP_ACP,
            0,
            wide_chars.as_ptr(),
            wide_chars.len() as i32,
            vec.as_mut_ptr(),
            len,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        vec.set_len(len as usize);
    }
    Ok(unsafe { CString::from_vec_unchecked(vec) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_string_into_ansi_c_string() {
        let mut data = Vec::<(&str, &[u8])>::new();
        data.push(("Hello", b"Hello"));
        match unsafe { GetACP() } {
            932 => {
                // Japanese code page
                data.push(("こんにちは", b"\x82\xb1\x82\xf1\x82\xc9\x82\xbf\x82\xcd"));
            }
            1252 => {
                // Western European code page
                data.push(("Grüß Gott", b"Gr\xfc\xdf Gott"));
            }
            _ => (),
        }
        for data in data {
            assert_eq!(
                os_string_into_ansi_c_string(data.0.into(), "dummy").unwrap(),
                CString::new(data.1).unwrap()
            );
        }
        os_string_into_ansi_c_string("crab 🦀".into(), "dummy").unwrap_err();
    }
}
