# egui_grafica — findings & design decisions

The rationale behind the `egui_grafica` architecture. Records *why*, so the
decisions don't get re-litigated.

## Scope

egui_grafica is a hybrid of KiCad schematic-entry and Simulink — also
Visio/draw.io-style diagramming — with round-trip DSL ⇄ graphics sync. It
is a *graph* editor (nodes, ports, edges) with a geometry layer.

## The data model is the asset

`Scene` (nodes + edges + settings) is the single source of truth. All
mutation flows through `Registry` — a typed surface that keeps edits
observable and would map one-to-one onto an out-of-process server if one is
ever added. The renderer and interaction layer never poke `Scene` fields
directly.

## Relational connections

Edges reference `(NodeId, PortId)`, not coordinates. Moving a node — or a
port along its perimeter — makes every attached wire follow automatically.
This is the single most important property of the model.

## The `.canvas` DSL — keyword-block syntax

Chosen over a decorator style and over S-expressions for **autocomplete
friendliness**: with line-oriented, keyword-led statements, "what is valid
at the cursor" is computable from the enclosing block plus the line's first
keyword. The source file is the authoritative artifact; the GUI is one
structured editor for it.

## Interaction state machine

Canvas interaction is an explicit FSM (`CanvasFsm`): a press from `Idle`
picks a gesture by what was hit; release/cancel returns to `Idle` and clears
context. egui is immediate-mode, so transitions are driven by per-frame
`Response` polling (`drag_started`/`dragged`/`drag_stopped`) rather than
retained-mode event callbacks — that is the only adaptation.

The `Connecting` state is a spatial gesture: drag a port and while the
cursor stays over its node it slides along the perimeter; once it leaves,
the gesture latches into drawing a connection (restoring the port's anchor
so a connect-drag never nudges it).

## Hit-test at the press origin

A press that drifts a few pixels before egui recognises a drag must be
hit-tested at the *true press point*, not the drifted point — otherwise
clicks on thin wires and small ports miss and fall through to panning.

## Routing is its own module

`router.rs` owns path computation — the renderer draws the router's
polyline, the interaction layer hit-tests it; neither re-derives the path.
Manual (hand-routed) wires store waypoints relative to the live endpoints,
so a re-routed wire keeps its shape when its nodes move.

## hypercurve is the geometry foundation

Decision (James, 2026-05-18): the exact-geometry kernel `hypercurve` is the
geometry foundation for routing and shapes.

- `f32` stays the at-rest format (model, DSL, painter, mouse) — an `f32` is
  a dyadic rational, so the crossing into the kernel is lossless.
- Geometric *reasoning* happens in the kernel: certified bezier flattening
  (a provable flatness bound, not an arbitrary segment count), and exact
  point-in-contour hit-testing for node shapes.
- `Real → f32` is the only lossy step, confined to the `geometry` bridge,
  at the rendering/interaction edge.

## Memory accuracy note

An earlier internal note recorded "no geometry kernel" as a settled
decision. It was not — it was a recommendation. The decision above
(adopt hypercurve) is the actual one.
