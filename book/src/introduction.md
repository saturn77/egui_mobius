# Introduction

> **Book version:** 0.4.0 &nbsp;·&nbsp; **Last updated:** 2026-05-03 &nbsp;·&nbsp; tracks `egui_mobius` v0.4.0
>
> The book is *live* — it evolves alongside the framework. Each
> chapter footer notes the date of its last substantive revision.
> Versioning follows the framework's own version: when egui_mobius
> ships v0.5.0, this book becomes 0.5.0.

This book provides know-how for building solid, professional, and flexible graphical user
interfaces. The language of choice is Rust employing the `egui` GUI library and the `egui_mobius` framework.

`egui_mobius` is also the overall name of this mono-repo, and primarily consists of itself, `egui_mobius_reactive`, `egui_citizen`, `egui_lens` (the canonical reactive event logger), and `egui_quill` (the syntax-highlighted editor).

Overall egui_citizen is the preferred general pattern of working with egui_mobius in the sense that most modern applications will typically have dockable panels to make the Ui ergonomic and modern. Coupled with background threading, these are the two primary focus areas of GUI design, well supported by the citizen pattern: 

- Dockable panels & modern Ui 
- Threading and support for async operation 

This book goes through `egui_citizen` and `egui_mobius_reactive` to illustrate the fundamental design pattern and provide explanations of the underlying code to facilitate a deep level of understanding when
employing the framework.  


![CopperForge running on egui_citizen — a docked layout with a 3D gerber view, settings, terminal, and logger panels updating live as the user drives the app.](images/citizen-copper.gif)

*[CopperForge](https://github.com/Atlantix-EDA/CopperForge) — a
real-world `egui_citizen` + `egui_dock` application for PCB gerber
inspection. Each docked region is a citizen-panel; the panels share
state through reactive cells, and the 3D rendering thread is
coordinated through the dispatcher.*


It must also be noted that there are three general levels to a mobius-citizen application. One is
where shared state is between panels, and the dispatcher simply does
state management of panels. A second is where the dispatcher is extended
to handle backend processing. Finally the third level is where signals
and slots are employed between the dispatcher and the backend. The
first two stages employ citizen and mobius_reactive while the third
stage uses all elements of the ecosystem. Thus the signals and slots
are explored via `egui_mobius` for third level applications.

Worked examples for each level live in `examples/`:

- **Level 1** — `getting_started`, `citizen_dock`
- **Level 2** — `filter_plotter`, `citizen_fetch`
- **Level 3** — `citizen_signal_async`

This guide is written organically, with the human focus, and is meant to be free-flowing and logical and yet easy to read. 

> **Who this book is for**
>
> This book is best served by those familiar with egui, and some
> notion of dockable widgets either through use of something like Qt
> Ads (Qt's Advanced Docking Widgets) or through **egui_dock** itself
> Familiarity with shared memory concepts and threading is important. 

The general scope and area of application of egui_citizen is building 
engineering tooling and small to medium sized applications, particularly
where dockable panels help organize the app and provide the modern
Ui polish. When one develops tools, it is often done after not looking
at a framework for a while. 

So a framework or pattern that is easy to pick back up is essential ! 

Basically one of the goals of egui_citizen is to provide a consistent
basis in which to make applications such as:  

- Instrumentation apps (serial / usb port monitors, modbus, plotting)
- Simulation front end apps (provide parameters, thread simulation)
- Project apps (database management, project tracking)
- 2D and 3D graphics interfacing with egui 
- Web interfacing and storage (fetch / async)

`egui_citizen` provides a pattern in which reactive state can simply be shared between panels, with a minimal dispatcher. The other main approach is to use backend processes in which the dispatcher becomes
more involved in the overall design. This guide will illustrate both
approaches. 

`egui_mobius` Signals and Slots can be employed by the dispatcher for
more advanced applications, in which case there is likely multiple background threads all receiving a signal from the dispatcher. Each
background thread can then send a Signal back to a Slot on the dispatcher for task / thread completion. 

This book presents background material related to an examination of 
`Dynamic<T>`, `egui_dock`, and general vocabulary associated with the
`egui_mobius` traits. Concepts are then presented, in an appropriate
level of detail, followed by the Tutorial. A user may go directly to
the Tutorial and refer back to concepts sections as needed. Finally
the book ends with suggestions on patterns and a reference sheet. 






