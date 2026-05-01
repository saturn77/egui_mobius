//! Async backend: receives a `WorkRequest` off the signal bus, sleeps
//! to simulate work, returns a `WorkResponse`.
//!
//! `wire_backend` builds the full Tokio-backed pipeline and returns the
//! UI-side handles: a `Signal<WorkRequest>` the UI uses to submit work,
//! and a `Slot<WorkResponse>` the UI starts to receive completions on.
//! The returned `BackendHandle` owns the `AsyncDispatcher` so the Tokio
//! runtime stays alive for the program duration.

use std::time::{Duration, Instant};

use egui_mobius::dispatching::AsyncDispatcher;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;

use crate::state::{WorkRequest, WorkResponse};

/// The async work function. Sleeps for `req.duration_ms` then computes
/// `sin(seed * 2π)` as a stand-in for "real" backend computation. The
/// elapsed time is measured wall-clock so a UI smoke test can confirm
/// the request actually went off-thread.
async fn work(req: WorkRequest) -> WorkResponse {
    let started = Instant::now();
    tokio::time::sleep(Duration::from_millis(req.duration_ms as u64)).await;
    let value = (req.seed * 2.0 * std::f64::consts::PI).sin();
    WorkResponse {
        value,
        elapsed_ms: started.elapsed().as_millis() as u32,
    }
}

/// Owns the `AsyncDispatcher` so the Tokio runtime it spun up stays
/// alive for the program duration. Drop this and the runtime dies and
/// queued work goes silent.
pub struct BackendHandle {
    _dispatcher: AsyncDispatcher<WorkRequest, WorkResponse>,
}

/// Build the work pipeline. Returns:
/// - `Signal<WorkRequest>` — UI thread submits work via `.send(req)`
/// - `Slot<WorkResponse>` — UI thread calls `.start(handler)` on this
///   to receive completions and update `SharedState`
/// - `BackendHandle` — keep alive on the App struct
pub fn wire_backend() -> (Signal<WorkRequest>, Slot<WorkResponse>, BackendHandle) {
    let (work_signal, work_slot) = factory::create_signal_slot::<WorkRequest>();
    let (result_signal, result_slot) = factory::create_signal_slot::<WorkResponse>();

    let dispatcher = AsyncDispatcher::<WorkRequest, WorkResponse>::new();
    dispatcher.attach_async(
        work_slot,
        result_signal,
        |req| async move { work(req).await },
    );

    (
        work_signal,
        result_slot,
        BackendHandle {
            _dispatcher: dispatcher,
        },
    )
}
