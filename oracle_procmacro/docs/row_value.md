A derive macro to implement the [`RowValue`] trait

All of the structure named fields have to implement [`FromSql`]
or have a `row_value` attribute with a function name.

## Examples

When all field data types implement [`FromSql`], set the `#[derive(RowValue)]`
attribute to the struct.

```
# use oracle::RowValue;
#[derive(RowValue)]
struct Employee {
    id: u32,
    name: String,
    salary: u64,
    #[row_value(rename = "manager")]
    manager_id: Option<u32>,
}
```

The above code is equivalent to the following:

```
# use oracle::{Result, Row, RowValue, SqlValue};
struct Employee {
    id: u32,
    name: String,
    salary: u64,
    manager_id: Option<u32>,
}

impl RowValue for Employee {
    fn get(row: &Row) -> Result<Self> {
        Ok(Employee {
            id: row.get("id")?,
            name: row.get("name")?,
            salary: row.get("salary")?,
            manager_id: row.get("manager")?,
        })
    }
}
```

**Note:** The `manager` column value is set to the `manager_id` field due to
the `#[row_value(rename = "manager")]` attribute.

When a struct contains a field, whose type is defined in the same crate,
it is better to implement [`FromSql`] for the type.

```
# use oracle::Error;
# use oracle::Result;
# use oracle::RowValue;
# use oracle::SqlValue;
# use oracle::sql_type::FromSql;

enum Gender {
  Male,
  Female,
  X(String),
}

impl FromSql for Gender {
    fn from_sql(val: &SqlValue) -> Result<Self> {
        let gender = val.get::<String>()?;
        Ok(match gender.as_str() {
            "M" => Gender::Male,
            "F" => Gender::Female,
            _ => Gender::X(gender),
        })
    }
}

#[derive(RowValue)]
struct Employee {
    id: u32,
    name: String,
    gender: Gender,
}
```

When a struct contains a field, whose type is defined in an external crate,
use `row_value` attribute with a function name to convert
[`Row`] to the field type.

```
# use uuid::Uuid;
# use std::boxed::Box;
# use oracle::{Error, Result, Row, RowValue};

fn oracle_column_to_uuid(row: &Row, column_name: &str) -> Result<Uuid> {
    let uuid_str: String = row.get(column_name)?;
    Uuid::parse_str(&uuid_str).map_err(|err| Error::ParseError(Box::new(err)))
}

#[derive(RowValue)]
struct Employee {
    id: u32,
    name: String,
    #[row_value(with = "oracle_column_to_uuid")]
    uuid: Uuid, // https://docs.rs/uuid/0.8/uuid/struct.Uuid.html
}
```

[`RowValue`]: trait.RowValue.html
[`FromSql`]: sql_type/trait.FromSql.html
[`Row`]: struct.Row.html
[`SqlValue`]: struct.SqlValue.html
