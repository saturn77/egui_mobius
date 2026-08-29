# serial_monitor

The citizen pattern against real hardware: a dockable serial monitor
with three citizens — **Monitor** (port picker, baud, connect, TX line
entry), **Console** (received lines), and **Logger** (citizen lifecycle
and TX events).

```bash
cargo run -p serial_monitor
```

## Architecture

The canonical citizen-app shape, with a hardware backend:

- `citizens/monitor.rs` pushes intents (`Connect`, `Send`, …) onto its
  **outbox**; the drain loop in `main.rs` forwards them through
  `monitor_actions::handle` — the dispatch path.
- `backend.rs` runs the serial worker thread (adapted from
  SaturnGridSim): commands arrive over an `mpsc` channel; received
  lines and connection state are written straight into shared
  `Dynamic` cells, and Path A carries them to the Console and Monitor
  on the next frame. One writer per cell throughout.
- The registry does exactly one thing: one-hot activation from tab
  clicks, logged by the Logger via the lifecycle drain.

## No hardware handy?

Create a virtual port pair and feed one end:

```bash
socat -d -d pty,raw,echo=0 pty,raw,echo=0
# socat prints two /dev/pts/N paths. Refresh + connect to one, then:
while true; do echo "tick $(date +%T)" > /dev/pts/M; sleep 1; done
```

Lines appear in the Console; anything you Send lands on the other pty.

On Linux, `serialport` needs libudev (`sudo apt install libudev-dev`),
and your user typically needs to be in the `dialout` group for real
devices.
