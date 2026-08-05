// mobius-lang — Application Composition Language (working draft)
#let doc-version = "v0.1.0"
#let inset-size = 11pt
#let git-hash = sys.inputs.at("githash", default: "unknown")
#let build-time = sys.inputs.at("buildtime", default: datetime.today().display())

#show outline.entry: it => { link(it.element.location(), it) }

#set page(
  paper: "us-letter",
  margin: 1in,
  header: [
    #grid(columns: (1fr, 1fr), align: (left, right),
      [_mobius-lang_ Composition Language], [_Version_ - #doc-version])
    #line(length: 100%, stroke: 0.5pt)
  ],
  footer: context [
    #grid(columns: (1fr, 2fr, 1fr), align: (left, center, right),
      [], [], [Page #counter(page).display()])
  ]
)

#set text(font: "Libertinus Serif", size: 13pt)
#set par(justify: true)
#set heading(numbering: "1.")

#show raw.where(block: true): it => {
  set text(font: "DejaVu Sans Mono", size: 9pt)
  block(fill: luma(245), inset: inset-size, radius: 4pt, width: 100%, it)
}
#show raw.where(block: false): it => {
  set text(font: "DejaVu Sans Mono", size: 10pt)
  box(fill: luma(240), inset: (x: 3pt, y: 0pt), outset: (y: 3pt), radius: 2pt, it)
}

#let todo(body) = text(fill: rgb("#b00020"), style: "italic")[[TODO: #body]]
#let open(body) = text(fill: rgb("#8a5a00"), style: "italic")[[OPEN: #body]]

// ============================================================
// Title Page
// ============================================================
#align(center)[
  #text(size: 18pt, weight: "bold")[_mobius-lang_ Composition Language]

  #v(0.5em)
  #text(size: 14pt, fill: gray)[A Code-First GUI Application Composition Language]

  #v(1em)
  #text(size: 12pt)[
    *Prepared by* \ James Bonanno \ \<james\@atlantixeng.com\> \ \
    *Git Version:* #raw(git-hash) \
    *Last Updated:* #build-time
  ]

  #v(1.2in)
  #align(left)[
    #text(size: 14pt, weight: "bold")[Abstract]

    #v(0.5em)
    #text(size: 12pt)[
      _mobius-lang_ is a *code-first application composition language* for
      `egui_mobius`. You describe a GUI application as source — clean, composable,
      diffable text — and the compiler resolves Citizens from a registry of
      compiled-in panel types, checks every shared-state and signal edge, and
      elaborates the description into a wired, running application: reactive
      state, signal routing, and dock layout.

      The language composes; it does not implement. A Citizen's interior — its
      widgets, its rendering, its private state — is Rust, compiled and
      registered. What the language owns is the layer the Rust builder pattern
      expresses poorly: *which* Citizens exist, *what state they share and in
      which direction*, *which events they emit and who handles them*, and
      *where they sit*. The topology is the text.

      The declaration form is the RPV convention — _Rust, Python, Verilog_ —
      where every declaration reads `let name : type = value`.
    ]
  ]
]

#pagebreak()
#outline(title: "Table of Contents", indent: auto)
#pagebreak()

// ============================================================
= Introduction <sec:intro>
// ============================================================

A GUI application built on `egui_mobius` is a set of Citizens plus the reactive
fabric that joins them: shared `Dynamic<T>` values, derived values, and
signal/slot event channels routed through the application's dispatcher runtime.
Today that fabric is authored as imperative Rust — constructor calls, clones
handed around, channels wired by hand in `main()`. It works, but the
application's *shape* is buried in a sequence of method calls: you cannot look
at the wiring code and see the graph; you reconstruct it.

mobius-lang makes the shape the source:

/ Composition over copy-paste: An application is assembled from reusable,
  registered Citizens; the same panel type instantiates into many applications
  with different bindings.
/ Git-native review: The wiring of an application — who writes which value,
  who observes it, which events flow where — is a text diff, not an
  archaeology expedition through `main()`.
/ Topology as text: Shared-state edges and signal edges are declarations. The
  reader sees the graph; the elaborator checks it.
/ Live iteration: Wiring and layout are data. Editing a `.mobius` file can
  re-elaborate the running application without recompiling, within the
  vocabulary of compiled-in Citizens (@sec:elab).
/ Generation and automation: A structured, semantic description of an
  application is a natural target for tooling and LLM-assisted design, where
  a pile of builder calls is not.

The compiler pipeline:

```
source (.mobius) --> parse --> resolve + check --> IR --> execute
                    (AST,     (Citizen registry,   |     (Rust calls: Dynamic<T>,
                     spans)    edge type-check,    |      signals/slots, dispatcher,
                               one-writer rule)    |      dock layout)
                                                   +--> [later] emit Rust source
```

The language is deliberately small. It has no widgets, no rendering, no
expressions of consequence. It is the `main()` you wish you could diff.

#pagebreak()
// ============================================================
= Lineage & Framework Contract <sec:lineage>
// ============================================================

The declaration discipline is the RPV convention — *Rust, Python, Verilog* —
with the composable brace-block style of Scala/SpinalHDL. The compiler is a
hand-written lexer, a recursive-descent parser producing a span-carrying AST
with preserved comments, a serializer that round-trips source, and an
elaboration pass that resolves names against a registry.

== Citizens are modular plugins

The unit the language composes is the *citizen*: a panel-granularity modular
plugin, as established in the egui_mobius book (_What citizen is (and is
not)_). The honest peer set is Eclipse RCP views, the VS Code contribution
model, and Qt Creator's plugin architecture — professional tools are built
from panels — with the differentiator that citizens stay lightweight and
reactive rather than declarative-config driven. The pattern's stance is
*total observation, partial routing*: the `Dispatcher` observes every
citizen's lifecycle, while data is routed to the backend only on request and
citizens may share state directly with each other. The invariant that keeps
the graph sound:

#quote(block: true)[No atom shares data in a way the `Dispatcher` cannot
see.]

mobius-lang enforces this invariant *by construction*: an edge exists only
by being declared in source, every declared edge is resolved and recorded in
the IR, and the elaborated application retains that record — the entire data
graph is observable because it cannot be expressed any other way.

== Framework contract

The runtime semantics are fixed by the `egui_mobius` architecture (see the
egui_mobius book) and the language must not distort them:

/ Shared state is `Dynamic<T>`: Thread-safe shared values; clones share
  storage; readers observe via subscription (`on_change`) or `Derived<T>`.
  One writer, many readers.
/ The Dispatcher is application-level: One dispatcher runtime per
  application, owned by the application. Citizens never contain or reference
  it; they own only their signal endpoints.
/ Results return through shared state: Events flow *out* of a Citizen over
  signals; backend handlers write results *into* shared `Dynamic<T>` values,
  which every reader — including the emitting Citizen — observes reactively.
  There is no outcome message and no host-mediated apply step.
/ Citizens are compiled Rust: The registry's vocabulary is whatever was
  compiled in. The language instantiates and wires; it does not define
  Citizen interiors.

#pagebreak()
// ============================================================
= Core Entity Types <sec:entities>
// ============================================================

The language declares six kinds of thing:

/ `app`: The composition root — exactly one per source tree. Owns the shared
  state, the Citizen instances, the signal routes, and the layout. The
  analog of Verilog's top-level module.
/ shared value: An application-owned reactive value, declared `let name :
  type = default` at app scope. Elaborates to a `Dynamic<T>`.
/ `struct` / `interface`: A named bundle of shared values. A bare `struct`
  groups values that travel together; an `interface` adds `modport`s —
  directional *views* of the bundle that citizens bind to (@sec:state).
/ citizen instance: A `let` binding whose value is a call on a registered
  Citizen type, with named arguments binding the Citizen's ports to shared
  values (@sec:citizens).
/ `signal` field: A typed `Signal`/`Slot` edge declared *inside an
  interface* alongside the state fields — the push half of the boundary.
  Produced through an `emit` view, consumed on a threaded `Slot` through a
  `drain` view (@sec:signals).
/ handler: A named backend endpoint — a `Slot` run off the UI thread,
  either on a dedicated thread (`Slot::start`) or as an async task
  (`start_async` / `AsyncDispatcher`). Bound to an interface's backend
  modport; implemented in Rust, it answers by writing shared state.

#pagebreak()
// ============================================================
= Declarations: the RPV Form <sec:rpv>
// ============================================================

Every declaration reads `let name : type = value`, terminated by `;`.
Named arguments (`port = value`) are order-independent. Comments are `//` and
are preserved through parse and re-serialization, so source round-trips
intact.

Types in source are *names*, held as strings in the AST and resolved at
elaboration against the registry. `f32`, `bool`, and `String` are primitive
shared-value types; any
registered Rust type name (`Scene`, `Vec<f32>`) is admissible where the
registry declares it #open[exact grammar for generic type names — likely a
restricted subset, not full Rust syntax].

#pagebreak()
// ============================================================
= Shared State & Interfaces <sec:state>
// ============================================================

Shared values are owned by the application, never by a Citizen — that is what
makes them shareable, and it keeps the topology a graph rather than a
parent-owns-child tree.

```rust
app SignalBench {
    let amplitude : f32  = 1.0;
    let enabled   : bool = true;
    let trace     : Vec<f32> = [];
    // ...
}
```

Each elaborates to a `Dynamic<T>`; clones (shared storage) are handed to the
Citizens whose ports bind them.

== The interface is the complete boundary

By the nature of a software application, a citizen's boundary carries *two*
kinds of traffic: shared state (the pull edges — reactive, observed) and
dispatched work (the push edges — a `Signal`/`Slot` pair whose consumer runs
*off* the UI thread, on a dedicated thread or an async task). The
invariant demands that *both* be visible; an interface that described only
state would be lying about being the interface. So an `interface` declares
both kinds of field, and a `modport` — as in SystemVerilog, where it gives
each end of a bus its directional view — states everything one party may do
at the boundary:

```rust
interface Bench {
    // shared state — pull edges, reactive Dynamic<T> values
    amplitude : f32 = 1.0
    enabled   : bool = true
    trace     : Vec<f32> = []

    // dispatched work — push edge; a Signal/Slot pair, the Slot off the UI thread
    commands  : signal BenchCmd

    // modports: one party's complete view of the boundary — what it may
    // write (out), observe (in), produce (emit), and consume (drain).
    modport controls (out amplitude, out enabled, in trace, emit commands)   // the input panel
    modport display  (in  amplitude, in  enabled,  in trace)                 // read-only observers
    modport backend  (in  amplitude, out trace,    drain commands)           // the worker: drains commands, answers through trace
}
```

Four verbs, two per edge kind:

/ `out` / `in`: state — the holder writes the `Dynamic<T>`, or observes it.
  `out` fields arrive as writable clones; `in` fields arrive observe-only
  (a subscription/`Derived` handle, not a writable clone).
/ `emit` / `drain`: dispatched work — the holder produces events onto a
  `Signal`, or consumes them on a `Slot`. A `drain` binding stands up an
  `egui_mobius` `Slot` running on its own thread (`Slot::start`) — or an
  async task (`start_async`) — so the consumer runs *off* the UI thread. It
  answers by writing shared `Dynamic<T>` state, which the UI observes
  reactively; there is no reply edge.

A bare `struct` remains available for grouping plain values with no
directions and no signals.

== The one-writer rule

`Dynamic<T>` does not police writers; the language does. Elaboration checks
that each shared value has *at most one* `out` binding across all citizen
instances and handlers, and rejects the composition otherwise, with spans on
both offending bindings. "One writer ⇒ many readers" stops being a convention
and becomes a diagnostic.

#open[whether a value may declare `inout`-style multiple writers behind an
explicit decorator (`@multi_writer`), or whether that door stays closed.]

#pagebreak()
// ============================================================
= Citizens & the Registry <sec:citizens>
// ============================================================

Citizen *types* live in the registry: compiled-in Rust types registered by the
host binary, each entry declaring its constructor and its *port table* — the
named ports it exposes, each with a type and a direction. The registry is
the resolver for every reference the source cannot define itself.
Instantiation is Verilog module
instantiation in spirit: a named type, ports bound by name, one line per
instance.

`egui_mobius` ships a *standard library* of registered citizens wrapping the
curated ecosystem — `PlotPanel` (`egui_plot`), `QuillEditor` (`egui_quill`),
`LensLogger` (`egui_lens`) — so a full plotter is one instantiation line;
the source places it and names the shared values it displays, and the
drawing stays in Rust.

Instantiation is a `let` binding whose value is a call; named arguments bind
ports to shared values or modport views:

```rust
app SignalBench {
    let amplitude : f32 = 1.0;
    let enabled   : bool = true;
    let trace     : Vec<f32> = [];

    let controls = ControlsPanel(amplitude = amplitude, enabled = enabled);
    let plot     = PlotPanel(trace = trace, amplitude = amplitude);
    let log      = LensLogger(observe = amplitude);
}
```

Elaboration resolves `ControlsPanel` in the registry, checks each named
argument against the port table — unknown port, missing required port, type
mismatch, direction violation (binding a writable port to an `in` view) — and
constructs the instance with the bound clones. Diagnostics carry source spans
and registry knowledge: _"`PlotPanel` has no port `amplitud`; nearest is
`amplitude : in f32`."_

== The registry as a language service

Because the registry knows every Citizen's ports and types, it serves the
tooling as well as the elaborator: completion of citizen types and port
names, hover documentation from the registered entry, and dead-edge analysis
(a shared value nobody reads). #todo[registry API sketch — likely a
`CitizenRegistry` with `register::<T>(name, ports, constructor)` in the host
binary.]

#pagebreak()
// ============================================================
= Citizen Interiors: Widget Primitives <sec:interiors>
// ============================================================

The language follows Verilog's two-level split. Verilog has *primitives*
(gates) built into the language and *modules* built from them; mobius-lang
has *widget primitives* — the standard egui input controls — built into the
language, and *citizens* as the module construct. Rich widgets (a plotter,
the quill editor, the lens logger) are not primitives; they are library
citizens, instantiated by name like any module (@sec:citizens).

== Why the primitive set is closed

Every standard input control is a typed binding and nothing more: a `bool`
edits through a checkbox, an `f32` through a slider or drag with a range, a
`String` through a text box, an enumeration through a combo or radio group,
and an action fires an event. Because the set is closed and each primitive
is pure data, the interpreter renders them generically — a citizen built
from primitives is *live-reloadable source*, requiring no Rust and no
compile.

== Declaring a citizen from primitives

A `citizen` declaration in source has the same signature form as a
registered one — a name and ports — with a body of layout containers and
primitives:

```rust
citizen ControlsPanel (bench : Bench.controls) {
    column {
        checkbox "Enabled"              <-> bench.enabled;
        slider   "Amplitude" 0.0..10.0  <-> bench.amplitude;
        text     "Label"                <-> bench.label;
        // fires into the `commands` signal field — legal because the
        // `controls` modport marks it `emit`
        button   "Apply"                ->  bench.commands.send(BenchCmd::Apply);
    }
}
```

Two binding arrows:

/ `<->`: two-way binding — the widget edits a port the modport marks
  `out`. The port's type selects the admissible primitives; binding a
  slider to a `bool` is an elaboration error.
/ `->`: event — the widget fires into a `signal` field the modport marks
  `emit`. Firing into a field the view does not `emit` is an elaboration
  error.

== The primitive vocabulary

/ input: `checkbox`, `toggle` (`bool`); `slider`, `drag` (`f32`/integer,
  with range); `text` (`String`); `combo`, `radio` (enumeration);
  `button` (event).
/ readout: `label`, `value` (display a shared value), `progress`
  (`f32` fraction). Read-only; bind with `<-`.
/ layout: `column`, `row`, `group`, `separator`, `spacer`.

#open[exact vocabulary boundaries — e.g. color picker, date field — decided
by what egui offers natively and what stays a library citizen.]

== The boundary rule

*If it is a standard input device, it is source; if it draws, it is Rust.*
Output-side widgets beyond trivial readouts — plots, log views, editors,
custom canvases — are open-ended drawing and live in compiled library
citizens (`PlotPanel`, `QuillEditor`, `LensLogger`, and user-written ones),
registered and instantiated by name. This keeps one reload boundary: the
file is live, the registry is compiled.

#pagebreak()
// ============================================================
= Signals & Handlers <sec:signals>
// ============================================================

A `signal` field is a push-based work edge, declared inside an interface next
to the state it travels with (@sec:state). It is an `egui_mobius`
`Signal`/`Slot` pair: the `emit` side pushes onto the `Signal`, the `drain`
side is a `Slot` that runs off the UI thread — on a dedicated thread or an
async task. There is no standalone route declaration: *routing falls out of
the modports*. A handler is bound to a modport like any citizen:

```rust
@wiring {
    let bench    = Bench();
    let controls = ControlsPanel(bench = bench.controls);  // emit commands
    // the worker drains commands and answers through `trace` (out)
    bind "bench_worker" = bench.backend;
}
```

Elaboration creates the `Signal`/`Slot` pair for each `signal` field, hands
the `Signal<T>` end to every `emit` holder, and starts the `Slot` running the
`drain` holder — here the handler registered as `"bench_worker"`. The host
chooses whether that `Slot` runs on a dedicated thread (`Slot::start`) or as
an async task (`start_async` / `AsyncDispatcher`); either way it is off the
UI thread, so backend work never blocks rendering. The event type is resolved against the registry like any
other type; an unknown handler name, a signal with an `emit` holder but no
`drain` holder (dangling), or two `drain` holders (ambiguous) are elaboration
errors, not runtime surprises.

There is deliberately no "response" edge. A handler that produces a result
writes it into a shared value its modport marks `out`; every observer sees
it reactively. The return path is state, per the framework contract
(@sec:lineage) — signals carry *events*, state carries *results*. The
`backend` modport in @sec:state says all of this in one line: `(in
amplitude, out trace, drain commands)` — observe the settings, drain the
requests, answer through the trace.

#pagebreak()
// ============================================================
= Sections: `@wiring` & `@layout` <sec:sections>
// ============================================================

An application, like a board, is two pictures of one thing: the *connective*
picture — who shares what, which events flow where — and the *physical*
picture — where each Citizen sits in the dock. Both live as named sections
of the `app`; sections are organizational, not scoping.

```rust
app SignalBench {
    let amplitude : f32 = 1.0;
    let enabled   : bool = true;
    let trace     : Vec<f32> = [];

    @wiring {
        let controls = ControlsPanel(amplitude = amplitude, enabled = enabled);
        let plot     = PlotPanel(trace = trace, amplitude = amplitude);
        let log      = LensLogger(observe = amplitude);
    }

    @layout {
        dock(plot,     region = center);
        dock(controls, region = left,  fraction = 0.25);
        dock(log,      region = below, fraction = 0.20);
    }
}
```

`@layout` speaks *regions*, not pixels — Citizens dock, detach, and resize;
the host resolves regions to rectangles each frame. Directive statements
(`dock(...)`) use the same named-argument form as instantiation, and the
directive vocabulary is small: `dock`, `tab` (stack two citizens in one
node), `breakaway` (own viewport). #open[the exact region set; whether
`egui_dock` split ordering needs explicit expression or the elaborator
derives it from region + fraction.]

#pagebreak()
// ============================================================
= Elaboration <sec:elab>
// ============================================================

Elaboration turns a parsed source file into a running application, in order:

1. *Resolve* — every citizen type, event type, and handler name against the
   registry; every port argument against the type's port table.
2. *Check* — port types and directions; the one-writer rule per shared
   value; no dangling signal (unknown handler) and, as warnings, dead state
   (no reader) and unbound optional ports.
3. *Lower* — emit the IR (@sec:ir): a flat, fully resolved description of
   the composition. The IR is only ever constructed from a composition that
   already passed every check — the back end performs, it never decides.
4. *Execute* — walk the IR: construct `Dynamic<T>` values, build instances
   with bound clones and observe-only views, create signal/slot pairs,
   register slots with the application dispatcher, hand layout directives
   to the dock host.

== The live-reload boundary

Because citizen interiors are compiled and the registry's vocabulary is
fixed at build time, the reload boundary is precise:

/ Live (re-elaborate on file change): bindings, signal routes, layout,
  default values, adding/removing *instances* of registered types.
/ Compile (touch Rust): a new Citizen type, a new event type, a new
  handler, a changed port table.

The boundary is the one EDA draws at the part library: you may rewire a
board freely; you may not invent a new IC at elaboration time.

#pagebreak()
// ============================================================
= The IR & Debugging <sec:ir>
// ============================================================

The IR is the contract between the front end and every back end — the
interpreter today, a Rust-source emitter later. It is flat and fully
resolved: no names left to look up, no decisions left to make.

== The IR is a netlist

mobius-lang has no runtime control flow — no branches, no loops, nothing
that executes over the application's lifetime. The IR therefore is not a
program; it is a *netlist*. The execute stage builds the described structure
once, at startup, in milliseconds — and from that moment on, everything
running in the process is compiled Rust: citizen interiors, handlers,
`Dynamic`, the dispatcher, egui. Nothing mobius-flavored executes after
wire-up.

== Textual form

The IR has a canonical textual form — greppable, diffable, and
snapshot-testable. `--emit tokens | ast | ir` dumps each stage from the CLI.
For the baseline application (@sec:baseline):

```
value    v0  amplitude : f32       = 1.0    writer=i0
value    v1  enabled   : bool      = true   writer=i0
value    v2  trace     : Vec<f32>  = []     writer=h0
instance i0  ControlsPanel   bind amplitude=v0(rw) enabled=v1(rw) trace=v2(ro)
instance i1  PlotPanel       bind trace=v2(ro) amplitude=v0(ro)
instance i2  LensLogger      bind observe=v0(ro)
signal   s0  BenchCmd  i0 -> handler "bench_worker" (h0)
layout   dock i1 center | dock i0 left 0.25 | dock i2 below 0.20
```

When something misbehaves, diff the IR against what you expected: wrong IR
means a front-end bug; right IR means a back-end bug. That bisection is the
debugging workflow for the pipeline itself.

== Spans are never dropped

Source span → AST node → IR entity. Every IR line remembers the source
location that produced it, so elaboration failures *and* runtime wiring
errors report `signal_bench.mobius:14`, never an anonymous internal state.

== Debugging model: inspect the structure, step the Rust

The division of labor is the one EDA already uses — you do not single-step
a netlist:

/ Structure wrong?: Inspect, don't step. The validation pass is the ERC;
  the IR dump and the live graph panel are the schematic view. The
  application retains the IR it was elaborated from, so a debug panel can
  render the live graph — every `Dynamic` with its current value, writable
  vs observe-only holders, signal routes, and the source line that created
  each edge.
/ Behavior wrong?: That is a Rust bug in a citizen or handler, and it is
  plain `gdb` — breakpoints, backtraces, unchanged from debugging any egui
  code. A breakpoint on `Dynamic::set` identifies a writer at runtime; the
  one-writer rule means the IR already names who it must be.
/ Execute-stage doubt?: The interpreter loop traces each call it makes
  (`wire: Signal<BenchCmd> i0 -> slot "bench_worker"`). The trace is a
  replay of the wiring, short enough to read in full.

Because the language is declarative-only, the class of bugs it can express
is small and *structural* — a wrong binding, route, or region — all visible
in the dump or caught by validation. Races, off-by-ones, and corrupted
state machines can only live in Rust, where the debugger already works.

== The emit escape hatch

If a composition matures to where stepping the wiring itself in a debugger
is wanted, `emit` generates the equivalent Rust source *from the same IR* —
compile it and step through it like a hand-written `main()`. Both back ends
consume one IR; the escape hatch costs nothing until used.

#pagebreak()
// ============================================================
= Open Questions <sec:open>
// ============================================================

+ *Naming.* `mobius-lang` is a working title; the file extension `.mobius`
  is assumed throughout. #open[final name]
+ *Grammar reuse.* Whether the lexer, RPV declaration grammar, and
  span/comment machinery are factored into a reusable `rpv-core` crate
  shared with sibling in-house languages, or owned outright by this
  compiler. Factoring favors the language family; forking favors
  independent evolution.
+ *Parameters on instantiation.* Verilog signatures mix parameters and
  ports, told apart by type. Citizen constructors likely want the same
  (`PlotPanel(history = 4096, trace = trace)`) — does the port table
  simply mark entries as parameter-typed?
+ *Derived values in source.* Whether `let rms : f32 = derived(trace, ...)`
  belongs in the language (elaborating to `Derived<T>`), or derived state
  stays a Rust-side concern registered like any other value.
+ *Arrays of citizens.* Verilog-style generate loops suggest
  `for i in 0..4 { let ch[i] = ScopeChannel(...); }` for multi-channel
  instruments. Deferred until a design needs it.
+ *Theme.* Whether visual theme belongs in `@layout`, its own section, or
  outside the language entirely.

#pagebreak()
// ============================================================
= A Baseline Application (current syntax) <sec:baseline>
// ============================================================

The complete source for the running example — one file, everything the
application *is* apart from the Rust interiors of its three Citizens and one
handler:

```rust
// SignalBench — bench controls, live plot, and a logger.
app SignalBench {

    interface Bench {
        // shared state — pull edges, reactive Dynamic<T> values
        amplitude : f32 = 1.0
        enabled   : bool = true
        trace     : Vec<f32> = []

        // dispatched data — push edge, drained by the dispatcher
        commands  : signal BenchCmd

        // modports: each party's complete view of the boundary —
        // write (out), observe (in), produce (emit), consume (drain)
        modport controls (out amplitude, out enabled, in trace, emit commands)
        modport display  (in  amplitude, in  enabled,  in trace)
        modport backend  (in  amplitude, out trace,    drain commands)
    }

    // An input panel built from widget primitives — pure source, live.
    citizen ControlsPanel (bench : Bench.controls) {
        column {
            checkbox "Enabled"              <-> bench.enabled;
            slider   "Amplitude" 0.0..10.0  <-> bench.amplitude;
            button   "Apply"                ->  bench.commands.send(BenchCmd::Apply);
        }
    }

    @wiring {
        let bench    = Bench();
        let controls = ControlsPanel(bench = bench.controls);
        let plot     = PlotPanel(bench = bench.display);    // stdlib citizen
        let log      = LensLogger(observe = bench.display); // stdlib citizen

        // the worker drains `commands`; results come back through
        // `trace` (out) — written by the handler, observed by the plot.
        bind "bench_worker" = bench.backend;
    }

    @layout {
        dock(plot,     region = center);
        dock(controls, region = left,  fraction = 0.25);
        dock(log,      region = below, fraction = 0.20);
    }
}
```

The host binary registers the `BenchCmd` type and the `"bench_worker"`
handler; `PlotPanel` and `LensLogger` come registered from the standard
library; `ControlsPanel` is defined in source from primitives. Everything
above is data except one handler and the plot's drawing code — both Rust,
both debugged with a debugger.
