//! Array state + generate loops: elaboration must expand arrays into
//! per-element values and unroll `for` loops into concrete, indexed widgets.

use mobius_lang::ir::{InstanceKind, IrWidget, IrWidgetTarget};
use mobius_lang::{Registry, lower, parse};

const FANS: &str = r#"
app FanBench {
    interface Bench {
        selected : [bool; 6] = {false}
        rpm      : [f32; 3]  = {10.0, 20.0, 30.0}
        modport ui (out selected, in rpm)
    }
    citizen SignalSelect (b : Bench.ui) {
        column {
            for i in 0..3 {
                row {
                    checkbox "Fan {i}" <-> b.selected[i];
                    label    "rpm"     <-  b.rpm[i];
                }
            }
        }
    }
    @wiring {
        let bench  = Bench();
        let select = SignalSelect(b = bench.ui);
    }
    @layout {
        dock(select, region = center);
    }
}
"#;

fn lower_ok(source: &str) -> mobius_lang::Ir {
    let (file, _) = parse(source).unwrap();
    lower(&file.apps[0], &Registry::new()).expect("must lower")
}

#[test]
fn array_expands_to_per_element_values() {
    let ir = lower_ok(FANS);
    // 6 selected + 3 rpm
    assert_eq!(ir.values.len(), 9);
    assert_eq!(ir.values[0].name, "bench.selected.0");
    assert_eq!(ir.values[5].name, "bench.selected.5");
    assert!(
        ir.values[0..6]
            .iter()
            .all(|v| v.default.as_deref() == Some("false"))
    );

    // Explicit per-element defaults.
    let rpm: Vec<_> = ir
        .values
        .iter()
        .filter(|v| v.name.starts_with("bench.rpm."))
        .map(|v| v.default.clone().unwrap())
        .collect();
    assert_eq!(rpm, ["10.0", "20.0", "30.0"]);
}

#[test]
fn out_view_writes_every_element() {
    let ir = lower_ok(FANS);
    // Each `selected` element has the citizen as its single writer.
    for v in ir
        .values
        .iter()
        .filter(|v| v.name.starts_with("bench.selected."))
    {
        assert!(v.writer.is_some(), "{} has no writer", v.name);
    }
}

#[test]
fn for_loop_unrolls_into_indexed_widgets() {
    let ir = lower_ok(FANS);
    let widgets = &ir.instances[0].widgets;
    assert_eq!(ir.instances[0].kind, InstanceKind::Source);

    // Outer column, then one row per loop pass (3), each with 2 widgets.
    let IrWidget::Container { kind, children } = &widgets[0] else {
        panic!("expected column");
    };
    assert_eq!(kind, "column");
    assert_eq!(children.len(), 3, "for i in 0..3 → 3 rows");

    // Row 2 (i = 2): checkbox bound to selected.2, label to rpm.2.
    let IrWidget::Container { children: row, .. } = &children[2] else {
        panic!("expected row");
    };
    let IrWidget::Primitive { label, target, .. } = &row[0] else {
        panic!()
    };
    assert_eq!(label.as_deref(), Some("Fan 2"), "{{i}} interpolated");
    assert!(matches!(target, IrWidgetTarget::Write { value } if value == "bench.selected.2"));
    let IrWidget::Primitive { target, .. } = &row[1] else {
        panic!()
    };
    assert!(matches!(target, IrWidgetTarget::Read { value } if value == "bench.rpm.2"));
}

fn expect_error(source: &str, needle: &str) {
    let (file, _) = parse(source).unwrap();
    let diagnostics = lower(&file.apps[0], &Registry::new()).unwrap_err();
    assert!(
        diagnostics.iter().any(|d| d.message.contains(needle)),
        "expected an error containing {needle:?}, got: {:#?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn loop_out_of_array_bounds_is_an_error() {
    expect_error(
        r#"
app A {
    interface S { xs : [bool; 4] = {false}  modport ui (out xs) }
    citizen P (s : S.ui) {
        column { for i in 0..6 { checkbox "x" <-> s.xs[i]; } }
    }
    @wiring { let s = S(); let p = P(s = s.ui); }
    @layout { dock(p, region = center); }
}
"#,
        "out of bounds",
    );
}

#[test]
fn indexing_a_scalar_is_an_error() {
    expect_error(
        r#"
app A {
    interface S { on : bool = false  modport ui (out on) }
    citizen P (s : S.ui) {
        column { for i in 0..2 { checkbox "x" <-> s.on[i]; } }
    }
    @wiring { let s = S(); let p = P(s = s.ui); }
    @layout { dock(p, region = center); }
}
"#,
        "not an array",
    );
}

#[test]
fn unindexed_array_is_an_error() {
    expect_error(
        r#"
app A {
    interface S { xs : [bool; 2] = {false}  modport ui (out xs) }
    citizen P (s : S.ui) {
        column { checkbox "x" <-> s.xs; }
    }
    @wiring { let s = S(); let p = P(s = s.ui); }
    @layout { dock(p, region = center); }
}
"#,
        "must be indexed",
    );
}

#[test]
fn wrong_initializer_count_is_an_error() {
    expect_error(
        r#"
app A {
    interface S { xs : [bool; 3] = {true, false}  modport ui (out xs) }
    @wiring { let s = S(); }
}
"#,
        "initializers but length",
    );
}

#[test]
fn scroll_container_wraps_a_channel_list() {
    let ir = lower_ok(
        r#"
app Rig {
    interface Bus {
        duty : [f32; 32] = {0.0}
        modport ctl (out duty)
    }
    citizen DutyCmd (b : Bus.ctl) {
        scroll {
            column {
                for ch in 0..32 { slider "Ch {ch}" 0.0..100.0 <-> b.duty[ch]; }
            }
        }
    }
    @wiring { let bus = Bus(); let cmd = DutyCmd(b = bus.ctl); }
    @layout { dock(cmd, region = center); }
}
"#,
    );
    // 32 channels expanded.
    assert_eq!(ir.values.len(), 32);
    // scroll → column → 32 sliders.
    let IrWidget::Container { kind, children } = &ir.instances[0].widgets[0] else {
        panic!("scroll");
    };
    assert_eq!(kind, "scroll");
    let IrWidget::Container { kind, children } = &children[0] else {
        panic!("column");
    };
    assert_eq!(kind, "column");
    assert_eq!(children.len(), 32);
}
