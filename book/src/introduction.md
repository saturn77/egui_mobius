# Introduction

`egui_mobius` primarily consists of itself, `egui_mobius_reactive`, and `egui_citizen.` The citizen crate is a design pattern on top of egui_mobius_reactive which facilitates robust development of 
flexible, maintainable applications. 

Overall egui_citizen is the preferred general pattern of working with egui_mobius in the sense that most modern applications will typically have dockable panels to make the Ui ergonomic and modern. Coupled with background threading, these are the two primary focus areas of GUI design, well supported by the citizen pattern: 

- Dockable panels & modern Ui 
- Threading and support for async operation 

This book goes through `egui_citizen` and `egui_mobius_reactive` to illustrate the fundamental design pattern and provide explanations of the underlying code so that one knows what the framework and patterns are doing under the hood. 

![CopperForge running on egui_citizen — a docked layout with a 3D gerber view, settings, terminal, and logger panels updating live as the user drives the app.](images/citizen-copper.gif)

*[CopperForge](https://github.com/Atlantix-EDA/CopperForge) — a
real-world `egui_citizen` + `egui_dock` application for PCB gerber
inspection. Each docked region is a citizen-panel; the panels share
state through reactive cells, and the 3D rendering thread is
coordinated through the dispatcher.*

This guide is written organically, with the human focus, and is meant to be free-flowing and logical and yet easy to read. 

> **Who this book is for**
>
> This book is best served by those familiar with egui, and some
> notion of dockable widgets either through use of something like Qt
> Ads (Qt's Advanced Docking Widgets) or through **egui_dock** itself. > Familiarty with shared memory concepts and threading is important. 

The general scope and area of application of egui_citizen is building 
engineering tooling and small to medium sized applications, particularly
where dockable panels help organize the app and provide the modern
Ui polish. Whe one develops tools, it is often done after not looking
at a framework for a while. 

So a framework or pattern that is easy to pick back up is essential ! 

Basically one of the goals of egui_citizen is to provide a consistent
basis in which to make applications such as:  

- Instrumentation apps (serial / usb port monitors, modbus, plotting)
- Simulation front end apps (provide parameters, thread simulation)
- Project apps (datbase management, project tracking)
- 2D and 3D graphics interfacing with egui 
- Web interfacing and storage (fetch / async)

`egui_citizen` provides a pattern in which reactive state can simply be shared between panels, with a minimal dispatcher. The other main approach is to use backend processes in which the dispatcher becomes
more involved in the overall design. This guide will illustrate both
approaches. 


![A typical multi-panel egui_citizen / egui_dock app: top and bottom ribbons frame a 2×2 dock layout — Project / Settings (top-left), Plotter 1 / Plotter 2 (top-right), Logger (bottom-left), and Terminal / Shell (bottom-right).](images/Basic_App.drawio.png)

*A typical multi-panel app layout. Every labelled region is a
candidate citizen-panel; the ribbons are app-shared chrome.*





