# Change Log

## 0.3.1 (2019-10-05)

Changes:

* Add a new method [`Connection.status()`][] and a new enum [`ConnStatus`][]
  to get the status of the connection.

## 0.3.0 (2019-10-02)

Incompatible changes:

* Remove `ConnParam` enum and add `Connector` struct to use builder pattern.
  * The fourth argument of `Connection::connect` was removed.  
    From:
    ```rust
    let conn = Connection::connect(username, password, connect_string, &[])?;
    ```
    To:
    ```rust
    let conn = Connection::connect(username, password, connect_string)?;
    ```
  * Use `Connector` to use additional parameters.  
    From:
    ```rust
    let conn = Connection::connect(username, password, connect_string, &[ConnParam::Sysdba])?;
    ```
    To:
    ```rust
    let conn = Connector::new(username, password, connect_string).privilege(Privilege::Sysdba).connect()?;
    ```
    or
    ```rust
    let mut connector = Connector::new(username, password, connect_string);
    connector.privilege(Privilege::Sysdba);
    let conn = connector.connect().unwrap();
    ```

* Add a new submodule `sql_type` and move the following structs, enums and traits from the root module to the submodule.
  * `Collection`
  * `IntervalDS`
  * `IntervalYM`
  * `Object`
  * `ObjectType`
  * `ObjectTypeAttr`
  * `OracleType`
  * `Timestamp`
  * `FromSql`
  * `ToSql`
  * `ToSqlNull`

* Remove `client_version()` method and add `Version::client()` method instead.

* Remove the associated type `Item` from `RowValue` trait.

* Add `&Connection` argument to trait methods: `ToSql.oratype` and `ToSqlNull.oratype_for_null`.

* Iterator for `&ResultSet<T>` was removed and that for `ResultSet<T>`
  was added again for better ergonomics.
  Change `for row_result in &result_set {...}` to either `for row_result in result_set {...}` if the `ResultSet` can be consumed
  or to `for row_result in result_set.by_ref() {...}` otherwise.

Changes:

* Implement `FusedIterator` for `ResultSet`.
* The return value of [`Connection.object_type()`][] is cached in the connection.  
  When "CREATE TYPE", "ALTER TYPE" or "DROP TYPE" is executed, the cache clears.
* Add `Connection.clear_object_type_cache()`.
* Update ODPI-C to version 3.2.2.

## 0.2.2 (2019-09-29)

* Implement `Sync` and `Send` for `Connection`. ([GH-14][])

## 0.2.1 (2019-04-14)

Changes:

* Fix memory corruption when using object and collection data types.
* Update ODPI-C to version 3.1.3.

## 0.2.0 (2018-10-02)

Incompatible changes:

* Make errors usable across threads ([GH-6][])

## 0.1.2 (2018-09-22 - yanked because of packaging miss)

Changes:

* Change the license to the Universal Permissive License v 1.0 and/or the Apache License v 2.0.

* Update ODPI-C to 3.0.0, which includes support for Oracle 18c client.

* New methods
  * [`Statement.row_count()`][]

## 0.1.1 (2018-07-16)

Changes:

* Allow fetching BLOB data as `Vec<u8>`.
* Implement `ToSql` and `ToSqlNull` for `&[u8]`.
* Implement `Debug` for `Connection`, `Statement` and so on.
* Update ODPI-C to version 2.4.2.

## 0.1.0 (2018-04-15)

Changes:

* New methods
  * [`Statement.is_query()`][]
  * [`Statement.is_plsql()`][]
  * [`Statement.is_ddl()`][]
  * [`Statement.is_dml()`][]

Incompatible changes:

* Iterator for `ResultSet<T>` was removed and that for `&ResultSet<T>`
  was added in order not to consume `ResultSet<T>` by for-loop.  
  Change `for row_result in result_set {...}` to `for row_result in &result_set {...}`.

## 0.0.8 (2018-03-25)

Fixed bugs:

* Fix an error when a column value converted from the database character set to
  UTF-8 becomes longer than the column size. ([GH-3][])

Incompatible changes:

* BindIndex and ColumnIndex were sealed and cannot be implemented for types outside of the crate.

* The `Other` variant of [`StatetmentType`][] enum was removed. `Commit`, `Rollback`, `ExplainPlan`, `Call` and `Unknown` variants were added to the enum.

* Change the return type of [`ObjectType.new_object()`][] from `Option<Object>` to `Result<Object>`.

* Change the return type of [`ObjectType.new_collection()`][] from `Option<Collection>` to `Result<Collection>`.

## 0.0.7 (2018-03-18)

The method to prepare statements was changed for future extension.

Changes:

* New methods and structs
  * [`Statement.returned_values()`][] to support RETURNING INTO clause.
  * [`StmtParam`][] struct to specify prepared statement parameters.

Incompatible changes:

* Changed Methods
  * [`Connection.prepare()`][]. The `params` argument was added.

* Removed methods
  * `Statement.set_fetch_array_size()`. Use [`StmtParam::FetchArraySize`][] instead.

## 0.0.6 (2018-03-11)

Methods for establishing connections were changed in order to avoid
incompatible changes when connection pooling is supported in future.

Changes:

* New methods and enums.
  * [`Connection::connect()`][]
  * [`ConnParam`][]

* Deprecated methods.
  * `Connnection::new()`. Use [`Connection::connect()`][] instead.

Incompatible changes:

* Renamed variants.
  * `Error::NoMoreData` &#x2192; [`Error::NoDataFound`][]
* Removed structs and enums.
  * `Connector` (connection builder). Use [`ConnParam`][] in order to specify extra connection parameters instead.
  * `AuthMode`. Use [`ConnParam`][] to specify authentication mode instead.
* Methods whose return type was changed from `&String` to `&str`.
  * [`Connection.tag()`][]
  * [`ColumnInfo.name()`][]
  * [`DbError.message()`][]
  * [`DbError.fn_name()`][]
  * [`DbError.action()`][]
  * [`ObjectType.schema()`][]
  * [`ObjectType.name()`][]
  * [`ObjectTypeAttr.name()`][]
* Methods whose return type was changed from `&Vec<...>` to `&[...]`.
  * [`Row.sql_values()`][]
  * [`ResultSet.column_info()`][]
  * [`ObjectType.attributes()`][]

## 0.0.5 (2018-03-04)

New features:

* Add query methods to `Connection` to fetch rows without using `Statement`.
  * [`Connection.query()`][]
  * [`Connection.query_named()`][]
  * [`Connection.query_as()`][]
  * [`Connection.query_as_named()`][]
* Add query_row methods to `Statement` to fetch a first row without using `ResultSet`.
  * [`Statement.query_row()`][]
  * [`Statement.query_row_named()`][]
  * [`Statement.query_row_as()`][]
  * [`Statement.query_row_as_named()`][]

Incompatible changes:

* Merge `RowResultSet` struct into `RowValueResultSet` and rename it to `ResultSet`.

## 0.0.4 (2018-02-25)

New features:

* Add query methods to `Statement` to fetch rows as iterator.
  * [`Statement.query()`][]
  * [`Statement.query_named()`][]
  * [`Statement.query_as()`][]
  * [`Statement.query_as_named()`][]
* Add query_row methods to `Connection` to fetch a first row without using `Statement`.
  * [`Connection.query_row()`][]
  * [`Connection.query_row_named()`][]
  * [`Connection.query_row_as()`][]
  * [`Connection.query_row_as_named()`][]
* Autocommit mode.

Incompatible changes:

* Execute methods fail for select statements. Use query methods instead.
  * [`Connection.execute()`][]
  * [`Connection.execute_named()`][]
  * [`Statement.execute()`][]
  * [`Statement.execute_named()`][]
* Renamed traits, methods and variants.
  * `ColumnValues` &#x2192; [`RowValue`][]
  * `Row.values()` &#x2192; [`Row.get_as()`][]
  * `Row.columns()` &#x2192; [`Row.sql_values()`][]
  * `Error::Overflow` &#x2192; [`Error::OutOfRange`][]
* Removed methods.
  * Statement.column_count()
  * Statement.column_names()
  * Statement.column_info()
  * Statement.fetch()
  * SqlValue.clone()

[GH-3]: https://github.com/kubo/rust-oracle/issues/3
[GH-6]: https://github.com/kubo/rust-oracle/issues/6
[GH-14]: https://github.com/kubo/rust-oracle/issues/14
[`ColumnInfo.name()`]: https://docs.rs/oracle/*/oracle/struct.ColumnInfo.html#method.name
[`Connection::connect()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.connect
[`Connection.execute()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.execute
[`Connection.execute_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.execute_named
[`Connection.object_type()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.object_type
[`Connection.prepare()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.prepare
[`Connection.query()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query
[`Connection.query_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_named
[`Connection.query_as()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as
[`Connection.query_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as_named
[`Connection.query_row()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row
[`Connection.query_row_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_named
[`Connection.query_row_as()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as
[`Connection.query_row_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as_named
[`Connection.status()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.status
[`Connection.tag()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.tag
[`ConnParam`]: https://docs.rs/oracle/0.2.*/oracle/enum.ConnParam.html
[`ConnStatus`]: https://docs.rs/oracle/*/oracle/enum.ConnStatus.html
[`DbError.action()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.action
[`DbError.fn_name()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.fn_name
[`DbError.message()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.message
[`Error::NoDataFound`]: https://docs.rs/oracle/*/oracle/enum.Error.html#variant.NoDataFound
[`Error::OutOfRange`]: https://docs.rs/oracle/*/oracle/enum.Error.html#variant.OutOfRange
[`ObjectType.attributes()`]: https://docs.rs/oracle/0.2.*/oracle/struct.ObjectType.html#method.attributes
[`ObjectType.name()`]: https://docs.rs/oracle/0.2.*/oracle/struct.ObjectType.html#method.name
[`ObjectType.new_collection()`]: https://docs.rs/oracle/0.2.*/oracle/struct.ObjectType.html#method.new_collection
[`ObjectType.new_object()`]: https://docs.rs/oracle/0.2.*/oracle/struct.ObjectType.html#method.new_object
[`ObjectType.schema()`]: https://docs.rs/oracle/0.2.*/oracle/struct.ObjectType.html#method.schema
[`ObjectTypeAttr.name()`]: https://docs.rs/oracle/0.2.*/oracle/struct.ObjectTypeAttr.html#method.name
[`ResultSet.column_info()`]: https://docs.rs/oracle/*/oracle/struct.ResultSet.html#method.column_info
[`Row.sql_values()`]: https://docs.rs/oracle/*/oracle/struct.Row.html#method.sql_values
[`Row.get_as()`]: https://docs.rs/oracle/*/oracle/struct.Row.html#method.get_as
[`RowValue`]: https://docs.rs/oracle/*/oracle/trait.RowValue.html
[`StatetmentType`]: https://docs.rs/oracle/*/oracle/enum.StatementType.html
[`Statement.execute()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.execute
[`Statement.execute_named()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.execute_named
[`Statement.query()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query
[`Statement.query_named()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_named
[`Statement.query_as()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_as
[`Statement.query_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_as_named
[`Statement.query_row()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row
[`Statement.query_row_named()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row_named
[`Statement.query_row_as()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row_as
[`Statement.query_row_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.query_row_as_named
[`Statement.returned_values()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.returned_values
[`Statement.row_count()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.row_count
[`Statement.is_query()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.is_query
[`Statement.is_plsql()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.is_plsql
[`Statement.is_ddl()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.is_ddl
[`Statement.is_dml()`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.id_dml
[`StmtParam`]: https://docs.rs/oracle/*/oracle/enum.StmtParam.html
[`StmtParam::FetchArraySize`]: https://docs.rs/oracle/*/oracle/enum.StmtParam.html#variant.FetchArraySize
