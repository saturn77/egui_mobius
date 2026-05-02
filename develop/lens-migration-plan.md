# egui_lens migration plan

This is a fresh-session task plan. It captures everything decided in
the session of 2026-05-01 so it can be picked up cold without
re-deriving anything. Read this whole document before running any of
the steps in §5.

## 1. Why this exists

`egui_lens` is the more modern, reactive logger built on
`Dynamic<T>` from `egui_mobius_reactive`. It currently lives in its
own repo (`saturn77/egui_lens`) because it was broken out before the
ecosystem's structural decisions were made.

Three things motivate folding it back into the workspace:

1. **Version drift.** Lens is on egui 0.32; this workspace is on egui
   0.34. Drift only widens. Workspace inheritance solves this once
   and for all.
2. **One canonical reactive logger.** The workspace currently has
   five logger implementations:
   - `egui_mobius_components::event_logger` (predecessor, signal/slot
     based, see §3 below)
   - `clock_reactive`'s `LoggerPanel` (bespoke)
   - `filter_plotter`'s `panels/logger.rs` (bespoke)
   - `citizen_signal_async`'s `panels/logger.rs` (bespoke)
   - `egui_lens::ReactiveEventLogger` (the modern shape, external)

   New contributors asking "where's the logger?" find five answers.
   Lens-in-workspace makes one answer authoritative.
3. **It's a citizen-friendly story.** Lens is a widget, not a
   citizen. Wrapping it in a `Citizen`-impl panel struct is a few
   lines. Once lens is in the workspace, the bespoke `LoggerPanel`s
   in the examples can be progressively replaced with
   `Citizen`-wrapping-`ReactiveEventLogger`.

The overall direction was confirmed in conversation: bring lens in,
deprecate `egui_mobius_components`, leave `egui_mobius_widgets` alone
(it has 9.5k crates.io downloads and lens itself depends on it).

## 2. Current state of egui_lens (as of 2026-05-01)

**Local checkout:**
`/home/james/raid_one/software_projects/atlantix/Egui/egui_lens/`
(note: lowercase `atlantix`, not `Atlantix` — conversation initially
got this wrong)

**Repo layout:**
```
egui_lens/                     ← workspace, not a single crate
├── Cargo.toml                 ← [workspace] manifest
├── crates/egui_lens/          ← THE crate we care about
├── examples/diskforge/        ← see §6 — has been spun out separately
└── examples/basic_custom/     ← decision pending — see §6
```

**Versions:**
- egui = `0.32` (workspace must reach `0.34`)
- eframe = `0.32`
- egui_dock = `0.17.0`
- egui_extras = `0.32`
- tokio = `1.44.1`
- egui_plot = `0.33.0`
- depends on `egui_mobius_widgets = "0.3.0-alpha.32"` ← fold to
  workspace inheritance once in
- depends on `egui_mobius_reactive = "0.3.0-alpha.32"` ← same
- depends on `egui_mobius = "0.3.0-alpha.32"` ← same

**Branch state:** as of session, lens was on
`1-support-for-displaying-logtracing-messages`, not `main`. Confirm
which branch is canonical before copying. The latest commit
("update: Update to egui 0.32 and related crates") suggests the
0.31→0.32 mechanical bump is done; we're inheriting that work.

## 3. What `egui_mobius_components` actually contains

Audited 2026-05-01. The crate is **literally just the event logger**
— no other components. The lib.rs's "## Components" section lists
exactly one item. File layout:

```
crates/egui_mobius_components/src/components/event_logger/
├── log_colors.rs
├── log_type.rs
├── logger.rs
├── logger_state.rs
├── messages/
├── platform/banner.rs
├── platform/details.rs
├── processor.rs
├── prelude.rs
└── serialization/color32_serde.rs
```

Per `lib.rs`: *"Components use the egui_mobius signal/slot
architecture for reactive UI."* So it's the **predecessor logger**,
built on signal/slot before `Dynamic<T>` existed. Lens supersedes
it architecturally.

Three pieces in components might not exist in lens (audit before
deciding to drop):

- `serialization/color32_serde.rs` — `egui::Color32` serde adapter.
  Lens may already have this; if not, port it in.
- `platform/banner.rs`, `platform/details.rs` — UI features.
  Unknown without reading. Worth confirming whether lens has
  equivalents.

## 4. The decision: option C — plain copy, no git history

User chose option C explicitly. Trade-off accepted: lens's per-file
git history doesn't transfer into egui_mobius — it remains
discoverable in the standalone `saturn77/egui_lens` repo, which
will be **archived** (not deleted) post-migration so historical
references survive.

Two alternatives that were considered and declined:

- **Option A (`git subtree add` of full lens repo):** wrong tool
  because lens is itself a workspace; can't nest workspaces.
- **Option B (`git subtree split` of `crates/egui_lens/` then
  merge):** preserves history but more complex; the user values
  speed of resolution over per-file blame here.

## 5. Migration steps

Each step is its own commit. Verify between steps. Don't batch.

### 5.1. Pre-flight

```bash
cd /home/james/raid_one/software_projects/atlantix/Egui/egui_mobius
git checkout master
git pull --ff-only origin master
git status   # MUST be clean (config.json runtime state OK to stash)
```

Confirm lens's source state: navigate to lens repo, decide which
branch is canonical (`main` vs feature branch), checkout that. Run
`cargo build` in lens itself first to make sure it's not broken on
its own — we don't want to import broken code.

### 5.2. Copy the crate source

```bash
# from inside egui_mobius repo
cp -r /home/james/raid_one/software_projects/atlantix/Egui/egui_lens/crates/egui_lens \
      crates/egui_lens
```

Don't copy lens's workspace `Cargo.toml`, `target/`, `.git/`, or
the `examples/` directory.

### 5.3. Rewrite `crates/egui_lens/Cargo.toml` for workspace inheritance

Convert direct version pins to `workspace = true`:

```toml
[package]
name    = "egui_lens"
version = "0.3.0-alpha.34"      # bump to match the rest of the workspace
edition.workspace      = true
rust-version.workspace = true
authors                = ["James Bonanno <atlantix-eda@proton.me>"]
description            = "Modular reactive event logger component for egui applications"
homepage               = "https://github.com/saturn77/egui_mobius"
repository             = "https://github.com/saturn77/egui_mobius"
license.workspace      = true
categories             = ["gui", "development-tools"]
keywords               = ["egui", "logger", "reactive", "egui-mobius"]
include                = ["LICENSE-MIT", "**/*.rs", "Cargo.toml"]

[dependencies]
egui                  = { workspace = true }
eframe                = { workspace = true, features = ["default"] }
                                                     # ↑ same lean-eframe pattern as the 9 examples we fixed in 4a391e8
egui_extras           = { workspace = true }
egui_mobius_reactive  = { workspace = true }
egui_mobius           = { workspace = true }       # IFF lens still uses egui_mobius signals
egui_mobius_widgets   = { workspace = true }
chrono                = { workspace = true }
serde                 = { workspace = true }
serde_json            = { workspace = true }
```

Cross-check against lens's existing direct deps; some (like `dirs`,
`once_cell`, `image`) may not be in egui_mobius's
`[workspace.dependencies]` and need to either be added there or
kept as direct deps in lens's Cargo.toml.

### 5.4. Add to workspace members

Edit egui_mobius's root `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/egui_mobius",
    "crates/egui_mobius_widgets",
    "crates/egui_mobius_reactive",
    "crates/egui_mobius_components",
    "crates/egui_citizen",
    "crates/egui_lens",            # ← NEW
]

[workspace.dependencies]
# ... existing entries ...
egui_lens = { version = "0.3.0-alpha.34", path = "crates/egui_lens" }
```

### 5.5. Build — see what breaks

```bash
cargo build --workspace 2>&1 | tee /tmp/lens-build-log.txt
```

Expected breakage:
- **egui 0.32 → 0.34 API churn.** Common drift areas: `Frame`
  margins, `Visuals` field renames, `ViewportBuilder` changes,
  removal of deprecated APIs. Walk each error, port idiomatically.
  Lens recently did the 0.31→0.32 bump so the team has the muscle
  for this kind of work; same exercise.
- **Possible feature-flag wrangling.** If lens's eframe needs
  things `default-features = false` strips (winit backends,
  accesskit, persistence), apply the `features = ["default"]`
  pattern. We did this for the 9 affected examples in commit
  `4a391e8`; same recipe applies here.
- **Possible egui_mobius signal/slot API drift.** If lens still
  uses old `egui_mobius` signal patterns, those may have evolved
  since `0.3.0-alpha.32`. Port to current shape.

### 5.6. Run lens's own tests

```bash
cargo test -p egui_lens
cargo test --workspace --quiet
```

Lens may have its own integration tests/examples. Get them green
before considering 5.5 complete.

### 5.7. Verify with a real consumer

The natural consumer is `examples/logger_component`, which
currently uses `egui_mobius_components`'s logger. As a smoke test,
**migrate it from `egui_mobius_components` to `egui_lens`**:

```bash
# Edit examples/logger_component/Cargo.toml — swap dep
# Edit examples/logger_component/src/main.rs — swap imports + usage
cargo run -p logger_component
```

If logger_component runs cleanly with lens, lens-in-workspace is
proven. If you want to be thorough, also build the wasm target:

```bash
rustup target add wasm32-unknown-unknown   # already done
cd examples/filter_plotter
trunk build       # ensure lens's deps don't break wasm filter_plotter
```

### 5.8. Commit

Single focused commit:

```
chore(egui_lens): import into workspace, bump to egui 0.34

Brings egui_lens (modular reactive event logger) from the
saturn77/egui_lens standalone repo into this workspace as
crates/egui_lens. Pinned to workspace deps; bumped from egui 0.32
to 0.34 along the way.

Source attribution: https://github.com/saturn77/egui_lens (will be
archived post-merge). Per-file git history not preserved — option C
from develop/lens-migration-plan.md, accepted in exchange for speed
of consolidation.

Migrate examples/logger_component from egui_mobius_components to
egui_lens as a smoke test. Components crate is now redundant; see
§6 of the plan for deprecation path.
```

### 5.9. Push and merge via PR

Don't commit-and-push direct to master for a change of this size —
open a PR so the diff is reviewable as a unit. Title:
*"Bring egui_lens into the workspace"*. Mention the plan doc in the
body.

## 6. Decisions deferred

These were flagged during planning but punted:

### 6.1. `examples/diskforge` from lens repo

**Resolved.** DiskForge has its own standalone repo at
`saturn77/DiskForge`. The copy in
`egui_lens/examples/diskforge/` is the predecessor and **does not
come along** in the migration. Drop it from the copy in 5.2.

### 6.2. `examples/basic_custom` from lens repo

**Open.** A simple custom-logger example. Decide:

- Skip it (delete during 5.2 — simplest)
- Port it as `examples/lens_basic/` in egui_mobius (preserves the
  pedagogical value)
- Move it to a future `egui_lens` book chapter as inline code

Recommendation: skip it for the migration commit; if it has
teaching value, port it as a follow-up issue under #20 (book
examples) or #25 (example audit).

### 6.3. `egui_mobius_components` deprecation

After 5.7 (logger_component migrated to lens), components has zero
internal consumers. External: 683 crates.io downloads. Two options:

**A — Soft deprecation.** Publish `0.3.0-alpha.34` with
`#[deprecated]` re-exports pointing at `egui_lens`, README says
"this crate is superseded by egui_lens, install that instead."
Friendly to existing users.

**B — Freeze.** Stop publishing new versions. Existing 683-download
users keep working with what they have. We just remove the crate
from `[workspace.members]` and `crates/`.

Either is defensible. **A is friendlier and adds maybe 30 minutes
of work.** Worth doing.

Track this as a separate follow-up issue once 5.x lands.

### 6.4. Replacing bespoke LoggerPanels in examples

After lens is in the workspace, the bespoke loggers in
`clock_reactive`, `filter_plotter`, `citizen_signal_async` could be
replaced with `ReactiveEventLogger` wrapped in a citizen-impl
panel. Separate decision per example; tracks against #25 (example
audit) and #26 (clock_reactive citizen migration).

Don't do this in the migration PR. Let lens land cleanly first,
then evaluate per-example.

## 7. Post-migration cleanup

Once 5.x is merged into master:

1. **Archive `saturn77/egui_lens`**. GitHub Settings → Danger Zone
   → Archive this repository. Preserves history publicly,
   prevents accidental new commits, surfaces a banner pointing
   readers at the new home.
2. **Update `saturn77/egui_lens` README** before archiving with a
   redirect notice: *"This crate has been merged into
   egui_mobius/crates/egui_lens. New issues and PRs there."*
3. **Update crates.io metadata** for `egui_lens` so the
   repository link points at egui_mobius (not the archived repo).
4. **Open the components-deprecation follow-up issue** (§6.3).

## 8. Cross-references

- Commit `4a391e8` — Linux runtime fix; same `features = ["default"]`
  pattern this migration will use for lens's eframe.
- Commit `2361f29` — logger virtualization; possible inspiration for
  perf-tuning lens's UI render path if needed later.
- Commit `ed6e28f` — wasm-PR cleanup; same kind of in-flight
  trim/revert work this migration may need.
- Issue #25 — example audit; informs §6.4 decisions.
- Issue #26 — clock_reactive → citizen migration; will likely
  consume lens once it's in the workspace.
- This plan supersedes the casual scoping done at end of session
  2026-05-01.

## 9. Estimated time

Honest scope: **1–3 hours of focused work**, longer if egui 0.32→0.34
has surprising breakage in lens's render code. Pick a fresh session;
this is not a momentum-finisher task.
