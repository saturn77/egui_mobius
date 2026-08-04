//! The full pipeline against real reactive machinery: parse → lower →
//! wire, then drive the wired app the way a UI host and a backend thread
//! would. No egui — the fabric is testable headless.

use mobius_lang::{Host, Registry, lower, parse, wire};

const BASELINE: &str = include_str!("../examples/signal_bench.mobius");

#[derive(Debug, PartialEq)]
enum BenchCmd {
    Apply,
}

fn wired() -> mobius_lang::WiredApp {
    let registry = Registry::new()
        .citizen("PlotPanel")
        .citizen("LensLogger")
        .handler("bench_worker")
        .event("BenchCmd");
    let (file, _) = parse(BASELINE).unwrap();
    let ir = lower(&file.apps[0], &registry).unwrap();
    wire(&ir, &Host::new()).expect("baseline must wire")
}

#[test]
fn values_have_defaults_and_shared_storage() {
    let app = wired();
    let amplitude = app.value::<f32>("bench.amplitude").unwrap();
    assert_eq!(amplitude.get(), 1.0);
    assert!(app.value::<bool>("bench.enabled").unwrap().get());
    assert!(
        app.value::<Vec<f32>>("bench.trace")
            .unwrap()
            .get()
            .is_empty()
    );

    // The controls instance's writable clone shares storage with the
    // app-level handle — clones are the same underlying Dynamic.
    let controls = &app.instances[0];
    let knob = controls.bindings.write::<f32>("bench.amplitude").unwrap();
    knob.set(2.5);
    assert_eq!(amplitude.get(), 2.5);
}

#[test]
fn read_bindings_are_observe_only_views() {
    let app = wired();
    let plot = &app.instances[1];
    // Plot has read access to amplitude, no write access.
    assert!(plot.bindings.read::<f32>("bench.amplitude").is_some());
    assert!(plot.bindings.write::<f32>("bench.amplitude").is_none());

    // A wrong-type read fails the checked downcast rather than lying.
    assert!(plot.bindings.read::<bool>("bench.amplitude").is_none());
}

#[test]
fn intent_roundtrip_through_backend_thread() {
    let app = wired();

    // UI side: the controls citizen fires its Apply button twice.
    let controls = &app.instances[0];
    let emit = controls.bindings.emit("bench.commands").unwrap();
    emit.send(BenchCmd::Apply);
    emit.send(BenchCmd::Apply);

    // Backend side: the handler drains commands, computes, and answers
    // through `trace` — its only writable value — from a real thread.
    let handler = &app.handlers[0];
    assert_eq!(handler.name, "bench_worker");
    let commands = handler.drains.first().unwrap();
    let events = commands.drain::<BenchCmd>();
    assert_eq!(events, vec![BenchCmd::Apply, BenchCmd::Apply]);

    let amplitude = handler.bindings.read::<f32>("bench.amplitude").unwrap();
    let trace = handler.bindings.write::<Vec<f32>>("bench.trace").unwrap();
    let worker = std::thread::spawn(move || {
        let amp = amplitude.get();
        trace.set(vec![0.0, amp, 0.0]);
    });
    worker.join().unwrap();

    // UI side again: the plot's observe-only view sees the result. The
    // return path is state — no response message existed anywhere.
    let plot = &app.instances[1];
    let seen = plot.bindings.read::<Vec<f32>>("bench.trace").unwrap();
    assert_eq!(seen.get(), vec![0.0, 1.0, 0.0]);
}

#[test]
fn layout_passes_through() {
    let app = wired();
    assert_eq!(app.layout.len(), 3);
    assert_eq!(app.layout[0].region, "center");
    assert_eq!(app.layout[1].fraction, Some(0.25));
}

#[test]
fn unregistered_value_type_is_a_host_error() {
    let source = r#"
app NeedsScene {
    interface S {
        scene : Scene
        modport ui (out scene)
    }
    @wiring {
        let s = S();
    }
}
"#;
    let registry = Registry::new();
    let (file, _) = parse(source).unwrap();
    let ir = lower(&file.apps[0], &registry).unwrap();
    let error = wire(&ir, &Host::new()).unwrap_err();
    assert!(error.message.contains("`Scene` is not registered"));
    assert_eq!(
        error.span.line, 4,
        "error points at the field's source line"
    );
}

#[test]
fn host_registers_custom_value_types() {
    #[derive(Clone, Default, PartialEq, Debug)]
    struct Scene {
        nodes: usize,
    }

    let source = r#"
app NeedsScene {
    interface S {
        scene : Scene
        modport ui (out scene)
    }
    @wiring {
        let s = S();
    }
}
"#;
    let registry = Registry::new();
    let (file, _) = parse(source).unwrap();
    let ir = lower(&file.apps[0], &registry).unwrap();
    let host = Host::new().value_type::<Scene, _>("Scene", |_| None);
    let app = wire(&ir, &host).unwrap();
    assert_eq!(
        app.value::<Scene>("s.scene").unwrap().get(),
        Scene::default()
    );
}
