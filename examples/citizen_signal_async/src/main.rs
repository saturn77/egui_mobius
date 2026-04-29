//! citizen_signal_async — citizen pattern + egui_mobius signal/slot + Tokio backend.
//!
//! Phase 1 — types and dispatcher routing in place. eframe shell and
//! panels follow. See `task_plan.md` and `task_progress.md`.

mod dispatcher;
mod messages;
mod state;

fn main() {
    println!("citizen_signal_async — types in place; eframe shell next.");
}
