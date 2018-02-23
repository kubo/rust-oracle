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

use std::rc::Rc;

use ColumnIndex;
use ColumnValues;
use FromSql;
use Result;
use SqlValue;
use Statement;

/// Row in a result set of a select statement
pub struct Row {
    pub(crate) column_names: Rc<Vec<String>>,
    pub(crate) column_values: Vec<SqlValue>,
}

impl Row {
    /// Gets the column value at the specified index.
    pub fn get<I, T>(&self, colidx: I) -> Result<T> where I: ColumnIndex, T: FromSql {
        let pos = colidx.idx(&self.column_names)?;
        self.column_values[pos].get()
    }

    /// Returns column values as a vector of SqlValue
    pub fn sql_values(&self) -> &Vec<SqlValue> {
        &self.column_values
    }

    /// Gets column values as specified type.
    ///
    /// Type inference for the return type doesn't work. You need to specify
    /// it explicitly such as `row.get_as::<(i32, String>()`.
    /// See [ColumnValues][] for available return types.
    ///
    /// [ColumnValues]: trait.ColumnValues.html
    ///
    /// ```no_run
    /// let conn = oracle::Connection::new("scott", "tiger", "").unwrap();
    /// let mut stmt = conn.execute("select empno, ename from emp", &[]).unwrap();
    ///
    /// while let Ok(row) = stmt.fetch() {
    ///     // Gets a row as `(i32, String)`.
    ///     let (empno, ename) = row.get_as::<(i32, String)>().unwrap();
    ///     println!("{},{}", empno, ename);
    /// }
    /// ```
    pub fn get_as<T>(&self) -> Result<<T>::Item> where T: ColumnValues {
        <T>::get(self)
    }
}

pub struct Rows<'stmt> {
    stmt: &'stmt Statement<'stmt>,
}

impl<'stmt> Rows<'stmt> {
    pub(crate) fn new(stmt: &'stmt Statement<'stmt>) -> Rows<'stmt> {
        Rows {
            stmt: stmt,
        }
    }
}

impl<'stmt> Iterator for Rows<'stmt> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stmt.next() {
            Some(Ok(row)) => {
                let num_cols = row.column_values.len();
                let mut column_values = Vec::with_capacity(num_cols);
                for val in &row.column_values {
                    match val.dup(self.stmt.conn) {
                        Ok(dupval) => column_values.push(dupval),
                        Err(err) => return Some(Err(err)),
                    }
                }
                Some(Ok(Row {
                    column_names: row.column_names.clone(),
                    column_values: column_values,
                }))
            },
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}
