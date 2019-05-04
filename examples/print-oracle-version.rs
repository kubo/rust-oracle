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

extern crate oracle;

use oracle::{Connection, Version};

fn main() {
    let client_ver = Version::client().unwrap();
    println!("Oracle Client Version: {}", client_ver);

    let conn = Connection::connect("scott", "tiger", "").unwrap();
    let (server_ver, banner) = conn.server_version().unwrap();
    println!("Oracle Server Version: {}", server_ver);
    println!("--- Server Version Banner ---");
    println!("{}", banner);
    println!("-----------------------------");
}
