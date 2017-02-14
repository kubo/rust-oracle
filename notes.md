# Notes for who takes over this driver

## Files

* odpi - submodule to checkout [ODPI-C]
* src/ffi.rs - one-to-one mapping of functions and macros in [dpi.h][]
* src/odpi.rs - wrapper for functions and macros in src/ffi.rs
* src/main.rs - sample code. This file must be moved to other location.
* ... others

## How columns are defined

Query metadata are retrieved as [dpiQueryInfo][]. [ColumnInfo][] and [OracleType][] are
created from them via [ColumnInfo::new][] and [from_query_info][] respectively. OracleType
is also used to [define column types][stmt.define] by end users. If not all columns are defined explicitly,
they are [implicitly defined according to the OracleType in ColumnInfo][stmt.define_columns] just before fetch.

ODPI-C requires dpiVar to define columns. OracleType provides [parameters to create a dpiVar][dpiConn_newVar]
via [var_create_param][]. A dpiVar is created via [DpiVar::new] and passed to [dpiStmt_define][] via
[DpiStatement::define].

## How columns are fetched

`stmt.fetch()` calls the ODPI-C function [dpiStmt_fetch] and collects values via
[dpiStmt_getQueryValue][] as dpiData. The dpiData and dpiNativeTypeNum, which indicates
type of dpiData, are packed as DpiData. DpiData and OracleType are packed as ValueRef.
ValueRef is public and dpiData and DpiData are private in the crate. If I develop further,
I may remove DpiData and merge it with ValueRef.

When end users get values via `row.get(index)`, the `from` method in the FromSql trait
is called with a ValueRef argument. The `form` method calls a proper method of ValueRef
such as `as_int64()` and `as_string()`. A ValueRef method checks whether the requested
type is compatible with OracleType and calls a proper method of DpiData. It may convert
data type, for example i64 to f64. A DpiData method, called by ValueRef, checks whether
the requested type is compatible with dpiNativeTypeNum and converts C values in dpiData
to Rust values.

## How number columns are defined and fetched

This driver's way to define number columns is far from ideal. If the scale of a number
column is zero and the precision is less than 18, it is defined as int64. Otherwise the
number is defined with DPI_ORACLE_TYPE_NUMBER and DPI_NATIVE_TYPE_BYTES. This means that
ODPI-C fetches the column as Oracle Number and converts it to a string to return to the
caller, the driver in this case. When end users get the value as i64 or f64, the string
returned by ODPI-C is converted to to i64 or f64 respectively.

The aim of this is to prevent incorrect conversion from Oracle number to Rust number.
I don't care the case that the return type of `row.get(..)` isn't suitable for the number
in the database. For example, if you use i32 to fetch 10.2, the number is truncated to
10. End users must take care whether the Rust type is suitable.

I have another choice to define number columns as double internally if int64 isn't suitable
according to the column metadata. However number columns are sometimes declared as number
without precision and scale even though they contain integer data only. Moreover, precision
and scale information may be lost by arithmetic in SQL statements. Therefore, integer
columns are sometimes treated as unsuitable for int64 by the driver even though column
metadata are checked. If such columns are implicitly defined as double and end users use
i64 as the return type of `row.get(..)`, numbers in the database may be incorrectly
converted to i64 because the precision of double is smaller than i64. This is easily
fixed by defining the column explicitly as `stmt.define(1, oracle::OracleType::Int64);`. 
However I don't like it. I'd like to permit end users to use any Rust number type
without defining it explicitly as long as the type is suitable for numbers in the database.

If you don't like this internal behaviour and want to use double instead of string,
change the following code:

* Replace DPI_NATIVE_TYPE_BYTES with DPI_NATIVE_TYPE_DOUBLE in [this line][numdef].
* Fix [ValueRef.as_int64()][num2int64]
* Fix [ValueRef.as_uint64()][num2uint64]
* Fix [ValueRef.as_float()][num2float]
* Fix [ValueRef.as_double()][num2double]

## Another idea to define columns

to be written

[dpiQueryInfo]:         https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/ffi.rs#L454-L466

[OracleType]:           https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L131-L186
[from_query_info]:      https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L190-L223
[var_create_param]:     https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L226-L288

[ColumnInfo]:           https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L1192-L1196
[ColumnInfo::new]:      https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L1211-L1217

[stmt.define]:          https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/lib.rs#L225-L230
[stmt.define_columns]:  https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/lib.rs#L250-L260

[DpiStatement::define]: https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L1151-L1156

[DpiVar]:               https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L1235-L1239
[DpiVar::new]:          https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L1242-L1254

[numdef]:               https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/odpi.rs#L249-L250
[num2int64]:            https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/value_ref.rs#L63-L66
[num2uint64]:           https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/value_ref.rs#L100-L103
[num2float]:            https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/value_ref.rs#L149-L152
[num2double]:           https://github.com/kubo/rust-oracle-wip/blob/4c6cada847/src/value_ref.rs#L127-L130

[ODPI-C]:               https://oracle.github.io/odpi/
[dpiStmt_define]:       https://oracle.github.io/odpi/doc/public_functions/dpiStmt.html#c.dpiStmt_define
[dpiStmt_fetch]:        https://oracle.github.io/odpi/doc/public_functions/dpiStmt.html#c.dpiStmt_fetch
[dpiStmt_getQueryValue]: https://oracle.github.io/odpi/doc/public_functions/dpiStmt.html#c.dpiStmt_getQueryValue
[dpiConn_newVar]:       https://oracle.github.io/odpi/doc/public_functions/dpiConn.html#c.dpiConn_newVar
[dpi.h]:                https://github.com/oracle/odpi/blob/master/include/dpi.h
