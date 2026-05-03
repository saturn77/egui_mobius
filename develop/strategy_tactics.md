# egui_mobius — Strategy and Tactics

Drafted 2026-05-03. This document is the central "where are we going
and how do we get there" reference for the egui_mobius ecosystem.
It folds in the seven tactical priorities the maintainer identified,
the 90-day cadence tied to the No Starch publishing pitch, and the
strategic positioning that ties them together.

Read this whole document before starting any of the work in §4.

## 1. Why this exists

The ecosystem has reached a phase where forward momentum requires
coordination, not just execution. We have:

- Five crates in the workspace, each with their own download
  trajectory and architectural era.
- Two external crates (`egui_lens` and a planned `mobius-ecs`/
  designer integration) waiting to be folded in.
- A book that's increasingly the canonical reference but still
  needs expansion.
- A real production app (`saturn-grid-sim`) earning revenue and
  another (`CopperForge`) anchoring the public story.
- A 90-day window to ship 1.0 and pitch a print book to No Starch
  Press from a position of strength.

Each tactical item below earns its place because it serves *that*
strategic moment — building the case for "egui_mobius is the
serious choice for production Rust GUI development in 2026."

## 2. Where we are (state of the union, 2026-05-03)

### Workspace crates

| Crate | Total dl | Last 90d | Architectural era |
|---|---:|---:|---|
| `egui_mobius` (umbrella) | 10,205 | 356 | Original — signal/slot patterns |
| `egui_mobius_reactive` | 9,595 | 201 | Reactive primitives — modern |
| `egui_mobius_widgets` | 9,475 | 178 | Stateful widget toolkit — keep |
| `egui_mobius_components` | 683 | 18 | Predecessor logger — to deprecate |
| `egui_citizen` | not published | — | Modern citizen pattern |

### External

| Crate | Total dl | Last 90d | Status |
|---|---:|---:|---|
| `egui_lens` | 616 | 50 | Standalone — folding in (see lens-migration-plan.md) |
| `mobius-ecs` / designer | ? | ? | External — folding in (see §4.4) |

### Examples (14)

`citizen_dock`, `citizen_fetch`, `citizen_signal_async`, `clock_async`,
`clock_reactive`, `dashboard`, `dashboard_async`, `filter_plotter`,
`getting_started`, `logger_component`, `reactive`, `reactive_slider`,
`realtime_plot`, `ui_refresh_events`. Per-example justification audit
is item §4.2 below; tracks against issue #25.

### Book

mdbook in `book/`. Sections: introduction, background (vocabulary,
Dynamic<T>, egui_dock), concepts (citizen, state, dispatcher, messages,
coupling, inside-dynamic), tutorial (filter_plotter end-to-end, with
WASM run-it section), patterns, pitfalls, reference. Estimated 200 of
~400 print-pages of equivalent content.

### Open issues (worth knowing)

- #19 — wasm support (filter_plotter landed; broader story open)
- #21 — egui-rad-builder (Tim Schmidt) integration
- #22 — gallery example showcasing all features
- #25 — example audit (drives §4.2)
- #26 — clock_reactive → citizen migration

## 3. Where we're going (strategic vision)

Three positioning bets:

1. **egui_mobius is the citizen-pattern framework for production
   Rust GUI.** Not a widget kit, not a wrapper, not "yet another
   reactive layer" — a complete architectural pattern with reactive
   state, panel lifecycle, dispatcher, async backend wiring, all
   composable, all with an end-to-end tutorial.

2. **First-mover in the print-book niche for Rust GUI.** No book
   exists today on Slint, Iced, egui, or any Rust GUI framework
   that bridges the language and the framework. The book becomes a
   strategic asset in 90 days if we hit 1.0 and 50k+ downloads.

3. **Cross-platform from day one.** Native desktop is table stakes;
   WASM is now proven (filter_plotter); mobile/touch is the next
   frontier (§4.5). The framework that ships on all three becomes
   the default choice, not "the desktop one."

The 90-day cadence (§6) sequences the tactical work to land the
1.0 release with these three positioning claims demonstrably true.

## 4. Tactical priorities

### 4.1 Expand the WASM tutorial example

**Goal.** Take `filter_plotter` (current WASM reference) further
so the browser demo showcases more of what the framework actually
does — not just a filter and a plot, but the citizen pattern,
multi-panel reactivity, theme support, and persistence.

**Current state.** `filter_plotter` runs in the browser via trunk
(landed via PR #23 with cleanups in `ed6e28f`). It demonstrates:
filter backend, plot panel, settings panel, logger panel, message
routing through dispatcher.

**Doesn't yet demonstrate (good candidates for expansion):**

- **Derived<T>**: a count badge, filtered log view, "current state
  summary" header. Pairs with §4.6 book content.
- **Theme switching**: dark/light toggle persisted to browser
  localStorage.
- **Multi-dock layouts**: more than one tab area, drag-tabs-between
  to demonstrate `egui_dock` flexibility.
- **Async operations**: a simulated network fetch or a background
  computation that showcases citizen messaging.
- **Browser-specific persistence**: settings saved to localStorage,
  surviving page reloads.
- **Mobile-friendly mode**: detect viewport, swap to single-panel
  mode on narrow widths. Pairs with §4.5.

**Concrete next steps.**

1. Pick 3-4 features from the list to expand into. Recommend:
   Derived<T> showcase (already in §4.6's gravity well), theme
   switching with persistence, and one multi-dock layout demo.
2. Each addition keeps the tutorial chapter coherent — don't
   bloat the example into a kitchen sink. Better: spin off
   `examples/wasm_gallery/` as a separate showcase that combines
   features (ties to #22).
3. Deploy the demo to a public URL (e.g., `mobius.demo.foo` or a
   GitHub Pages site). Every README, blog post, and pitch links
   here. This is one of the strongest catalysts for download
   growth (§6).

**Open questions.**

- Is `filter_plotter` the right host for expansion, or does the
  expanded version belong in a sibling example? Decision feeds §4.2.
- What's the public URL strategy? GitHub Pages is free; the WASM
  artifact is small enough.

**Refs.** #19 (wasm support), #22 (gallery), commit `ed6e28f`.

### 4.2 Example audit — clarify which examples exemplify the framework

**Goal.** Each example in `examples/` either earns its slot with a
clear pedagogical role, or gets refreshed/merged/deleted. The
directory has accumulated organically; some examples teach the
current vision, some are snapshots of older shapes.

**Current state.** 14 examples. Issue #25 already opens this
audit. Per-example questions: what does it teach, is it current,
is it duplicative, is it book-anchored, does it run on Linux/wasm.

**Concrete next steps.**

1. Walk through each example with the issue's five-question rubric.
2. Decide per example: keep / refresh / merge / move to book / delete.
3. Document decisions in a table inside the issue or a sibling
   `develop/example-audit-decisions.md`.
4. Execute the deletes and merges in a single PR ("examples: audit
   cleanup"). Refreshes can land separately as their own PRs.

**Suspected redundancies (need verification):**

- `clock_reactive` vs `clock_async` — overlap?
- `dashboard` vs `dashboard_async` — overlap?
- `reactive` vs `reactive_slider` — distinct enough?
- `logger_component` after lens migration — likely deleted (§4.3
  smoke-tests lens against this)
- `citizen_dock` vs `citizen_fetch` vs `citizen_signal_async` —
  all three teach citizens; rationalize the progression

**Suspected high-value keepers:**

- `getting_started` (Level 1 entry point)
- `filter_plotter` (Level 2 tutorial, WASM reference)
- `citizen_signal_async` (Level 3 reference)

**Open questions.**

- Should each surviving example be tied to a book chapter? Yes,
  arguably — strengthens the "the book is the canonical guide"
  story.
- Which examples should support WASM? Per-example decision after
  the audit.

**Refs.** #25 (example audit issue), §4.1 above (WASM expansion
consumes audit decisions), §4.3 below (lens deprecates
`logger_component`'s use of components).

### 4.3 Integrate `egui_lens` into the monorepo

**Goal.** Bring `egui_lens` into the workspace as `crates/egui_lens/`,
solving version drift permanently and making lens the canonical
reactive logger across the ecosystem.

**Current state.** Detailed plan exists at
`develop/lens-migration-plan.md` (committed `2ff9c1f`). Includes:
why, current state of lens, the option-C decision (plain copy, no
git history), step-by-step migration, post-migration cleanup
(archive `saturn77/egui_lens`).

**This document doesn't repeat that plan.** Read
`lens-migration-plan.md` for the operational detail.

**Strategic bet.** Lens-in-workspace makes "the canonical reactive
logger" a one-line answer for new users and contributors, and
unblocks deprecating `egui_mobius_components`. After lens lands,
the bespoke `LoggerPanel` in `clock_reactive`, `filter_plotter`,
and `citizen_signal_async` becomes a per-example refactor decision
(part of §4.2's audit).

**Refs.** `develop/lens-migration-plan.md`, commit `2ff9c1f`.

### 4.4 Integrate mobius-ecs / mobius-designer into the monorepo (BIG)

**Goal.** Bring `mobius-ecs` (ECS-based UI templating framework)
and `mobius-designer` (visual UI designer) into the egui_mobius
workspace. Solves the same version-drift problem as lens but at
significantly larger scope, and resolves a strategic question about
the framework's relationship to ECS-style data modeling.

**Current state (audited 2026-05-03).**

Local paths:
- `/home/james/raid_one/software_projects/atlantix/Egui/mobius-ecs/`
  — the framework itself, `version = "0.2.0"`, `edition = 2024`
- `/home/james/raid_one/software_projects/atlantix/Egui/mobius-designer/`
  — a separate top-level path
- `/home/james/raid_one/software_projects/atlantix/Egui/mobius-ecs/mobius-designer/`
  — designer also lives inside the ecs repo

**Two designer paths exist — which is canonical?** Open question
for the maintainer (see end of section).

`mobius-ecs` shape:
- Description: *"ECS-based UI templating framework for egui
  applications"*
- Core dep: `bevy_ecs = "0.16.1"` — substantial commitment to
  Bevy's ECS as the data layer
- egui pinned at 0.32 (same drift as lens; will need bump to 0.34
  during migration)
- Includes a binary `mobius` and several examples (`demo`,
  `morphorm_demo`)
- Other notable deps: `egui_taffy 0.8`, `morphorm 0.7`,
  `egui_tool_windows 0.1.3`, `chrono` with serde, `image`,
  `egui_dock 0.17` with serde features

`mobius-designer` is the in-house visual UI builder:
- *"Mobius Designer is a visual UI design tool for egui
  applications, similar to Qt Designer."*
- Drag-and-drop UI creation with real-time code generation
- Generates production-ready egui code with integrated
  egui_mobius signals/slots
- Built on the mobius-ecs framework (depends on it)
- Advanced layout tools — alignment, distribution, grid snapping
- Currently `version = "0.2.0"`, badged for `egui 0.32`,
  `bevy_ecs 0.16.1`, `egui_mobius 0.3.0-alpha.32`

**Strategic implication for issue #21.** This audit changes the
calculus on Tim Schmidt's `egui-rad-builder` integration:
mobius-designer **is the in-house equivalent** of egui-rad-builder.
That doesn't necessarily make #21 redundant, but it changes the
question from "should we integrate egui-rad-builder?" to
"should we integrate egui-rad-builder *given that we have
mobius-designer*?" — which is a different and arguably weaker
case. Decision deferred but should be revisited after this work
lands.

**Architectural relationship to citizen.**

```
egui_mobius_reactive  → reactive primitives (Dynamic<T>)
egui_citizen          → panel lifecycle pattern (UI layer)
mobius-ecs            → component data layer (bevy_ecs based)
mobius-designer       → visual UI builder using mobius-ecs
```

This is the architectural seam to think hard about. mobius-ecs and
egui_citizen *might* be orthogonal (ECS provides component-based
data modeling; citizen provides UI lifecycle; they compose) — or
they *might* overlap (both have notions of "entity with
lifecycle"). Pre-integration audit needs to determine which.

**Why this is the "big one."**

- Larger codebase than lens by an order of magnitude (binary +
  framework + designer + multiple examples)
- Brings `bevy_ecs` as a heavy transitive dependency — real
  build-time and binary-size cost
- The visual designer is itself a significant app surface
- Same egui 0.32 → 0.34 drift as lens, but applied to more code
- Likely affects `saturn-grid-sim` and other downstream apps if
  they consume mobius-ecs
- Architectural integration potentially conflicts with citizen
  pattern (needs scoping)

**Concrete next steps (skeleton — full plan deferred to a
dedicated `develop/mobius-ecs-migration-plan.md` once scoped).**

1. **Pre-flight audit.**
   - Resolve which `mobius-designer` is canonical (top-level vs
     inside `mobius-ecs/`).
   - Map mobius-ecs's public API surface — what do external
     consumers use?
   - Identify downstream consumers (`saturn-grid-sim`,
     `CopperForge`, etc.).
   - Build mobius-ecs and the designer locally to verify they
     work as-is before any migration.
2. **Architectural reconciliation.**
   - Determine whether ECS components and citizen panels overlap
     conceptually. If yes, design the unified story before code
     migration. If orthogonal, document the composition pattern.
   - Decide whether mobius-ecs's `Dynamic<T>` usage (currently on
     `egui_mobius_reactive 0.3.0-alpha.32`) needs API updates for
     the latest version.
3. **Migration shape.**
   - Same "option C" pattern as lens — plain copy, no git history,
     archive the standalone repos post-merge.
   - mobius-ecs becomes `crates/mobius_ecs/` (note: underscore for
     Rust convention).
   - mobius-designer becomes either `crates/mobius_designer/` or
     `examples/designer/` depending on whether it's a library or
     primarily a binary.
4. **egui 0.32 → 0.34 bump.** Same mechanical exercise as lens but
   over a larger codebase. Plan for ~2-4 hours of API porting.
5. **bevy_ecs dependency analysis.** Bevy's ECS is substantial.
   Confirm we want this dep at the workspace level, or whether
   mobius-ecs should be feature-gated.
6. **Smoke tests.** Examples that exercise both framework and
   designer should build and run.
7. **Archive standalone repos.** Both `mobius-ecs` (or wherever
   it's hosted) and `mobius-designer` if separate.

**Why this doesn't happen in 90 days.** Per §6, this is too big a
unit of work to fold in pre-1.0 without significant risk to the
publishing-pitch timeline. Recommend sequencing **after** the
1.0 ship and the No Starch pitch is in flight. The exception:
if the maintainer decides that ECS *belongs* in 1.0's story
(strategic), then earlier integration is justified — but at the
cost of slipping the 1.0 date.

**Open questions (for the maintainer).**

- Which `mobius-designer` path is canonical — the top-level
  `Egui/mobius-designer/` or the one inside `mobius-ecs/`?
- Is `mobius-ecs` published to crates.io? (Quick `crates io
  search` would tell us — worth checking for download numbers.)
- Are `saturn-grid-sim` and/or `CopperForge` direct consumers of
  `mobius-ecs`? If yes, integration must preserve their builds.
- Is the ECS direction core to your 1.0 story, or a parallel
  track? This determines whether §6 should sequence ECS earlier
  or later.
- Does mobius-designer subsume the value egui-rad-builder would
  bring (#21), or is there room for both?
- Do we want `bevy_ecs` as a workspace-wide concept, or
  feature-gated to mobius-ecs only?

**Refs.** #21 (egui-rad-builder — relationship now in question),
`develop/lens-migration-plan.md` (template for the eventual
`mobius-ecs-migration-plan.md`).

### 4.5 Mobile functionality (touch-friendly widgets)

**Goal.** Make egui_mobius a serious choice for mobile (iOS,
Android) deployment, not just desktop and browser. Touch
ergonomics, responsive layouts, virtual keyboard handling.

**Current state.** egui itself supports mobile via eframe's wgpu
backend, but ergonomics need attention. Hit areas in standard egui
widgets are desktop-tuned (mouse-precision); touch needs ≥44pt
(iOS HIG) or ≥48dp (Material) targets. The framework hasn't been
exercised on mobile yet.

**Concrete next steps.**

1. **Audit existing widgets in `egui_mobius_widgets`** for touch
   suitability. `StatefulButton`, `StyledButton` likely need a
   touch-friendly variant or feature flag.
2. **Build a mobile test harness.** Likely an Android app shell
   embedding the wasm build, or a native eframe app on iOS via
   the iOS toolchain. One tracer-bullet example.
3. **Identify citizen-pattern friction points on mobile.** Tab
   switching by touch? Dock layouts on small screens? Virtual
   keyboard interaction with text inputs? Each is a real concern.
4. **Decide on a packaging strategy.** Feature flag (`mobile`)?
   Separate crate (`egui_mobius_mobile`)? Or inline mobile-friendly
   defaults?

**Open questions.**

- Which platform first — iOS or Android? Different toolchain
  costs, different audience.
- Does the citizen pattern's "one-hot panel activation" map well
  to mobile single-screen-at-a-time UX, or does it need adapting?
- Is there a "responsive" story (single layout, adapts to viewport
  width) or "platform-distinct layouts" story?
- Does this work create a `crates/egui_mobius_mobile/` or extend
  existing crates with a feature flag?

**Why now.** Mobile + WASM + Desktop is the cross-platform pitch
that makes the No Starch case (§3 bet 3). Without mobile, the
framework is "desktop and web," which is a smaller story than
"everywhere Rust runs."

**Refs.** #19 (wasm — natural pairing), §4.1 (mobile-friendly
mode in wasm gallery), §4.7 (dock wrapper might need mobile
considerations).

### 4.6 Expand the generic backend pattern in the book

**Goal.** Generalize `filter_plotter`'s backend pattern (trait +
mock + real implementations) into a documented, reusable design
pattern for any citizen app. Add a chapter to the book.

**Current state.** `filter_plotter` defines a `BackendKind` trait
with `FilterParams`/`Traces<T>` data types, with `InProcessIir` as
the only current implementation. The tutorial mentions the pattern
in passing but doesn't generalize it.

**The pattern, distilled.**

```rust,ignore
// 1. Plain data types (Copy if small, Clone otherwise)
pub struct Params { /* snapshot of reactive state */ }
pub struct Result { /* what the backend produces */ }

// 2. Trait — backend boundary
pub trait Backend {
    fn run(&mut self, params: Params) -> Result;
    // ... or async version with channel return for §4.5 mobile work
}

// 3. Real and mock implementations
pub struct RealBackend { /* IIR, serial port, network, etc. */ }
pub struct MockBackend { /* canned responses for tests/demos */ }

impl Backend for RealBackend { /* ... */ }
impl Backend for MockBackend { /* ... */ }
```

**Why this earns its keep.**

- Lets developers test citizen apps without real I/O.
- Gives a clear seam for swapping backends (file-based → network,
  emulated → hardware).
- Tutorial-friendly: "here's the same app with a mock; here it is
  with the real thing."
- Extends naturally to async (`AsyncBackend` trait with channel
  semantics).

**Concrete next steps.**

1. **Write a new book chapter** under `concepts/` titled
   "Backends: the citizen-app I/O boundary." Walks through
   `filter_plotter`'s pattern, generalizes the trait, adds the
   mock-vs-real story, gives an async variant.
2. **Add a `MockBackend` to `filter_plotter`** as a worked example
   the chapter references.
3. **Extract the trait into a workspace crate?** Possibly. A
   `egui_mobius_backend` crate with `Backend`, `AsyncBackend`,
   `MockBackend` could ship the pattern as code, not just docs.
   Decision: if the trait is genuinely 5+ lines and most apps
   would want it, extract. If it's idiosyncratic per-app, keep it
   as a book pattern only.

**Open questions.**

- Does this become its own crate or stay as a documented pattern?
- Sync vs async — one trait or two? `filter_plotter` is sync;
  citizen_signal_async is async; the pattern should bridge both.
- How does this relate to the dispatcher's `handle()` for
  message routing? They serve overlapping but distinct purposes —
  worth disambiguating in the chapter.

**Refs.** Tutorial chapter `book/src/tutorial/writing-a-citizen-app.md`,
§4.1 (WASM expansion may demonstrate mock backends in the browser),
issue #20 (closed but conceptually related to book content
expansion).

### 4.7 Thin wrapper around `egui_dock` (and maybe `egui_tiles`)

**Goal.** Reduce the boilerplate of setting up a dock layout in
egui_mobius apps. The current `TabViewer` impl in every example is
~50-100 lines of mostly-identical scaffolding. A thin abstraction
named, say, `MobiusDock` would let apps declare a layout
declaratively and get the rest for free.

**Current state.** Every dock-using example
(`citizen_dock`, `filter_plotter`, `citizen_signal_async`,
`clock_reactive` via taffy) writes its own `TabViewer` impl,
`Tab` enum, dock-state setup, and tab-button click routing. There's
genuine reuse opportunity.

**The wrapper sketch.**

```rust,ignore
let dock = MobiusDock::new()
    .add_panel(SettingsPanel::new(state.clone()))
    .add_panel(PlotPanel::new(state.clone()))
    .add_panel(LoggerPanel::new(state.clone()))
    .layout(DockLayout::SplitHorizontal { left: 0.3 });

dock.show(ctx, &mut self.dispatcher);
```

Internally it wires up the `TabViewer`, citizen activation on tab
clicks, drain-loop for messages — the boilerplate goes away.

**Concrete next steps.**

1. **Prototype against `egui_dock`.** Write the abstraction, port
   `citizen_dock` to use it, measure boilerplate reduction.
2. **Evaluate `egui_tiles` as alternative backend.** Tiles is a
   different splitting model (more flexible nested splits, less
   tab-centric). The wrapper should ideally be backend-agnostic
   so it can support both.
3. **Decide naming and home.** Likely a new workspace crate
   `egui_mobius_dock` or fold into `egui_citizen` itself if the
   abstraction is small enough.
4. **Migrate examples.** Once the wrapper is solid, move the
   examples to use it — concrete proof of the boilerplate savings.

**Trade-off to surface.** Abstractions over `egui_dock` can either
be **thin** (mostly pass-through with conveniences) or **thick**
(opinionated layout DSL). Thin is safer (easier to escape if you
need raw `egui_dock` features) but offers smaller savings. Thick
is more leveraged but locks you in. Recommend starting **thin** —
prove the boilerplate savings first, then consider thick if there
are clear patterns the thin version keeps repeating.

**Open questions.**

- One backend (`egui_dock` only) or two (`egui_dock` and `egui_tiles`)?
  Two is more work but future-proofs against either dock library
  changing API or going dormant.
- Where does the wrapper live? `egui_citizen` (since it's
  citizen-aware), new crate `egui_mobius_dock`, or its own thing?
- Does this block on lens migration? No — independent, parallelizable.

**Refs.** §4.5 (mobile dock layouts), §4.2 (example audit may
identify which dock examples to migrate first).

## 5. Sequencing — what comes first

Some items are independent; some have dependencies. The dependency
graph:

```
4.3 lens migration  ─┐
                     ├─► 4.2 example audit (decides what to
4.6 backend chapter ─┤    migrate, which examples to keep)
                     │
                     ├─► 4.1 WASM expansion (consumes audit
                     │    decisions)
                     │
                     ├─► 4.7 dock wrapper (consumes audit;
                     │    mobile considerations from 4.5)
                     │
4.5 mobile track   ──┘  (parallel; informs 4.7 wrapper)

4.4 mobius-ecs/designer  ── independent track, big scope
```

Roughly: **lens first (it's already planned, low risk), then audit
(unblocks WASM expansion and dock wrapper), then WASM expansion +
backend chapter + dock wrapper in parallel, then mobile and
mobius-ecs as standalone tracks.**

## 6. 90-day cadence (tied to No Starch pitch window)

The maintainer has committed to a 90-day pre-pitch window where
the goal is hitting 50k+ downloads, shipping 1.0, and accumulating
real-world traction. Tactics map to weeks:

### Weeks 1-3 (mid-May → early June)

- **Land lens migration** (§4.3) — execute the existing plan.
- **Run the example audit** (§4.2) — produce decisions, file PRs.
- **Deploy WASM filter_plotter to a public URL** (§4.1 step 3) —
  small change, big leverage.
- **Decide the citizen-piece rename** (currently `civimo`/`civility`/
  `citivo` candidates) — needs to land before any new crate publication.

### Weeks 4-8 (June)

- **Ship 1.0** — close the open architecture decisions, freeze the
  API, publish to crates.io.
- **Submit to TWIR** with the 1.0 announcement.
- **First blog post.** "Why egui_mobius exists / what it solves."
  Concrete; references saturn-grid-sim and CopperForge.
- **Show HN tied to saturn-grid-sim.** Real product, real
  framework, real revenue — the credibility story.
- **WASM expansion** (§4.1) — at least the public demo URL +
  one new feature in filter_plotter.
- **Backend pattern chapter** (§4.6) — write it; helps the
  expanded book pitch.

### Weeks 9-12 (July)

- **YouTube walkthrough.** 15-30 min building a citizen app from
  scratch.
- **Second blog post.** Technical / comparative — "Rust GUI in 2026."
- **Mobile track kickoff** (§4.5) — at least a tracer-bullet on
  one platform.
- **Dock wrapper prototype** (§4.7).
- **Monitor downloads.** If trajectory looks good, draft No Starch
  pitch in the last week of the window.

### What doesn't happen in 90 days

- **mobius-ecs / designer integration** (§4.4) — too big to fold
  in pre-1.0. Schedule for the post-pitch window unless the
  maintainer disagrees and wants to consolidate before 1.0 (which
  is a defensible alternative — see §7).

## 7. Decisions deferred

Open questions where the right answer is "don't decide yet":

1. **Citizen-piece final name.** `civimo` vs `civility` vs `citivo`
   vs others. Discord is leaning `citivo`; the maintainer leans
   `civimo`; my pick was `civility`. Deadline: before first
   crates.io publication of that crate.

2. **mobius-ecs/designer integration timing.** Pre-1.0 (riskier
   but unifies the story) vs post-1.0 (safer but leaves a known
   loose end). Need maintainer scoping (§4.4) to decide.

3. **Dock wrapper backend coverage.** `egui_dock`-only vs both
   `egui_dock` and `egui_tiles`. Decide after thin-wrapper
   prototype (§4.7 step 1).

4. **Backend pattern as crate vs documented pattern.** Decide
   after writing the chapter (§4.6) — if the trait is genuinely
   reusable, extract; if it's mostly per-app, leave as docs.

5. **Mobile packaging.** Feature flag vs separate crate. Decide
   after the audit and tracer-bullet (§4.5).

6. **Bespoke-LoggerPanel replacement.** Once lens is in the
   workspace, each example with a bespoke logger has a per-example
   migration decision. Tracks against #25 audit and #26 clock
   migration.

## 8. Cross-references

### Plans

- `develop/lens-migration-plan.md` — operational detail for §4.3.
- `develop/task_plan.md` — earlier API simplification thinking;
  some of it is absorbed into 1.0 prep.
- `MEMORY.md` (in Claude memory) — durable cross-session decisions.

### Issues

- #19 — wasm support for examples (umbrella, partially closed).
- #21 — egui-rad-builder integration (possibly redundant with §4.4).
- #22 — gallery example showcasing all features (consumes §4.1).
- #25 — example audit (drives §4.2).
- #26 — clock_reactive → citizen migration (consumes §4.3 lens
  + §4.6 backend pattern).

### Recent landed work (commits)

- `4a391e8` — Linux runtime fix for stripped eframe defaults.
- `2361f29` — clock_reactive logger virtualization.
- `ed6e28f` — wasm-PR cleanup (workspace revert, AI-doc trim).
- `2ff9c1f` — lens migration plan committed.
- `12ce64e` — book sync (Dynamic<T> chapter move + Derived/ValueExt
  + WASM tutorial).

## 9. Open questions for the maintainer

Before this document can move from "framing" to "executable plan,"
these need fill-in:

1. **§4.4 mobius-ecs / designer.** Initial audit done 2026-05-03.
   Remaining open questions: canonical path of `mobius-designer`
   (top-level vs nested in `mobius-ecs/`); crates.io publication
   status of `mobius-ecs`; downstream consumers (`saturn-grid-sim`,
   `CopperForge`); whether ECS belongs in the 1.0 story; relation
   to egui-rad-builder (#21); workspace-wide vs feature-gated
   `bevy_ecs` dependency.
2. **§4.5 mobile.** Which platform first — iOS or Android?
   Or build the harness in a way that targets both equivalently?
3. **§7 deferred decisions.** Any of these you want to close
   sooner rather than later? In particular: the citizen-piece
   rename has a hard deadline ("before first publication") that's
   coming up in weeks 1-3.
4. **§6 90-day cadence.** Bandwidth-realistic? Solo work + client
   work + the framework load is heavy. Adjust if any week's bullets
   feel like wishful thinking.

This document is a working draft. Iterate on it; don't preserve it
as scripture.
