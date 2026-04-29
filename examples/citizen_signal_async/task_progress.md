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
- [ ] Minimal eframe app shell in `main.rs` (no panels yet, just a
      central panel saying "ready")

## Phase 2 — backend wiring

- [ ] `src/backend.rs` — async work function + helper that builds the
      `AsyncDispatcher`, signal/slot pairs, and returns the handles
      the UI needs (`Signal<WorkRequest>`, `Slot<WorkResponse>`)
- [ ] Hook the result slot to write `last_result`, `in_flight`,
      `log` in `SharedState`
- [ ] Manual smoke test: send a synthetic `WorkRequest` from a button
      in the central panel; verify `last_result` updates after the
      sleep

## Phase 3 — citizen panels

- [ ] `src/panels/control.rs`
- [ ] `src/panels/result.rs` (with spinner on `in_flight`)
- [ ] `src/panels/logger.rs`
- [ ] `src/tabs.rs` — `TabKind`, `Tab`, `TabViewer`
- [ ] `src/main.rs` updated to dock layout + drain loop

## Phase 4 — citizen activation logging

- [ ] Tab clicks call `citizen_dispatcher.activate(&id)`
- [ ] `drain_messages()` appended to `state.log` so Logger shows
      both backend events and citizen lifecycle events

## Phase 5 — polish

- [ ] README.md inside the example explaining what to look at
- [ ] Reference this example from the book's existing
      "signals and slots can be employed by the dispatcher"
      paragraph
- [ ] Add to `examples/README.md` under the citizen pattern section

## Notes / decisions made along the way

(Append dated notes here as design choices come up. Keep terse.)
