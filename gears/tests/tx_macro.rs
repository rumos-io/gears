#![cfg(feature = "macros_test")]

#[test]
fn simple_structure() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assets/app_msg/simple_struct.rs");
}

#[test]
fn simple_enum() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assets/app_msg/simple_enum.rs");
}
