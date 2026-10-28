//! Parser error recovery: multiple errors per pass, partial AST retained.

use mobius_lang::{parse, parse_with_recovery};

/// One error in a citizen body, one in a section — both reported, and the
/// well-formed parts of the app survive in the AST.
const TWO_ERRORS: &str = r#"
app A {
    interface I {
        x : bool = false
        modport v (out x)
    }
    citizen C (p : I.v) {
        row {
            button "a" <-> p.x
        }
    }
    @wiring {
        let i = I()
        let c = C(p = i.v);
    }
}
"#;

#[test]
fn reports_every_error_and_keeps_the_partial_ast() {
    let out = parse_with_recovery(TWO_ERRORS);
    assert_eq!(
        out.errors.len(),
        2,
        "one widget error + one stmt error: {:?}",
        out.errors
    );
    // Both errors anchor to real source positions, in order.
    assert!(out.errors[0].span.line < out.errors[1].span.line);

    // The app survived with its well-formed items intact.
    let app = &out.file.apps[0];
    assert_eq!(app.name, "A");
    assert_eq!(app.items.len(), 3, "interface + citizen + @wiring all kept");
}

#[test]
fn fail_fast_parse_returns_the_first_recovered_error() {
    let first = parse_with_recovery(TWO_ERRORS).errors.remove(0);
    let err = parse(TWO_ERRORS).unwrap_err();
    assert_eq!(err.span, first.span);
    assert_eq!(err.message, first.message);
}

#[test]
fn clean_source_recovers_nothing() {
    let source = include_str!("../../mobius_codegen/tests/fixtures/ribbon.mobius");
    let out = parse_with_recovery(source);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    assert_eq!(out.file.apps.len(), 1);
}

/// Recovery must terminate on pathological input (unclosed everything).
#[test]
fn truncated_input_terminates() {
    for source in [
        "app A { citizen C (p : I.v) { row {",
        "app A { interface I { x :",
        "app",
        "app A { @wiring { let x",
        "} } }",
    ] {
        let out = parse_with_recovery(source);
        assert!(!out.errors.is_empty(), "{source:?} must error");
    }
}
