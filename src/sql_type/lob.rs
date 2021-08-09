// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2021 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
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
use crate::io::SeekInChars;
use crate::new_odpi_str;
use crate::sql_type::FromSql;
use crate::sql_type::OracleType;
use crate::sql_type::ToSql;
use crate::sql_type::ToSqlNull;
use crate::to_odpi_str;
use crate::Connection;
use crate::Context;
use crate::Error;
use crate::Result;
use crate::SqlValue;
use std::cmp;
use std::convert::TryInto;
use std::fmt;
use std::io::{self, Read, Seek, Write};
use std::os::raw::c_char;
use std::ptr;
use std::str;

#[cfg(not(test))]
const MIN_READ_SIZE: usize = 400;

#[cfg(test)]
const MIN_READ_SIZE: usize = 20;

fn utf16_len(s: &[u8]) -> io::Result<usize> {
    let s = map_to_io_error(str::from_utf8(s))?;
    Ok(s.chars().fold(0, |acc, c| acc + c.len_utf16()))
}

fn map_to_io_error<T, E>(res: std::result::Result<T, E>) -> io::Result<T>
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    res.map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

pub struct LobLocator {
    ctxt: &'static Context,
    pub(crate) handle: *mut dpiLob,
    pos: u64,
}

impl LobLocator {
    fn from_raw(ctxt: &'static Context, handle: *mut dpiLob) -> Result<LobLocator> {
        chkerr!(ctxt, dpiLob_addRef(handle));
        Ok(LobLocator {
            ctxt: ctxt,
            handle: handle,
            pos: 0,
        })
    }

    fn close(&mut self) -> Result<()> {
        chkerr!(self.ctxt, dpiLob_close(self.handle));
        Ok(())
    }

    fn read_bytes(&mut self, amount: usize, buf: &mut [u8]) -> Result<usize> {
        let mut len = buf.len() as u64;
        chkerr!(
            self.ctxt,
            dpiLob_readBytes(
                self.handle,
                self.pos + 1,
                amount as u64,
                buf.as_mut_ptr() as *mut c_char,
                &mut len
            )
        );
        Ok(len as usize)
    }

    /// read for `Blob` and `Bfile`
    fn read_binary(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = map_to_io_error(self.read_bytes(buf.len(), buf))?;
        self.pos += len as u64;
        Ok(len)
    }

    /// read for `Clob` and `Nclob`
    fn read_chars(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() > MIN_READ_SIZE {
            let len = map_to_io_error(self.read_bytes(buf.len(), buf))?;
            self.pos += utf16_len(&buf[0..len])? as u64;
            Ok(len)
        } else {
            let mut tmp = [0u8; MIN_READ_SIZE];
            let buf_len = if buf.len() == 1 { 2 } else { buf.len() };
            let len = map_to_io_error(self.read_bytes(buf_len, &mut tmp))?;
            let len = cmp::min(len, buf.len());
            let s = match str::from_utf8(&tmp[0..len]) {
                Ok(s) => s,
                Err(err) if err.error_len().is_some() => return map_to_io_error(Err(err)),
                Err(err) if err.valid_up_to() == 0 => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "too small buffer to read characters",
                    ));
                }
                Err(err) => unsafe { str::from_utf8_unchecked(&tmp[0..err.valid_up_to()]) },
            };
            buf[0..s.len()].copy_from_slice(s.as_bytes());
            self.pos += s.chars().fold(0, |acc, c| acc + c.len_utf16()) as u64;
            Ok(s.len())
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>, nls_ratio: usize) -> io::Result<usize> {
        let too_long_data_err = || {
            io::Error::new(
                io::ErrorKind::Other,
                "The length of LOB data is too long to store a buffer",
            )
        };
        let lob_size = map_to_io_error(self.size())?;
        if self.pos >= lob_size {
            return Ok(0);
        }
        let rest_size: usize = (lob_size - self.pos)
            .try_into()
            .map_err(|_| too_long_data_err())?;
        let buf_len = buf.len();
        let rest_byte_size = rest_size
            .checked_mul(nls_ratio)
            .filter(|n| {
                if let Some(len) = buf_len.checked_add(*n) {
                    len <= isize::MAX as usize
                } else {
                    false
                }
            })
            .ok_or_else(too_long_data_err)?;
        buf.reserve(rest_byte_size);
        unsafe {
            buf.set_len(buf_len + rest_byte_size);
        }
        let result = self.read_bytes(rest_size, &mut buf[buf_len..]);
        unsafe {
            buf.set_len(buf_len + result.as_ref().unwrap_or(&0));
        }
        map_to_io_error(result)
    }

    /// read_to_end for `BLOB` and `BFILE`
    fn read_binary_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let len = self.read_to_end(buf, 1)?;
        self.pos += len as u64;
        Ok(len)
    }

    /// read_to_end for `CLOB` and `NCLOB`
    fn read_chars_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let start_pos = buf.len();
        let len = self.read_to_end(buf, 4)?;
        self.pos += utf16_len(&buf[start_pos..])? as u64;
        Ok(len)
    }

    fn write_bytes(&mut self, buf: &[u8]) -> Result<usize> {
        let len = buf.len() as u64;
        chkerr!(
            self.ctxt,
            dpiLob_writeBytes(
                self.handle,
                self.pos + 1,
                buf.as_ptr() as *const c_char,
                len,
            )
        );
        Ok(len as usize)
    }

    /// write for `BLOB` and `BFILE`
    fn write_binary(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = map_to_io_error(self.write_bytes(buf))?;
        self.pos += len as u64;
        Ok(len)
    }

    /// write for `CLOB` and `NCLOB`
    fn write_chars(&mut self, buf: &[u8]) -> io::Result<usize> {
        map_to_io_error(str::from_utf8(buf))?;
        let len = map_to_io_error(self.write_bytes(buf))?;
        self.pos += utf16_len(&buf[0..len])? as u64;
        Ok(len)
    }

    fn size(&self) -> Result<u64> {
        let mut size = 0;
        chkerr!(self.ctxt, dpiLob_getSize(self.handle, &mut size));
        Ok(size)
    }

    fn trim(&mut self, new_size: u64) -> Result<()> {
        chkerr!(self.ctxt, dpiLob_trim(self.handle, new_size));
        if self.pos > new_size {
            self.pos = new_size;
        }
        Ok(())
    }

    fn chunk_size(&self) -> Result<u32> {
        let mut size = 0;
        chkerr!(self.ctxt, dpiLob_getChunkSize(self.handle, &mut size));
        Ok(size)
    }

    fn open_resource(&mut self) -> Result<()> {
        chkerr!(self.ctxt, dpiLob_openResource(self.handle));
        Ok(())
    }

    fn close_resource(&mut self) -> Result<()> {
        chkerr!(self.ctxt, dpiLob_closeResource(self.handle));
        Ok(())
    }

    fn is_resource_open(&self) -> Result<bool> {
        let mut is_open = 0;
        chkerr!(
            self.ctxt,
            dpiLob_getIsResourceOpen(self.handle, &mut is_open)
        );
        Ok(is_open != 0)
    }

    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            io::SeekFrom::Start(offset) => Some(offset),
            io::SeekFrom::End(offset) => {
                let size = map_to_io_error(self.size())?;
                if offset < 0 {
                    size.checked_sub((-offset) as u64)
                } else {
                    size.checked_add(offset as u64)
                }
            }
            io::SeekFrom::Current(offset) => {
                if offset < 0 {
                    self.pos.checked_sub((-offset) as u64)
                } else {
                    self.pos.checked_add(offset as u64)
                }
            }
        }
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot seek {:?} from offset {}", pos, self.pos),
            )
        })?;
        Ok(self.pos)
    }

    fn directory_and_file_name(&self) -> Result<(String, String)> {
        let mut dir_alias = new_odpi_str();
        let mut file_name = new_odpi_str();
        chkerr!(
            self.ctxt,
            dpiLob_getDirectoryAndFileName(
                self.handle,
                &mut dir_alias.ptr,
                &mut dir_alias.len,
                &mut file_name.ptr,
                &mut file_name.len
            )
        );
        Ok((dir_alias.to_string(), file_name.to_string()))
    }

    fn set_directory_and_file_name(&self, directory_alias: &str, file_name: &str) -> Result<()> {
        let dir_alias = to_odpi_str(directory_alias);
        let file_name = to_odpi_str(file_name);
        chkerr!(
            self.ctxt,
            dpiLob_setDirectoryAndFileName(
                self.handle,
                dir_alias.ptr,
                dir_alias.len,
                file_name.ptr,
                file_name.len
            )
        );
        Ok(())
    }

    fn file_exists(&self) -> Result<bool> {
        let mut exists = 0;
        chkerr!(self.ctxt, dpiLob_getFileExists(self.handle, &mut exists));
        Ok(exists != 0)
    }
}

impl Clone for LobLocator {
    fn clone(&self) -> Self {
        unsafe { dpiLob_addRef(self.handle) };
        LobLocator { ..*self }
    }
}

impl fmt::Debug for LobLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lob {{ handle: {:?}, pos: {} }} ", self.handle, self.pos)
    }
}

impl Drop for LobLocator {
    fn drop(&mut self) {
        unsafe { dpiLob_release(self.handle) };
    }
}

/// A reference to Oracle data type `BLOB` or `BFILE`
///
/// This struct implements [`Read`], [`Write`] and [`Seek`] to
/// read and write bytes; and seek to a position in a LOB.
///
/// # Examples
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util;
/// # use oracle::sql_type::Blob;
/// use std::io::BufReader;
/// use std::io::Read;
/// # let conn = test_util::connect()?;
/// # conn.execute(
/// #     "insert into TestBLOBs values (1, UTL_RAW.CAST_TO_RAW('BLOB DATA'))",
/// #     &[],
/// # )?;
///
/// let sql = "select BLOBCol from TestBLOBS where IntCol = 1";
/// let mut stmt = conn.statement(sql).lob_locator().build()?;
/// let blob = stmt.query_row_as::<Blob>(&[])?;
/// let mut buf_reader = BufReader::with_capacity(blob.chunk_size()? * 16, blob);
/// let mut buf = [0u8; 4];
/// assert_eq!(buf_reader.read(&mut buf)?, 4); // read the first four bytes
/// assert_eq!(&buf, b"BLOB");
/// assert_eq!(buf_reader.read(&mut buf)?, 4); // read the next four bytes
/// assert_eq!(&buf, b" DAT");
/// assert_eq!(buf_reader.read(&mut buf)?, 1); // read the last one byte
/// assert_eq!(&buf[0..1], b"A");
/// assert_eq!(buf_reader.read(&mut buf)?, 0); // end of blob
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Blob {
    pub(crate) lob: LobLocator,
}

impl Blob {
    pub(crate) fn from_raw(ctxt: &'static Context, handle: *mut dpiLob) -> Result<Blob> {
        Ok(Blob {
            lob: LobLocator::from_raw(ctxt, handle)?,
        })
    }

    /// Returns a reference to a new temporary LOB which may subsequently be
    /// written and bound to a statement.
    pub fn new(conn: &Connection) -> Result<Blob> {
        let mut handle = ptr::null_mut();
        chkerr!(
            conn.ctxt(),
            dpiConn_newTempLob(conn.handle(), DPI_ORACLE_TYPE_BLOB, &mut handle)
        );
        Blob::from_raw(conn.ctxt(), handle)
    }

    /// Closes the LOB.
    pub fn close(&mut self) -> Result<()> {
        self.lob.close()
    }

    /// Returns the size of the data stored in the LOB in bytes.
    pub fn size(&self) -> Result<u64> {
        self.lob.size()
    }

    /// Returns the chunk size, in bytes, of the internal LOB. Reading and writing
    /// to the LOB in multiples of this size will improve performance.
    pub fn chunk_size(&self) -> Result<usize> {
        Ok(self.lob.chunk_size()?.try_into()?)
    }

    /// Opens the LOB resource for writing. This will improve performance when
    /// writing to the LOB in chunks and there are functional or extensible indexes
    /// associated with the LOB. If this function is not called, the LOB resource
    /// will be opened and closed for each write that is performed. A call to the
    /// [`close_resource`] should be done before performing a
    /// call to the function [`Connection.commit`].
    ///
    /// [`close_resource`]: #method.close_resource
    /// [`Connection.commit`]: Connection#method.commit
    pub fn open_resource(&mut self) -> Result<()> {
        self.lob.open_resource()
    }

    /// Closes the LOB resource. This should be done when a batch of writes has
    /// been completed so that the indexes associated with the LOB can be updated.
    /// It should only be performed if a call to function
    /// [`open_resource`] has been performed.
    ///
    /// [`open_resource`]: #method.open_resource
    pub fn close_resource(&mut self) -> Result<()> {
        self.lob.close_resource()
    }

    /// Returns a boolean value indicating if the LOB resource has been opened by
    /// making a call to the function [`open_resource`].
    ///
    /// [`open_resource`]: #method.open_resource
    pub fn is_resource_open(&self) -> Result<bool> {
        self.lob.is_resource_open()
    }

    /// Trims the data in the LOB so that it only contains the specified amount of
    /// data.
    pub fn trim(&mut self, new_size: u64) -> Result<()> {
        self.lob.trim(new_size)
    }

    /// Returns the directory alias name and file name for a BFILE type LOB.
    pub fn directory_and_file_name(&self) -> Result<(String, String)> {
        self.lob.directory_and_file_name()
    }

    /// Sets the directory alias name and file name for a BFILE type LOB.
    pub fn set_directory_and_file_name<D, F>(
        &mut self,
        directory_alias: D,
        file_name: F,
    ) -> Result<()>
    where
        D: AsRef<str>,
        F: AsRef<str>,
    {
        self.lob
            .set_directory_and_file_name(directory_alias.as_ref(), file_name.as_ref())
    }

    /// Returns a boolean value indicating if the file referenced by the BFILE type
    /// LOB exists or not.
    pub fn file_exists(&self) -> Result<bool> {
        self.lob.file_exists()
    }
}

impl FromSql for Blob {
    fn from_sql(val: &SqlValue) -> Result<Self> {
        val.to_blob()
    }
}

impl ToSqlNull for Blob {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::BLOB)
    }
}

impl ToSql for Blob {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::BLOB)
    }

    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_blob(self)
    }
}

impl Read for Blob {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lob.read_binary(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lob.read_binary_to_end(buf)
    }
}

impl Seek for Blob {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.lob.seek(pos)
    }
}

impl Write for Blob {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lob.write_binary(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A reference to Oracle data type `CLOB` or `NCLOB`
///
/// This struct implements [`Read`] and [`Write`] to read and write
/// characters. [`Read.read`] fails when `buf` is too small
/// to store one character. [`Write.write`] fails when `buf` contains
/// invalid UTF-8 byte sequence.
///
/// This also implements [`SeekInChars`] to seek to a position in characters.
/// Note that there is no way to seek in bytes.
///
/// # Notes
///
/// The size of LOBs returned by the [`size`] method and positions in
/// [`SeekInChars`] are inaccurate if a character in the LOB requires
/// more than one UCS-2 codepoint. That's becuase Oracle stores CLOBs
/// and NCLOBs using the UTF-16 encoding and the number of characters
/// is defined by the number of UCS-2 codepoints.
///
/// [`size`]: #method.size
/// [`Read.read`]: Read#tymethod.read
/// [`Write.write`]: Write#tymethod.write
#[derive(Clone, Debug)]
pub struct Clob {
    pub(crate) lob: LobLocator,
}

impl Clob {
    pub(crate) fn from_raw(ctxt: &'static Context, handle: *mut dpiLob) -> Result<Clob> {
        Ok(Clob {
            lob: LobLocator::from_raw(ctxt, handle)?,
        })
    }

    /// Returns a reference to a new temporary CLOB or NCLOB which may subsequently be
    /// written and bound to a statement.
    ///
    /// The `oratype` argument must be either `&OracleType::CLOB` or `&OracleType::NCLOB`.
    pub fn new(conn: &Connection, oratype: &OracleType) -> Result<Clob> {
        let lob_type = match oratype {
            &OracleType::CLOB => DPI_ORACLE_TYPE_CLOB,
            &OracleType::NCLOB => DPI_ORACLE_TYPE_NCLOB,
            _ => return Err(Error::InvalidOperation(format!("The oratype argument must be either OracleType::CLOB or OracleType::NCLOB but {:?}", oratype))),
        };
        let mut handle = ptr::null_mut();
        chkerr!(
            conn.ctxt(),
            dpiConn_newTempLob(conn.handle(), lob_type, &mut handle)
        );
        Clob::from_raw(conn.ctxt(), handle)
    }

    /// Closes the LOB.
    pub fn close(&mut self) -> Result<()> {
        self.lob.close()
    }

    /// Returns the size of the data stored in the LOB in characters.
    ///
    /// See also [Notes](#notes).
    pub fn size(&self) -> Result<u64> {
        self.lob.size()
    }

    // /// Returns the chunk size, in bytes, of the internal LOB. Reading and writing
    // /// to the LOB in multiples of this size will improve performance.
    // pub fn chunk_size(&self) -> Result<usize> {
    //     Ok(self.lob.chunk_size()?.try_into()?)
    // }

    /// Opens the LOB resource for writing. This will improve performance when
    /// writing to the LOB in chunks and there are functional or extensible indexes
    /// associated with the LOB. If this function is not called, the LOB resource
    /// will be opened and closed for each write that is performed. A call to the
    /// [`close_resource`] should be done before performing a
    /// call to the function [`Connection.commit`].
    ///
    /// [`close_resource`]: #method.close_resource
    /// [`Connection.commit`]: Connection#method.commit
    pub fn open_resource(&mut self) -> Result<()> {
        self.lob.open_resource()
    }

    /// Closes the LOB resource. This should be done when a batch of writes has
    /// been completed so that the indexes associated with the LOB can be updated.
    /// It should only be performed if a call to function
    /// [`open_resource`] has been performed.
    ///
    /// [`open_resource`]: #method.open_resource
    pub fn close_resource(&mut self) -> Result<()> {
        self.lob.close_resource()
    }

    /// Returns a boolean value indicating if the LOB resource has been opened by
    /// making a call to the function [`open_resource`].
    ///
    /// [`open_resource`]: #method.open_resource
    pub fn is_resource_open(&self) -> Result<bool> {
        self.lob.is_resource_open()
    }

    /// Trims the data in the LOB so that it only contains the specified amount of
    /// data.
    ///
    /// The new size is the number of UCS-2 codepoints. See [Notes](#notes).
    pub fn trim(&mut self, new_size: u64) -> Result<()> {
        self.lob.trim(new_size)
    }
}

impl FromSql for Clob {
    fn from_sql(val: &SqlValue) -> Result<Self> {
        val.to_clob()
    }
}

impl ToSqlNull for Clob {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::CLOB)
    }
}

impl ToSql for Clob {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::CLOB)
    }

    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_clob(self)
    }
}

impl Read for Clob {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lob.read_chars(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lob.read_chars_to_end(buf)
    }
}

impl SeekInChars for Clob {
    fn seek_in_chars(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.lob.seek(pos)
    }
}

impl Write for Clob {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lob.write_chars(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;
    use lazy_static::lazy_static;
    use std::io::Read;
    use std::io::Seek;
    use std::io::Write;

    // one-byte characters in UTF-8
    const CRAB_CHARS: [&'static str; 4] = [
        // one-byte characters in UTF-8
        "crab",
        // two-byte characters in UTF-8
        //   D0  BA  D1  80  D0  B0  D0  B1
        //  208 186 209 128 208 176 208 177
        "краб",
        // three-byte character in UTF-8
        //   E8  9F  B9
        //  232 159 185
        "蟹",
        // four-byte character in UTF-8
        //   F0  9F  A6  80
        //  240 159 169 128
        "🦀",
    ];

    // simple pseudo-random number generator which returns same sequence
    struct Rand {
        next: u64,
    }

    impl Rand {
        fn new() -> Rand {
            Rand { next: 1 }
        }
    }

    impl Iterator for Rand {
        type Item = u16;
        fn next(&mut self) -> Option<Self::Item> {
            // https://pubs.opengroup.org/onlinepubs/9699919799/functions/rand.html#tag_16_473_06_02
            self.next = self.next.overflowing_mul(1103515245).0;
            self.next = self.next.overflowing_add(12345).0;
            Some(((self.next / 65536) % 32768) as u16)
        }
    }

    lazy_static! {
        static ref TEST_DATA: String = {
            Rand::new()
                .take(100)
                .map(|n| CRAB_CHARS[(n as usize) % CRAB_CHARS.len()])
                .collect::<Vec<_>>()
                .join("")
        };
    }

    #[test]
    fn read_write_blob() -> Result<()> {
        let conn = test_util::connect()?;
        let mut lob = Blob::new(&conn)?;
        assert_eq!(lob.seek(io::SeekFrom::Current(0)).unwrap(), 0);
        lob.write(TEST_DATA.as_bytes()).unwrap();
        assert_eq!(
            lob.seek(io::SeekFrom::Current(0)).unwrap(),
            TEST_DATA.len() as u64
        );

        lob.open_resource()?;
        assert!(lob.is_resource_open()?);
        lob.write(TEST_DATA.as_bytes()).unwrap();
        lob.close_resource()?;
        assert!(!lob.is_resource_open()?);

        lob.seek(io::SeekFrom::Start(0)).unwrap();
        let mut buf = vec![0; TEST_DATA.len()];
        let len = lob.read(&mut buf).unwrap();
        assert_eq!(len, TEST_DATA.len());
        assert_eq!(TEST_DATA.as_bytes(), buf);

        let len = lob.read(&mut buf).unwrap();
        assert_eq!(len, TEST_DATA.len());
        assert_eq!(TEST_DATA.as_bytes(), buf);
        assert_eq!(
            lob.seek(io::SeekFrom::Current(0)).unwrap(),
            TEST_DATA.len() as u64 * 2,
        );

        lob.trim(TEST_DATA.len() as u64)?;
        Ok(())
    }

    #[test]
    fn query_blob() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let conn = test_util::connect()?;
        let mut lob = Blob::new(&conn)?;
        lob.write(b"BLOB DATA")?;
        conn.execute("insert into TestBLOBs values (1, :1)", &[&lob])?;
        let sql = "select BLOBCol from TestBLOBs where IntCol = 1";

        // query blob as binary
        let mut stmt = conn.statement(sql).build()?;
        assert_eq!(stmt.query_row_as::<Vec<u8>>(&[])?, b"BLOB DATA");

        // query blob as Blob
        let mut stmt = conn.statement(sql).lob_locator().build()?;
        let mut buf = Vec::new();
        stmt.query_row_as::<Blob>(&[])?
            .read_to_end(&mut buf)
            .unwrap();
        assert_eq!(buf, b"BLOB DATA");

        // error when querying blob as Blob without `StatementBuilder.lob_locator()`.
        let mut stmt = conn.statement(sql).build()?;
        assert_eq!(stmt.query_row_as::<Blob>(&[]).unwrap_err().to_string(), "invalid operation: Please use StatementBuilder.lob_locator() to fetch LOB data as Blob");
        Ok(())
    }

    #[test]
    fn read_write_clob() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let conn = test_util::connect()?;
        let mut lob = Clob::new(&conn, &OracleType::CLOB)?;
        let test_data_len = utf16_len(TEST_DATA.as_bytes())? as u64;
        assert_eq!(lob.seek_in_chars(io::SeekFrom::Current(0))?, 0);
        assert_eq!(lob.write(TEST_DATA.as_bytes())?, TEST_DATA.len());
        assert_eq!(lob.stream_position_in_chars()?, test_data_len);
        assert_eq!(lob.size()?, test_data_len);

        lob.seek_in_chars(io::SeekFrom::Start(0))?;
        let mut buf = vec![0; TEST_DATA.len()];
        let mut offset = 0;
        while offset < buf.len() {
            let mut len = lob.read(&mut buf[offset..])?;
            if len == 0 {
                len = lob.read_to_end(&mut buf)?;
                if len == 0 {
                    panic!(
                        "lob.read returns zero. (lob: {:?}, buf.len(): {}, offset: {}, buf: {:?}, data: {:?})",
                        lob.lob,
                        buf.len(),
                        offset,
                        &buf[0..offset],
                        *TEST_DATA
                    );
                }
            }
            offset += len as usize;
        }
        assert_eq!(offset, TEST_DATA.len());
        assert_eq!(TEST_DATA.as_bytes(), buf);

        lob.write(&"🦀".as_bytes()[0..1]).unwrap_err();
        lob.write(&"🦀".as_bytes()[0..2]).unwrap_err();
        lob.write(&"🦀".as_bytes()[0..3]).unwrap_err();
        lob.write(&"🦀".as_bytes()[0..4])?;

        lob.seek_in_chars(io::SeekFrom::Current(-2))?;
        lob.read(&mut buf[0..1]).unwrap_err(); // one byte buffer for four byte UTF-8
        lob.read(&mut buf[0..2]).unwrap_err(); // two bytes buffer for four byte UTF-8
        lob.read(&mut buf[0..3]).unwrap_err(); // three bytes buffer for four byte UTF-8
        buf.fill(0);
        assert_eq!(lob.read(&mut buf[0..4])?, 4);
        assert_eq!(&buf[0..4], "🦀".as_bytes());
        lob.seek_in_chars(io::SeekFrom::Current(-2))?;
        buf.fill(0);
        assert_eq!(lob.read(&mut buf[0..5])?, 4);
        assert_eq!(&buf[0..4], "🦀".as_bytes());

        lob.seek_in_chars(io::SeekFrom::Current(-3))?;
        lob.read(&mut buf[0..1]).unwrap_err(); // one byte buffer for two byte UTF-8
        buf.fill(0);
        assert_eq!(lob.read(&mut buf[0..2])?, 2);
        assert_eq!(&buf[0..2], "б".as_bytes());
        lob.seek_in_chars(io::SeekFrom::Current(-1))?;
        buf.fill(0);
        assert_eq!(lob.read(&mut buf[0..3])?, 2);
        assert_eq!(&buf[0..2], "б".as_bytes());
        lob.seek_in_chars(io::SeekFrom::Current(-1))?;
        buf.fill(0);
        assert_eq!(lob.read(&mut buf[0..4])?, 2);
        assert_eq!(&buf[0..2], "б".as_bytes());
        lob.seek_in_chars(io::SeekFrom::Current(-1))?;
        buf.fill(0);
        assert_eq!(lob.read(&mut buf[0..5])?, 2);
        assert_eq!(&buf[0..2], "б".as_bytes());
        lob.seek_in_chars(io::SeekFrom::Current(-1))?;
        buf.fill(0);
        assert_eq!(lob.read(&mut buf[0..6])?, 6);
        assert_eq!(&buf[0..6], "б🦀".as_bytes());

        Ok(())
    }
}
