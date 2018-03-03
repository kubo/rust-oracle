// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
// ------------------------------------------------------
//
// Copyright 2018 Kubo Takehiro <kubo@jiubao.org>
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

use std::marker::PhantomData;
use std::rc::Rc;

use binding::*;

use ColumnIndex;
use ColumnInfo;
use Connection;
use FromSql;
use Result;
use SqlValue;
use Statement;

pub struct RowSharedData {
    column_names: Vec<String>,
    conn_handle: *mut dpiConn,
}

impl Drop for RowSharedData {
    fn drop(&mut self) {
        if !self.conn_handle.is_null() {
            unsafe { dpiConn_release(self.conn_handle) };
        }
    }
}

/// Row in a result set of a select statement
pub struct Row {
    pub(crate) shared: Rc<RowSharedData>,
    pub(crate) column_values: Vec<SqlValue>,
}

impl Row {
    pub(crate) fn new(conn: &Connection, column_names: Vec<String>, column_values: Vec<SqlValue>) -> Result<Row> {
        chkerr!(conn.ctxt,
                dpiConn_addRef(conn.handle));
        let shared = RowSharedData {
            column_names: column_names,
            conn_handle: conn.handle,
        };
        Ok(Row {
            shared: Rc::new(shared),
            column_values: column_values,
        })
    }

    /// Gets the column value at the specified index.
    pub fn get<I, T>(&self, colidx: I) -> Result<T> where I: ColumnIndex, T: FromSql {
        let pos = colidx.idx(&self.shared.column_names)?;
        self.column_values[pos].get()
    }

    /// Returns column values as a vector of SqlValue
    pub fn sql_values(&self) -> &Vec<SqlValue> {
        &self.column_values
    }

    #[cfg(feature = "restore-deleted")]
    #[deprecated(since="0.0.4", note="use `sql_value` instead")]
    #[doc(hidden)]
    pub fn columns(&self) -> &Vec<SqlValue> {
        self.sql_values()
    }

    /// Gets column values as specified type.
    ///
    /// Type inference for the return type doesn't work. You need to specify
    /// it explicitly such as `row.get_as::<(i32, String>()`.
    /// See [RowValue][] for available return types.
    ///
    /// [RowValue]: trait.RowValue.html
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    /// let mut stmt = conn.prepare("select empno, ename from emp").unwrap();
    ///
    /// for result in stmt.query(&[]).unwrap() {
    ///     let row = result.unwrap();
    ///     // Gets a row as `(i32, String)`.
    ///     let (empno, ename) = row.get_as::<(i32, String)>().unwrap();
    ///     println!("{},{}", empno, ename);
    /// }
    /// ```
    pub fn get_as<T>(&self) -> Result<<T>::Item> where T: RowValue {
        <T>::get(self)
    }
}

/// Result set
pub struct ResultSet<'stmt, T> where T: RowValue {
    stmt: &'stmt Statement<'stmt>,
    phantom: PhantomData<T>,
}

impl<'stmt, T> ResultSet<'stmt, T> where T: RowValue {
    pub(crate) fn new(stmt: &'stmt Statement<'stmt>) -> ResultSet<'stmt, T> {
        ResultSet {
            stmt: stmt,
            phantom: PhantomData,
        }
    }

    pub fn column_info(&self) -> &Vec<ColumnInfo> {
        &self.stmt.column_info
    }
}

impl<'stmt, T> Iterator for ResultSet<'stmt, T> where T: RowValue {
    type Item = Result<<T>::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stmt
            .next()
            .map(|row_result| row_result.and_then(|row| row.get_as::<T>()))
    }
}

/// A trait to get a row as specified type
///
/// This is the return type of [Connection.query_row_as][],
/// [Connection.query_row_as_named][] and [Row.get_as][].
///
/// The trait was added to fetch column values as a tuple.
/// The oracle crate provides implementations for a type
/// implementing [FromSql][] and tuples of types implementing
/// [FromSql][]. The number of elements in a tuple should
///  be 1 through 50.
///
/// ```no_run
/// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
///
/// let sql = "select * from emp where empno = :1";
///
/// // Gets the first column value in a row.
/// // Values after the second column are ignored.
/// let empno = conn.query_row_as::<u32>(sql, &[&7369]).unwrap();
///
/// // Gets the first two column values in a row.
/// // Values after the third column are ignored.
/// let tuple_of_empno_and_ename = conn.query_row_as::<(i32, String)>(sql, &[&7499]).unwrap();
/// ```
///
/// You can implement the trait for your own types. For example
/// when you have a struct whose members are `empno` and `ename`,
/// you can make the struct from `empno` and `ename` column values
/// as follows:
///
/// ```no_run
/// struct Emp {
///     empno: i32,
///     ename: String,
/// }
///
/// impl oracle::RowValue for Emp {
///     type Item = Emp;
///     fn get(row: &oracle::Row) -> oracle::Result<Emp> {
///         Ok(Emp {
///             empno: row.get("empno")?,
///             ename: row.get("ename")?,
///         })
///     }
/// }
///
/// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
/// let mut stmt = conn.prepare("select * from emp").unwrap();
///
/// // Gets rows as Emp
/// for result in stmt.query_as::<Emp>(&[]).unwrap() {
///     let emp = result.unwrap();
///     println!("{},{}", emp.empno, emp.ename);
/// }
/// ```
///
/// [FromSql]: trait.FromSql.html
/// [Connection.query_row_as]: struct.Connection.html#method.query_row_as
/// [Connection.query_row_as_named]: struct.Connection.html#method.query_row_as_named
/// [Row.get_as]: struct.Row.html#method.get_as
pub trait RowValue {
    type Item;
    fn get(row: &Row) -> Result<Self::Item>;
}

impl RowValue for Row {
    type Item = Row;
    fn get(row: &Row) -> Result<Row> {
        let num_cols = row.column_values.len();
        let mut column_values = Vec::with_capacity(num_cols);
        for val in &row.column_values {
            column_values.push(val.dup_by_handle(row.shared.conn_handle)?);
        }
        Ok(Row {
            shared: row.shared.clone(),
            column_values: column_values,
        })
    }
}

impl<T: FromSql> RowValue for T {
    type Item = T;
    fn get(row: &Row) -> Result<T> {
        Ok(row.get::<usize, T>(0)?)
    }
}

macro_rules! impl_row_value_for_tuple {
    ($(
        [$(($idx:tt, $T:ident))+],
    )+) => {
        $(
            impl<$($T:FromSql,)+> RowValue for ($($T,)+) {
                type Item = ($($T,)+);
                fn get(row: &Row) -> Result<($($T,)+)> {
                    Ok((
                        $(row.get::<usize, $T>($idx)?,)+
                    ))
                }
            }
        )+
    }
}

impl_row_value_for_tuple!{
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
