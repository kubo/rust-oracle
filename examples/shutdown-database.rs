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
use oracle::{Connector, Privilege, ShutdownMode};

fn main() {
    let username = "sys";
    let password = "change_on_install";
    let database = "";
    let shutdown_mode = ShutdownMode::Immediate;

    // connect as sysdba or sysoper
    let conn = Connector::new(username, password, database)
        .privilege(Privilege::Sysdba)
        .connect()
        .unwrap();

    // begin 'shutdown'
    conn.shutdown_database(shutdown_mode).unwrap();

    // close the database
    conn.execute("alter database close normal", &[]).unwrap();
    println!("Database closed.");

    // dismount the database
    conn.execute("alter database dismount", &[]).unwrap();
    println!("Database dismounted.");

    // finish 'shutdown'
    conn.shutdown_database(ShutdownMode::Final).unwrap();
    println!("ORACLE instance shut down.");
    conn.close().unwrap();
}
