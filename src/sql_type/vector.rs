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

//! VECTOR data type support
//!
//! Oracle Database 23ai introduced a new data type [VECTOR]. This module contains
//! rust types to support the Oracle type.
//!
//! In short:
//! * Insert VECTOR data by wrapping slice data such as `&[f32]` in [`VecRef`].
//! * Fetch VECTOR data as `Vec<_>` such as `Vec<f32>` when the vector dimension element format is known.
//! * Fetch VECTOR data as [`Vector`] type when the vector dimension element format is unknown.
//! * Fetch VECTOR data as [`Vector`] type when the method to be called takes a slice
//!   as an argument and you want to avoid the cost of memory allocation for `Vec<_>`.
//!
//! # Note
//!
//! Fetched [`Vector`] data should be dropped before the next fetch. That's because [`Vector`] and [`ResultSet`]
//! share an internal fetch buffer. When the next row is fetched from the result set and the vector
//! is alive and has a reference to the buffer, a new fetch buffer may be allocated. When the vector is dropped,
//! and only the result set has the reference, it is reused.
//!
//! # Examples
//!
//! Wrap slice data in [`VecRef`] to insert VECTOR data.
//!
//! ```
//! # use oracle::test_util;
//! # use oracle::sql_type::vector::VecRef;
//! # let conn = test_util::connect()?;
//! # if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
//! #     return Ok(());
//! # }
//! let mut stmt = conn
//!     .statement("insert into test_vector_type(id, vec) values (:1, :2)")
//!     .build()?;
//! // Insert &[f32] slice as Oracle type VECTOR(FLOAT32, 3).
//! stmt.execute(&[&1, &VecRef::Float32(&[0.0001, 100.0, 3.4])])?;
//!
//! // Insert &[f64] slice as Oracle type VECTOR(FLOAT64, 3).
//! stmt.execute(&[&2, &VecRef::Float64(&[5.6, 1000.3, 0.0838])])?;
//!
//! // Insert &[i8] slice as Oracle type VECTOR(INT8, 3).
//! stmt.execute(&[&3, &VecRef::Int8(&[1, 100, -30])])?;
//!
//! # if test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)? {
//! // Insert &[u8] slice as Oracle type VECTOR(BINARY, 24).
//! // Binary vectors require the database initialization parameter COMPATIBLE
//! // to be set to 23.5.0.0.0 or greater on Oracle Database.
//! stmt.execute(&[&4, &VecRef::Binary(&[128, 0, 225])])?;
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Fetch VECTOR data as `Vec<_>` when the vector dimension element format is known.
//!
//! ```
//! # use oracle::test_util;
//! # use oracle::sql_type::vector::VecRef;
//! # let conn = test_util::connect()?;
//! # if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
//! #     return Ok(());
//! # }
//! # let mut stmt = conn
//! #     .statement("insert into test_vector_type(id, vec) values (:1, :2)")
//! #     .build()?;
//! # stmt.execute(&[&1, &VecRef::Float32(&[0.0001, 100.0, 3.4])])?;
//! # stmt.execute(&[&2, &VecRef::Float64(&[5.6, 1000.3, 0.0838])])?;
//! # stmt.execute(&[&3, &VecRef::Int8(&[1, 100, -30])])?;
//! # if test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)? {
//! #     stmt.execute(&[&4, &VecRef::Binary(&[128, 0, 225])])?;
//! # }
//! // Fetch VECTOR(FLOAT32) data as Vec<f32>
//! let f32_vec = conn.query_row_as::<Vec<f32>>("select vec from test_vector_type where id = :1", &[&1])?;
//! assert_eq!(f32_vec, vec![0.0001, 100.0, 3.4]);
//!
//! // Fetch VECTOR(FLOAT64) data as Vec<f64>
//! let f64_vec = conn.query_row_as::<Vec<f64>>("select vec from test_vector_type where id = :1", &[&2])?;
//! assert_eq!(f64_vec, vec![5.6, 1000.3, 0.0838]);
//!
//! // Fetch VECTOR(INT8) data as Vec<i8>
//! let i8_vec = conn.query_row_as::<Vec<i8>>("select vec from test_vector_type where id = :1", &[&3])?;
//! assert_eq!(i8_vec, vec![1, 100, -30]);
//!
//! # if test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)? {
//! // Fetch VECTOR(BINARY) data as Vec<u8>
//! let binary_vec = conn.query_row_as::<Vec<u8>>("select vec from test_vector_type where id = :1", &[&4])?;
//! assert_eq!(binary_vec, vec![128, 0, 225]);
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Fetch VECTOR data as [`Vector`] type and check the dimension element format.
//!
//! ```
//! # use oracle::test_util;
//! # use oracle::sql_type::vector::VecRef;
//! # use oracle::sql_type::vector::Vector;
//! # let conn = test_util::connect()?;
//! # if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
//! #     return Ok(());
//! # }
//! # let mut stmt = conn
//! #     .statement("insert into test_vector_type(id, vec) values (:1, :2)")
//! #     .build()?;
//! # stmt.execute(&[&1, &VecRef::Float32(&[0.0001, 100.0, 3.4])])?;
//! # stmt.execute(&[&2, &VecRef::Float64(&[5.6, 1000.3, 0.0838])])?;
//! # stmt.execute(&[&3, &VecRef::Int8(&[1, 100, -30])])?;
//! # if test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)? {
//! #     stmt.execute(&[&4, &VecRef::Binary(&[128, 0, 225])])?;
//! # }
//! let mut rows = conn.query_as::<(i32, Vector)>("select id, vec from test_vector_type", &[])?;
//! for row_result in rows {
//!     let (id, vector) = row_result?;
//!     // Check the vector dimension element type at runtime.
//!     match vector.as_vec_ref() {
//!         // When id == 1, the vector data type is VECTOR(FLOAT32, 3).
//!         VecRef::Float32(slice) => {
//!             assert_eq!(id, 1);
//!             assert_eq!(slice, &[0.0001, 100.0, 3.4]);
//!         },
//!         // When id == 2, the vector data type is VECTOR(FLOAT64, 3).
//!         VecRef::Float64(slice) => {
//!             assert_eq!(id, 2);
//!             assert_eq!(slice, &[5.6, 1000.3, 0.0838]);
//!         },
//!         // When id == 3, the vector data type is VECTOR(INT8, 3).
//!         VecRef::Int8(slice) => {
//!             assert_eq!(id, 3);
//!             assert_eq!(slice, &[1, 100, -30]);
//!         },
//!         // When id == 4, the vector data type is VECTOR(BIANRY, 24).
//!         VecRef::Binary(slice) => {
//!             assert_eq!(id, 4);
//!             assert_eq!(slice, &[128, 0, 225]);
//!         },
//!         _ => panic!("unexpected format {}", vector.format()),
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Fetch VECTOR data as [`Vector`] type and get a reference to the internal
//! array data held by [`ResultSet`]. Use this when the method to be called takes a slice
//! as the vector data.
//!
//! ```
//! # use oracle::test_util;
//! # use oracle::sql_type::vector::VecRef;
//! # use oracle::sql_type::vector::Vector;
//! // Assume that you have a vector type
//! struct YourVectorType {
//!     // ...
//! }
//!
//! // The vector type has a method taking &[f32] slice
//! // but doesn't have a method taking Vec<f32>.
//! impl YourVectorType {
//!     pub fn from_slice(slice: &[f32]) -> YourVectorType {
//!        // ...
//! #      assert_eq!(slice, &[0.0001, 100.0, 3.4]);
//! #      YourVectorType{}
//!     }
//! }
//!
//! # let conn = test_util::connect()?;
//! # if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
//! #     return Ok(());
//! # }
//! # let mut stmt = conn
//! #     .statement("insert into test_vector_type(id, vec) values (:1, :2)")
//! #     .build()?;
//! # stmt.execute(&[&1, &VecRef::Float32(&[0.0001, 100.0, 3.4])])?;
//! // The following code is inefficient. That's becuase `Vec<f32>` is allocated
//! // only to be passed to from_slice().
//! let vec = conn.query_row_as::<Vec<f32>>("select vec from test_vector_type where id = :1", &[&1])?;
//! YourVectorType::from_slice(&vec);
//!
//! // The following code avoids memory allocation to create a temporary Vec data.
//! let vec = conn.query_row_as::<Vector>("select vec from test_vector_type where id = :1", &[&1])?;
//! YourVectorType::from_slice(vec.as_slice()?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [VECTOR]: https://docs.oracle.com/en/database/oracle/oracle-database/23/vecse/overview-ai-vector-search.html

use crate::private;
use crate::sql_type::FromSql;
use crate::sql_type::OracleType;
use crate::sql_type::SqlValue;
use crate::sql_type::ToSql;
use crate::sql_type::ToSqlNull;
use crate::Connection;
use crate::DpiVar;
use crate::Error;
use crate::ErrorKind;
use crate::Result;
#[cfg(doc)]
use crate::ResultSet;
use odpic_sys::*;
use std::fmt;
use std::os::raw::c_void;
use std::rc::Rc;
use std::slice;

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
    // The 'static lifetime in the returned value is incorrect.
    // Its actual lifetime is that of data referred by info.
    pub(crate) unsafe fn from_dpi(info: dpiVectorInfo) -> Result<VecRef<'static>> {
        match info.format as u32 {
            DPI_VECTOR_FORMAT_FLOAT32 => Ok(VecRef::Float32(slice::from_raw_parts(
                info.dimensions.asFloat,
                info.numDimensions as usize,
            ))),
            DPI_VECTOR_FORMAT_FLOAT64 => Ok(VecRef::Float64(slice::from_raw_parts(
                info.dimensions.asDouble,
                info.numDimensions as usize,
            ))),
            DPI_VECTOR_FORMAT_INT8 => Ok(VecRef::Int8(slice::from_raw_parts(
                info.dimensions.asInt8,
                info.numDimensions as usize,
            ))),
            DPI_VECTOR_FORMAT_BINARY => Ok(VecRef::Binary(slice::from_raw_parts(
                info.dimensions.asPtr as *const u8,
                (info.numDimensions / 8) as usize,
            ))),
            _ => Err(Error::internal_error(format!(
                "unknown vector format {}",
                info.format
            ))),
        }
    }

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

    pub(crate) fn oracle_type(&self) -> OracleType {
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

/// Vector data retrieved from the Oracle database
///
/// See the [module-level documentation](index.html) for more.
///
/// # Note
///
/// Fetched [`Vector`] data should be dropped before the next fetch. That's because [`Vector`] and [`ResultSet`]
/// share an internal fetch buffer. When the next row is fetched from the result set and the vector
/// is alive and has a reference to the buffer, a new fetch buffer may be allocated. When the vector is dropped,
/// and only the result set has the reference, it is reused.
#[derive(Debug)]
pub struct Vector {
    // The 'static lifetime is incorrect. Its actual lifetime is same with DpiVar.
    vec_ref: VecRef<'static>,
    // _var must be held until the end of lifetime.
    // Otherwise vec_ref may points to freed memory region.
    _var: Rc<DpiVar>,
}

impl Vector {
    pub(crate) fn new(vec_ref: VecRef<'static>, var: Rc<DpiVar>) -> Result<Vector> {
        Ok(Vector { vec_ref, _var: var })
    }

    /// Returns vector dimension element format.
    ///
    /// **Note:** It doesn't return `VecFmt::Flexible`.
    pub fn format(&self) -> VecFmt {
        self.vec_ref.format()
    }

    /// Returns the internal [`VecRef`] data.
    pub fn as_vec_ref(&self) -> &VecRef {
        &self.vec_ref
    }

    /// Gets the containing vector data as slice
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::test_util;
    /// # use oracle::sql_type::vector::VecRef;
    /// # use oracle::sql_type::vector::Vector;
    /// # let conn = test_util::connect()?;
    /// # if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
    /// #     return Ok(());
    /// # }
    /// # let mut stmt = conn
    /// #     .statement("insert into test_vector_type(id, vec) values (:1, :2)")
    /// #     .build()?;
    /// # stmt.execute(&[&1, &VecRef::Float32(&[0.0001, 100.0, 3.4])])?;
    /// // Fetch VECTOR(FLOAT32) data from Oracle.
    /// let vec = conn.query_row_as::<Vector>("select vec from test_vector_type where id = 1", &[])?;
    ///
    /// // Gets as a slice of [f32]
    /// assert_eq!(vec.as_slice::<f32>()?, &[0.0001, 100.0, 3.4]);
    ///
    /// // Fails for other types.
    /// assert!(vec.as_slice::<f64>().is_err());
    /// assert!(vec.as_slice::<i8>().is_err());
    /// assert!(vec.as_slice::<u8>().is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_slice<T>(&self) -> Result<&[T]>
    where
        T: VectorFormat,
    {
        T::vec_ref_to_slice(&self.vec_ref)
    }
}

impl FromSql for Vector {
    fn from_sql(val: &SqlValue) -> Result<Vector> {
        val.to_vector()
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
    use crate::sql_type::vector::Vector;
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

    #[test]
    fn vec_from_sql() -> Result<()> {
        let conn = test_util::connect()?;

        if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
            return Ok(());
        }
        let binary_vec = test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)?;
        conn.execute("delete from test_vector_type", &[])?;
        let mut expected_data = vec![];
        conn.execute("insert into test_vector_type(id, vec) values(1, TO_VECTOR('[1.0, 2.25, 3.5]', 3, FLOAT32))", &[])?;
        expected_data.push((1, "FLOAT32", VecRef::Float32(&[1.0, 2.25, 3.5])));
        conn.execute("insert into test_vector_type(id, vec) values(2, TO_VECTOR('[4.0, 5.25, 6.5]', 3, FLOAT64))", &[])?;
        expected_data.push((2, "FLOAT64", VecRef::Float64(&[4.0, 5.25, 6.5])));
        conn.execute(
            "insert into test_vector_type(id, vec) values(3, TO_VECTOR('[7, 8, 9]', 3, INT8))",
            &[],
        )?;
        expected_data.push((3, "INT8", VecRef::Int8(&[7, 8, 9])));
        if binary_vec {
            conn.execute("insert into test_vector_type(id, vec) values(4, TO_VECTOR('[10, 11, 12]', 24, BINARY))", &[])?;
            expected_data.push((4, "BINARY", VecRef::Binary(&[10, 11, 12])));
        }
        let mut index = 0;
        for row_result in conn.query(
            "select id, vector_dimension_format(vec), vec from test_vector_type order by id",
            &[],
        )? {
            let row = row_result?;
            assert!(index < expected_data.len());
            let data = &expected_data[index];
            assert_eq!(row.get::<_, i32>(0)?, data.0);
            assert_eq!(row.get::<_, String>(1)?, data.1);
            match data.2 {
                VecRef::Float32(slice) => assert_eq!(row.get::<_, Vec<f32>>(2)?, slice),
                VecRef::Float64(slice) => assert_eq!(row.get::<_, Vec<f64>>(2)?, slice),
                VecRef::Int8(slice) => assert_eq!(row.get::<_, Vec<i8>>(2)?, slice),
                VecRef::Binary(slice) => assert_eq!(row.get::<_, Vec<u8>>(2)?, slice),
            }
            index += 1;
        }
        assert_eq!(index, expected_data.len());
        Ok(())
    }

    #[test]
    fn vector_from_sql() -> Result<()> {
        let conn = test_util::connect()?;

        if !test_util::check_version(&conn, &test_util::VER23, &test_util::VER23)? {
            return Ok(());
        }
        let binary_vec = test_util::check_version(&conn, &test_util::VER23_5, &test_util::VER23_5)?;
        conn.execute("delete from test_vector_type", &[])?;
        let mut expected_data = vec![];
        conn.execute("insert into test_vector_type(id, vec) values(1, TO_VECTOR('[1.0, 2.25, 3.5]', 3, FLOAT32))", &[])?;
        expected_data.push((1, "FLOAT32", VecRef::Float32(&[1.0, 2.25, 3.5])));
        conn.execute("insert into test_vector_type(id, vec) values(2, TO_VECTOR('[4.0, 5.25, 6.5]', 3, FLOAT64))", &[])?;
        expected_data.push((2, "FLOAT64", VecRef::Float64(&[4.0, 5.25, 6.5])));
        conn.execute(
            "insert into test_vector_type(id, vec) values(3, TO_VECTOR('[7, 8, 9]', 3, INT8))",
            &[],
        )?;
        expected_data.push((3, "INT8", VecRef::Int8(&[7, 8, 9])));
        if binary_vec {
            conn.execute("insert into test_vector_type(id, vec) values(4, TO_VECTOR('[10, 11, 12]', 24, BINARY))", &[])?;
            expected_data.push((4, "BINARY", VecRef::Binary(&[10, 11, 12])));
        }
        let rows = conn
            .statement(
                "select id, vector_dimension_format(vec), vec from test_vector_type order by id",
            )
            .fetch_array_size(2) // This must be lower than number of total rows in order to check Vector holds `_var`.
            .build()?
            .query_as::<(i32, String, Vector)>(&[])?
            .collect::<Result<Vec<_>>>()?;
        let mut index = 0;
        for row in rows {
            assert!(index < expected_data.len());
            let data = &expected_data[index];
            assert_eq!(row.0, data.0);
            assert_eq!(row.1, data.1);
            assert_eq!(row.2.as_vec_ref(), &data.2);
            index += 1;
        }
        assert_eq!(index, expected_data.len());
        Ok(())
    }
}
