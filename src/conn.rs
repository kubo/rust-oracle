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

//! Type definitions for connection
//!
//! Some types at the top-level module will move here in future.
use crate::binding::*;

#[derive(Debug, Copy, Clone, PartialEq)]
/// [Session Purity](https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-12410EEC-FE79-42E2-8F6B-EAA9EDA59665)
pub enum Purity {
    /// Must use a new session
    New,
    /// Reuse a pooled session
    Self_,
}

impl Purity {
    pub(crate) fn to_dpi(&self) -> dpiPurity {
        match self {
            Purity::New => DPI_PURITY_NEW,
            Purity::Self_ => DPI_PURITY_SELF,
        }
    }
}
