# Query Methods

There are many methods to query rows as follows. This page
explains how to use them depending on the context.

* Connection methods:
  1. <code>fn [query][cq](&self, sql: &str, params: &[&[ToSql][]]) -> Result\<[ResultSet][]\<[Row][]>></code>
  2. <code>fn [query_named][cqn](&self, sql: &str, params: &[(&str, &[ToSql][])]) -> Result\<[ResultSet][]\<[Row][]>></code>
  3. <code>fn [query_as][cqa]\<T>(&self, sql: &str, params: &[&[ToSql][]]) -> Result\<[ResultSet][]\<T>> where T: [RowValue][]</code>
  4. <code>fn [query_as_named][cqan]\<T>(&self, sql: &str, params: &[(&str, &[ToSql][])]) -> Result\<[ResultSet][]\<T>> where T: [RowValue][]</code>
  5. <code>fn [query_row][cqr](&self, sql: &str, params: &[&[ToSql][]]) -> Result\<[Row][]></code>
  6. <code>fn [query_row_named][cqrn](&self, sql: &str, params: &[(&str, &[ToSql][])]) -> Result\<[Row][]></code>
  7. <code>fn [query_row_as][cqra]\<T>(&self, sql: &str, params: &[&[ToSql][]]) -> Result\<\<T>::Item> where T: [RowValue][]</code>
  8. <code>fn [query_row_as_named][cqran]\<T>(&self, sql: &str, params: &[(&str, &[ToSql][])]) -> Result\<\<T>::Item> where T: [RowValue][]</code>

* Statement methods:
  1. <code>fn [query][sq](&mut self, params: &[&[ToSql][]]) -> Result\<[ResultSet][]\<[Row][]>></code>
  2. <code>fn [query_named][sqn](&mut self, params: &[(&str, &[ToSql][])]) -> Result\<[ResultSet][]\<[Row][]>></code>
  3. <code>fn [query_as][sqa]\<'a, T>(&'a mut self, params: &[&[ToSql][]]) -> Result\<[ResultSet][]\<'a, T>> where T: [RowValue][]</code>
  4. <code>fn [query_as_named][sqan]\<'a, T>(&'a mut self, params: &[(&str, &[ToSql][])]) -> Result\<[ResultSet][]\<'a, T>> where T: [RowValue][]</code>
  5. <code>fn [query_row][sqr](&mut self, params: &[&[ToSql][]]) -> Result\<[Row][]></code>
  6. <code>fn [query_row_named][sqrn](&mut self, params: &[(&str, &[ToSql][])]) -> Result\<[Row][]></code>
  7. <code>fn [query_row_as][sqra]\<T>(&mut self, params: &[&[ToSql][]]) -> Result\<\<T>::Item> where T: [RowValue][]</code>
  8. <code>fn [query_row_as_named][sqran]\<T>(&mut self, params: &[(&str, &[ToSql][])]) -> Result\<\<T>::Item> where T: [RowValue][]</code>

The next table is a brief summary of the following sections.

struct | method | bind parameter | predictable column type and column by position | max. number of rows | same SQL except parameters |
-----------|--------------------|------------|--------|---------|--------|
Connection | [query][cq]        | positional | no     | unknown | no     |
" | [query_named][cqn]          | named      | no     | unknown | no     |
" | [query_as][cqa]             | positional | yes    | unknown | no     |
" | [query_as_named][cqan]      | named      | yes    | unknown | no     |
" | [query_row][cqr]            | positional | no     | 1       | no     |
" | [query_row_named][cqrn]     | named      | no     | 1       | no     |
" | [query_row_as][cqra]        | positional | yes    | 1       | no     |
" | [query_row_as_named][cqran] | named      | yes    | 1       | no     |
Statement  | [query][sq]        | positional | no     | unknown | yes    |
" | [query_named][sqn]          | named      | no     | unknown | yes    |
" | [query_as][sqa]             | positional | yes    | unknown | yes    |
" | [query_as_named][sqan]      | named      | yes    | unknown | yes    |
" | [query_row][sqr]            | positional | no     | 1       | yes    |
" | [query_row_named][sqrn]     | named      | no     | 1       | yes    |
" | [query_row_as][sqra]        | positional | yes    | 1       | yes    |
" | [query_row_as_named][sqran] | named      | yes    | 1       | yes    |

## With and without `_named`

When an SQL statement has no parameters or positional parameters, use methods which don't end with `_named`.
When it has named parameters, use methods which end with `_named`.

```rust
// SQL statement with no parameters
let sql_text = "select empno, ename from emp where deptno = 10 and sal >= 1000";
let rows = conn.query_as::<(i32, String)>(sql_text, &[])?;

// SQL statement with positional parameters.
// Needless to say, parameter names don't affect the position.
// For example, `:deptno` or even `:3` as the first.
let sql_text = "select empno, ename from emp where deptno = :1 and sal >= :2";
let rows = conn.query_as::<(i32, String)>(sql_text, &[&10, &1000])?;

// SQL statement with named parameters
let sql_text = "select empno, ename from emp where deptno = :deptno and sal >= :sal";
let rows = conn.query_as_named::<(i32, String)>(sql_text, &[("deptno", &10), ("sal", &1000)])?;
```

## With and without `_as`

When the column types in a query result are predictable and columns
are retrieved by positions, use methods including `_as`. Otherwise,
use methods not including `_as`.

```rust
// When you know the column types of the statement is number and character,
// fetch the result as tuples of `i32` and `String`.
let sql_text = "select empno, ename from emp where deptno = 10 and sal >= 1000";
let rows = conn.query_as::<(i32, String)>(sql_text, &[])?;
// This is same with the following:
//   let rows: ResultSet<(i32, String)> = conn.query_as(sql_text, &[])?;

for row_result in rows { // row_result: Result<(i32, String)>
    let (empno, ename) = row_result?; // empno: i32, ename: String
    println!("empno: {}, ename: {}", empno, ename);
}
```

The above could be rewritten as follows. However it is a bit inefficient.
The above converts column values in internal [`Row`][] data into `i32` and
`String` directly. The following clones the internal `Row` to use it as
an iterator result and then converts column values in it into `i32` and
`String`.

```rust
// The rows are fetched as `Row`.
let sql_text = "select empno, ename from emp where deptno = 10 and sal >= 1000";
let rows = conn.query(sql_text, &[])?;

for row_result in rows { // Internal Row data are cloned to make iterator results.
    let row = row_result?; // row: Row
    let empno: i32 = row.get(0)?;
    let ename: String = row.get(1)?;
    println!("empno: {}, ename: {}", empno, ename);
}
```

When column types are unpredicted at compile time, `_as` methods aren't available.
Use methods not including `_as`.

```rust
let sql_text = "...";
let stmt = conn.prepare(sql_text, &[])?;
if stmt.is_query() {
    let rows = stmt.query(&[])?;
    let column_info = rows.column_info();

    for row_result in &rows {
        let row = row_result?;
        println!("Row");
        for (colidx, val) in row.sql_values().enumerate() {
            println!("  {}: {}", column_info[colidx].name(), val.to_string());
        }
    }
}
```

Even when column types are predicted, `_as` methods aren't available if
you need to get column values by their names. Use `row.get("column_name")`
as follows.

```rust
let sql_text = "select empno, ename from emp where deptno = 10 and sal >= 1000";
let rows = conn.query(sql_text, &[])?;

for row_result in rows {
    let row = row_result?;
    let empno: i32 = row.get("empno")?;
    let ename: String = row.get("ename")?;
    println!("empno: {}, ename: {}", empno, ename);
}
```

If you implement [`RowValue`][] for your type, you can fetch rows as the type.

```rust
struct EmpRow {
    empno: i32,
    ename: String,
}

impl RowValue for EmpRow {
    type Item = EmpRow;
    fn get(row: &Row) -> Result<EmpRow> {
        Ok(EmpRow {
            empno: row.get("empno")?,
            ename: row.get("ename")?,
        })
    }
}

let sql_text = "select empno, ename from emp where deptno = 10 and sal >= 1000";
let rows = conn.query_as::<EmpRow>(sql_text, &[])?;

for row_result in rows {
    let row = row_result?; // row: EmpRow
    println!("empno: {}, ename: {}", row.empno, row.ename);
}
```

## With and without `_row`

When a query returns at most one row, use methods which start with `query_row`.
They fetch only the first row when it is found or return `Err<Error::NoDataFound>`
when no rows are found.

```rust
let sql_text = "select ename from emp where empno = :1";
let ename = conn.query_row_as::<String>(sql_text, &[&100])?;
```

`query_row` methods in `Connection` struct are efficient compared with
the others when at most one row is fetched. That's because they allocate
memory for only one row but the others allocate memory for 100 rows
to reduce the number of network round trips in case that many rows are
fetched.

When `query_row` methods in `Statement` struct is used, use
`StmtParam::FetchArraySize(1)` to reduce memory usage as corresponding
methods in `Connection` do.

```rust
let sql_text = "select ename from emp where empno = :1";
let stmt = conn.prepare(sql_text, &[StmtParam::FetchArraySize(1)])?

let empnos = [100, 101, 102];

for empno in empnos {
    let ename = stmt.query_row_as::<String>(&[&empno])?;
    println!("empno: {}, ename: {}", empno, ename);
}
```

## Connection methods and Statement methods

When an ad-hoc query is executed, use query methods of Connection.

```rust
let sql_text = "select empno, ename from emp where deptno = :1";
let rows = conn.query_as::<(i32, String)>(sql_text, &[&10])?;

for row_result in &rows {
    let row = row_result?;
    println!("empno: {}, ename: {}", row.0, row.1);
}
```

When you execute same SQL except parameters, use query methods of Statement.

```rust
let sql_text = "select empno, ename from emp where deptno = :1";
let stmt = conn.prepare(sql_text, &[])?;

for deptno in [10, 20, 30] {
    let rows = stmt.query_as::<(i32, String)>(&[&deptno])?;

    println!("deptno: {}", deptno);
    for row_result in &rows {
        let row = row_result?;
	println!("  empno: {}, ename: {}", row.0, row.1);
    }
}
```

When a SQL is executed, DBMS does the followings: (1) parse the SQL,
(2) make a query plan, (3) execute the query plan and then (4) returns
the result. When query methods of Connection are used, all four steps
are performed. When query methods of Statement are used, only the last
two steps are performed. (This is extremely rough sketch. There are many
other considerations; [library cache][], [cursor sharing][],
[client statement cache][] and so on.)

[cq]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query
[cqn]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_named
[cqa]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as
[cqan]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as_named
[cqr]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row
[cqrn]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_named
[cqra]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as
[cqran]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as_named
[sq]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query
[sqn]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_named
[sqa]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_as
[sqan]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_as_named
[sqr]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row
[sqrn]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row_named
[sqra]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row_as
[sqran]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row_as_named
[ResultSet]: https://docs.rs/oracle/*/oracle/struct.ResultSet.html
[Row]: https://docs.rs/oracle/*/oracle/struct.Row.html
[`Row`]: https://docs.rs/oracle/*/oracle/struct.Row.html
[RowValue]: https://docs.rs/oracle/*/oracle/trait.RowValue.html
[`RowValue`]: https://docs.rs/oracle/*/oracle/trait.RowValue.html
[ToSql]: https://docs.rs/oracle/*/oracle/trait.ToSql.html
[library cache]: https://docs.oracle.com/en/database/oracle/oracle-database/12.2/cncpt/memory-architecture.html#GUID-DE757E9C-3437-408A-8598-3EB4C8E2A3B0
[cursor sharing]: https://docs.oracle.com/en/database/oracle/oracle-database/12.2/tgsql/improving-rwp-cursor-sharing.html#GUID-971F4652-3950-4662-82DE-713DDEED317C
[client statement cache]: https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnoci/more-oci-advanced-topics.html#GUID-75169FE4-DE2C-431F-BBA7-3691C7C33360
