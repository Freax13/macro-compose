use enum_from_str_macro::FromStr;
use std::str::FromStr;

#[derive(Eq, PartialEq, Debug, FromStr)]
pub enum Foo {
    Bar,
    Baz,
}

#[test]
fn test_from_str() {
    assert_eq!(Foo::from_str("Bar"), Ok(Foo::Bar));
    assert_eq!(Foo::from_str("Baz"), Ok(Foo::Baz));
    assert_eq!(Foo::from_str("Qux"), Err(FooError("Qux".to_string())));
}
