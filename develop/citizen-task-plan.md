# egui-citizen — Task Plan

## Phase 1: Foundation (current)
- [x] Core crate: Citizen trait, CitizenState, CitizenMessage, CitizenRegistry
- [x] Basic example: citizen_dock (three algo tabs + reactive plot + logger)
- [x] Standalone workspace repo with crates/ and examples/ structure
- [x] Citizen trait rename: `state()` → `citizen_state()` (book-driven,
      makes room for parallel `panel_state()` if PanelState becomes
      first-class)
- [ ] Decide: keep egui_mobius_reactive dependency or reimplement Dynamic<T> natively
- [ ] Root README — dense, emphasizing panels, threading, message coordination, dispatcher, real-world use
- [ ] LICENSE file
- [ ] Git init + initial commit

## Phase 1.5: Book (mdBook, in progress)
Plan in `develop/book-plan.md`. Scaffold and the highest-value
chapters are drafted; rest are stubs awaiting prose.

- [x] mdBook scaffold: `book/book.toml`, `book/src/SUMMARY.md`,
      `book/.gitignore`, all chapter files created
- [x] `introduction.md` — content-complete (framework framing, egui_dock
      / Qt ADS parallel, vocabulary, full Dynamic<T> deep dive)
- [x] `concepts/state.md` — content-complete (reactive lifecycle)
- [x] `concepts/coupling.md` — content-complete (two coupling paths,
      dual-wired atoms, source-of-truth discipline)
- [x] `concepts/inside-dynamic.md` — content-complete (Dynamic<T>
      notifier subsystem internals, set/on_change mechanics, OS-thread
      cost, no-unsubscribe and no-coalescing implications)
- [x] `patterns/state-shape.md` — content-complete (three-struct model
      with PanelState convention)
- [x] `concepts/problem.md` — content-complete (per-frame ui() vs
      one-shot on_tab_button distinction; on_tab_button-name-obscures-
      role discoverability framing from 2026-04-13 Adanos020 exchange)
- [x] `concepts/citizen.md` — content-complete (trait surface,
      minimum-viable impl, CitizenId-as-const guidance)
- [x] `concepts/dispatcher.md` — content-complete (three jobs,
      eight-method API, one-dispatcher-per-app rule, registration
      topology diagram from Basic_App_State.drawio)
- [x] `concepts/messages.md` — content-complete (six variants,
      CopperForge-shaped AppMessage example, intent-vs-outcome,
      sub-domain nesting)
- [x] `tutorial/writing-a-citizen-app.md` — content-complete
      (replaces the three stubs first-citizen / with-egui-dock /
      two-panels with a single end-to-end walkthrough of the
      `examples/filter_plotter` biquad lowpass demo)
- [x] `patterns/stored-vs-stateless.md` — content-complete (both
      lawful forms with code, the CitizenState::default() trap, the
      CopperForge stored-vs-stateless split, decision rule)
- [x] `pitfalls.md` — content-complete (six items each with broken
      snippet + what-goes-wrong + fix)
- [x] `reference.md` — content-complete (single-page cheat sheet)
- [x] CI: GitHub Pages deploy via `mdbook build`
      (.github/workflows/book.yml; requires Pages to be enabled in
      repo settings before first deploy)
- [ ] Decide later whether to wire `mdbook test` against the live
      crate (would require `[dev-dependencies]` and converting many
      `rust,ignore` blocks to runnable)

## Phase 2: Serial Plotter Example
- [ ] Serial plotter with citizen-based dock layout:
  - Serial Config panel (citizen) — port selection, baud rate, connect/disconnect
  - Plot panel (citizen) — real-time scrolling waveform plot via egui_plot
  - Logger panel (citizen) — raw data stream + parsed messages
  - Settings panel (citizen) — plot config, sample rate, buffer depth
- [ ] Threaded serial I/O with crossbeam channels
- [ ] RP2350 embedded firmware (TBD):
  - USB serial CDC output
  - Configurable waveform generator (sine, square, triangle, noise)
  - CSV protocol: timestamp, channel, value
- [ ] README showing how to flash RP2350 and run the plotter

## Phase 3: Reactive Layer Decision
- [ ] Evaluate: keep egui_mobius_reactive as dep vs reimplement
  - Pro keep: proven, published, maintained
  - Pro reimplement: zero external deps, citizen owns its full stack
  - Middle ground: thin reactive crate under crates/egui_citizen_reactive
- [ ] If reimplementing, bring over Dynamic<T>, Derived<T>, SignalRegistry

## Phase 4: Dispatcher
- [ ] Formal dispatcher crate (crates/egui_citizen_dispatcher or similar)
- [ ] Typed message routing: citizen-to-citizen and citizen-to-backend
- [ ] Integration with threaded backends (serial, modbus, network)

## Future
- [ ] Citizen group support (multiple independent one-hot groups)
- [ ] Derive macro for Citizen trait boilerplate
- [ ] Persistence — save/restore citizen state and dock layout across sessions
- [ ] Template repo for cargo-generate
- [ ] **Lifecycle hooks** — have `Dispatcher::activate()` invoke
      `panel.on_activate()` (and `.on_deactivate()`, `.on_click()`)
      callbacks instead of writing `state.active.set(true)` directly.
      Currently the `Citizen` trait defines these hooks but the
      framework never calls them — they're sugar for panel code to
      invoke manually. Making the dispatcher call them turns the
      trait into a load-bearing part of the framework and gives panels
      a synchronous place to do side-effect work on activation
      (start a fetch, allocate a buffer, log) without having to detect
      state edges manually in `show()`. Implementation requires the
      dispatcher to hold panel handles (`Box<dyn Citizen>` or
      similar) — a real architectural change, not a refactor.
