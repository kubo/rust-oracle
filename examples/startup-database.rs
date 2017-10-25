// Rust Oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2017 Kubo Takehiro <kubo@jiubao.org>
//
// Redistribution and use in source and binary forms, with or without modification, are
// permitted provided that the following conditions are met:
//
//    1. Redistributions of source code must retain the above copyright notice, this list of
//       conditions and the following disclaimer.
//
//    2. Redistributions in binary form must reproduce the above copyright notice, this list
//       of conditions and the following disclaimer in the documentation and/or other materials
//       provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHORS ''AS IS'' AND ANY EXPRESS OR IMPLIED
// WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
// ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
// ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// The views and conclusions contained in the software and documentation are those of the
// authors and should not be interpreted as representing official policies, either expressed
// or implied, of the authors.

extern crate oracle;

fn main() {
    let username = "sys";
    let password = "change_on_install";
    let database = "";
    let auth_mode = oracle::AuthMode::SYSDBA;

    // connect as sysdba or sysoper with prelim_auth mode
    let mut connector = oracle::Connector::new(username, password, database);
    connector.auth_mode(auth_mode);
    connector.prelim_auth(true);
    let conn = connector.connect().unwrap();

    // start up database. The database is not mounted at this time.
    conn.startup_database(&[]).unwrap();
    conn.close().unwrap();

    // connect as sysdba or sysoper **without** prelim_auth mode
    connector.prelim_auth(false);
    let conn = connector.connect().unwrap();

    // mount and open the database
    conn.execute("alter database mount", &()).unwrap();
    println!("Database mounted.");
    conn.execute("alter database open", &()).unwrap();
    println!("Database opened.");
    conn.close().unwrap();
}
