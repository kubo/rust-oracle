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

//! The module defines types to be set to the associate type [`OciAttr::Mode`]
#[cfg(doc)]
use crate::oci_attr::OciAttr;
use crate::private;
#[cfg(doc)]
use crate::Connection;
#[cfg(doc)]
use crate::Statement;

/// Access mode to restrict the associate type [`OciAttr::Mode`]
///
/// By using the [sealed trait pattern], no types outside of the `oracle` crate implement this.
///
/// [sealed trait pattern]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed
pub trait Mode: private::Sealed {}

/// Access mode to restrict the type parameters of [`Connection::oci_attr`] and [`Statement::oci_attr`]
pub trait ReadMode: Mode {}

/// Access mode to restrict the type parameters of [`Connection::set_oci_attr`] and [`Statement::set_oci_attr`]
pub trait WriteMode: Mode {}

/// Read only mode, which implements [`ReadMode`]
#[derive(Debug)]
pub struct Read;
impl private::Sealed for Read {}
impl Mode for Read {}
impl ReadMode for Read {}

/// Write only mode, which implements [`WriteMode`]
#[derive(Debug)]
pub struct Write;
impl private::Sealed for Write {}
impl Mode for Write {}
impl WriteMode for Write {}

/// Read write mode, which implements both [`ReadMode`] and [`WriteMode`]
#[derive(Debug)]
pub struct ReadWrite;
impl private::Sealed for ReadWrite {}
impl Mode for ReadWrite {}
impl ReadMode for ReadWrite {}
impl WriteMode for ReadWrite {}
