// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2025 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use crate::Error;
use crate::Result;
use odpic_sys::*;
use std::fmt;

/// Vector dimension element format
///
/// This is used in a tuple element of [`OracleType::Vector`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum VecFmt {
    /// FLOAT32 (32-bit IEEE floating-point numbers)
    Float32 = DPI_VECTOR_FORMAT_FLOAT32 as u8,
    /// FLOAT64 (64-bit IEEE floating-point numbers)
    Float64 = DPI_VECTOR_FORMAT_FLOAT64 as u8,
    /// INT8 (8-bit integers)
    Int8 = DPI_VECTOR_FORMAT_INT8 as u8,
    /// BINARY (packed UINT8 bytes where each dimension is a single bit)
    Binary = DPI_VECTOR_FORMAT_BINARY as u8,
    /// Flexible format, corresponding to `*` in vector column definitions such as `VECTOR(1024, *)`
    Flexible = 0,
}

impl VecFmt {
    pub(crate) fn from_dpi(format: u8) -> Result<VecFmt> {
        match format as u32 {
            0 => Ok(VecFmt::Flexible),
            DPI_VECTOR_FORMAT_FLOAT32 => Ok(VecFmt::Float32),
            DPI_VECTOR_FORMAT_FLOAT64 => Ok(VecFmt::Float64),
            DPI_VECTOR_FORMAT_INT8 => Ok(VecFmt::Int8),
            DPI_VECTOR_FORMAT_BINARY => Ok(VecFmt::Binary),
            _ => Err(Error::internal_error(format!(
                "unknown vector format {}",
                format
            ))),
        }
    }
}

impl fmt::Display for VecFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            match self {
                VecFmt::Float32 => f.pad("FLOAT32"),
                VecFmt::Float64 => f.pad("FLOAT64"),
                VecFmt::Int8 => f.pad("INT8"),
                VecFmt::Binary => f.pad("BINARY"),
                VecFmt::Flexible => f.pad("*"),
            }
        } else {
            match self {
                VecFmt::Float32 => f.pad("Float32"),
                VecFmt::Float64 => f.pad("Float64"),
                VecFmt::Int8 => f.pad("Int8"),
                VecFmt::Binary => f.pad("Binary"),
                VecFmt::Flexible => f.pad("Flexible"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sql_type::vector::VecFmt;
    use crate::sql_type::OracleType;
    use crate::test_util;
    use crate::Result;
    use std::iter::zip;

    #[test]
    fn column_info() -> Result<()> {
        let conn = test_util::connect()?;

        // Check test_vector_type column info
        if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
            return Ok(());
        }
        let expected_colinfo = [
            ("ID", OracleType::Number(38, 0), "NUMBER(38)"),
            (
                "VEC",
                OracleType::Vector(0, VecFmt::Flexible),
                "VECTOR(*, *)",
            ),
            (
                "FIXED_DIM",
                OracleType::Vector(2, VecFmt::Flexible),
                "VECTOR(2, *)",
            ),
            (
                "F32",
                OracleType::Vector(0, VecFmt::Float32),
                "VECTOR(*, FLOAT32)",
            ),
            (
                "F64",
                OracleType::Vector(4, VecFmt::Float64),
                "VECTOR(4, FLOAT64)",
            ),
            ("I8", OracleType::Vector(0, VecFmt::Int8), "VECTOR(*, INT8)"),
        ];
        let rows = conn.query("select * from test_vector_type", &[])?;
        let colinfo = rows.column_info();
        assert_eq!(colinfo.len(), expected_colinfo.len());
        for (ci, expected) in zip(colinfo, &expected_colinfo) {
            assert_eq!(ci.name(), expected.0);
            assert_eq!(ci.oracle_type(), &expected.1);
            assert_eq!(ci.oracle_type().to_string(), expected.2);
        }

        // Check test_binary_vector column info
        if !test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)? {
            return Ok(());
        }
        let expected_colinfo = [
            ("ID", OracleType::Number(38, 0), "NUMBER(38)"),
            (
                "VEC",
                OracleType::Vector(0, VecFmt::Binary),
                "VECTOR(*, BINARY)",
            ),
        ];
        let rows = conn.query("select * from test_binary_vector", &[])?;
        let colinfo = rows.column_info();
        assert_eq!(colinfo.len(), expected_colinfo.len());
        for (ci, expected) in zip(colinfo, &expected_colinfo) {
            assert_eq!(ci.name(), expected.0);
            assert_eq!(ci.oracle_type(), &expected.1);
            assert_eq!(ci.oracle_type().to_string(), expected.2);
        }
        Ok(())
    }
}
