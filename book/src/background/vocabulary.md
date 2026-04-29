## Key vocabulary

Three terms appear throughout this book. Fix them in your head now —
every chapter that follows leans on these:
- **`Dynamic<T>`** — the reactive primitive that citizen-panels and
  atoms both sit on top of. A thread-safe, observable cell that any
  number of handles can point at. Writes through any handle are
  visible through every other handle. Covered in depth below. There is
  also a correspondent `Derived<T>` from `egui_mobius_reactive` that can automatically produce side effects. 
- **citizen-panel** — a dock panel that carries a persistent identity
  ([`CitizenId`](concepts/citizen.md)) and reactive lifecycle state
  ([`CitizenState`](concepts/state.md)), wired into a central
  [`Dispatcher`](concepts/dispatcher.md). The citizen-panel is the
  unit of organization in an `egui_citizen` app.
- **atom** — a single widget inside a citizen-panel: a slider, a
  button, a text field, a checkbox. Atoms are where user input
  originates. They fire events on their citizen-panel's behalf and
  often hold their own reactive state that other panels or backend
  threads read. See the
  [coupling chapter](concepts/coupling.md) for how an atom can wire
  into panel-to-panel state sharing, panel-to-backend messaging, or
  both at once.