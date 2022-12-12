# Change Log

## 0.5.6 (20XX-XX-XX)

Fixed Issues:

* Fix RowValue which can reference an invalid Ok that defined outside (contributed by [GH-62])

Changes:

* [`Connection::prepare`] was marked as deprecated.

Internal Changes:

* Update ODPI-C to 4.4.1. (see [ODPI-C release notes](https://oracle.github.io/odpi/doc/releasenotes.html))
* Suppress 'cargo clippy' warnings
* Use atomic types instead of RefCell and Mutex

## 0.5.5 (2022-05-11)

New features:

* [`RowValue` derive macro] (contributed by [GH-49])
* Add query methods for Statement which take ownership (contributed by [GH-50])
* Support connection pooling
  * [`pool`]
  * [`Connection::close_with_mode`]
  * [`Connection::tag`]
  * [`Connection::tag_found`]
  * [`Connection::is_new_connection`]
* Add methods related to statement caching
  * [`Connector::stmt_cache_size`]
  * [`StatementBuilder::exclude_from_cache`]
  * [`StatementBuilder::tag`]
* Support Advanced Queuing experimentally when `aq_unstable` feature is enabled
  Breaking changes may be introduced by a minor release.

Fixed bugs:

* Fix resource leaks when statement are explicitly closed by [`Statement::close`].
* Fix syntax typo in doc comment about `row.get_as` ([GH-54])

Internal Changes:

* Update ODPI-C to 4.3.0. (see ODPI-C release notes: [4.3.0](https://oracle.github.io/odpi/doc/releasenotes.html#version-4-3-november-4-2021))

## 0.5.4 (2022-01-20)

Fixed bugs:

* Fix resource leaks when Oracle object datatypes are used. ([GH-48][])

## 0.5.3 (2021-08-15)

New features:

* Add [`Connection::statement`][] and [`StatementBuilder`][] to create
  [`Statement`][] and deprecate [`Connection::prepare`][].
* Customize prefetch row size ([`StatementBuilder::prefetch_rows`]) [GH-40]
* Read/Write LOBs as streams ([`sql_type::Lob`][], [`sql_type::Clob`][], [`sql_type::Nclob`][] and [`sql_type::Blob`][])
* Ref cursors including implicit statement results ([`sql_type::RefCursor`][]) [GH-38]
* Add [`Connection::oci_attr`][], [`Connection::set_oci_attr`][], [`Statement::oci_attr`][] and [`Statement::oci_attr`][]
  to support OCI handle attributes.
* `Impl ToSql for &'a [u8; N]`

Internal Changes:

* Bind LOB columns as string or binary by default

## 0.5.2 (2021-06-11)

* Update ODPI-C to 4.2.1. (see ODPI-C release notes: [4.2.0](https://oracle.github.io/odpi/doc/releasenotes.html#version-4-2-may-18-2021) and [4.2.1](https://oracle.github.io/odpi/doc/releasenotes.html#version-4-2-1-june-1-2021))
* Make it possible to get rowid as string ([GH-36][])
* Make it possible to bind boolean values (PL/SQL only)

## 0.5.1 (2021-04-18)

* Add [Batch DML][`Batch`] feature ([GH-29][])
* Implement `FromStr` for `Version`.
* Add `OracleType::Json` variant. (Note: JSON data type has not been supported yet.)

## 0.4.0 (2021-03-07)

Changes:

* Based on ODPI-C 4.1.0
* The return value of [`DbError.offset()`][] was changed from u16 to u32.

## 0.3.3 (2020-10-25)

Changes:

* Add new methods [`Connection.call_timeout()`][] and [`Connection.set_call_timeout()`][]

## 0.3.2 (2019-11-14)

Changes:

* Fix SEGV while getting Object type's FLOAT data type attributes. ([GH-19][])

* Add workaround to use with [tracing-core](https://crates.io/crates/tracing-core). ([GH-18][])

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
  * [`Connection::prepare`][]. The `params` argument was added.

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
[GH-18]: https://github.com/kubo/rust-oracle/issues/18
[GH-19]: https://github.com/kubo/rust-oracle/issues/19
[GH-29]: https://github.com/kubo/rust-oracle/issues/29
[GH-36]: https://github.com/kubo/rust-oracle/issues/36
[GH-38]: https://github.com/kubo/rust-oracle/issues/38
[GH-40]: https://github.com/kubo/rust-oracle/issues/40
[GH-48]: https://github.com/kubo/rust-oracle/issues/48
[GH-49]: https://github.com/kubo/rust-oracle/issues/49
[GH-50]: https://github.com/kubo/rust-oracle/issues/50
[GH-54]: https://github.com/kubo/rust-oracle/issues/54
[GH-62]: https://github.com/kubo/rust-oracle/pull/62
[`pool`]: https://docs.rs/oracle/latest/oracle/pool/index.html
[`Batch`]: https://docs.rs/oracle/*/oracle/struct.Batch.html
[`ColumnInfo.name()`]: https://docs.rs/oracle/*/oracle/struct.ColumnInfo.html#method.name
[`Connection::connect()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.connect
[`Connection.call_timeout()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.call_timeout
[`Connection.execute()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.execute
[`Connection.execute_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.execute_named
[`Connection.object_type()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.object_type
[`Connection::prepare`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.prepare
[`Connection.query()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query
[`Connection.query_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_named
[`Connection.query_as()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as
[`Connection.query_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_as_named
[`Connection.query_row()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row
[`Connection.query_row_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_named
[`Connection.query_row_as()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as
[`Connection.query_row_as_named()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.query_row_as_named
[`Connection::statement`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.statement
[`Connection.status()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.status
[`Connection.set_call_timeout()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.set_call_timeout
[`Connection.tag()`]: https://docs.rs/oracle/*/oracle/struct.Connection.html#method.tag
[`Connector::stmt_cache_size`]: https://docs.rs/oracle/*/oracle/struct.Connector.html#method.stmt_cache_size
[`ConnParam`]: https://docs.rs/oracle/0.2.*/oracle/enum.ConnParam.html
[`ConnStatus`]: https://docs.rs/oracle/*/oracle/enum.ConnStatus.html
[`DbError.action()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.action
[`DbError.fn_name()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.fn_name
[`DbError.message()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.message
[`DbError.offset()`]: https://docs.rs/oracle/*/oracle/struct.DbError.html#method.offset
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
[`RowValue` derive macro]: https://www.jiubao.org/rust-oracle/oracle/derive.RowValue.html
[`Statement::close`]: https://docs.rs/oracle/*/oracle/struct.Statement.html#method.close
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
[`StatementBuilder`]: https://docs.rs/oracle/*/oracle/struct.StatementBuilder.html
[`StatementBuilder::exclude_from_cache`]: https://docs.rs/oracle/latest/oracle/struct.StatementBuilder.html#method.exclude_from_cache
[`StatementBuilder::prefetch_rows`]: https://docs.rs/oracle/latest/oracle/struct.StatementBuilder.html#method.prefetch_rows
[`StatementBuilder::tag`]: https://docs.rs/oracle/latest/oracle/struct.StatementBuilder.html#method.tag
[`StmtParam`]: https://docs.rs/oracle/*/oracle/enum.StmtParam.html
[`StmtParam::FetchArraySize`]: https://docs.rs/oracle/*/oracle/enum.StmtParam.html#variant.FetchArraySize
