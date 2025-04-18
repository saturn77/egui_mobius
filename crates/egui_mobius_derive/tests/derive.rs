#[test]
fn test_macro_compiles() {
    let t = trybuild::TestCases::new();
    t.pass("tests/fixtures/test_widget.rs");
}

