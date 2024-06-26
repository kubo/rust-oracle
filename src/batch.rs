// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2021 Kubo Takehiro <kubo@jiubao.org>
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

use crate::binding::*;
use crate::chkerr;
use crate::error::DPI_ERR_BUFFER_SIZE_TOO_SMALL;
use crate::private;
use crate::sql_type::OracleType;
use crate::sql_type::ToSql;
use crate::sql_value::BufferRowIndex;
use crate::statement::QueryParams;
use crate::to_rust_str;
use crate::Connection;
use crate::DbError;
use crate::Error;
use crate::OdpiStr;
use crate::Result;
use crate::SqlValue;
#[cfg(doc)]
use crate::Statement;
use crate::StatementType;
use std::convert::TryFrom;
use std::fmt;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

#[cfg(test)]
const MINIMUM_TYPE_LENGTH: u32 = 1;
#[cfg(not(test))]
const MINIMUM_TYPE_LENGTH: u32 = 64;

// round up to the nearest power of two
fn po2(mut size: u32) -> u32 {
    if size < MINIMUM_TYPE_LENGTH {
        size = MINIMUM_TYPE_LENGTH;
    }
    1u32 << (32 - (size - 1).leading_zeros())
}

fn oratype_size(oratype: &OracleType) -> Option<u32> {
    match oratype {
        &OracleType::Varchar2(size)
        | &OracleType::NVarchar2(size)
        | &OracleType::Char(size)
        | &OracleType::NChar(size)
        | &OracleType::Raw(size) => Some(size),
        _ => None,
    }
}

#[derive(Clone)]
struct BindType {
    oratype: Option<OracleType>,
}

impl BindType {
    fn new(oratype: &OracleType) -> BindType {
        BindType {
            oratype: match oratype {
                OracleType::Varchar2(size) => Some(OracleType::Varchar2(po2(*size))),
                OracleType::NVarchar2(size) => Some(OracleType::NVarchar2(po2(*size))),
                OracleType::Char(size) => Some(OracleType::Char(po2(*size))),
                OracleType::NChar(size) => Some(OracleType::NChar(po2(*size))),
                OracleType::Raw(size) => Some(OracleType::Raw(po2(*size))),
                _ => None,
            },
        }
    }

    fn reset_size(&mut self, new_size: u32) {
        self.oratype = match self.oratype {
            Some(OracleType::Varchar2(_)) => Some(OracleType::Varchar2(po2(new_size))),
            Some(OracleType::NVarchar2(_)) => Some(OracleType::NVarchar2(po2(new_size))),
            Some(OracleType::Char(_)) => Some(OracleType::Char(po2(new_size))),
            Some(OracleType::NChar(_)) => Some(OracleType::NChar(po2(new_size))),
            Some(OracleType::Raw(_)) => Some(OracleType::Raw(po2(new_size))),
            _ => None,
        };
    }

    fn as_oratype(&self) -> Option<&OracleType> {
        self.oratype.as_ref()
    }
}

/// A builder to create a [`Batch`] with various configuration
pub struct BatchBuilder<'conn, 'sql> {
    conn: &'conn Connection,
    sql: &'sql str,
    batch_size: usize,
    with_batch_errors: bool,
    with_row_counts: bool,
    query_params: QueryParams,
}

impl<'conn, 'sql> BatchBuilder<'conn, 'sql> {
    pub(crate) fn new(
        conn: &'conn Connection,
        sql: &'sql str,
        batch_size: usize,
    ) -> BatchBuilder<'conn, 'sql> {
        BatchBuilder {
            conn,
            sql,
            batch_size,
            with_batch_errors: false,
            with_row_counts: false,
            query_params: QueryParams::new(),
        }
    }

    /// See ["Error Handling"](Batch#error-handling)
    pub fn with_batch_errors(&mut self) -> &mut BatchBuilder<'conn, 'sql> {
        self.with_batch_errors = true;
        self
    }

    /// See ["Affected Rows"](Batch#affected-rows)
    pub fn with_row_counts(&mut self) -> &mut BatchBuilder<'conn, 'sql> {
        self.with_row_counts = true;
        self
    }

    pub fn build(&self) -> Result<Batch<'conn>> {
        let batch_size = u32::try_from(self.batch_size).map_err(|err| {
            Error::out_of_range(format!("too large batch size {}", self.batch_size)).add_source(err)
        })?;
        let conn = self.conn;
        let sql = OdpiStr::new(self.sql);
        let mut handle: *mut dpiStmt = ptr::null_mut();
        chkerr!(
            conn.ctxt(),
            dpiConn_prepareStmt(
                conn.handle(),
                0,
                sql.ptr,
                sql.len,
                ptr::null(),
                0,
                &mut handle
            )
        );
        let mut info = MaybeUninit::uninit();
        chkerr!(
            conn.ctxt(),
            dpiStmt_getInfo(handle, info.as_mut_ptr()),
            unsafe {
                dpiStmt_release(handle);
            }
        );
        let info = unsafe { info.assume_init() };
        if info.isDML == 0 && info.isPLSQL == 0 {
            unsafe {
                dpiStmt_release(handle);
            }
            let msg = format!(
                "could not use {} statement",
                StatementType::from_enum(info.statementType)
            );
            return Err(Error::invalid_operation(msg));
        };
        let mut num = 0;
        chkerr!(
            conn.ctxt(),
            dpiStmt_getBindCount(handle, &mut num),
            unsafe {
                dpiStmt_release(handle);
            }
        );
        let bind_count = num as usize;
        let mut bind_names = Vec::with_capacity(bind_count);
        let mut bind_values = Vec::with_capacity(bind_count);
        if bind_count > 0 {
            let mut names: Vec<*const c_char> = vec![ptr::null_mut(); bind_count];
            let mut lengths = vec![0; bind_count];
            chkerr!(
                conn.ctxt(),
                dpiStmt_getBindNames(handle, &mut num, names.as_mut_ptr(), lengths.as_mut_ptr()),
                unsafe {
                    dpiStmt_release(handle);
                }
            );
            bind_names = Vec::with_capacity(num as usize);
            for i in 0..(num as usize) {
                bind_names.push(to_rust_str(names[i], lengths[i]));
                bind_values.push(SqlValue::for_bind(
                    conn.conn.clone(),
                    self.query_params.clone(),
                    batch_size,
                ));
            }
        };
        Ok(Batch {
            conn,
            handle,
            statement_type: StatementType::from_enum(info.statementType),
            bind_count,
            bind_names,
            bind_values,
            bind_types: vec![None; bind_count],
            batch_index: 0,
            batch_size,
            with_batch_errors: self.with_batch_errors,
            with_row_counts: self.with_row_counts,
            query_params: self.query_params.clone(),
        })
    }
}

/// Statement batch, which inserts, updates or deletes more than one row at once
///
/// Batching is efficient when the network distance between the client and
/// the server is long. When a network round trip requires 1ms, inserting
/// 10k rows using [`Statement`] consumes at least 10s excluding time spent
/// in the client and the server. If 1000 rows are sent in a batch, it
/// decreases to 10ms.
///
/// # Usage
///
/// 1. [`conn.batch(sql_stmt, batch_size).build()`](Connection::batch) to create [`Batch`].
/// 2. [`append_row()`](#method.append_row) for each row. Rows in the batch are sent to
///    the server when the number of appended rows reaches the batch size.  
///    **Note:** The "batch errors" option mentioned later changes this behavior.
/// 3. [`execute()`](#method.execute) in the end to send rows which
///    have not been sent by `append_row()`.
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util;
/// # let conn = test_util::connect()?;
/// # conn.execute("delete from TestTempTable", &[])?;
/// let sql_stmt = "insert into TestTempTable values(:1, :2)";
/// let batch_size = 100;
/// let mut batch = conn.batch(sql_stmt, batch_size).build()?;
/// for i in 0..1234 { // iterate 1234 times.
///     // send rows internally every 100 iterations.
///     batch.append_row(&[&i, &format!("value {}", i)])?;
/// }
/// batch.execute()?; // send the rest 34 rows.
/// // Check the number of inserted rows.
/// assert_eq!(conn.query_row_as::<i32>("select count(*) from TestTempTable", &[])?, 1234);
/// # Ok::<(), Error>(())
/// ```
///
/// # Error Handling
///
/// There are two modes when invalid data are in a batch.
///
/// 1. Stop executions at the first failure and return the error information.
/// 2. Execute all rows in the batch and return an array of the error information.
///
/// ## Default Error Handling
///
/// `append_row()` and `execute()` stop executions at the first failure and return
/// the error information. There are no ways to know which row fails.
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util;
/// # let conn = test_util::connect()?;
/// # conn.execute("delete from TestTempTable", &[])?;
/// let sql_stmt = "insert into TestTempTable values(:1, :2)";
/// let batch_size = 10;
/// let mut batch = conn.batch(sql_stmt, batch_size).build()?;
/// batch.append_row(&[&1, &"first row"])?;
/// batch.append_row(&[&2, &"second row"])?;
/// batch.append_row(&[&1, &"first row again"])?; // -> ORA-00001: unique constraint violated.
/// batch.append_row(&[&3, &"third row ".repeat(11)])?; // -> ORA-12899: value too large for column
/// batch.append_row(&[&4, &"fourth row"])?;
/// let result = batch.execute();
/// match result {
///     Err(Error::OciError(dberr)) => {
///         assert_eq!(dberr.code(), 1);
///         assert!(dberr.message().starts_with("ORA-00001: "));
///     }
///     _ => panic!("Unexpected batch result: {:?}", result),
/// }
///
/// // Check the inserted rows.
/// let mut stmt = conn
///     .statement("select count(*) from TestTempTable where intCol = :1")
///     .build()?;
/// assert_eq!(stmt.query_row_as::<i32>(&[&1])?, 1);
/// assert_eq!(stmt.query_row_as::<i32>(&[&2])?, 1);
/// assert_eq!(stmt.query_row_as::<i32>(&[&3])?, 0);
/// assert_eq!(stmt.query_row_as::<i32>(&[&4])?, 0);
/// # Ok::<(), Error>(())
/// ```
///
/// ## Error Handling with batch errors
///
/// **Note:** This feature is available only when both the client and the server are Oracle 12.1 or upper.
///
/// [`BatchBuilder::with_batch_errors`] changes
/// the behavior of `Batch` as follows:
/// * `execute()` executes all rows in the batch and return an array of the error information
///   with row positions in the batch when the errors are caused by invalid data.
/// * `append_row()` doesn't send rows internally when the number of appended rows reaches
///   the batch size. It returns an error when the number exceeds the size instead.
///
/// ```
/// # use oracle::Error;
/// # use oracle::test_util::{self, check_version, VER12_1};
/// # let conn = test_util::connect()?;
/// # if !check_version(&conn, &VER12_1, &VER12_1)? {
/// #     return Ok(()); // skip this test
/// # }
/// # conn.execute("delete from TestTempTable", &[])?;
/// let sql_stmt = "insert into TestTempTable values(:1, :2)";
/// let batch_size = 10;
/// let mut batch = conn.batch(sql_stmt, batch_size).with_batch_errors().build()?;
/// batch.append_row(&[&1, &"first row"])?;
/// batch.append_row(&[&2, &"second row"])?;
/// batch.append_row(&[&1, &"first row again"])?; // -> ORA-00001: unique constraint violated.
/// batch.append_row(&[&3, &"third row ".repeat(11)])?; // -> ORA-12899: value too large for column
/// batch.append_row(&[&4, &"fourth row"])?;
/// let result = batch.execute();
/// match result {
///     Err(Error::BatchErrors(mut errs)) => {
///         // sort by position because errs may not preserve order.
///         errs.sort_by(|a, b| a.offset().cmp(&b.offset()));
///         assert_eq!(errs.len(), 2);
///         assert_eq!(errs[0].code(), 1);
///         assert_eq!(errs[1].code(), 12899);
///         assert_eq!(errs[0].offset(), 2); // position of `[&1, &"first row again"]`
///         assert_eq!(errs[1].offset(), 3); // position of `[&3, &"third row ".repeat(11)]`
///         assert!(errs[0].message().starts_with("ORA-00001: "));
///         assert!(errs[1].message().starts_with("ORA-12899: "));
///     }
///     _ => panic!("Unexpected batch result: {:?}", result),
/// }
///
/// // Check the inserted rows.
/// let mut stmt = conn
///     .statement("select count(*) from TestTempTable where intCol = :1")
///     .build()?;
/// assert_eq!(stmt.query_row_as::<i32>(&[&1])?, 1);
/// assert_eq!(stmt.query_row_as::<i32>(&[&2])?, 1);
/// assert_eq!(stmt.query_row_as::<i32>(&[&3])?, 0); // value too large for column
/// assert_eq!(stmt.query_row_as::<i32>(&[&4])?, 1);
/// # Ok::<(), Error>(())
/// ```
///
/// # Affected Rows
///
/// **Note:** This feature is available only when both the client and the server are Oracle 12.1 or upper.
///
/// Use [`BatchBuilder::with_row_counts`] and [`Batch::row_counts`] to get affected rows
/// for each input row.
///
/// ```
/// # use oracle::Error;
/// # use oracle::sql_type::OracleType;
/// # use oracle::test_util::{self, check_version, VER12_1};
/// # let conn = test_util::connect()?;
/// # if !check_version(&conn, &VER12_1, &VER12_1)? {
/// #     return Ok(()); // skip this test
/// # }
/// # conn.execute("delete from TestTempTable", &[])?;
/// # let sql_stmt = "insert into TestTempTable values(:1, :2)";
/// # let batch_size = 10;
/// # let mut batch = conn.batch(sql_stmt, batch_size).build()?;
/// # batch.set_type(1, &OracleType::Int64)?;
/// # batch.set_type(2, &OracleType::Varchar2(1))?;
/// # for i in 0..10 {
/// #    batch.append_row(&[&i])?;
/// # }
/// # batch.execute()?;
/// let sql_stmt = "update TestTempTable set stringCol = :stringCol where intCol >= :intCol";
/// let mut batch = conn.batch(sql_stmt, 3).with_row_counts().build()?;
/// batch.append_row_named(&[("stringCol", &"a"), ("intCol", &9)])?; // update 1 row
/// batch.append_row_named(&[("stringCol", &"b"), ("intCol", &7)])?; // update 3 rows
/// batch.append_row_named(&[("stringCol", &"c"), ("intCol", &5)])?; // update 5 rows
/// batch.execute()?;
/// assert_eq!(batch.row_counts()?, &[1, 3, 5]);
/// # Ok::<(), Error>(())
/// ```
///
/// # Bind Parameter Types
///
/// Parameter types are decided by the value of [`Batch::append_row`], [`Batch::append_row_named`]
/// or [`Batch::set`]; or by the type specified by [`Batch::set_type`]. Once the
/// type is determined, there are no ways to change it except the following case.
///
/// For user's convenience, when the length of character data types is too short,
/// the length is extended automatically. For example:
/// ```no_run
/// # use oracle::Error;
/// # use oracle::sql_type::OracleType;
/// # use oracle::test_util;
/// # let conn = test_util::connect()?;
/// # let sql_stmt = "dummy";
/// # let batch_size = 10;
/// let mut batch = conn.batch(sql_stmt, batch_size).build()?;
/// batch.append_row(&[&"first row"])?; // allocate 64 bytes for each row
/// batch.append_row(&[&"second row"])?;
/// //....
/// // The following line extends the internal buffer length for each row.
/// batch.append_row(&[&"assume that data length is over 64 bytes"])?;
/// # Ok::<(), Error>(())
/// ```
/// Note that extending the internal buffer needs memory copy from existing buffer
/// to newly allocated buffer. If you know the maximum data length, it is better
/// to set the size by [`Batch::set_type`].
pub struct Batch<'conn> {
    pub(crate) conn: &'conn Connection,
    handle: *mut dpiStmt,
    statement_type: StatementType,
    bind_count: usize,
    bind_names: Vec<String>,
    bind_values: Vec<SqlValue<'conn>>,
    bind_types: Vec<Option<BindType>>,
    batch_index: u32,
    batch_size: u32,
    with_batch_errors: bool,
    with_row_counts: bool,
    query_params: QueryParams,
}

impl<'conn> Batch<'conn> {
    /// Closes the batch before the end of its lifetime.
    pub fn close(&mut self) -> Result<()> {
        chkerr!(self.conn.ctxt(), dpiStmt_close(self.handle, ptr::null(), 0));
        Ok(())
    }

    pub fn append_row(&mut self, params: &[&dyn ToSql]) -> Result<()> {
        self.check_batch_index()?;
        for (i, param) in params.iter().enumerate() {
            self.bind_internal(i + 1, *param)?;
        }
        self.append_row_common()
    }

    pub fn append_row_named(&mut self, params: &[(&str, &dyn ToSql)]) -> Result<()> {
        self.check_batch_index()?;
        for param in params {
            self.bind_internal(param.0, param.1)?;
        }
        self.append_row_common()
    }

    fn append_row_common(&mut self) -> Result<()> {
        if self.with_batch_errors {
            self.set_batch_index(self.batch_index + 1);
        } else {
            self.set_batch_index(self.batch_index + 1);
            if self.batch_index == self.batch_size {
                self.execute()?;
            }
        }
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let result = self.execute_sub();
        // reset all values to null regardless of the result
        let num_rows = self.batch_index;
        self.batch_index = 0;
        for bind_value in &mut self.bind_values {
            for i in 0..num_rows {
                bind_value.buffer_row_index = BufferRowIndex::Owned(i);
                bind_value.set_null()?;
            }
            bind_value.buffer_row_index = BufferRowIndex::Owned(0);
        }
        result
    }

    fn execute_sub(&mut self) -> Result<()> {
        if self.batch_index == 0 {
            return Ok(());
        }
        let mut exec_mode = DPI_MODE_EXEC_DEFAULT;
        if self.conn.autocommit() {
            exec_mode |= DPI_MODE_EXEC_COMMIT_ON_SUCCESS;
        }
        if self.with_batch_errors {
            exec_mode |= DPI_MODE_EXEC_BATCH_ERRORS;
        }
        if self.with_row_counts {
            exec_mode |= DPI_MODE_EXEC_ARRAY_DML_ROWCOUNTS;
        }
        chkerr!(
            self.conn.ctxt(),
            dpiStmt_executeMany(self.handle, exec_mode, self.batch_index)
        );
        self.conn.ctxt().set_warning();
        if self.with_batch_errors {
            let mut errnum = 0;
            chkerr!(
                self.conn.ctxt(),
                dpiStmt_getBatchErrorCount(self.handle, &mut errnum)
            );
            if errnum != 0 {
                let mut errs = Vec::with_capacity(errnum as usize);
                chkerr!(
                    self.conn.ctxt(),
                    dpiStmt_getBatchErrors(self.handle, errnum, errs.as_mut_ptr())
                );
                unsafe { errs.set_len(errnum as usize) };
                return Err(Error::make_batch_errors(
                    errs.iter().map(DbError::from_dpi_error).collect(),
                ));
            }
        }
        Ok(())
    }

    /// Returns the number of bind parameters
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// # conn.execute("delete from TestTempTable", &[])?;
    /// let sql_stmt = "insert into TestTempTable values(:intCol, :stringCol)";
    /// let mut batch = conn.batch(sql_stmt, 100).build()?;
    /// assert_eq!(batch.bind_count(), 2);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn bind_count(&self) -> usize {
        self.bind_count
    }

    /// Returns an array of bind parameter names
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// # conn.execute("delete from TestTempTable", &[])?;
    /// let sql_stmt = "insert into TestTempTable values(:intCol, :stringCol)";
    /// let batch = conn.batch(sql_stmt, 100).build()?;
    /// assert_eq!(batch.bind_names(), &["INTCOL", "STRINGCOL"]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn bind_names(&self) -> Vec<&str> {
        self.bind_names.iter().map(|name| name.as_str()).collect()
    }

    fn check_batch_index(&self) -> Result<()> {
        if self.batch_index < self.batch_size {
            Ok(())
        } else {
            Err(Error::out_of_range(format!(
                "over the max batch size {}",
                self.batch_size
            )))
        }
    }

    /// Set the data type of a bind parameter
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util;
    /// # use oracle::sql_type::OracleType;
    /// # let conn = test_util::connect()?;
    /// # conn.execute("delete from TestTempTable", &[])?;
    /// let sql_stmt = "insert into TestTempTable values(:intCol, :stringCol)";
    /// let mut batch = conn.batch(sql_stmt, 100).build()?;
    /// batch.set_type(1, &OracleType::Int64)?;
    /// batch.set_type(2, &OracleType::Varchar2(10))?;
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set_type<I>(&mut self, bindidx: I, oratype: &OracleType) -> Result<()>
    where
        I: BatchBindIndex,
    {
        let pos = bindidx.idx(self)?;
        if self.bind_types[pos].is_some() {
            return Err(Error::invalid_operation(format!(
                "type at {} has set already",
                bindidx
            )));
        }
        self.bind_values[pos].init_handle(oratype)?;
        chkerr!(
            self.conn.ctxt(),
            bindidx.bind(self.handle, self.bind_values[pos].handle()?)
        );
        self.bind_types[pos] = Some(BindType::new(oratype));
        Ok(())
    }

    /// Set a parameter value
    ///
    /// ```
    /// # use oracle::Error;
    /// # use oracle::test_util;
    /// # let conn = test_util::connect()?;
    /// # conn.execute("delete from TestTempTable", &[])?;
    /// let sql_stmt = "insert into TestTempTable values(:intCol, :stringCol)";
    /// let mut batch = conn.batch(sql_stmt, 100).build()?;
    /// // The below three lines are same with `batch.append_row(&[&100, &"hundred"])?`.
    /// batch.set(1, &100)?; // set by position 1
    /// batch.set(2, &"hundred")?; // set at position 2
    /// batch.append_row(&[])?;
    /// // The below three lines are same with `batch.append_row(&[("intCol", &101), ("stringCol", &"hundred one")])?`
    /// batch.set("intCol", &101)?; // set by name "intCol"
    /// batch.set("stringCol", &"hundred one")?; // set by name "stringCol"
    /// batch.append_row(&[])?;
    /// batch.execute()?;
    /// let sql_stmt = "select * from TestTempTable where intCol = :1";
    /// assert_eq!(conn.query_row_as::<(i32, String)>(sql_stmt, &[&100])?, (100, "hundred".to_string()));
    /// assert_eq!(conn.query_row_as::<(i32, String)>(sql_stmt, &[&101])?, (101, "hundred one".to_string()));
    /// # Ok::<(), Error>(())
    /// ```
    pub fn set<I>(&mut self, index: I, value: &dyn ToSql) -> Result<()>
    where
        I: BatchBindIndex,
    {
        self.check_batch_index()?;
        self.bind_internal(index, value)
    }

    fn bind_internal<I>(&mut self, bindidx: I, value: &dyn ToSql) -> Result<()>
    where
        I: BatchBindIndex,
    {
        let pos = bindidx.idx(self)?;
        if self.bind_types[pos].is_none() {
            // When the parameter type has not bee specified yet,
            // assume the type from the value
            let oratype = value.oratype(self.conn)?;
            let bind_type = BindType::new(&oratype);
            self.bind_values[pos].init_handle(bind_type.as_oratype().unwrap_or(&oratype))?;
            chkerr!(
                self.conn.ctxt(),
                bindidx.bind(self.handle, self.bind_values[pos].handle()?)
            );
            self.bind_types[pos] = Some(bind_type);
        }
        match self.bind_values[pos].set(value) {
            Err(err) if err.dpi_code() == Some(DPI_ERR_BUFFER_SIZE_TOO_SMALL) => {
                let bind_type = self.bind_types[pos].as_mut().unwrap();
                if bind_type.as_oratype().is_none() {
                    return Err(err);
                }
                let new_oratype = value.oratype(self.conn)?;
                let new_size = oratype_size(&new_oratype).ok_or(err)?;
                bind_type.reset_size(new_size);
                // allocate new bind handle.
                let mut new_sql_value = SqlValue::for_bind(
                    self.conn.conn.clone(),
                    self.query_params.clone(),
                    self.batch_size,
                );
                new_sql_value.init_handle(bind_type.as_oratype().unwrap())?;
                // copy values in old to new.
                for idx in 0..self.batch_index {
                    chkerr!(
                        self.conn.ctxt(),
                        dpiVar_copyData(
                            new_sql_value.handle()?,
                            idx,
                            self.bind_values[pos].handle()?,
                            idx
                        )
                    );
                }
                new_sql_value.buffer_row_index = BufferRowIndex::Owned(self.batch_index);
                new_sql_value.set(value)?;
                chkerr!(
                    self.conn.ctxt(),
                    bindidx.bind(self.handle, new_sql_value.handle()?)
                );
                self.bind_values[pos] = new_sql_value;
                Ok(())
            }
            x => x,
        }
    }

    fn set_batch_index(&mut self, batch_index: u32) {
        self.batch_index = batch_index;
        for bind_value in &mut self.bind_values {
            bind_value.buffer_row_index = BufferRowIndex::Owned(batch_index);
        }
    }

    /// Returns the number of affected rows
    ///
    /// See ["Affected Rows"](Batch#affected-rows)
    pub fn row_counts(&self) -> Result<Vec<u64>> {
        let mut num_row_counts = 0;
        let mut row_counts = ptr::null_mut();
        chkerr!(
            self.conn.ctxt(),
            dpiStmt_getRowCounts(self.handle, &mut num_row_counts, &mut row_counts)
        );
        Ok(unsafe { slice::from_raw_parts(row_counts, num_row_counts as usize) }.to_vec())
    }

    /// Returns statement type
    pub fn statement_type(&self) -> StatementType {
        self.statement_type
    }

    /// Returns true when the SQL statement is a PL/SQL block.
    pub fn is_plsql(&self) -> bool {
        matches!(
            self.statement_type,
            StatementType::Begin | StatementType::Declare | StatementType::Call
        )
    }

    /// Returns true when the SQL statement is DML (data manipulation language).
    pub fn is_dml(&self) -> bool {
        matches!(
            self.statement_type,
            StatementType::Insert
                | StatementType::Update
                | StatementType::Delete
                | StatementType::Merge
        )
    }
}

impl<'conn> Drop for Batch<'conn> {
    fn drop(&mut self) {
        unsafe { dpiStmt_release(self.handle) };
    }
}

/// A trait implemented by types that can index into bind values of a batch
///
/// This trait is sealed and cannot be implemented for types outside of the `oracle` crate.
pub trait BatchBindIndex: private::Sealed + fmt::Display {
    /// Returns the index of the bind value specified by `self`.
    #[doc(hidden)]
    fn idx(&self, batch: &Batch) -> Result<usize>;
    /// Binds the specified value by using a private method.
    #[doc(hidden)]
    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32;
}

impl BatchBindIndex for usize {
    #[doc(hidden)]
    fn idx(&self, batch: &Batch) -> Result<usize> {
        let num = batch.bind_count();
        if 0 < num && *self <= num {
            Ok(*self - 1)
        } else {
            Err(Error::invalid_bind_index(*self))
        }
    }

    #[doc(hidden)]
    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        dpiStmt_bindByPos(stmt_handle, *self as u32, var_handle)
    }
}

impl BatchBindIndex for &str {
    #[doc(hidden)]
    fn idx(&self, batch: &Batch) -> Result<usize> {
        let bindname = self.to_uppercase();
        batch
            .bind_names()
            .iter()
            .position(|&name| name == bindname)
            .ok_or_else(|| Error::invalid_bind_name(*self))
    }

    #[doc(hidden)]
    unsafe fn bind(&self, stmt_handle: *mut dpiStmt, var_handle: *mut dpiVar) -> i32 {
        let s = OdpiStr::new(self);
        dpiStmt_bindByName(stmt_handle, s.ptr, s.len, var_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;
    use crate::ErrorKind;

    #[derive(Debug)]
    struct TestData {
        int_val: i32,
        string_val: &'static str,
        error_code: Option<i32>,
    }

    impl TestData {
        const fn new(int_val: i32, string_val: &'static str, error_code: Option<i32>) -> TestData {
            TestData {
                int_val,
                string_val,
                error_code,
            }
        }
    }

    // ORA-00001: unique constraint violated
    const ERROR_UNIQUE_INDEX_VIOLATION: Option<i32> = Some(1);

    // ORA-12899: value too large for column
    const ERROR_TOO_LARGE_VALUE: Option<i32> = Some(12899);

    const TEST_DATA: [TestData; 10] = [
        TestData::new(0, "0", None),
        TestData::new(1, "1111", None),
        TestData::new(2, "222222222222", None),
        TestData::new(3, "3333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333", None),
        TestData::new(4, "44444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444", ERROR_TOO_LARGE_VALUE),
        TestData::new(1, "55555555555555", ERROR_UNIQUE_INDEX_VIOLATION),
        TestData::new(6, "66666666666", None),
        TestData::new(2, "7", ERROR_UNIQUE_INDEX_VIOLATION),
        TestData::new(8, "8", None),
        TestData::new(3, "9999999999999999999999999", ERROR_UNIQUE_INDEX_VIOLATION),
    ];

    fn append_rows_then_execute(batch: &mut Batch, rows: &[&TestData]) -> Result<()> {
        for row in rows {
            batch.append_row(&[&row.int_val, &row.string_val])?;
        }
        batch.execute()?;
        Ok(())
    }

    fn check_rows_inserted(conn: &Connection, expected_rows: &[&TestData]) -> Result<()> {
        let mut rows =
            conn.query_as::<(i32, String)>("select * from TestTempTable order by intCol", &[])?;
        let mut expected_rows = expected_rows.to_vec();
        expected_rows.sort_by(|a, b| a.int_val.cmp(&b.int_val));
        for expected_row in expected_rows {
            let row_opt = rows.next();
            assert!(row_opt.is_some());
            let row = row_opt.unwrap()?;
            assert_eq!(row.0, expected_row.int_val);
            assert_eq!(row.1, expected_row.string_val);
        }
        assert!(rows.next().is_none());
        Ok(())
    }

    #[test]
    fn batch_insert() {
        let conn = test_util::connect().unwrap();
        let rows: Vec<&TestData> = TEST_DATA
            .iter()
            .filter(|data| data.error_code.is_none())
            .collect();
        let mut batch = conn
            .batch("insert into TestTempTable values(:1, :2)", rows.len())
            .build()
            .unwrap();
        append_rows_then_execute(&mut batch, &rows).unwrap();
        check_rows_inserted(&conn, &rows).unwrap();
    }

    #[test]
    fn batch_execute_twice() {
        let conn = test_util::connect().unwrap();
        let rows_total: Vec<&TestData> = TEST_DATA
            .iter()
            .filter(|data| data.error_code.is_none())
            .collect();
        let (rows_first, rows_second) = rows_total.split_at(rows_total.len() / 2);
        let mut batch = conn
            .batch("insert into TestTempTable values(:1, :2)", rows_first.len())
            .build()
            .unwrap();
        append_rows_then_execute(&mut batch, rows_first).unwrap();
        append_rows_then_execute(&mut batch, rows_second).unwrap();
        check_rows_inserted(&conn, &rows_total).unwrap();
    }

    #[test]
    fn batch_with_error() {
        let conn = test_util::connect().unwrap();
        let rows: Vec<&TestData> = TEST_DATA.iter().collect();
        let expected_rows: Vec<&TestData> = TEST_DATA
            .iter()
            .take_while(|data| data.error_code.is_none())
            .collect();
        let mut batch = conn
            .batch("insert into TestTempTable values(:1, :2)", rows.len())
            .build()
            .unwrap();
        match append_rows_then_execute(&mut batch, &rows) {
            Err(err) if err.kind() == ErrorKind::OciError => {
                let errcode = TEST_DATA
                    .iter()
                    .find(|data| data.error_code.is_some())
                    .unwrap()
                    .error_code;
                assert_eq!(err.oci_code(), errcode);
            }
            x => {
                panic!("got {:?}", x);
            }
        }
        check_rows_inserted(&conn, &expected_rows).unwrap();
    }

    #[test]
    fn batch_with_batch_errors() {
        let conn = test_util::connect().unwrap();
        let rows: Vec<&TestData> = TEST_DATA.iter().collect();
        let expected_rows: Vec<&TestData> = TEST_DATA
            .iter()
            .filter(|row| row.error_code.is_none())
            .collect();
        let mut batch = conn
            .batch("insert into TestTempTable values(:1, :2)", rows.len())
            .with_batch_errors()
            .build()
            .unwrap();
        match append_rows_then_execute(&mut batch, &rows) {
            Err(err) if err.batch_errors().is_some() => {
                let expected_errors: Vec<(u32, i32)> = TEST_DATA
                    .iter()
                    .enumerate()
                    .filter(|row| row.1.error_code.is_some())
                    .map(|row| (row.0 as u32, row.1.error_code.unwrap()))
                    .collect();
                let actual_errors: Vec<(u32, i32)> = err
                    .batch_errors()
                    .unwrap()
                    .iter()
                    .map(|dberr| (dberr.offset(), dberr.code()))
                    .collect();
                assert_eq!(expected_errors, actual_errors);
            }
            x => {
                panic!("got {:?}", x);
            }
        }
        check_rows_inserted(&conn, &expected_rows).unwrap();
    }
}
