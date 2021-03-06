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

use oracle::{Connector, Privilege, Result};

fn main() -> Result<()> {
    let username = "sys";
    let password = "change_on_install";
    let database = "";

    // connect as sysdba or sysoper with prelim_auth mode
    let conn = Connector::new(username, password, database)
        .privilege(Privilege::Sysdba)
        .prelim_auth(true)
        .connect()?;

    // start up database. The database is not mounted at this time.
    conn.startup_database(&[])?;
    conn.close()?;

    // connect as sysdba or sysoper **without** prelim_auth mode
    let conn = Connector::new(username, password, database)
        .privilege(Privilege::Sysdba)
        .connect()?;

    // mount and open the database
    conn.execute("alter database mount", &[])?;
    println!("Database mounted.");
    conn.execute("alter database open", &[])?;
    println!("Database opened.");
    conn.close()?;
    Ok(())
}
