# citizen-ADF — owning the loop, renting the toolkit

Design notes on the longer-horizon shape of the Citizen Application
Development Framework: citizen-ADF takes control of the render loop and
treats the underlying GUI framework — egui today, masonry later,
perhaps gpui eventually — as a provider of windowing, widgets, and
layout rather than as the host.

Captures a design discussion from the Atlantix-EDA Discord, building on
[`egui_dock_citizen.md`](egui_dock_citizen.md).

## The proposition

> citizen-ADF owns the rendering loop and uses winit, widgets, and
> layout from the *selected* underlying framework.

The destination is sound. Two conditions decide whether it succeeds or
collapses into a lowest-common-denominator mess.

## Condition 1 — winit is the floor; widgets are not

A framework-abstraction layer that makes toolkits 1:1 interchangeable
fails: it can only expose the *intersection* of what every backend
supports, controls behave differently on each, and you ship the lowest
common denominator. That trap is real and must be designed out.

The escape is the choice of shared floor. The common substrate is
**winit and raw events** — windowing and input — not widgets. winit is
not a widget API; it is the event/surface layer that `egui-winit` and
`masonry` already both sit on. Sharing it is safe precisely because it
is substrate, not controls.

So the layering is:

| Layer | Owns |
|-------|------|
| **winit** | windows, raw input events — the common floor |
| **citizen-ADF** | the loop, lifecycle, dispatch, per-citizen render + input offset |
| **citizens** | framework-*native* rendering of their own controls |

A citizen is native to its framework. An egui citizen renders the egui
way; a masonry citizen renders the masonry way. They are **not**
interchangeable, and that is the point — it is the maintainer's
guidance that "a citizen is in charge of how its controls are rendered,"
made structural. "Selected framework" is therefore per-citizen, never a
global switch that pretends the controls are identical.

The ADF is a **loop and coordination** backbone. It is never a widget
compatibility layer.

## Condition 2 — the cost is what you praised about egui

Owning the loop means re-shouldering everything `eframe` currently gives
for free:

- surface configuration and resize,
- the event pump and frame pacing,
- multi-viewport,
- IME and text input,
- AccessKit / accessibility,
- DPI and scale-factor handling.

The "it just works" quality attributed to egui is in large part
*eframe*, not egui. Take the loop and you inherit every platform
papercut eframe absorbs today.

## Sequencing — let the requirement pull you down

Owning the loop is justified only when a concrete capability demands it:

1. **Heterogeneous citizens** — egui *and* masonry in one process,
   immediate and retained side by side. `eframe` is egui-only; masonry
   owns its own loop. Hosting both means owning winit yourself and
   dispatching render and input to each citizen with its own offset — a
   render call to each, offset graphics, offset input passthrough.
2. **Render or input control eframe will not expose.**

Until one of those bites, eframe-hosted citizens plus breakaway via
eframe's viewport API likely reach the goal *without* re-shouldering the
platform burden. `egui_grafica` already proves you can drop to wgpu
paint callbacks inside eframe — the lower level is reachable from within
the harness.

**Test breakaway on eframe first.** If it stalls on a wall eframe will
not break through, that is the signal to take the loop — with a concrete
reason, not a speculative one. Architectural ambition should not push
you under eframe; an unmet requirement should pull you there.

## gpui is a different animal

egui and masonry render into a surface the host can manage, so "ADF owns
the loop, framework supplies widgets and layout" fits them. gpui's value
is its deeply integrated GPU renderer and its own loop. It resists being
treated as "just widgets + layout" — when the roadmap reaches it, gpui
may want to *be* the loop rather than sit beneath one. Plan for gpui as
a distinct integration, not a drop-in third backend.

## Relationship to breakaway panels

Breakaway — see [`egui_dock_citizen.md`](egui_dock_citizen.md) — is the
near-term stepping stone and the natural probe. It exercises
multi-viewport and per-citizen hosting on top of eframe. How far it gets
on eframe alone is the empirical answer to "when must the ADF own the
loop." Build breakaway first; let it tell you whether the loop has to
move.

## Reference point — pane_ui

`pane_ui` (https://github.com/Avon662/pane_ui) is a useful existence
proof for the bottom of this stack. It is a solo, ground-up
immediate-mode UI library built directly on **winit 0.30 + wgpu 29 +
glyphon** for text — no egui, no masonry underneath. UI is defined in
RON files and redrawn each frame.

What it tells us:

- **The floor is one-person-buildable.** Owning winit + wgpu + text is
  not a moonshot, and it sits on the same wgpu 29 / winit 0.30 substrate
  `egui_grafica` already targets. `glyphon` — cosmic-text on wgpu — is
  the concrete answer to the text problem the moment you stop renting
  egui's glyph atlas.
- **It took the path this doc argues against.** pane_ui owns the loop
  *and* builds its own widgets — fourteen widget types, a styling
  system, RON layout. That is precisely the cost Condition 2 warns of:
  once you are down at winit+wgpu, the pull is to also own widgets, and
  then you have built a whole toolkit. The citizen-ADF bet is the
  opposite — own the loop, **rent** native widgets per citizen. pane_ui
  is the alternative we are choosing not to take; it illustrates the
  temptation the bet is designed to resist, not a confirmation of it.

Two pieces transfer:

- **Its integration modes name our decision.** *Standalone* owns the
  window and loop; *Overlay* renders into a caller-owned wgpu surface
  while the caller keeps the loop; *Headless* has neither. Overlay is
  the same seam `egui_grafica` uses to inject wgpu paint callbacks
  today — the Standalone-vs-Overlay split *is* "own the loop vs. ride
  eframe's loop."
- **RON-defined UI with hot-reload** rhymes with grafica's `.canvas`
  DSL — data-driven interface, reload without recompile.

What it does not do: any docking, detachable / floating panels, or
multi-viewport. It is orthogonal to the breakaway work — no help there.

## Summary

- Agree with the destination: citizen-ADF owning the loop, renting
  windowing / widgets / layout from a selected framework.
- Hard guardrail: share winit/events, never unify widgets — no
  compatibility layer, no lowest common denominator.
- Real cost: owning the loop re-implements eframe's platform plumbing.
- Sequence: eframe-first; own the loop only when heterogeneous hosting
  or render control forces it.
- gpui is a separate integration, not a third drop-in backend.
