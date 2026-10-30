# Signals and threaded backends

Path A cells answer "what is the current value?" Signals answer "this
just happened" — fire-and-forget events with a typed payload, declared
on the same interface and governed by the same roles. This chapter's
example is the Tray pilot's real contract.

## Declaring signals

```mobius
interface TraySettings {
    sort : SortOrder = SortOrder::Newest

    open : signal f32          // payload: which capture (index)
    delete : signal f32

    role tray (write sort, emit open, emit delete)
    role host (read sort, drain open, drain delete)
}
```

Two new verbs: **`emit`** (this role may fire the signal) and
**`drain`** (this role consumes what was fired). Like `write`, emit
rights are exclusive per seat — and the generator takes them literally:
only a citizen holding an `emit` role grows an outbox for that signal.
A drain-side citizen physically cannot fire it; the field doesn't exist.

## Firing: the `->` event operator

```mobius
for i in 0..4 {
    row {
        label -- t.labels[i];
        button "Open" -> t.open(i);      // index rides the signal
        button "Delete" -> t.delete(i);
    }
}
```

`->` is deliberately not a wire. A wire is continuous assignment; an
event has a direction and a moment. The call's argument is the payload —
an enum variant, a number, a string — typed against the signal's
declaration. Inside a generate loop the loop variable substitutes into
the payload, so row 2's button fires `open(2.0)`.

## What gets generated

Each signal becomes a typed outbox on the emitting citizen and a
routing stub in the app:

```rust
// on the Tray citizen (emit side)
outbox_open: Vec<f32>,
outbox_delete: Vec<f32>,

// in the app's update(), once per frame
for cmd in std::mem::take(&mut self.tray.outbox_open) {
    actions::handle_open(cmd, &self.tray_if);
}
```

```rust
mod actions {
    pub fn handle_open(cmd: f32, _tray_settings: &TraySettings) {
        eprintln!("[open] {cmd:?}");   // scaffolding — replace with real work
    }
}
```

The stubs are scaffolding by design: generate once, then edit. The
generated project is standalone — yours.

## Pushing work to a thread

The handler runs on the UI thread, so anything slow gets spawned. The
pattern is short because the pieces already cooperate: interfaces
derive `Clone`, and `Dynamic<T>` is the thread-safe cell the whole
framework is built on ([Inside Dynamic](../concepts/inside-dynamic.md)).
Clone the interface into the thread, do the work, write results back
into cells — the UI sees them next frame through ordinary Path A:

```rust
pub fn handle_open(cmd: f32, tray: &TraySettings) {
    let tray = tray.clone();                    // cheap: Arc'd cells
    std::thread::spawn(move || {
        let img = load_capture(cmd as usize);   // slow: disk, decode
        tray.labels[cmd as usize].set(img.label);
        tray.thumbs[cmd as usize].set(img.thumb_path);
    });
}
```

That is the whole threading story: **events down (signal → drain →
spawn), state back (cells)**. No channels to plumb, no `&mut` to
schedule, no callback into egui — the thread writes a cell, and
whichever citizens read that cell render the new value on their next
frame. The one-writer rule keeps this honest under concurrency too:
the spawned worker is acting *as* the role that drained the signal,
and it writes only cells that role holds the pen for.

## Choosing between a cell and a signal

| | cell (`write`/`read`) | signal (`emit`/`drain`) |
|---|---|---|
| answers | "what is the value now?" | "what just happened?" |
| consumed | never — always readable | exactly once, by the drain |
| carries | the current `T` | a payload per firing |
| backpressure | last write wins | queued until drained |

If losing intermediate values is fine (a slider position), it's a cell.
If every occurrence matters (open *this* capture, delete *that* one),
it's a signal.
