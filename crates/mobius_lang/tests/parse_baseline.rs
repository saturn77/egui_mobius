//! The baseline application from the language spec must parse completely and
//! produce the expected structure.

use mobius_lang::ast::*;
use mobius_lang::parse;

const BASELINE: &str = include_str!("../examples/signal_bench.mobius");

fn baseline_app() -> App {
    let (file, _) = parse(BASELINE).expect("baseline must parse");
    assert_eq!(file.apps.len(), 1);
    file.apps.into_iter().next().unwrap()
}

#[test]
fn app_name_and_item_shape() {
    let app = baseline_app();
    assert_eq!(app.name, "SignalBench");
    assert_eq!(app.items.len(), 4); // interface, citizen, @wiring, @layout
}

#[test]
fn interface_carries_state_and_signal_fields() {
    let app = baseline_app();
    let interface = app
        .items
        .iter()
        .find_map(|item| match item {
            AppItem::Interface(interface) => Some(interface),
            _ => None,
        })
        .expect("Bench interface");
    assert_eq!(interface.name, "Bench");
    assert_eq!(interface.fields.len(), 4);

    let state_names: Vec<_> = interface
        .fields
        .iter()
        .filter(|f| matches!(f.kind, FieldKind::State { .. }))
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(state_names, ["amplitude", "enabled", "trace"]);

    let signal = interface
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Signal { .. }))
        .expect("signal field");
    assert_eq!(signal.name, "commands");
    let FieldKind::Signal { ty } = &signal.kind else {
        unreachable!()
    };
    assert_eq!(ty.path.segments, ["BenchCmd"]);

    // trace : Vec<f32> = []
    let trace = &interface.fields[2];
    let FieldKind::State { ty, default } = &trace.kind else {
        panic!("trace must be state")
    };
    assert_eq!(ty.path.segments, ["Vec"]);
    assert_eq!(ty.params[0].path.segments, ["f32"]);
    assert!(matches!(default, Some(Expr::Array(items, _)) if items.is_empty()));
}

#[test]
fn modports_carry_all_four_verbs() {
    let app = baseline_app();
    let interface = app
        .items
        .iter()
        .find_map(|item| match item {
            AppItem::Interface(interface) => Some(interface),
            _ => None,
        })
        .unwrap();
    assert_eq!(interface.modports.len(), 3);

    let controls = &interface.modports[0];
    assert_eq!(controls.name, "controls");
    let verbs: Vec<_> = controls.entries.iter().map(|e| e.verb).collect();
    assert_eq!(verbs, [Verb::Out, Verb::Out, Verb::In, Verb::Emit]);
    assert_eq!(controls.entries[3].field, "commands");

    let backend = &interface.modports[2];
    assert_eq!(backend.name, "backend");
    let verbs: Vec<_> = backend.entries.iter().map(|e| e.verb).collect();
    assert_eq!(verbs, [Verb::In, Verb::Out, Verb::Drain]);
}

#[test]
fn primitives_citizen_bindings() {
    let app = baseline_app();
    let citizen = app
        .items
        .iter()
        .find_map(|item| match item {
            AppItem::Citizen(citizen) => Some(citizen),
            _ => None,
        })
        .expect("ControlsPanel");
    assert_eq!(citizen.name, "ControlsPanel");
    assert_eq!(citizen.ports.len(), 1);
    assert_eq!(citizen.ports[0].ty.path.segments, ["Bench", "controls"]);

    let WidgetNode::Container { kind, children, .. } = &citizen.body[0] else {
        panic!("expected column container")
    };
    assert_eq!(kind, "column");
    assert_eq!(children.len(), 3);

    let WidgetNode::Primitive(checkbox) = &children[0] else {
        panic!()
    };
    assert_eq!(checkbox.kind, "checkbox");
    assert_eq!(checkbox.label.as_deref(), Some("Enabled"));
    assert!(matches!(&checkbox.binding, Binding::TwoWay(path)
        if path.segments == ["bench", "enabled"]));

    let WidgetNode::Primitive(slider) = &children[1] else {
        panic!()
    };
    assert!(matches!(
        slider.range,
        Some((Expr::Float(lo, _), Expr::Float(hi, _))) if lo == 0.0 && hi == 10.0
    ));

    let WidgetNode::Primitive(button) = &children[2] else {
        panic!()
    };
    let Binding::Event(Expr::Call { path, args, .. }) = &button.binding else {
        panic!("button must fire an event call")
    };
    assert_eq!(path.segments, ["bench", "commands", "send"]);
    assert!(matches!(&args[0], Expr::Path(p) if p.segments == ["BenchCmd", "Apply"]));
}

#[test]
fn wiring_instances_and_bind() {
    let app = baseline_app();
    let wiring = app
        .items
        .iter()
        .find_map(|item| match item {
            AppItem::Section(section) if section.name == "wiring" => Some(section),
            _ => None,
        })
        .expect("@wiring section");
    assert_eq!(wiring.stmts.len(), 5);

    let Stmt::Instance { name, ty, args, .. } = &wiring.stmts[1] else {
        panic!("expected controls instance")
    };
    assert_eq!(name, "controls");
    assert_eq!(ty.segments, ["ControlsPanel"]);
    assert_eq!(args[0].name, "bench");
    assert!(matches!(&args[0].value, Expr::Path(p)
        if p.segments == ["bench", "controls"]));

    let Stmt::Bind { handler, view, .. } = &wiring.stmts[4] else {
        panic!("expected bind statement")
    };
    assert_eq!(handler, "bench_worker");
    assert_eq!(view.segments, ["bench", "backend"]);
}

#[test]
fn layout_directives() {
    let app = baseline_app();
    let layout = app
        .items
        .iter()
        .find_map(|item| match item {
            AppItem::Section(section) if section.name == "layout" => Some(section),
            _ => None,
        })
        .expect("@layout section");
    assert_eq!(layout.stmts.len(), 3);

    let Stmt::Directive { name, args, .. } = &layout.stmts[1] else {
        panic!("expected dock directive")
    };
    assert_eq!(name, "dock");
    assert!(matches!(&args[0], DirectiveArg::Positional(Expr::Path(p))
        if p.segments == ["controls"]));
    let DirectiveArg::Named(region) = &args[1] else {
        panic!()
    };
    assert_eq!(region.name, "region");
    let DirectiveArg::Named(fraction) = &args[2] else {
        panic!()
    };
    assert!(matches!(fraction.value, Expr::Float(f, _) if f == 0.25));
}

#[test]
fn comments_survive_lexing() {
    let (_, comments) = parse(BASELINE).unwrap();
    assert!(
        comments
            .iter()
            .any(|c| c.text.contains("modports: each party's complete view"))
    );
}

#[test]
fn parse_errors_point_at_source() {
    let err = parse("app Broken {\n    let x = ;\n}").unwrap_err();
    assert_eq!(err.span.line, 2);
    assert!(err.message.contains("expected"));
}
