#[test]
fn simple() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assets/proto/simple.rs");
}

#[test]
fn different_fields() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assets/proto/different_fields.rs");
}
