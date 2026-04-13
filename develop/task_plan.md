# egui-mobius — Task Plan

## Architecture clarity

egui-mobius has two distinct layers that should be documented separately:

- **egui_mobius_reactive** — foundational. `Dynamic<T>` is the reactive primitive that egui-citizen's `CitizenState` is built on. This is load-bearing infrastructure.
- **egui_mobius** — optional toolkit. Signals, slots, factory — useful patterns for typed backend threading, but interchangeable with crossbeam channels.

### Tasks
- [ ] Document the two-layer distinction in README
- [ ] Make it clear that egui_mobius_reactive is the substrate, egui_mobius is the toolkit

## API simplification

The current signals/slots surface is verbose compared to crossbeam:

```rust
// Current: three concepts, mutable binding
let (signal, mut slot) = factory::create_signal_slot::<MyRequest>();
slot.start(move |request: MyRequest| { ... });
signal.emit(MyRequest::Fetch { url });

// Crossbeam: two concepts, immediately obvious
let (tx, rx) = unbounded();
tx.send(MyRequest::Fetch { url });
```

### Tasks
- [ ] Evaluate simplified channel-like API that preserves typed auto-dispatch
- [ ] Prototype: `let (send, recv) = channel::<MyRequest>();`
- [ ] If API changes: deprecate old factory pattern, add new alongside it
- [ ] Update egui_mobius_template repo
- [ ] Update all examples

## Documentation
- [ ] Update README to reflect relationship with egui-citizen
- [ ] Add getting-started guide (like egui-citizen's)
- [ ] Document when to use signals/slots vs crossbeam channels
