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
use crate::DpiStmt;
use crate::Error;
use crate::Result;
use crate::ResultSet;
use crate::Row;
use crate::RowValue;
use crate::SqlValue;
#[cfg(doc)]
use crate::Statement;
#[cfg(doc)]
use crate::StatementBuilder;

/// Result set output by or returned by a PL/SQL block or a stored procedure
///
/// This struct has four query methods, which are similar to [`Statement`]'s query methods
/// excluding `params` arguments. The latter methods internally execute statements with
/// specified `params` and then get query results. On the other hand, the former
/// get query results executed in a PL/SQL block or a stored procedure. So there are no
/// `params` arguments in this struct.
///
/// When settings about queries are set to [`StatementBuilder`], they are
/// also applied to ref cursors output by a PL/SQL. It is worth to call
/// [`StatementBuilder::fetch_array_size(1)`][`StatementBuilder::fetch_array_size]
/// in order to reduce memory usage when a ref cursor contains at most one row.
///
/// # Examples
///
/// Ref cursor as an output parameter
/// ```
/// # use oracle::Error;
/// # use oracle::sql_type::RefCursor;
/// # use oracle::test_util;
/// # let conn = test_util::connect()?;
/// let sql = r#"
/// begin
///   open :cursor for select IntCol, StringCol from TestStrings order by IntCol;
/// end;
/// "#;
/// let mut stmt = conn.statement(sql).build()?;
/// stmt.execute(&[&None::<RefCursor>])?;
///
/// let mut cursor: RefCursor = stmt.bind_value(1)?;
/// let mut n = 1;
/// for row_result in cursor.query_as::<(i32, String)>()? {
///     let (int_col, string_col) = row_result?;
///     assert_eq!(int_col, n);
///     assert_eq!(string_col, format!("String {}", n));
///     n += 1;
/// }
/// # Ok::<(), Error>(())
/// ```
///
/// Ref cursor returned by a PL/SQL block
/// ```
/// # use oracle::Error;
/// # use oracle::sql_type::RefCursor;
/// # use oracle::test_util::{self, check_version, VER12_1};
/// # let conn = test_util::connect()?;
/// # if !check_version(&conn, &VER12_1, &VER12_1)? {
/// #     return Ok(()); // skip this test
/// # }
/// let sql = r#"
/// declare
///   cursor1 SYS_REFCURSOR;
/// begin
///   open cursor1 for select IntCol, StringCol from TestStrings order by IntCol;
///   dbms_sql.return_result(cursor1);
/// end;
/// "#;
/// let mut stmt = conn.statement(sql).build()?;
/// stmt.execute(&[])?;
///
/// // Get the result set.
/// let mut opt_cursor = stmt.implicit_result()?;
/// assert!(opt_cursor.is_some());
/// let mut cursor = opt_cursor.unwrap();
/// let mut n = 1;
/// for row_result in cursor.query_as::<(i32, String)>()? {
///     let (int_col, string_col) = row_result?;
///     assert_eq!(int_col, n);
///     assert_eq!(string_col, format!("String {}", n));
///     n += 1;
/// }
/// # Ok::<(), Error>(())
/// ```
///
pub struct RefCursor {
    stmt: Stmt,
}

impl RefCursor {
    pub(crate) fn from_handle(
        conn: Conn,
        handle: DpiStmt,
        query_params: QueryParams,
    ) -> Result<RefCursor> {
        chkerr!(
            conn.ctxt(),
            dpiStmt_setFetchArraySize(handle.raw, query_params.fetch_array_size)
        );
        let mut num_query_columns = 0;
        chkerr!(
            conn.ctxt(),
            dpiStmt_getNumQueryColumns(handle.raw, &mut num_query_columns)
        );
        let mut stmt = Stmt::new(conn, handle, query_params, "".into());
        stmt.init_row(num_query_columns as usize)?;
        Ok(RefCursor { stmt })
    }

    /// Gets rows as an iterator of [`Row`]s.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::sql_type::RefCursor;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// let sql = r#"
    /// begin
    ///   open :cursor for select IntCol, StringCol from TestStrings order by IntCol;
    /// end;
    /// "#;
    /// let mut stmt = conn.statement(sql).build()?;
    /// stmt.execute(&[&None::<RefCursor>])?;
    ///
    /// let mut cursor: RefCursor = stmt.bind_value(1)?;
    /// let mut n = 1;
    /// for row_result in cursor.query()? {
    ///     let row = row_result?;
    ///     let int_col: i32 = row.get(0)?;
    ///     let string_col: String = row.get(1)?;
    ///     assert_eq!(int_col, n);
    ///     assert_eq!(string_col, format!("String {}", n));
    ///     n += 1;
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn query(&mut self) -> Result<ResultSet<Row>> {
        Ok(ResultSet::<Row>::new(&mut self.stmt))
    }

    /// Gets rows as an itertor of the specified type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::sql_type::RefCursor;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// let sql = r#"
    /// begin
    ///   open :cursor for select IntCol, StringCol from TestStrings order by IntCol;
    /// end;
    /// "#;
    /// let mut stmt = conn.statement(sql).build()?;
    /// stmt.execute(&[&None::<RefCursor>])?;
    ///
    /// let mut cursor: RefCursor = stmt.bind_value(1)?;
    /// let mut n = 1;
    /// for row_result in cursor.query_as::<(i32, String)>()? {
    ///     let (int_col, string_col) = row_result?;
    ///     assert_eq!(int_col, n);
    ///     assert_eq!(string_col, format!("String {}", n));
    ///     n += 1;
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn query_as<T>(&mut self) -> Result<ResultSet<T>>
    where
        T: RowValue,
    {
        Ok(ResultSet::<T>::new(&mut self.stmt))
    }

    /// Gets one row as [`Row`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::sql_type::RefCursor;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// let sql = r#"
    /// begin
    ///   open :cursor for select StringCol from TestStrings where IntCol = :IntCol;
    /// end;
    /// "#;
    /// let mut stmt = conn.statement(sql).fetch_array_size(1).build()?;
    /// stmt.execute(&[&None::<RefCursor>, &1])?;
    ///
    /// let mut cursor: RefCursor = stmt.bind_value(1)?;
    /// let string_col: String = cursor.query_row()?.get(0)?;
    /// assert_eq!(string_col, "String 1");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn query_row(&mut self) -> Result<Row> {
        self.query()?.next().unwrap_or(Err(Error::no_data_found()))
    }

    /// Gets one row as the specified type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::sql_type::RefCursor;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// let sql = r#"
    /// begin
    ///   open :cursor for select StringCol from TestStrings where IntCol = :IntCol;
    /// end;
    /// "#;
    /// let mut stmt = conn.statement(sql).fetch_array_size(1).build()?;
    /// stmt.execute(&[&None::<RefCursor>, &1])?;
    ///
    /// let mut cursor: RefCursor = stmt.bind_value(1)?;
    /// assert_eq!(cursor.query_row_as::<String>()?, "String 1");
    /// # Ok::<(), Error>(())
    /// ```
    pub fn query_row_as<T>(&mut self) -> Result<T>
    where
        T: RowValue,
    {
        self.query_as()?
            .next()
            .unwrap_or(Err(Error::no_data_found()))
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
        Err(Error::invalid_operation(
            "cannot bind RefCursor as an IN parameter",
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
    use crate::statement::LobBindType;
    use crate::test_util;

    fn params_from_ref_cursor(cursor: &RefCursor) -> (LobBindType, QueryParams) {
        let sql_value = &cursor.stmt.row.as_ref().unwrap().column_values[0];
        (sql_value.lob_bind_type, sql_value.query_params.clone())
    }

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

        let params = params_from_ref_cursor(&ref_cursor);
        assert_eq!(params.0, LobBindType::Bytes);
        assert_eq!(params.1.fetch_array_size, DPI_DEFAULT_FETCH_ARRAY_SIZE);
        assert_eq!(params.1.prefetch_rows, None);
        assert_eq!(params.1.lob_bind_type, LobBindType::Bytes);
        Ok(())
    }

    #[test]
    fn out_ref_cursor_with_statement_builder_options() -> Result<()> {
        let conn = test_util::connect()?;
        let sql = "begin pkg_TestOutCursors.TestOutCursor(:1, :2); end;";
        let mut stmt = conn
            .statement(sql)
            .fetch_array_size(2)
            .prefetch_rows(3)
            .lob_locator()
            .build()?;
        stmt.execute(&[&1, &None::<RefCursor>])?;
        let mut ref_cursor: RefCursor = stmt.bind_value(2)?;
        let row = ref_cursor.query_row_as::<(i32, String)>()?;
        assert_eq!(row.0, 1);
        assert_eq!(row.1, "String 1");

        // Options specified by StatementBuilder propagates to ref cursors.
        let params = params_from_ref_cursor(&ref_cursor);
        assert_eq!(params.0, LobBindType::Locator);
        assert_eq!(params.1.fetch_array_size, 2);
        assert_eq!(params.1.prefetch_rows, Some(3));
        assert_eq!(params.1.lob_bind_type, LobBindType::Locator);
        Ok(())
    }
}
