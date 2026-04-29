# Progress — citizen_signal_async

Build checklist. Update as work proceeds. Each phase should leave the
crate in a buildable state — `cargo check -p citizen_signal_async`
clean before moving on.

## Phase 0 — scaffold (this commit)

- [x] Crate directory created at `examples/citizen_signal_async/`
- [x] `Cargo.toml` with workspace deps (egui_mobius,
      egui_mobius_reactive, egui_citizen, egui, egui_dock, eframe,
      tokio)
- [x] Stub `src/main.rs` (placeholder `fn main()`)
- [x] `task_plan.md`, `task_findings.md`, `task_progress.md` written
- [x] Added to root `Cargo.toml` workspace `members`
- [x] `cargo check -p citizen_signal_async` clean

## Phase 1 — types + state + dispatcher routing

- [x] `src/state.rs` — `SharedState`, `ParamsState`, `WorkRequest`, `WorkResponse`
- [x] `src/messages.rs` — `AppMessage` enum
- [x] `src/dispatcher.rs` — `register_citizens`, `drain_citizen`,
      `handle()`, `append_log()`, `format_citizen()`. Citizen IDs
      live here for now; will move to `tabs.rs` in Phase 3.
- [x] `cargo check` clean (17 unused-item warnings; expected until
      Phase 2 wires the bus and Phase 3 builds the panels)
- [x] Minimal eframe app shell in `main.rs` (CentralPanel with sliders,
      Compute button, in-flight spinner, last-result label, log
      scrollback) — folded into Phase 2 since the shell and the
      backend wiring are tested together

## Phase 2 — backend wiring + smoke-test shell

- [x] `src/backend.rs` — `wire_backend()` builds the `AsyncDispatcher`,
      both signal/slot pairs, attaches the async `work()` function;
      returns `Signal<WorkRequest>`, `Slot<WorkResponse>`,
      `BackendHandle` (keepalive for the Tokio runtime)
- [x] Result slot handler in `App::new` writes `last_result`,
      `in_flight`, log line; calls `ctx.request_repaint()` so the UI
      paints the new value next frame
- [x] AppMessage drain in `update()` — `outbox` filled by Compute
      button, drained by `dispatcher::handle`, which forwards to
      `work_signal.send(req)`
- [x] `cargo check -p citizen_signal_async` clean (2 unused-item
      warnings; expected until Phase 3 wires panels through `AppMessage`)
- [ ] **Manual smoke test (user)**: `cargo run -p citizen_signal_async`,
      move sliders, click Compute. Expect log lines: `[ui] submit:`,
      `[backend] result:`, and `last result:` updating after the
      slider's duration. Spinner shown while in-flight.

## Phase 3 — citizen panels + dock layout

- [x] `src/panels/mod.rs` — module root
- [x] `src/panels/control.rs` — duration / seed sliders + Compute
      button → `outbox.push(AppMessage::Compute)`
- [x] `src/panels/result.rs` — last_result heading + in_flight spinner
- [x] `src/panels/logger.rs` — log scrollback (mirrors filter_plotter)
- [x] `src/tabs.rs` — `TabKind`, `Tab`, `TabViewer`. Citizen IDs moved
      here from `dispatcher.rs` (their canonical home alongside
      `TabKind`).
- [x] `src/main.rs` rewritten: `DockArea` + `TabViewer`, layout is
      `Result | (Control / Logger)`. Drain loop unchanged.
- [x] `cargo check` clean (4 unused-item warnings on panel
      `citizen_id` / `citizen_state` fields — same warnings
      filter_plotter has; convention is to leave them in)
- [ ] **Manual smoke test (user)**: dock layout opens, Compute on the
      Control panel updates the Result panel, tab clicks log
      `[citizen] X activated/deactivated` lines in the Logger.

## Phase 4 — citizen activation logging (already done in Phase 3)

- [x] Tab clicks call `citizen_dispatcher.activate(&id)` (via
      `TabViewer::on_tab_button`)
- [x] `drain_messages()` appended to `state.log` so Logger shows both
      backend events and citizen lifecycle events

## Phase 5 — polish

- [ ] README.md inside the example explaining what to look at
- [ ] Reference this example from the book's existing
      "signals and slots can be employed by the dispatcher"
      paragraph
- [ ] Add to `examples/README.md` under the citizen pattern section

## Notes / decisions made along the way

(Append dated notes here as design choices come up. Keep terse.)
