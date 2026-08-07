//! The `@style` section lowers to resolved style intent on the IR.

use mobius_lang::{Registry, lower, parse};

fn lower_ok(source: &str) -> mobius_lang::Ir {
    let (file, _) = parse(source).unwrap();
    lower(&file.apps[0], &Registry::new()).expect("must lower")
}

#[test]
fn style_section_resolves() {
    let ir = lower_ok(
        r#"
app A {
    @style {
        theme        = tokyo_night;
        font_scale   = 110;
        item_spacing = 8;
        slider_width = 160;
    }
    interface S { level : f32 = 0.0  modport ui (out level) }
    @wiring { let s = S(); }
}
"#,
    );
    assert_eq!(ir.style.theme.as_deref(), Some("tokyo_night"));
    assert_eq!(ir.style.font_scale, Some(110.0));
    assert_eq!(ir.style.item_spacing, Some(8.0));
    assert_eq!(ir.style.slider_width, Some(160.0));
}

#[test]
fn no_style_section_is_default() {
    let ir = lower_ok(
        r#"app A { interface S { x : f32 = 0.0  modport ui (out x) } @wiring { let s = S(); } }"#,
    );
    assert_eq!(ir.style, mobius_lang::ir::IrStyle::default());
}

fn expect_error(source: &str, needle: &str) {
    let (file, _) = parse(source).unwrap();
    let diagnostics = lower(&file.apps[0], &Registry::new()).unwrap_err();
    assert!(
        diagnostics.iter().any(|d| d.message.contains(needle)),
        "expected {needle:?}, got: {:#?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn unknown_style_key_errors() {
    expect_error(
        r#"app A { @style { colour = red; } @wiring { } }"#,
        "unknown @style key `colour`",
    );
}

#[test]
fn wrong_style_value_type_errors() {
    expect_error(
        r#"app A { @style { font_scale = big; } @wiring { } }"#,
        "`font_scale` expects a number",
    );
}
