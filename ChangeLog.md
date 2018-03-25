# Change Log

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
[`ColumnInfo.name()`]: https://docs.rs/oracle/*/oracle/struct.ColumnInfo.html#method.name
[`Connection::connect()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.connect
[`Connection.execute()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.execute
[`Connection.execute_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.execute_named
[`Connection.prepare()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.prepare
[`Connection.query()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query
[`Connection.query_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_named
[`Connection.query_as()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as
[`Connection.query_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as_named
[`Connection.query_row()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row
[`Connection.query_row_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_named
[`Connection.query_row_as()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as
[`Connection.query_row_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as_named
[`Connection.tag()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.tag
[`ConnParam`]: https://docs.rs/oracle/*/oracle/enum.ConnParam.html
[`DbError.action()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.action
[`DbError.fn_name()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.fn_name
[`DbError.message()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.message
[`Error::NoDataFound`]: https://docs.rs/oracle/*/oracle/enum.Error.html#variant.NoDataFound
[`Error::OutOfRange`]: https://docs.rs/oracle/*/oracle/enum.Error.html#variant.OutOfRange
[`ObjectType.attributes()`]: https://docs.rs/oracle/*/oracle/struct.ObjectType.html#method.attributes
[`ObjectType.name()`]: https://docs.rs/oracle/*/oracle/struct.ObjectType.html#method.name
[`ObjectType.new_collection()`]: https://docs.rs/oracle/*/oracle/struct.ObjectType.html#method.new_collection
[`ObjectType.new_object()`]: https://docs.rs/oracle/*/oracle/struct.ObjectType.html#method.new_object
[`ObjectType.schema()`]: https://docs.rs/oracle/*/oracle/struct.ObjectType.html#method.schema
[`ObjectTypeAttr.name()`]: https://docs.rs/oracle/*/oracle/struct.ObjectTypeAttr.html#method.name
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
[`StmtParam`]: https://docs.rs/oracle/*/oracle/enum.StmtParam.html
[`StmtParam::FetchArraySize`]: https://docs.rs/oracle/*/oracle/enum.StmtParam.html#variant.FetchArraySize
