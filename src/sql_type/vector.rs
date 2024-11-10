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

use crate::private;
use crate::sql_type::OracleType;
use crate::sql_type::SqlValue;
use crate::sql_type::ToSql;
use crate::sql_type::ToSqlNull;
use crate::Connection;
use crate::Error;
use crate::ErrorKind;
use crate::Result;
use odpic_sys::*;
use std::fmt;
use std::os::raw::c_void;

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

/// Reference to vector dimension elements
///
/// See the [module-level documentation](index.html) for more.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum VecRef<'a> {
    /// Wraps `[f32]` slice data as Oracle data type `VECTOR(FLOAT32)`
    Float32(&'a [f32]),
    /// Wraps `[f64]` slice data as Oracle data type `VECTOR(FLOAT64)`
    Float64(&'a [f64]),
    /// Wraps `[i8]` slice data as Oracle data type `VECTOR(INT8)`
    Int8(&'a [i8]),
    /// Wraps `[u8]` slice data as Oracle data type `VECTOR(BINARY)`
    Binary(&'a [u8]),
}

impl VecRef<'_> {
    pub(crate) fn to_dpi(&self) -> Result<dpiVectorInfo> {
        match self {
            VecRef::Float32(slice) => Ok(dpiVectorInfo {
                format: DPI_VECTOR_FORMAT_FLOAT32 as u8,
                numDimensions: slice.len().try_into()?,
                dimensionSize: 4,
                dimensions: dpiVectorDimensionBuffer {
                    asPtr: slice.as_ptr() as *mut c_void,
                },
            }),
            VecRef::Float64(slice) => Ok(dpiVectorInfo {
                format: DPI_VECTOR_FORMAT_FLOAT64 as u8,
                numDimensions: slice.len().try_into()?,
                dimensionSize: 8,
                dimensions: dpiVectorDimensionBuffer {
                    asPtr: slice.as_ptr() as *mut c_void,
                },
            }),
            VecRef::Int8(slice) => Ok(dpiVectorInfo {
                format: DPI_VECTOR_FORMAT_INT8 as u8,
                numDimensions: slice.len().try_into()?,
                dimensionSize: 1,
                dimensions: dpiVectorDimensionBuffer {
                    asPtr: slice.as_ptr() as *mut c_void,
                },
            }),
            VecRef::Binary(slice) => Ok(dpiVectorInfo {
                format: DPI_VECTOR_FORMAT_BINARY as u8,
                numDimensions: (slice.len() * 8).try_into()?,
                dimensionSize: 1,
                dimensions: dpiVectorDimensionBuffer {
                    asPtr: slice.as_ptr() as *mut c_void,
                },
            }),
        }
    }

    /// Returns vector dimension element format
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::sql_type::vector::VecFmt;
    /// # use oracle::sql_type::vector::VecRef;
    /// // Refernce to float32 vector data.
    /// let vec_ref = VecRef::Float32(&[0.001, 3.0, 5.3]);
    ///
    /// assert_eq!(vec_ref.format(), VecFmt::Float32);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn format(&self) -> VecFmt {
        match self {
            VecRef::Float32(_) => VecFmt::Float32,
            VecRef::Float64(_) => VecFmt::Float64,
            VecRef::Int8(_) => VecFmt::Int8,
            VecRef::Binary(_) => VecFmt::Binary,
        }
    }

    /// Gets the containing vector data as slice
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::sql_type::vector::VecRef;
    /// // Refernce to float32 vector data.
    /// let vec_ref = VecRef::Float32(&[0.001, 3.0, 5.3]);
    ///
    /// // Gets as a slice of [f32]
    /// assert_eq!(vec_ref.as_slice::<f32>()?, &[0.001, 3.0, 5.3]);
    ///
    /// // Errors for other vector dimension types.
    /// assert!(vec_ref.as_slice::<f64>().is_err());
    /// assert!(vec_ref.as_slice::<i8>().is_err());
    /// assert!(vec_ref.as_slice::<u8>().is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_slice<T>(&self) -> Result<&[T]>
    where
        T: VectorFormat,
    {
        T::vec_ref_to_slice(self)
    }

    fn oracle_type(&self) -> OracleType {
        match self {
            VecRef::Float32(slice) => OracleType::Vector(slice.len() as u32, VecFmt::Float32),
            VecRef::Float64(slice) => OracleType::Vector(slice.len() as u32, VecFmt::Float64),
            VecRef::Int8(slice) => OracleType::Vector(slice.len() as u32, VecFmt::Int8),
            VecRef::Binary(slice) => OracleType::Vector(slice.len() as u32 * 8, VecFmt::Binary),
        }
    }
}

impl<'a, T> From<&'a [T]> for VecRef<'a>
where
    T: VectorFormat,
{
    fn from(s: &'a [T]) -> VecRef<'a> {
        T::slice_to_vec_ref(s)
    }
}

impl<'a, T> TryFrom<VecRef<'a>> for &'a [T]
where
    T: VectorFormat,
{
    type Error = Error;

    fn try_from(s: VecRef) -> Result<&[T]> {
        T::vec_ref_to_slice(&s)
    }
}

impl ToSqlNull for VecRef<'_> {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Vector(0, VecFmt::Flexible))
    }
}

impl ToSql for VecRef<'_> {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::Vector(0, VecFmt::Flexible))
    }
    fn to_sql(&self, val: &mut SqlValue) -> Result<()> {
        val.set_vec_ref(self, "VecRef")
    }
}

/// Trait for vector dimension element type
///
/// This trait is sealed and cannot be implemented for types outside of the `oracle` crate.
pub trait VectorFormat: private::Sealed + Sized {
    #[doc(hidden)]
    fn slice_to_vec_ref(s: &[Self]) -> VecRef;
    #[doc(hidden)]
    fn vec_ref_to_slice<'a>(s: &VecRef<'a>) -> Result<&'a [Self]>;
}

/// For the element type of Oracle data type `VECTOR(FLOAT32)`
impl VectorFormat for f32 {
    fn slice_to_vec_ref(s: &[Self]) -> VecRef {
        VecRef::Float32(s)
    }
    fn vec_ref_to_slice<'a>(s: &VecRef<'a>) -> Result<&'a [Self]> {
        match s {
            VecRef::Float32(s) => Ok(s),
            _ => Err(Error::new(
                ErrorKind::InvalidTypeConversion,
                format!("Could not convert {} to &[f32]", s.oracle_type()),
            )),
        }
    }
}

/// For the element type of Oracle data type `VECTOR(FLOAT64)`
impl VectorFormat for f64 {
    fn slice_to_vec_ref(s: &[Self]) -> VecRef {
        VecRef::Float64(s)
    }
    fn vec_ref_to_slice<'a>(s: &VecRef<'a>) -> Result<&'a [Self]> {
        match s {
            VecRef::Float64(s) => Ok(s),
            _ => Err(Error::new(
                ErrorKind::InvalidTypeConversion,
                format!("Could not convert {} to &[f64]", s.oracle_type()),
            )),
        }
    }
}

/// For the element type of Oracle data type `VECTOR(INT8)`
impl VectorFormat for i8 {
    fn slice_to_vec_ref(s: &[Self]) -> VecRef {
        VecRef::Int8(s)
    }
    fn vec_ref_to_slice<'a>(s: &VecRef<'a>) -> Result<&'a [Self]> {
        match s {
            VecRef::Int8(s) => Ok(s),
            _ => Err(Error::new(
                ErrorKind::InvalidTypeConversion,
                format!("Could not convert {} to &[i8]", s.oracle_type()),
            )),
        }
    }
}

/// For the element type of Oracle data type `VECTOR(BINARY)`
impl VectorFormat for u8 {
    fn slice_to_vec_ref(s: &[Self]) -> VecRef {
        VecRef::Binary(s)
    }
    fn vec_ref_to_slice<'a>(s: &VecRef<'a>) -> Result<&'a [Self]> {
        match s {
            VecRef::Binary(s) => Ok(s),
            _ => Err(Error::new(
                ErrorKind::InvalidTypeConversion,
                format!("Could not convert {} to &[u8]", s.oracle_type()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sql_type::vector::VecFmt;
    use crate::sql_type::vector::VecRef;
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

    #[test]
    fn to_sql() -> Result<()> {
        let conn = test_util::connect()?;

        if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
            return Ok(());
        }
        let binary_vec = test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)?;
        conn.execute("delete from test_vector_type", &[])?;
        let mut stmt = conn
            .statement("insert into test_vector_type(id, vec) values(:1, :2)")
            .build()?;
        let mut expected_data = vec![];
        stmt.execute(&[&1, &VecRef::Float32(&[1.0, 1.25, 1.5])])?;
        expected_data.push((1, "FLOAT32", "[1.0E+000,1.25E+000,1.5E+000]"));
        stmt.execute(&[&2, &VecRef::Float64(&[2.0, 2.25, 2.5])])?;
        expected_data.push((2, "FLOAT64", "[2.0E+000,2.25E+000,2.5E+000]"));
        stmt.execute(&[&3, &VecRef::Int8(&[3, 4, 5])])?;
        expected_data.push((3, "INT8", "[3,4,5]"));
        if binary_vec {
            stmt.execute(&[&4, &VecRef::Binary(&[6, 7, 8])])?;
            expected_data.push((4, "BINARY", "[6,7,8]"));
        }
        let mut index = 0;
        for row_result in conn.query_as::<(i32, String, String)>(
            "select id, vector_dimension_format(vec), from_vector(vec) from test_vector_type order by id",
            &[],
        )? {
            let row = row_result?;
            assert!(index < expected_data.len());
            let data = &expected_data[index];
            assert_eq!(row.0, data.0);
            assert_eq!(row.1, data.1);
            assert_eq!(row.2, data.2);
            index += 1;
        }
        assert_eq!(index, expected_data.len());
        Ok(())
    }
}
