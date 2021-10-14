#[test]
fn compile() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/database.rs");
    t.compile_fail("tests/fail/service_struct.rs");
}
