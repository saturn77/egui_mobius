//! Enum state + `combo`/`radio`: an enum field lowers to a variant-backed
//! string, and combo/radio widgets carry the enum's variants as options.

use mobius_lang::ir::{IrWidget, IrWidgetTarget};
use mobius_lang::{Registry, lower, parse};

const RIG: &str = r#"
app Rig {
    enum Mode { AC, DC }
    interface Bus {
        mode : Mode = DC
        modport ctl (out mode)
    }
    citizen Cmd (b : Bus.ctl) {
        column {
            combo "Mode" <-> b.mode;
            radio "Mode" <-> b.mode;
        }
    }
    @wiring { let bus = Bus(); let cmd = Cmd(b = bus.ctl); }
    @layout { dock(cmd, region = center); }
}
"#;

fn lower_ok(source: &str) -> mobius_lang::Ir {
    let (file, _) = parse(source).unwrap();
    lower(&file.apps[0], &Registry::new()).expect("must lower")
}

#[test]
fn enum_field_lowers_to_variant_backed_string() {
    let ir = lower_ok(RIG);
    let mode = &ir.values[0];
    assert_eq!(mode.name, "bus.mode");
    assert_eq!(mode.ty, "String", "enum is a variant-constrained string");
    assert_eq!(mode.default.as_deref(), Some("\"DC\""));
}

#[test]
fn combo_and_radio_carry_variants() {
    let ir = lower_ok(RIG);
    let IrWidget::Container { children, .. } = &ir.instances[0].widgets[0] else {
        panic!("column");
    };
    for widget in children {
        let IrWidget::Primitive {
            kind,
            options,
            target,
            ..
        } = widget
        else {
            panic!("primitive");
        };
        assert!(kind == "combo" || kind == "radio");
        assert_eq!(options, &["AC", "DC"], "{kind} offers the enum variants");
        assert!(matches!(target, IrWidgetTarget::Write { value } if value == "bus.mode"));
    }
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
fn default_must_be_a_variant() {
    expect_error(
        r#"
app A {
    enum Mode { AC, DC }
    interface B { mode : Mode = PWM  modport ui (out mode) }
    @wiring { let b = B(); }
}
"#,
        "not a variant of enum `Mode`",
    );
}

#[test]
fn combo_needs_an_enum_field() {
    expect_error(
        r#"
app A {
    interface B { level : f32 = 0.0  modport ui (out level) }
    citizen P (b : B.ui) {
        column { combo "x" <-> b.level; }
    }
    @wiring { let b = B(); let p = P(b = b.ui); }
    @layout { dock(p, region = center); }
}
"#,
        "needs an enum field",
    );
}

#[test]
fn mixed_array_enum_and_scalar_interface() {
    // The rig command panel: per-channel array + global enum + scalars.
    let ir = lower_ok(
        r#"
app Rig {
    enum Mode { AC, DC }
    interface Bus {
        duty    : [f32; 4] = {0.0}
        mode    : Mode = AC
        freq_hz : f32 = 50.0
        pwm_en  : bool = false
        modport ctl (out duty, out mode, out freq_hz, out pwm_en)
    }
    citizen Cmd (b : Bus.ctl) {
        column {
            combo "Mode" <-> b.mode;
            slider "Hz" 1.0..400.0 <-> b.freq_hz;
            checkbox "PWM" <-> b.pwm_en;
            for ch in 0..4 { slider "Ch {ch}" 0.0..100.0 <-> b.duty[ch]; }
        }
    }
    @wiring { let bus = Bus(); let cmd = Cmd(b = bus.ctl); }
    @layout { dock(cmd, region = center); }
}
"#,
    );
    // 4 duty + mode + freq_hz + pwm_en
    assert_eq!(ir.values.len(), 7);
    assert_eq!(
        ir.values.iter().find(|v| v.name == "bus.mode").unwrap().ty,
        "String"
    );
    assert_eq!(
        ir.values
            .iter()
            .filter(|v| v.name.starts_with("bus.duty."))
            .count(),
        4
    );
}
