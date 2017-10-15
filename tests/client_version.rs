extern crate oracle;
mod common;

#[test]
fn client_version() {
    let ver = match oracle::client_version() {
        Ok(ver) => ver,
        Err(err) => panic!("Failed to get client version: {}", err),
    };
    let conn = common::connect().unwrap();
    let mut stmt = conn.prepare("SELECT client_version FROM v$session_connect_info WHERE sid = SYS_CONTEXT('USERENV', 'SID')").unwrap();
    stmt.execute().unwrap();
    let row = stmt.fetch().unwrap();
    let ver_from_query: String = row.get(0).unwrap();
    assert_eq!(ver.to_string(), ver_from_query);
}
