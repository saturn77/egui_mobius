# Two panels talking

> **Stub.** Needs a new example: `examples/two_panels_reactive/`.

One panel's activation drives another panel's content. Pure reactive —
no messages, just `Dynamic<T>` reads in panel B's `show()`.

Frame this by contrast with the [problem chapter](../concepts/problem.md):
two panels racing on a shared bool become two panels reading from a
shared `Dynamic<bool>` and never colliding.
