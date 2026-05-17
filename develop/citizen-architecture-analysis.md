# What Citizen Is (and Is Not) — an architectural analysis

An outside-viewpoint analysis of the citizen pattern: what distinguishes it
from the Elm architecture, the invariant that keeps it sound, and where it
sits in the landscape of UI frameworks.

---

## 1. It is not Elm — "total observation, partial routing"

The Elm architecture's `update` function is **total**: every message in the
entire application funnels through one `update`. That totality is exactly what
makes Elm predictable, and exactly what makes it rigid.

Citizen takes the opposite stance. The Dispatcher **always observes** — state
tracking across every citizen is total — but it **only optionally routes**:
data is dispatched to the backend only when an atom requests it, and atoms in
any citizen may share data directly with atoms in any other citizen.

That split — *total observation, partial routing* — is the actual invention.

- It is **not Elm**: Elm has no optional edges; every message is mandatory
  through the one update loop.
- It is **not a plain pub/sub bus**: a bus has no central state model. It
  forwards and forgets.

What citizen has is a **stateful switchboard**. The topology is a graph, not a
tree: atoms and citizens are nodes, dispatch connections are edges, and the
Dispatcher is the layer that makes the graph *observable* even though it does
not make the graph *constrained*. This is why the "neural-network topology"
framing holds — it is a graph of nodes with signal propagation, not a
hierarchy.

## 2. The load-bearing invariant

A graph topology where any atom can talk to any atom is powerful, and it is
also the classic recipe for spaghetti. The same property that makes Elm
restrictive on purpose is the property citizen gives up.

What saves citizen from that fate is the total-observation half of the split:
communication is graph-shaped, but **state stays centrally legible** because
the Dispatcher observes everything.

This yields one invariant that must be protected, permanently:

> **No atom shares data in a way the Dispatcher cannot see.**

The moment a back-channel is introduced — "for performance," "just this once"
— the central legibility is gone, and a graph topology without central
observability is genuinely harder to reason about than an Elm tree. The
flexibility of citizen is only safe because of the discipline of total
observation. The pattern and the rule are inseparable.

## 3. The panel-oriented middle ground

Citizen is a bet on a specific granularity of UI composition: the **panel** —
larger than a component, smaller than an application. There is prior art that
both validates the bet and warns about its failure mode.

**Eclipse RCP (Rich Client Platform)** made exactly this bet: the panel/view
as the unit of a professional tool, a workbench that gives panels identity, a
contribution model for wiring them together. The entire Eclipse IDE and a
generation of EDA, CAD, and finance tools are RCP-shaped. That is the proof
the granularity is right.

The warning is also from RCP: it became a byword for *heavyweight* — XML
extension points, lifecycle ceremony, sluggishness. Citizen's differentiator
must remain that it is **lightweight and reactive**, not declarative-config
driven.

**Qt's dock-widgets + signals/slots** is the other close relative. Citizen's
atom-to-atom sharing *is* signals/slots — but value-typed and observed, rather
than callback-typed and invisible.

The peer set for citizen is therefore: Eclipse RCP, the VS Code contribution
model, Qt Creator's plugin architecture. It is **not** dioxus / leptos / iced
— those are component- or page-oriented and web-derived. They solve a
different problem at a different granularity. The claim that the
panel-oriented middle ground is where the most value lies is a claim that
professional tools are built from panels, not pages or screens — and the
roster of tools above bears that out.

## 4. The recursion — the pattern eats itself

The strongest sign that the abstraction is real: the **panel builder is
recursive**.

`egui_grafica` — nodes with ports, edges between ports, a registry as the
backend model — is *structurally identical* to "citizens with atoms, dispatch
connections between atoms, a Dispatcher as the model."

A RAD tool where you drag citizens into a dock layout and draw atom-to-atom
wires would, structurally, be `egui_grafica` pointed at its own framework. The
canvas built for diagrams is the panel builder's canvas. The typed-port
machinery (`Port.data_type`, `PortKind`) that validates "this output type
matches that input type" for a diagram validates citizen-to-citizen wiring for
free.

That is not a coincidence. Real abstractions tend to eat themselves like this
— the same graph showing up at two scales (the fine-grained reactive value
graph, and the coarse-grained citizen/dispatcher graph) is evidence the
abstraction describes something true rather than something arbitrary.

---

## Summary

| Aspect | Elm | Pub/sub bus | **Citizen** |
|--------|-----|-------------|-------------|
| State model | central, total | none | **central, total** |
| Message routing | total (mandatory) | partial (fire-and-forget) | **partial (optional)** |
| Topology | tree / loop | graph | **graph** |
| Observability | inherent | absent | **inherent (via Dispatcher)** |
| Failure mode | rigidity | unobservable spaghetti | spaghetti *if the invariant is broken* |

Citizen keeps Elm's observability without Elm's rigidity, by separating
*observing* state from *routing* data. The price of that flexibility is a
single non-negotiable rule: every data path is visible to the Dispatcher.
