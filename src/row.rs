// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2018 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use std::fmt;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::sql_type::FromSql;
use crate::statement::Stmt;
use crate::AssertSend;
use crate::ColumnIndex;
use crate::ColumnInfo;
#[cfg(doc)]
use crate::Connection;
use crate::Result;
use crate::SqlValue;
#[cfg(doc)]
use crate::Statement;

/// Row in a result set of a select statement
pub struct Row {
    pub(crate) column_info: Arc<Vec<ColumnInfo>>,
    pub(crate) column_values: Vec<SqlValue<'static>>,
}

impl Row {
    pub(crate) fn new(
        column_info: Vec<ColumnInfo>,
        column_values: Vec<SqlValue<'static>>,
    ) -> Result<Row> {
        Ok(Row {
            column_info: Arc::new(column_info),
            column_values,
        })
    }

    /// Gets the column value at the specified index.
    pub fn get<I, T>(&self, colidx: I) -> Result<T>
    where
        I: ColumnIndex,
        T: FromSql,
    {
        let pos = colidx.idx(&self.column_info)?;
        self.column_values[pos].get()
    }

    /// Returns column values as a vector of SqlValue
    pub fn sql_values(&self) -> &[SqlValue] {
        &self.column_values
    }

    /// Gets column values as specified type.
    ///
    /// Type inference for the return type doesn't work. You need to specify
    /// it explicitly such as `row.get_as::<(i32, String)>()`.
    /// See [`RowValue`] for available return types.
    ///
    /// ```no_run
    /// # use oracle::*;
    /// let conn = Connection::connect("scott", "tiger", "")?;
    /// let mut stmt = conn.statement("select empno, ename from emp").build()?;
    ///
    /// for result in stmt.query(&[])? {
    ///     let row = result?;
    ///     // Gets a row as `(i32, String)`.
    ///     let (empno, ename) = row.get_as::<(i32, String)>()?;
    ///     println!("{},{}", empno, ename);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    pub fn get_as<T>(&self) -> Result<T>
    where
        T: RowValue,
    {
        <T>::get(self)
    }

    pub fn column_info(&self) -> &[ColumnInfo] {
        &self.column_info
    }
}

impl AssertSend for Row {}

impl fmt::Debug for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Row {{ ")?;
        for (info, value) in self.column_info.iter().zip(&self.column_values) {
            write!(f, "{}: {:?} ", info.name(), value)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug)]
enum StmtHolder<'a> {
    Borrowed(&'a mut Stmt),
    Owned(Stmt),
}

/// Result set
///
/// # Remarks
///
/// The lifetime parameter `'a` is `'static` when this type is created by the following methods.
///
/// * [`Connection::query()`]
/// * [`Connection::query_named()`]
/// * [`Connection::query_as()`]
/// * [`Connection::query_as_named()`]
/// * [`Statement::into_result_set()`]
/// * [`Statement::into_result_set_named()`]
///
/// On the other hand, `'a` refers to [`Statement`] when it is created by the following methods.
///
/// * [`Statement::query()`]
/// * [`Statement::query_named()`]
/// * [`Statement::query_as()`]
/// * [`Statement::query_as_named()`]
///
#[derive(Debug)]
pub struct ResultSet<'a, T>
where
    T: RowValue,
{
    stmt: StmtHolder<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> ResultSet<'a, T>
where
    T: RowValue,
{
    pub(crate) fn new(stmt: &'a mut Stmt) -> ResultSet<'a, T> {
        ResultSet {
            stmt: StmtHolder::Borrowed(stmt),
            phantom: PhantomData,
        }
    }

    pub(crate) fn from_stmt(stmt: Stmt) -> ResultSet<'a, T> {
        ResultSet {
            stmt: StmtHolder::Owned(stmt),
            phantom: PhantomData,
        }
    }

    fn stmt(&self) -> &Stmt {
        match self.stmt {
            StmtHolder::Borrowed(ref stmt) => stmt,
            StmtHolder::Owned(ref stmt) => stmt,
        }
    }

    fn stmt_mut(&mut self) -> &mut Stmt {
        match self.stmt {
            StmtHolder::Borrowed(ref mut stmt) => stmt,
            StmtHolder::Owned(ref mut stmt) => stmt,
        }
    }

    pub fn column_info(&self) -> &[ColumnInfo] {
        &self.stmt().row.as_ref().unwrap().column_info
    }
}

unsafe impl<T> Send for ResultSet<'static, T> where T: RowValue {}

impl<'stmt, T> Iterator for ResultSet<'stmt, T>
where
    T: RowValue,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stmt_mut()
            .next()
            .map(|row_result| row_result.and_then(|row| row.get_as::<T>()))
    }
}

impl<'stmt, T> FusedIterator for ResultSet<'stmt, T> where T: RowValue {}

/// A trait to get a row as specified type
///
/// This is the return type of [`Connection::query_row_as`],
/// [`Connection::query_row_as_named`] and [`Row::get_as`].
///
/// The trait was added to fetch column values as a tuple.
/// The oracle crate provides implementations for a type
/// implementing [`FromSql`] and tuples of types implementing
/// [`FromSql`]. The number of elements in a tuple should
///  be 1 through 50.
///
/// ```no_run
/// # use oracle::*;
/// let conn = Connection::connect("scott", "tiger", "")?;
///
/// let sql = "select * from emp where empno = :1";
///
/// // Gets the first column value in a row.
/// // Values after the second column are ignored.
/// let empno = conn.query_row_as::<u32>(sql, &[&7369])?;
///
/// // Gets the first two column values in a row.
/// // Values after the third column are ignored.
/// let tuple_of_empno_and_ename = conn.query_row_as::<(i32, String)>(sql, &[&7499])?;
/// # Ok::<(), Error>(())
/// ```
///
/// You can implement the trait for your own types. For example
/// when you have a struct whose members are `empno` and `ename`,
/// you can make the struct from `empno` and `ename` column values
/// as follows:
///
/// ```no_run
/// # use oracle::{Connection, Error, Result, Row, RowValue};
/// struct Emp {
///     empno: i32,
///     ename: String,
/// }
///
/// impl RowValue for Emp {
///     fn get(row: &Row) -> std::result::Result<Emp, Error> {
///         Ok(Emp {
///             empno: row.get("empno")?,
///             ename: row.get("ename")?,
///         })
///     }
/// }
///
/// let conn = Connection::connect("scott", "tiger", "")?;
/// let mut stmt = conn.statement("select * from emp").build()?;
///
/// // Gets rows as Emp
/// for result in stmt.query_as::<Emp>(&[])? {
///     let emp = result?;
///     println!("{},{}", emp.empno, emp.ename);
/// }
/// # Ok::<(), Error>(())
/// ```
pub trait RowValue: Sized {
    fn get(row: &Row) -> Result<Self>;
}

impl RowValue for Row {
    fn get(row: &Row) -> Result<Row> {
        let num_cols = row.column_values.len();
        let mut column_values = Vec::with_capacity(num_cols);
        for val in &row.column_values {
            column_values.push(val.clone_except_fetch_array_buffer()?);
        }
        Ok(Row {
            column_info: row.column_info.clone(),
            column_values,
        })
    }
}

impl<T: FromSql> RowValue for T {
    fn get(row: &Row) -> Result<T> {
        row.get::<usize, T>(0)
    }
}

macro_rules! impl_row_value_for_tuple {
    ($(
        [$(($idx:tt, $T:ident))+],
    )+) => {
        $(
            impl<$($T:FromSql,)+> RowValue for ($($T,)+) {
                fn get(row: &Row) -> Result<($($T,)+)> {
                    Ok((
                        $(row.get::<usize, $T>($idx)?,)+
                    ))
                }
            }
        )+
    }
}

impl_row_value_for_tuple! {
    [(0,T0)],
    [(0,T0)(1,T1)],
    [(0,T0)(1,T1)(2,T2)],
    [(0,T0)(1,T1)(2,T2)(3,T3)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)(47,T47)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)(47,T47)(48,T48)],
    [(0,T0)(1,T1)(2,T2)(3,T3)(4,T4)(5,T5)(6,T6)(7,T7)(8,T8)(9,T9)
     (10,T10)(11,T11)(12,T12)(13,T13)(14,T14)(15,T15)(16,T16)(17,T17)(18,T18)(19,T19)
     (20,T20)(21,T21)(22,T22)(23,T23)(24,T24)(25,T25)(26,T26)(27,T27)(28,T28)(29,T29)
     (30,T30)(31,T31)(32,T32)(33,T33)(34,T34)(35,T35)(36,T36)(37,T37)(38,T38)(39,T39)
     (40,T40)(41,T41)(42,T42)(43,T43)(44,T44)(45,T45)(46,T46)(47,T47)(48,T48)(49,T49)],
}
