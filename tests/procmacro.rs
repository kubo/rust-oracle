use oracle::RowValue;

#[test]
fn procmacro_ok_shadow() {
    #[allow(dead_code)]
    enum AnotherResult<T, E> {
        Ok(T),
        Err(E),
    }

    #[allow(unused_imports)]
    use AnotherResult::Ok;

    #[derive(Debug, RowValue)]
    struct Foo {}
}
