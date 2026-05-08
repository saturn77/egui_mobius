# Toolkit-agnostic egui_mobius — Roadmap

A strategic forward-look. The thesis: egui_mobius's citizen pattern
is *conceptually* toolkit-agnostic, while the current implementation
is shot through with egui assumptions. Decoupling those assumptions
into a `mobius_core` plus per-toolkit adapters repositions the
framework from "the citizen-pattern framework for egui" to "the
citizen-pattern framework — currently egui-shaped, architecturally
neutral, ports to anywhere with dockable panels."

Captured 2026-05-08 from a Discord conversation with Tim Schmidt
(csgrs) and QuickFlash. Tim's framing was the catalyst — *"Like if
Tokio knew about GUIs"*. The user's three-bucket state model —
`AppState` / `CitizenState` / `LocalState` — is the abstraction
that makes it tractable.

This document is *not* a commitment to start the work. It is
preservation of the conversation's substance and a sketch of the
shape so that when the time is right, the path is already drawn.

## 1. Why this exists

Tim's diagnosis: *"It's a missing layer of abstraction no rust GUI
toolkits have yet."* egui_dock is panels-as-component; nothing in
the Rust ecosystem provides panels-as-application-layer with
state, lifecycle, and message routing. egui_mobius already does
this — *coupled to egui*. Decoupling it is the move from "useful
egui flavour" to "the layer the Rust GUI ecosystem is missing."

Successful frameworks tend to become glue layers rather than
feature kits. Tokio doesn't care if you use hyper or actix; it
provides the runtime layer they build on top of. Rails became
glue between web servers and databases. The egui_mobius
equivalent: *"we don't care if your panel renders with egui or
gpui or iced. We provide the citizen pattern, message bus,
reactive state, and dispatcher. The toolkit handles its own
render."*

The strategic positioning Tim called *"king-of-the-hill"* is
real. If this works, egui_mobius becomes the **interoperability
story for Rust GUIs**, not just an egui flavour.

## 2. The thesis: citizen pattern is toolkit-agnostic in concept

Re-reading the citizen pattern's primitives, none are
fundamentally egui:

| Primitive | Concept | Egui coupling today |
|---|---|---|
| `CitizenId` | stable opaque string identity | none — pure data |
| `CitizenState` | reactive lifecycle bucket — active, clicked, … | reactive layer is egui-coupled |
| `CitizenMessage` | enum of lifecycle events for backend bridging | none — pure data |
| `Dispatcher` | registry of citizens, one-hot activation, msg queue | none — pure logic |
| `Dynamic<T>` | reactive cell with subscribers | repaint signal is egui-coupled |
| `Citizen` trait | id + lifecycle accessors | currently leans on consumers calling egui APIs |

The data primitives are clean. The adhesion to egui lives in two
places: the *repaint signal* that `Dynamic<T>::set()` invokes when
a value changes, and the *render contract* a citizen satisfies in
its `show()` method.

## 3. Three-bucket state model — toolkit-agnostic form

The user's clarification, which the conversation crystallised:

- **`AppState`** — global shared state across panels. In-process,
  cross-toolkit-shareable today via `Arc<Mutex<T>>` since panels
  share an address space regardless of which toolkit renders
  them.
- **`PanelState` / `CitizenState`** — per-panel lifecycle bucket.
  Library-defined (active flag, click flag, lifecycle tracking).
  Owned by the dispatcher's registry.
- **`LocalState`** — per-citizen private data (slider values,
  combo selections, scroll positions). Plain fields on the
  citizen struct; never visible to other panels.

This shape transfers across toolkits *without modification*. The
Discord conversation kept circling this — Tim's *"the right
abstraction boundary for a different toolkit"* is exactly this.

## 4. The architectural shape — `mobius_core` + adapters

```
mobius_core (toolkit-agnostic)
├── Citizen trait                   (id, citizen_state, citizen_state_mut)
├── CitizenState, CitizenMessage    (lifecycle data)
├── Dispatcher                      (registry, activation, message queue)
├── Dynamic<T>                      (reactive cell + Repainter callback)
├── Repainter trait                 (toolkit's "redraw please" hook)
└── Three-bucket model docs

mobius_egui (egui adapter — what egui_mobius is today)
├── EguiRepainter                   (wraps egui::Context::request_repaint)
├── Render trait                    (extends Citizen with fn show(&mut Ui))
├── egui_dock integration           (TabViewer bridge)
└── re-exports core for compatibility

mobius_gpui (future)
├── GpuiRepainter
├── Render trait — gpui-shaped
└── gpui dock layer (probably needs to be built)

Shipped citizens (toolkit-specific by nature)
├── egui_lens, egui_quill, egui_3d_viewer    (egui — today)
└── gpui_lens, gpui_quill, gpui_3d_viewer    (future, if demand)
```

The data primitives in `mobius_core` are toolkit-agnostic. The
*render contract* lives in each toolkit's adapter. Citizens are
toolkit-specific because their job is to render.

## 5. What's actually egui-coupled today — audit

To estimate the work honestly:

- **`egui_mobius_reactive::Dynamic<T>`**: the cell itself is
  `Arc<Mutex<T>>`-backed and toolkit-neutral. The "wake the UI"
  signal is what's egui-tied — `set()` calls into egui's repaint
  scheduler somehow. Decoupling means abstracting that into a
  `Repainter` trait the citizen passes in at construction.
- **`egui_citizen`**: the `Citizen` trait surface itself looks
  toolkit-agnostic on inspection — it exposes id and state
  accessors. `Dispatcher` is pure logic. The egui assumptions
  hide in the *examples* and the convention that a citizen has a
  `fn show(ui: &mut egui::Ui)` method, which the trait does not
  actually require. So the trait may already be most of the way
  to core.
- **`egui_dock` integration**: the `TabViewer` bridge that maps
  egui_dock tab clicks to `Dispatcher::activate()` is egui-only
  by definition. Stays in the egui adapter.
- **Citizens themselves** (`egui_lens`, `egui_quill`,
  `egui_3d_viewer`): these *are* egui implementations. They keep
  their egui dep and live in the egui adapter's ecosystem.
  Equivalent gpui citizens get built when there's demand.

The decoupling project is smaller than the Discord conversation
implied. Most of the framework is already conceptually
toolkit-neutral; the lift is mostly about *making the abstraction
boundaries explicit* rather than rewriting internals.

## 6. Counterpoint — what Tim slightly underestimated

Tim's *"Mixing toolkits in dockable panels: feasible. Local
control of the render loop within each panel"* glosses one real
hard point:

**Cross-toolkit AppState** is *not* the hard part. In-process
panels share an address space; `Dynamic<T>` clones across
toolkits work today because `Arc<Mutex<T>>` is toolkit-blind.
What's egui-tied is the *repaint signal*, which is much smaller
than "the entire reactivity layer." A `Repainter` trait with one
method — `request_repaint(&self)` — covers it.

**Cross-toolkit dock hosting** *is* the hard part. egui_dock
owns the layout today. For multi-toolkit apps, the dock host
becomes a thinner shell — winit + wgpu directly, with each panel
handed its own render context. This is where the
QuickFlash-mentioned gpui-on-winit-wgpu fork becomes relevant —
that kind of underlying architecture is what makes
heterogeneous-toolkit panel hosting feasible. The mainline gpui,
which is coupled to Zed's needs, is harder.

## 7. Phased path forward

Each phase is a meaningful diff and a possible stopping point —
the framework is more useful at every step.

- **Phase 1 — `mobius_core` extraction.** Identify the
  egui-coupled spots, lift them behind trait boundaries, ship a
  new workspace crate that holds the toolkit-neutral primitives.
  Existing consumers — zicad, CopperForge, filter_plotter —
  continue to use the egui flavour through the adapter and notice
  no API change. Internal refactor only.
- **Phase 2 — proof-of-concept second adapter.** Pick the
  cheapest second toolkit — could be a minimal winit + wgpu shell
  using a thin custom dock, or iced, or the gpui-on-winit-wgpu
  fork. Build *one* citizen — the simplest possible — that can
  share `Dynamic<T>` state with an egui-rendered citizen in the
  same app. Proves the abstraction.
- **Phase 3 — production-ready second adapter.** Once the
  proof works, harden one specific toolkit and ship `gpui_lens`
  or equivalent. This is where dock hosting becomes real
  engineering work for whichever toolkit is chosen.

## 8. Cost — honest estimate

- Phase 1: 1–3 weeks. Internal refactor with no behaviour change
  for existing consumers. Low risk; mostly about drawing the
  trait boundaries cleanly.
- Phase 2: 1–2 months depending on toolkit choice. The hard part
  is dock hosting — if the chosen second toolkit has a usable
  dock library, this is fast; if not, it's slow.
- Phase 3: 3–6 months for production-grade quality including a
  shipped citizen or two.

Total realistic horizon for "credible toolkit-agnostic story
with two living adapters": 6 months from when Phase 1 starts.

## 9. When to start

Not yet. Three reasons to keep this on the shelf for now:

- **zicad is the proof point.** The framework's credibility comes
  from "real apps shipping with citizens" much more than from
  architectural purity. zicad at v0.5+ — project panel + file IO
  + async eval — makes the framework story *demonstrable*. Right
  now it's *promising*. Demonstrable beats promising.
- **Pivot risk during decoupling.** During the 1–3 weeks of
  Phase 1 internal refactor, application work effectively
  freezes. If zicad and CopperForge are mid-feature when the
  decoupling lands, integration friction multiplies. Better to
  reach a quiet stopping point on application work first.
- **The strongest pitch is "we have a real CSG IDE and a real
  PCB inspector, and the framework's clean enough to swap in
  gpui."** Reaching that pitch requires the apps reaching v0.5+
  before the framework decoupling. Reverse the order and the
  pitch becomes "we have a multi-toolkit framework with two
  half-built apps," which is weaker.

Trigger to revisit: zicad reaches v0.5+ AND there's external
demand from a non-egui toolkit user, OR Tim ships a csgrs viewer
based on this framework and that becomes the next driving use
case.

## 10. Open decisions when revisited

- **Naming**: `mobius_core` + `mobius_egui` + `mobius_gpui`? Or
  keep `egui_mobius` as the egui adapter and introduce
  `mobius_core` alongside? Renames are disruptive; additions are
  cheap. Lean: keep `egui_mobius` as-is, add `mobius_core`
  beside it, treat `egui_mobius` as the egui adapter going
  forward.
- **Repainter shape**: single trait method `request_repaint`?
  Channel-based? Subscriber callbacks? Lean: trait method,
  simplest, easiest for adapter authors to satisfy.
- **Async story**: signal/slot is currently Tokio-coupled, which
  is also a wasm problem — see `develop/wasm_login_demo_plan.md`.
  Toolkit-agnostic and wasm-compatible probably want the same
  underlying decoupling. Worth tackling together when revisited.
- **Second adapter target**: gpui (with dock work), iced,
  winit+wgpu thin shell, or something else. Decide when there's
  external demand pulling rather than internal speculation
  pushing.

## 11. References

- **Discord conversation, 2026-05-08** — Tim Schmidt and
  QuickFlash. Tim's *"Tokio for GUIs"* framing and *"missing
  layer no rust GUI toolkit has"* diagnosis. QuickFlash's pointer
  to a gpui fork running on winit + wgpu.
- **`develop/strategy_tactics.md`** — current strategic plan,
  framed as "egui_mobius is the citizen-pattern framework for
  egui." This roadmap reframes that to *"…currently egui-shaped
  but architecturally neutral."*
- **`develop/wasm_login_demo_plan.md`** — adjacent decoupling
  problem; Tokio-coupled signal/slot vs wasm. Worth tackling
  together.
- **`gpui`** — Zed's framework. <https://github.com/zed-industries/zed>
- **CopperForge** — first egui_mobius consumer; the citizen
  pattern's first real-world test.
- **zicad** — second consumer; lives or dies by the framework's
  ergonomics. The proof point that the abstraction works for a
  second domain.
