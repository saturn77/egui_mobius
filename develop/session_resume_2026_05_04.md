# Session resume — 2026-05-04 → next

Snapshot at the end of a session that landed the Editor citizen
(`egui_quill`), virtualized the lens render, bumped the workspace
to 0.4.0, and deployed the live wasm demo to GitHub Pages.

## Where things stand

**Crates published at v0.4.0** (all 0.3.0-alpha.X retired):
- `egui_mobius` — 10,205 historical downloads
- `egui_mobius_reactive` — 9,595
- `egui_mobius_widgets` — 9,475
- `egui_mobius_components` — *deprecated, frozen at 0.4.0*
- `egui_lens` — 617 historical downloads
- `egui_quill` — *new this session, not yet published to crates.io*
- `egui_citizen` — *workspace-only, not on crates.io pending rename
  decision*

**Live URLs:**
- Book: https://saturn77.github.io/egui_mobius/
- Demo: https://saturn77.github.io/egui_mobius/demo/filter_plotter/
  (auto-deployed from CI via `.github/workflows/book.yml`)

**Recent commits worth knowing:**
- `66bc5a4` — book: compile-time plug-ins caveat
- `ffacc01` — book: dispatcher-vs-egui_dock + citizens-as-plug-ins sections
- `0083597` — book: egui_quill chapter
- `f394681` — feat: egui_quill citizen crate
- `39424ac` — perf: lens virtualized via `ScrollArea::show_rows`
  (closes #27 + #29)

## Immediate next actions, in order

1. **Data Table citizen (#13)** — second canonical panel. Same
   pattern as quill: new sibling crate (e.g. `egui_grid` or
   similar), reactive state, atom set, bolted into filter_plotter
   as a fifth dock tab. ~1-2 days.

2. **Cloudflare Pages migration (#32)** — keystone for the
   announcement push. ~1-2 hours. Unlocks brotli compression on
   the wasm bundle.

3. **Announcement push** (parallelizable post-#32):
   - egui Discord show-and-tell channel
   - /r/rust post with live demo URL
   - egui GitHub Discussions
   - Rust Discord #showcase
   - Awesome Rust PR

4. **TWIR + Show HN + Lobsters** — reserved for the 1.0 ship
   moment, per the strategy doc 90-day cadence.

## Open issues snapshot (high-leverage subset)

- **#13** — Data Table citizen panel (next canonical citizen)
- **#21** — egui-rad-builder integration (parked pending mobius-designer)
- **#26** — clock_reactive → citizen migration
- **#30** — lens `impl Citizen` directly (drop wrapper struct)
- **#31** — Level 3 (signal/slot + AsyncDispatcher) WASM compat
  *(HIGH — closes the cross-platform pitch)*
- **#32** — Cloudflare Pages migration *(HIGH — announcement keystone)*
- **#33** — Canonical citizen panels (Project / Editor / Terminal /
  Settings; Editor done as quill, Data Table is #13)
- **#34** — Editor citizen *(closed by f394681)*
- **#35** — Hand-rolled HDL parsers (Verilog / SystemVerilog / VHDL)
  feeding quill via `Highlighter` trait — phased: lexer-only first,
  AST later

## Decisions still open

- **Citizen rename** — `civimo` / `civility` / `citivo` /
  `egui_citizen`. Currently named `egui_citizen`, not published. Hard
  deadline is "before first crates.io publication of that crate."
  Per `memory/project_naming_decision.md`, `egui_mobius` umbrella
  stays as-is.
- **mobius-ecs / mobius-designer integration (#33 §4.4)** —
  pre-1.0 vs post-1.0 timing. Currently parked.
- **#22 (gallery example)** — should be closed/renamed since
  filter_plotter absorbs that role.

## Key references

- `develop/strategy_tactics.md` — master 90-day plan + tactical
  priorities. The most important doc to re-read on resume.
- `develop/lens-migration-plan.md` — completed; pattern reference
  for future migrations.
- Memory directory at
  `/home/james/.claude/projects/-home-james-raid-one-software-projects-atlantix-Egui-egui-mobius/memory/` —
  durable preferences (terse responses, no Claude footer in commits,
  small commits, naming decision).

---

*This file is OK to delete after the next session picks up the thread.*
