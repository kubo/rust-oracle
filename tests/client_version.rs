// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2018 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

mod common;

use oracle::Version;

#[test]
fn client_version() {
    let ver = match Version::client() {
        Ok(ver) => ver,
        Err(err) => panic!("Failed to get client version: {}", err),
    };
    let conn = common::connect().unwrap();
    let ver_from_query = conn.query_row_as::<String>("SELECT client_version FROM v$session_connect_info WHERE sid = SYS_CONTEXT('USERENV', 'SID')", &[]).unwrap();
    assert_eq!(ver.to_string(), ver_from_query);
}
