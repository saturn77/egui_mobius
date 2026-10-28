# mobius_codegen roadmap

Accumulated from the 2026-10-28 RibbonDemo milestone session. Ordered
roughly by leverage, not urgency.

## Toolchain shape

The repo now has the HDL toolchain duality: `mobius_lang_host`
interprets a `.mobius` design live (simulation), `mobius_codegen`
emits it as a compilable cargo project (synthesis), one parser feeds
both. The items below close the gaps that duality exposes.

## TODO

- **Output directives / project pathing.** Today the output location
  and the egui_mobius dependency root arrive as CLI args
  (`mobius-gen <in> <out> [root]`). Polished flows need this declared,
  not retyped. Two candidate homes, HDL precedent in mind:
  - a `@project` block in the source (`out = "…"`, `crate_name`,
    `mobius = path|version`, asset dirs) — single-file ergonomics;
  - a `mobius.toml` next to the source — keeps the `.mobius` design
    portable, the way RTL stays pure while `.qsf` / `.xpr` / filelists
    carry pathing. Likely answer: support both, precedence
    CLI > `@project` > `mobius.toml` > defaults.

- **Route codegen through `lower()`.** Codegen walks the raw AST and
  skips elaboration, so generated projects miss the one-writer check —
  synthesis without lint. Consuming the lowered IR gives both back
  ends the same semantic gate.

- **Sim/synth equivalence.** Nothing yet asserts the interpreter and
  the generated code behave identically on the same source. Extend the
  gate test fixture to drive both paths and compare observable state.

- **Parser error recovery.** `parse()` is fail-fast (one `ParseError`).
  Recovery to sync points (`;`, `}`, section starts) returning
  `(partial AST, Vec<ParseError>)` makes `mobius-gen` report like a
  compiler and is the prerequisite for a usable LSP.

- **Diagnostics LSP.** Tier 1: `parse()` errors as live squiggles
  (lsp-server + lsp-types; spans already carry line/col). Tier 2:
  `lower()` diagnostics. Tier 3 (post-recovery): completion from the
  AST symbol tables — port fields after `.`, variants after `::`.
  The VS Code extension (`editors/mobius-vscode`) grows a ~30-line
  client activation.

- ~~**Second citizen milestone.**~~ Done: `dock()` → DockState/Tab/
  TabViewer generation, instance-derived `CitizenId`s via
  `citizen_panel!::with_id`, multi-instancing proven in
  `examples/dock` (Controls ×2 / Scope ×2 over Channel ×2), plus the
  System citizen (egui_lens prefab: builtin `LoggerState`/`LogColors`
  types, `logger` primitive, generated startup banner with deps +
  system details).

- **Example 3 — the terminal citizen (tutorial ladder).** James's
  sequence: ribbon (chrome) → dock (composition + logger) → terminal
  (shell citizen, CopperForge-style: bash passthrough + app
  subcommands). Each example builds on the previous — a tutorial of
  sorts. The terminal is likely the first *stateful* prefab (input
  buffer, history) and the first natural Path B consumer (commands as
  events into an actions module).

- **Self-truing deps table.** The About window's `DEPENDENCIES` const
  is a generation-time snapshot. A generated `build.rs` extracting
  resolved versions (ForgePM's pattern) keeps it honest as lockfiles
  drift.

- **Chrome-vs-citizen ribbon choice.** CopperTrace models the ribbon
  as a citizen; ShutterFrame deliberately keeps chrome outside the
  registry (no dock lifecycle) and adds responsive one/two-row layout
  from measured cluster widths. Eventually a declared choice; the
  responsive two-row behaviour is mechanical and generatable.

- **Grammar punch list** (from building the ribbon against real apps):
  hover text on buttons, labeled containers (true `menu "🎨" { … }`),
  conditional enablement bound to state, two-tone wordmark as syntax
  rather than camel-hump inference, modal declarations as widget trees
  instead of the `about()` directive special case.
