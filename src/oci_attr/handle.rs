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

//! The module defines types to be set to the associate type [`OciAttr::HandleType`]
use crate::binding::*;
#[cfg(doc)]
use crate::oci_attr::OciAttr;
use crate::private;
#[cfg(doc)]
use crate::Connection;

/// OCI handle type to restrict the associate type [`OciAttr::HandleType`]
///
/// By using the [sealed trait pattern], no types outside of the `oracle` crate implement this.
///
/// [sealed trait pattern]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed
pub trait HandleType: private::Sealed {}

/// OCI handle type related to `Connection` to restrict the type parameters of [`Connection::oci_attr`] and [`Connection::set_oci_attr`]
pub trait ConnHandle: HandleType {
    #[doc(hidden)]
    const HANDLE_TYPE: u32;
}

/// [`HandleType`] for [Service Context Handle Attributes]
///
/// [service context handle attributes]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-D8EE68EB-7E38-4068-B06E-DF5686379E5E
#[derive(Debug)]
pub struct SvcCtx {
    _unused: [usize; 0],
}
impl private::Sealed for SvcCtx {}
impl HandleType for SvcCtx {}
impl ConnHandle for SvcCtx {
    #[doc(hidden)]
    const HANDLE_TYPE: u32 = DPI_OCI_HTYPE_SVCCTX;
}

/// [`HandleType`] for [Server Handle Attributes]
///
/// [server handle attributes]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-2B6D06A7-8EDF-46FF-BDEF-320D293DCA65
#[derive(Debug)]
pub struct Server {
    _unused: [usize; 0],
}
impl private::Sealed for Server {}
impl HandleType for Server {}
impl ConnHandle for Server {
    #[doc(hidden)]
    const HANDLE_TYPE: u32 = DPI_OCI_HTYPE_SERVER;
}

/// [`HandleType`] for [Authentication Information Handle Attributes] and [User Session Handle Attributes]
///
/// [authentication information handle attributes]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-0193473A-20FE-4727-850E-41269F94BAD4
/// [user session handle attributes]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-FB263210-118E-4DB3-A840-1769EF0CB977
#[derive(Debug)]
pub struct Session {
    _unused: [usize; 0],
}
impl private::Sealed for Session {}
impl HandleType for Session {}
impl ConnHandle for Session {
    #[doc(hidden)]
    const HANDLE_TYPE: u32 = DPI_OCI_HTYPE_SESSION;
}

/// [`HandleType`] for [Statement Handle Attributes]
///
/// [statement handle attributes]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-A251CF91-EB9F-4DBC-8BB8-FB5EA92C20DE
#[derive(Debug)]
pub struct Stmt {
    _unused: [usize; 0],
}
impl private::Sealed for Stmt {}
impl HandleType for Stmt {}
