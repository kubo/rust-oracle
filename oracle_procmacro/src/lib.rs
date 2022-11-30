// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2022 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

//! See [`oracle`] instead.
//! Macros in this crate are documented in the crate also
//! and some links in docs are valid only there.
//!
//! [`oracle`]: https://www.jiubao.org/rust-oracle/oracle/index.html#derives

use proc_macro::TokenStream;

mod derive_row_value;
mod remove_stmt_lifetime;

#[doc = include_str!("../docs/row_value.md")]
#[proc_macro_derive(RowValue, attributes(row_value))]
pub fn derive_row_value(input: TokenStream) -> TokenStream {
    derive_row_value::derive_row_value(input)
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn remove_stmt_lifetime(_args: TokenStream, input: TokenStream) -> TokenStream {
    remove_stmt_lifetime::remove_stmt_lifetime(input)
}
