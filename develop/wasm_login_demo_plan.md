# Wasm + JS-bridge login demo — Task Plan

A future `examples/wasm_login_demo/` proving the egui_mobius story
in a browser context: gated content behind a login form, with the
gate enforced through a `wasm-bindgen` JS bridge so the demo
visibly demonstrates Rust↔JS interop rather than hiding it behind
a normal web shell.

Drafted 2026-05-07.

## Why this exists

`filter_plotter` already proves egui_mobius runs cleanly in the
browser. What it doesn't show is the **JS interop story**, which
is the next question consumers ask when evaluating wasm-shipped
egui apps for production:

- Can wasm call back into JS? (Yes — `wasm_bindgen`.)
- Can JS push events into wasm? (Yes — same.)
- How do you do auth, analytics, or browser APIs not exposed by
  `web-sys`? (Through the bridge.)

A login form is a natural vehicle because everyone understands the
scenario, the code surface is small, and it forces a non-trivial
exchange between the two layers.

## Architectural decision

Two viable shapes — the demo picks **the second**:

- **JS shell + gated wasm load.** Static HTML+JS login page; on
  success, JS triggers wasm bundle load and passes a token via URL
  param or window global. wasm app inherits an authenticated
  session. Production-shaped.
- **wasm-rendered form with JS bridge.** egui draws the login
  form; on submit, wasm calls a JS function via `wasm_bindgen`
  that does the auth round-trip; result returns to wasm. The wasm
  app gates its own dashboard rendering off the result.

The second is the more interesting demo because the *point* is
making the JS-bridge story concrete. The first is normal web
architecture and the wasm part doesn't really demonstrate
anything specific to egui_mobius.

## Scope guardrails

This is a **mechanism demo, not a production auth template**. The
demo:

- Validates against a hardcoded credential in JS (e.g. `demo` /
  `mobius`) so anyone can run it.
- Loudly labels itself as demo-only in the README and ideally in
  the rendered UI.
- Does not pretend to be secure — no real tokens, no real backend,
  no claim that this is how to build authenticated apps.

A follow-up demo can wire to a real auth provider (Auth0, Supabase,
custom backend). That's intentionally out of scope here.

## Layout

```
examples/wasm_login_demo/
├── Cargo.toml          # eframe + wasm-bindgen + web-sys + egui_mobius
├── Trunk.toml
├── index.html          # mounting div + small JS stub for the bridge fns
├── src/
│   ├── main.rs         # eframe::App with auth state, citizen routing
│   ├── login.rs        # citizen with username + password atoms
│   ├── auth.rs         # wasm-bindgen extern "C" stubs to JS + Rust callers
│   └── dashboard.rs    # post-login content — small reactive widget set
└── README.md           # how the bridge works, demo-only disclaimer
```

## Phases

Each phase is one visible diff in the browser. Roughly:

- **Phase 0 — wasm scaffold.** Trunk + index.html + a hello-world
  eframe::App that says "wasm boot OK" and uses the egui_mobius
  citizen pattern with one trivial citizen. Verify `trunk serve`
  works and the build artefact is sane.
- **Phase 1 — login citizen.** A citizen rendering username and
  password fields plus a Sign In button. No auth wired yet —
  clicking the button just toggles to the dashboard. Establishes
  the citizen-pattern state shape (Dynamic<LoginState> with
  username, password, error message).
- **Phase 2 — JS bridge plumbing.** Add `wasm-bindgen` extern fns
  in `auth.rs`. JS side defines `verify_credentials(u, p)` that
  returns a Promise resolving to a bool. Rust calls it from the
  Sign In handler and awaits via wasm-bindgen-futures. Logger
  citizen records the bridge call as a custom log entry so the
  user sees Rust↔JS round-trips happening live.
- **Phase 3 — gated dashboard.** App routes between Login citizen
  and Dashboard citizen based on auth state. Sign Out button on
  the dashboard clears state and routes back to login. Failed
  logins show an error message in the login citizen.
- **Phase 4 — bridge demo extras.** Inside the dashboard, a few
  buttons that call into JS for things only JS can easily do:
  - "What's my browser?" calls `navigator.userAgent` via JS,
    returns to wasm, displays.
  - "Take screenshot" or "Copy to clipboard" using the JS
    Clipboard API.
  - "Show JS console message" to demonstrate Rust → JS one-way
    communication.
  These keep the bridge story visible after login lands.
- **Phase 5 — polish + deploy.** Tokyo Night styling consistent
  with zicad / CopperForge, README walking through the bridge
  pattern, GitHub Pages deployment alongside the
  filter_plotter demo.

## Open decisions

- **Login state shape.** Plain struct on Dynamic, or use `egui_mobius`
  signals/slots for the JS round-trip? Probably plain Dynamic with
  `wasm-bindgen-futures` driving the async call — signals/slots
  add complexity the demo doesn't need.
- **JS demo functions.** Which browser APIs to showcase. User
  agent, clipboard, console.log are obvious starters; geolocation
  or notifications would be more visually impressive but might
  require permissions that confuse the demo.
- **Deploy target.** GitHub Pages alongside `filter_plotter` keeps
  the demos co-located. Cloudflare Pages would give brotli
  compression for smaller wasm bundles — worth doing once the
  demo is real.

## References

- `filter_plotter` — the existing wasm reference implementation in
  this workspace. Trunk config, deploy workflow, and the `#[cfg(target_arch = "wasm32")]`
  entrypoint pattern all transfer over.
- [`wasm-bindgen` book](https://rustwasm.github.io/wasm-bindgen/) —
  authoritative reference for `extern "C"` blocks, `js_namespace`,
  and the `JsValue` round-trip surface.
- [`web-sys`](https://docs.rs/web-sys) — browser APIs accessible
  from Rust without writing custom JS bridge functions.
- [`wasm-bindgen-futures`](https://docs.rs/wasm-bindgen-futures) —
  Rust async/await over JS Promises.
