# egui-citizen — Task Findings

## egui_dock ui() is NOT a focus callback
In `egui_dock`, `ui()` runs for every *visible* tab across all dock nodes, not just
the focused one. Setting state inside `ui()` causes per-frame races when panels are
undocked. Use `on_tab_button` + `response.clicked()` for state transitions instead.

## activate() must be an encoded set/reset
When multiple algo panels are visible simultaneously, exactly one must be "active".
`CitizenRegistry::activate()` sets one citizen active and deactivates all others —
an encoded set/reset. This pattern is fundamental and cannot be achieved with
egui_dock's built-in focus tracking, which doesn't reliably report the last-clicked tab.

## HCR register values are always positive
The Stabiliti firmware stores all breakpoint values as positive unsigned integers.
Sign and mirroring are handled in the embedded processor, not in the register map.
The GUI must match this convention — store positive, mirror in the plot.

## Dynamic<T> internal shape (2026-04-25)
`egui_mobius_reactive::Dynamic<T>` (v0.3.0-alpha.33) is **not**
`Arc<RwLock<T>>` — it is `Arc<Mutex<T>>` plus a separate
`Arc<parking_lot::Mutex<Vec<Sender<()>>>>` of notifier-channel senders.
`set()` writes through the inner `Mutex` and then sends `()` into every
registered notifier; `on_change(cb)` spawns a dedicated background
thread that waits on the receiver and invokes the callback. This means
`on_change` callbacks run *off* the UI thread — useful for triggering
backend work, less useful for UI-to-UI coupling (where polling via
`.get()` in each `ui()` is simpler and frame-coherent).

## Two coupling paths, with distinct timing (2026-04-25)
Stop conflating shared `Dynamic<T>` and dispatcher messages. They are
two independent mechanisms with different timing:
- **Shared `Dynamic<T>` (UI↔UI)** — instant, in-frame. Writer's
  `.set()` is visible to readers' `.get()` immediately.
- **Dispatcher messages (UI↔backend)** — queued. `.send()` appends;
  consumers don't see the message until the next `drain_messages()`.

Backend threads therefore observe Path B messages *one frame after*
the UI has already observed the Path A value. Fine for normal async
patterns, broken for any in-frame ordering dependency.

The dispatcher is **not** a reactive bus that observes arbitrary
`Dynamic<T>` writes. It only knows about `CitizenState`s registered
via `register()` and its queue only fills from `activate()` and
explicit `send()`.

## Atoms can dual-wire to both paths (2026-04-25)
A single widget inside a citizen-panel — what we now call an "atom" —
can fan out to both coupling paths from the same change handler:

```rust
if ui.add(slider).changed() {
    self.value.set(local);                              // Path A
    dispatcher.send(AppMessage::SliderChanged(local));  // Path B
}
```

Required discipline: pick a source of truth. Either the `Dynamic<T>`
is canonical and the message is a content-free ping (consumers
re-read shared state), or the message carries the value and the
`Dynamic<T>` is a UI mirror (consumers trust the message). Mixing
silently — some consumers reading the Dynamic, others trusting the
message — is where bugs live.

## Permissive type, disciplined use (2026-04-25)
`Dynamic<T>` is **multi-producer, multi-consumer** by type — any clone
can call `.set()`. The single-writer-per-field convention
(`Dispatcher` writes `active`, panel hooks write `clicked`/`selected`,
etc.) is a discipline layered on top, not a type guarantee. This
explains why two dispatchers in one app break the one-hot invariant —
they're two writers to the same logical field.

## Clone in Rust is per-type (2026-04-25)
There is no language-level "deep" or "shallow" default for `Clone`.
Each type's `Clone` impl decides:
- Owned types (`String`, `Vec<T>`, `Box<T>`, `HashMap`) duplicate
  heap data on `.clone()` — what C++ would call deep.
- Reference-counted types (`Arc<T>`, `Rc<T>`) are *documented* to
  clone as a refcount bump — same allocation, new handle.

`Mutex<T>` does not implement `Clone` at all, which is exactly why an
`Arc` wrapper is structurally necessary to share a `Mutex<T>`.
`Dynamic<T>::clone()` is therefore exactly two `Arc::clone` calls —
two refcount bumps, zero data duplication. Sharing is the contract,
not a quirk.

## Writing the book reveals API tensions (2026-04-25)
The act of writing concept chapters surfaces non-obvious places where
the API's vocabulary is unclear or misaligned. First example: the
`Citizen::state()` method name was ambiguous once `PanelState`
emerged as a parallel concept, prompting the rename to
`citizen_state()` to make room for a possible future `panel_state()`
sibling. Future chapters will likely surface more of these. Treat
"tension surfaced by docs" as a real signal, not a stylistic concern.
