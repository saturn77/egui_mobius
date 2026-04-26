# egui-citizen — Task Progress

## 2026-04-04
- Created standalone egui-citizen workspace repo
- Migrated egui_citizen crate from egui_mobius feature-egui-citizen branch
- Migrated citizen_dock basic example
- Scaffolded serial_plotter example (TBD implementation)
- Created develop/ directory with task_plan, task_progress, task_findings
- Origin: saturn-grid-sim tab-switching bug exposed need for panel lifecycle management

## 2026-04-25 — Book scaffolding + first content pass
- Scaffolded `book/` as mdBook (book.toml, SUMMARY.md, .gitignore for
  `book/book/`). Targets GitHub Pages via `mdbook build`.
- Drafted **content-complete** chapters (the four highest-value pages):
  - `book/src/introduction.md` — framework framing anchored in egui's
    own README ("egui is *not* a framework…"), Qt Advanced Docking
    System parallel for `egui_dock`, three-term vocabulary
    (citizen-panel, atom, `Dynamic<T>`), and a deep dive on `Dynamic<T>`
    (internal `Arc<Mutex<T>>` shape, API, `on_change`,
    permissive-type/disciplined-use framing, Clone-semantics aside).
  - `book/src/concepts/state.md` — reactive lifecycle and the trap
    where `CitizenState::default()` severs reactivity.
  - `book/src/concepts/coupling.md` — the two coupling paths (UI↔UI
    via shared `Dynamic<T>` vs UI↔backend via dispatcher messages),
    dual-wired atoms, source-of-truth discipline, timing asymmetry.
  - `book/src/patterns/state-shape.md` — three-struct model
    (`CitizenState` / `PanelState` / app-shared services), with the
    `PanelState` convention surfaced as a named pattern.
- Created stubs for the remaining first-cut chapters: `problem`,
  `citizen`, `dispatcher`, `messages`, `tutorial/{first-citizen,
  with-egui-dock, two-panels}`, `patterns/stored-vs-stateless`,
  `pitfalls`, `reference`.
- **API change driven by writing the book:** renamed
  `Citizen::state()` → `citizen_state()` and `state_mut()` →
  `citizen_state_mut()` across the trait def, default impls, both
  rustdoc snippets, and `examples/getting_started`. Workspace
  compiles clean. Frees up `panel_state()` as a parallel sibling if a
  `PanelState` associated type is added later.
- Saved persistent memory entries (user role, book scaffolding state,
  primary-source-grounding feedback) so future sessions resume cleanly.
- Added **`book/src/concepts/inside-dynamic.md`** as a full chapter on
  `Dynamic<T>` internals — the `Arc<parking_lot::Mutex<Vec<Sender<()>>>>`
  notifier subsystem. Covers: outside-in struct breakdown, why
  `parking_lot::Mutex` (not std), the doorbell pattern (`Sender<()>`),
  exact `set()` and `on_change` mechanics with a producer/consumer
  diagram, and five practical implications (one OS thread per
  subscriber, no unsubscribe API, no coalescing, off-UI-thread
  callbacks, lock contention). Closes with a three-row decision
  table: `.get()` polling vs `dispatcher.send` vs
  `Dynamic::on_change`. Cross-linked from the introduction's
  "Observing changes" subsection and from `coupling.md`'s Path B
  aside.

## 2026-04-25 (continued) — Remaining first-cut chapters + Pages CI

- Filled six stub chapters to content-complete: `concepts/problem`,
  `concepts/citizen`, `concepts/dispatcher`, `concepts/messages`,
  `patterns/stored-vs-stateless`, `pitfalls`.
- Wrote `book/src/reference.md` as a full single-page cheat sheet
  (Dispatcher table, Citizen trait table, CitizenState fields,
  CitizenMessage variants, CitizenId, common idioms).
- `problem.md` framing crystallized in a 2026-04-13 Discord exchange
  with Adanos020 (egui_dock maintainer): the
  on_tab_button-name-obscures-role observation. Translated into
  book voice; the verbatim Discord excerpt deliberately not quoted
  in the published doc.
- Added `Basic_App_State.drawio` diagram to
  `concepts/dispatcher.md` — every panel's `CitizenState` arrow
  points at the central dispatcher block (registration topology).
- Updated `concepts/messages.md` "Wrapping in your own AppMessage"
  section with a CopperForge-shaped 12-variant example,
  intent-vs-outcome framing (`DrcRunRequested` →
  `DrcCompleted`), section-divider-via-comments pattern, and
  sub-domain nesting (`HotkeyPressed(Hotkey)`).
- Added `.github/workflows/book.yml` — mdBook build to GitHub Pages
  on push to `master`, manual `workflow_dispatch` enabled. Requires
  Pages to be enabled in repo settings (Source: GitHub Actions)
  before first deploy. Target URL is
  `https://saturn77.github.io/egui-citizen/`.
- Added README book link: `[![Book]…]` badge in the header row plus
  a prominent blockquote callout linking to
  `book/src/introduction.md`. Works on GitHub today via the markdown
  renderer; repointable at the github.io URL once Pages deploys.
- **Status:** 13/13 first-cut chapters drafted (three tutorial
  chapters intentionally remain as stubs — `examples/` covers the
  hands-on path; tutorials can lag the conceptual material without
  blocking a soft launch).
