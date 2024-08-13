#![cfg(feature = "macros_test")]

#[test]
fn simple() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assets/paramskey/simple.rs");
}

#[test]
#[should_panic]
fn empty_key() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/assets/paramskey/empty_key.rs");
}

#[test]
#[should_panic]
fn duplicate_key() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/assets/paramskey/duplicate_key.rs");
}


