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
use oracle::{Connection, ConnParam};

fn main() {
    let username = "sys";
    let password = "change_on_install";
    let database = "";

    // connect as sysdba or sysoper with prelim_auth mode
    let params = [
        ConnParam::Sysdba, // or ConnParam::Sysoper
        ConnParam::PrelimAuth, // required to connect to idle database.
    ];
    let conn = Connection::connect(username, password, database, &params).unwrap();

    // start up database. The database is not mounted at this time.
    conn.startup_database(&[]).unwrap();
    conn.close().unwrap();

    // connect as sysdba or sysoper **without** prelim_auth mode
    let params = [
        ConnParam::Sysdba, // or ConnParam::Sysoper
    ];
    let conn = Connection::connect(username, password, database, &params).unwrap();

    // mount and open the database
    conn.execute("alter database mount", &[]).unwrap();
    println!("Database mounted.");
    conn.execute("alter database open", &[]).unwrap();
    println!("Database opened.");
    conn.close().unwrap();
}
