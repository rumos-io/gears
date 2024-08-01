#[test]
fn simple() {
    let t = trybuild::TestCases::new();
    t.pass("tests/assets/raw/simple.rs");
}
