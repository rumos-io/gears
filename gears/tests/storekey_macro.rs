#![cfg(feature = "macros_test")]

#[test]
fn simple() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assets/storekey/simple.rs");
}

#[test]
#[should_panic]
fn empty_key() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/assets/storekey/empty_key.rs");
}

#[test]
#[should_panic]
fn duplicate_key() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/assets/storekey/duplicate_key.rs");
}

#[test]
#[should_panic]
fn no_params() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/assets/storekey/no_params.rs");
}
