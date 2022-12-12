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

//! Type definitions for I/O in characters
use std::io::{Result, SeekFrom};

/// A cursor which can be moved within a stream of characters.
///
/// This is same with [`Seek`] except positions are numbered in characters, not in bytes.
///
/// [`Seek`]: std::io::Seek
pub trait SeekInChars {
    /// Seek to an offset, in characters, in a stream.
    ///
    /// A seek beyond the end of a stream is allowed, but behavior is defined
    /// by the implementation.
    ///
    /// If the seek operation completed successfully,
    /// this method returns the new position from the start of the stream.
    /// That position can be used later with [`SeekFrom::Start`].
    ///
    /// # Errors
    ///
    /// Seeking to a negative offset is considered an error.
    fn seek_in_chars(&mut self, pos: SeekFrom) -> Result<u64>;

    /// Returns the current seek position from the start of the stream.
    ///
    /// This is equivalent to `self.seek_in_chars(SeekFrom::Current(0))`.
    fn stream_position_in_chars(&mut self) -> Result<u64> {
        self.seek_in_chars(SeekFrom::Current(0))
    }
}
