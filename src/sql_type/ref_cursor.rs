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
use crate::binding::*;
use crate::chkerr;
use crate::connection::Conn;
use crate::sql_type::FromSql;
use crate::sql_type::OracleType;
use crate::sql_type::ToSql;
use crate::sql_type::ToSqlNull;
use crate::statement::QueryParams;
use crate::statement::Stmt;
use crate::Connection;
use crate::Error;
use crate::Result;
use crate::ResultSet;
use crate::Row;
use crate::RowValue;
use crate::SqlValue;

pub struct RefCursor {
    stmt: Stmt,
}

impl RefCursor {
    pub(crate) fn from_raw(
        conn: Conn,
        handle: *mut dpiStmt,
        query_params: QueryParams,
    ) -> Result<RefCursor> {
        let mut fetch_array_size = 0;
        chkerr!(
            conn.ctxt,
            dpiStmt_getFetchArraySize(handle, &mut fetch_array_size)
        );
        if fetch_array_size != query_params.fetch_array_size {
            return Err(Error::InternalError(format!(
                "invalid RefCursor fetch_array_size.  {} != {}",
                fetch_array_size, query_params.fetch_array_size
            )));
        }
        let mut num_query_columns = 0;
        chkerr!(
            conn.ctxt,
            dpiStmt_getNumQueryColumns(handle, &mut num_query_columns)
        );
        chkerr!(conn.ctxt, dpiStmt_addRef(handle));
        let mut stmt = Stmt::new(conn, handle, query_params);
        stmt.init_row(num_query_columns as usize)?;
        Ok(RefCursor { stmt: stmt })
    }

    pub fn query(&mut self) -> Result<ResultSet<Row>> {
        Ok(ResultSet::<Row>::new(&self.stmt))
    }

    pub fn query_as<'a, T>(&'a mut self) -> Result<ResultSet<'a, T>>
    where
        T: RowValue,
    {
        Ok(ResultSet::<T>::new(&self.stmt))
    }

    pub fn query_row(&mut self) -> Result<Row> {
        self.query()?.next().unwrap_or(Err(Error::NoDataFound))
    }

    pub fn query_row_as<T>(&mut self) -> Result<T>
    where
        T: RowValue,
    {
        self.query_as()?.next().unwrap_or(Err(Error::NoDataFound))
    }
}

impl ToSqlNull for RefCursor {
    fn oratype_for_null(_conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::RefCursor)
    }
}

impl ToSql for RefCursor {
    fn oratype(&self, _conn: &Connection) -> Result<OracleType> {
        Ok(OracleType::RefCursor)
    }

    fn to_sql(&self, _val: &mut SqlValue) -> Result<()> {
        Err(Error::InvalidOperation(
            "Cannot bind RefCursor as an IN parameter".into(),
        ))
    }
}

impl FromSql for RefCursor {
    fn from_sql(val: &SqlValue) -> Result<Self> {
        val.to_ref_cursor()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn out_ref_cursor() -> Result<()> {
        let conn = test_util::connect()?;
        let sql = "begin pkg_TestOutCursors.TestOutCursor(:1, :2); end;";
        let mut stmt = conn.statement(sql).build()?;
        stmt.execute(&[&1, &None::<RefCursor>])?;
        let mut ref_cursor: RefCursor = stmt.bind_value(2)?;
        let row = ref_cursor.query_row_as::<(i32, String)>()?;
        assert_eq!(row.0, 1);
        assert_eq!(row.1, "String 1");
        Ok(())
    }
}
