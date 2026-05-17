# What citizen is (and is not)

By this point you have met the [`Citizen` trait](citizen.md), the
[`Dispatcher`](dispatcher.md), and [`CitizenMessage`](messages.md), the
backend bridge. This chapter steps back from the parts and asks what
the pattern *is* — by contrasting it with the architecture people
most often assume it to be.

## It is not Elm

Citizen has Elm-like aspects, and the comparison comes up often
enough that it is worth being precise about where the two diverge.

The Elm architecture's `update` function is **total**: every message
in the entire application funnels through one `update`. That totality
is exactly what makes Elm predictable, and exactly what makes it
rigid. There are no optional edges — every message is mandatory,
through the one loop.

Citizen takes the opposite stance, and it is best stated as a single
phrase: **total observation, partial routing**.

- **Total observation.** The `Dispatcher` always tracks the state of
  every registered citizen. Nothing about a citizen's lifecycle is
  invisible to it.
- **Partial routing.** Data is dispatched to the backend only when an
  atom *requests* it. And atoms in any citizen may share data directly
  with atoms in any other citizen, without the backend being involved
  at all.

That split is the actual invention. It is not Elm — Elm has no
optional edges. It is also not a plain pub/sub bus — a bus forwards
and forgets; it has no central state model. What citizen has is a
**stateful switchboard**.

The topology that results is a *graph*, not a tree: atoms and
citizens are nodes, dispatch connections are edges, and the
`Dispatcher` is the layer that makes the graph **observable** even
though it does not make the graph **constrained**. This is why the
neural-network framing fits — it is a graph of nodes with signal
propagation, not a hierarchy.

## The invariant that keeps it sound

A graph topology where any atom can talk to any atom is powerful, and
it is also the classic recipe for spaghetti. The very property that
makes Elm restrictive on purpose is the property citizen gives up.

What saves citizen from that fate is the *other* half of the split.
Communication is graph-shaped, but **state stays centrally legible**,
because the `Dispatcher` observes everything.

That yields one invariant, and it must be protected permanently:

> **No atom shares data in a way the `Dispatcher` cannot see.**

The moment a back-channel is introduced — "for performance," "just
this once" — the central legibility is gone. A graph topology
*without* central observability is genuinely harder to reason about
than an Elm tree. The flexibility of citizen is only safe because of
the discipline of total observation. The pattern and the rule are
inseparable: keep every data path visible to the dispatcher, or you
do not have the citizen pattern any more.

## The panel-oriented middle ground

Citizen is a bet on a particular granularity of UI composition: the
**panel** — larger than a component, smaller than an application.
There is prior art that both validates the bet and warns about its
failure mode.

**Eclipse RCP** (Rich Client Platform) made exactly this bet: the
panel/view as the unit of a professional tool, a workbench that gives
panels identity, a contribution model for wiring them together. The
entire Eclipse IDE, and a generation of EDA, CAD, and finance tools,
are RCP-shaped. That is the proof the granularity is right. The
warning is also from RCP: it became a byword for *heavyweight* — XML
extension points, lifecycle ceremony, sluggishness. Citizen's
differentiator has to remain that it is lightweight and reactive,
not declarative-config driven.

**Qt's dock-widgets plus signals/slots** is the other close relative.
Citizen's atom-to-atom sharing *is* signals/slots — but value-typed
and observed, rather than callback-typed and invisible.

So the honest peer set for citizen is Eclipse RCP, the VS Code
contribution model, and Qt Creator's plugin architecture. It is *not*
dioxus, leptos, or iced — those are component- or page-oriented and
web-derived, solving a different problem at a different granularity.
The claim that the panel-oriented middle ground is where the most
value lies is really a claim that professional tools are built from
panels, not pages or screens. The roster of tools above bears it out.

## The pattern eats itself

The strongest sign that the abstraction is real is that the **panel
builder is recursive**.

The `egui_grafica` canvas — nodes with ports, edges between ports, a
registry as the backend model — is structurally identical to
"citizens with atoms, dispatch connections between atoms, a
`Dispatcher` as the model." A RAD tool where you drag citizens into a
dock layout and draw atom-to-atom wires would, structurally, be
`egui_grafica` pointed at its own framework. The canvas built for
diagrams *is* the panel builder's canvas.

That is not a coincidence. Real abstractions tend to eat themselves
like this — the same graph showing up at two scales (the fine-grained
reactive value graph, and the coarse-grained citizen/dispatcher
graph) is evidence the abstraction describes something true rather
than something arbitrary.

## Summary

| Aspect | Elm | Pub/sub bus | **Citizen** |
|--------|-----|-------------|-------------|
| State model | central, total | none | **central, total** |
| Message routing | total (mandatory) | partial (fire-and-forget) | **partial (optional)** |
| Topology | tree / loop | graph | **graph** |
| Observability | inherent | absent | **inherent (via `Dispatcher`)** |
| Failure mode | rigidity | unobservable spaghetti | spaghetti *if the invariant breaks* |

Citizen keeps Elm's observability without Elm's rigidity, by
separating *observing* state from *routing* data. The price of that
flexibility is a single non-negotiable rule: every data path is
visible to the `Dispatcher`.
