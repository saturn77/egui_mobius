//! Lowering the baseline application must produce the expected netlist,
//! and the elaboration checks must catch structural mistakes with spans.

use mobius_lang::ir::*;
use mobius_lang::{Registry, lower, parse};

const BASELINE: &str = include_str!("../examples/signal_bench.mobius");

fn registry() -> Registry {
    Registry::new()
        .citizen("PlotPanel")
        .citizen("LensLogger")
        .handler("bench_worker")
        .event("BenchCmd")
}

fn baseline_ir() -> Ir {
    let (file, _) = parse(BASELINE).unwrap();
    lower(&file.apps[0], &registry()).expect("baseline must lower cleanly")
}

#[test]
fn values_writers_and_readers() {
    let ir = baseline_ir();
    assert_eq!(ir.values.len(), 3);

    let amplitude = &ir.values[0];
    assert_eq!(amplitude.name, "bench.amplitude");
    assert_eq!(amplitude.ty, "f32");
    assert_eq!(amplitude.default.as_deref(), Some("1.0"));
    // ControlsPanel (i0) writes amplitude; plot, log, and the handler read it.
    assert_eq!(amplitude.writer, Some(Party::Instance(0)));
    assert_eq!(amplitude.readers.len(), 3);

    let trace = &ir.values[2];
    assert_eq!(trace.name, "bench.trace");
    assert_eq!(trace.ty, "Vec<f32>");
    // The handler is the only writer of trace.
    assert_eq!(trace.writer, Some(Party::Handler(0)));
}

#[test]
fn signal_routes_from_modports() {
    let ir = baseline_ir();
    assert_eq!(ir.signals.len(), 1);
    let commands = &ir.signals[0];
    assert_eq!(commands.name, "bench.commands");
    assert_eq!(commands.ty, "BenchCmd");
    assert_eq!(commands.emitters, vec![Party::Instance(0)]);
    assert_eq!(commands.drainer, Some(Party::Handler(0)));
    assert_eq!(ir.handlers[0].drains, vec![0]);
}

#[test]
fn instances_and_layout() {
    let ir = baseline_ir();
    assert_eq!(ir.instances.len(), 3);
    assert_eq!(ir.instances[0].ty, "ControlsPanel");
    assert_eq!(ir.instances[0].kind, InstanceKind::Source);
    assert_eq!(ir.instances[1].ty, "PlotPanel");
    assert_eq!(ir.instances[1].kind, InstanceKind::Registry);

    assert_eq!(ir.layout.len(), 3);
    assert_eq!(ir.layout[0].region, "center");
    assert_eq!(ir.layout[1].fraction, Some(0.25));
}

#[test]
fn emit_text_is_stable() {
    let text = baseline_ir().emit_text();
    let expected = "\
app      SignalBench
value    v0  bench.amplitude : f32      = 1.0  writer=i0
value    v1  bench.enabled   : bool     = true  writer=i0
value    v2  bench.trace     : Vec<f32> = []  writer=h0
instance i0  controls : ControlsPanel (source)  bench.amplitude=v0(rw) bench.enabled=v1(rw) bench.trace=v2(ro)
instance i1  plot : PlotPanel (registry)  bench.amplitude=v0(ro) bench.enabled=v1(ro) bench.trace=v2(ro)
instance i2  log : LensLogger (registry)  bench.amplitude=v0(ro) bench.enabled=v1(ro) bench.trace=v2(ro)
signal   s0  bench.commands : BenchCmd  emit=i0  drain=h0
handler  h0  \"bench_worker\"  bench.amplitude=v0(ro) bench.trace=v2(rw)
layout   dock i1 region=center
layout   dock i0 region=left fraction=0.25
layout   dock i2 region=below fraction=0.2
";
    assert_eq!(text, expected);
}

#[test]
fn one_writer_rule_reports_both_sites() {
    let source = r#"
app TwoWriters {
    interface S {
        level : f32 = 0.0
        modport a (out level)
        modport b (out level)
    }
    citizen P (s : S.a) {
        column { slider "L" 0.0..1.0 <-> s.level; }
    }
    citizen Q (s : S.b) {
        column { slider "L" 0.0..1.0 <-> s.level; }
    }
    @wiring {
        let s = S();
        let p = P(s = s.a);
        let q = Q(s = s.b);
    }
    @layout {
        dock(p, region = center);
    }
}
"#;
    let (file, _) = parse(source).unwrap();
    let diagnostics = lower(&file.apps[0], &registry()).unwrap_err();
    let one_writer = diagnostics
        .iter()
        .find(|d| d.message.contains("one-writer rule"))
        .expect("must flag the second writer");
    assert!(!one_writer.related.is_empty(), "must point at both sites");
}

#[test]
fn dangling_signal_is_an_error() {
    let source = r#"
app Dangling {
    interface S {
        level : f32 = 0.0
        go : signal BenchCmd
        modport ui (out level, emit go)
    }
    citizen P (s : S.ui) {
        column { button "Go" -> s.go.send(BenchCmd::Start); }
    }
    @wiring {
        let s = S();
        let p = P(s = s.ui);
    }
    @layout {
        dock(p, region = center);
    }
}
"#;
    let (file, _) = parse(source).unwrap();
    let diagnostics = lower(&file.apps[0], &registry()).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("dangling signal"))
    );
}

#[test]
fn wrong_verb_for_arrow_is_an_error() {
    let source = r#"
app WrongVerb {
    interface S {
        level : f32 = 0.0
        modport viewer (in level)
    }
    citizen P (s : S.viewer) {
        column { slider "L" 0.0..1.0 <-> s.level; }
    }
    @wiring {
        let s = S();
        let p = P(s = s.viewer);
    }
    @layout {
        dock(p, region = center);
    }
}
"#;
    let (file, _) = parse(source).unwrap();
    let diagnostics = lower(&file.apps[0], &registry()).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("`<->` needs `out`"))
    );
}

#[test]
fn unknown_citizen_type_is_an_error() {
    let source = r#"
app Unknown {
    interface S {
        level : f32 = 0.0
        modport ui (out level)
    }
    @wiring {
        let s = S();
        let p = Mystery(s = s.ui);
    }
    @layout {
        dock(p, region = center);
    }
}
"#;
    let (file, _) = parse(source).unwrap();
    let diagnostics = lower(&file.apps[0], &registry()).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("unknown citizen type `Mystery`"))
    );
}

#[test]
fn modport_verb_must_match_field_kind() {
    let source = r#"
app Mismatch {
    interface S {
        level : f32 = 0.0
        go : signal BenchCmd
        modport bad (emit level, out go)
    }
    @wiring {
        let s = S();
    }
}
"#;
    let (file, _) = parse(source).unwrap();
    let diagnostics = lower(&file.apps[0], &registry()).unwrap_err();
    assert_eq!(
        diagnostics
            .iter()
            .filter(|d| d.message.contains("does not apply to field"))
            .count(),
        2
    );
}
