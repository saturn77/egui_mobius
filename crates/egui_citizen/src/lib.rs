//! # egui_citizen
//!
//! Panel lifecycle and message dispatch for dockable egui applications.
//!
//! ## The problem
//!
//! In `egui_dock`, when multiple panels are visible across dock nodes, there is
//! no built-in way to track which panel the user last interacted with. Every
//! visible panel's `ui()` runs every frame — if two panels write to the same
//! state, whichever renders last wins. This is a per-frame race condition.
//!
//! ## The solution
//!
//! Give each dock panel a persistent identity ([`CitizenId`]), lifecycle state
//! ([`CitizenState`]), and route state transitions through a central
//! [`Dispatcher`]. State changes happen exactly once, on click — not every frame.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use egui_citizen::{Citizen, CitizenId, CitizenState, CitizenMessage, Dispatcher};
//!
//! // 1. Create a dispatcher and register panels
//! let mut dispatcher = Dispatcher::new();
//! let alpha_state = dispatcher.register(CitizenId::new("alpha"));
//! let beta_state = dispatcher.register(CitizenId::new("beta"));
//!
//! // 2. Activate a citizen (one-hot: one active, rest off)
//! dispatcher.activate(&CitizenId::new("alpha"));
//!
//! // 3. Drain messages after rendering
//! for msg in dispatcher.drain_messages() {
//!     match msg {
//!         CitizenMessage::Activated { id } => println!("{} activated", id),
//!         CitizenMessage::Deactivated { id } => println!("{} deactivated", id),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Two consumer paths
//!
//! State transitions produce both reactive state changes and messages:
//!
//! - **Panels** read [`CitizenState`] directly via `Dynamic<T>` — reactive,
//!   immediate, no polling. A plot panel can observe `alpha.active.get()` and
//!   switch its display without any wiring.
//!
//! - **Backend threads** receive [`CitizenMessage`] via
//!   [`Dispatcher::drain_messages()`] and route them over channels to serial
//!   ports, network connections, or compute tasks.
//!
//! ## With egui_dock
//!
//! Wire citizen activation into `TabViewer::on_tab_button()`:
//!
//! ```text
//! impl egui_dock::TabViewer for MyTabViewer<'_> {
//!     type Tab = MyTab;
//!
//!     fn on_tab_button(&mut self, tab: &mut MyTab, response: &egui::Response) {
//!         if response.clicked() {
//!             self.dispatcher.activate(&tab.citizen_id());
//!         }
//!     }
//!
//!     fn ui(&mut self, ui: &mut egui::Ui, tab: &mut MyTab) {
//!         tab.show(ui);
//!     }
//! }
//!
//! // After DockArea::show(), drain messages:
//! for msg in dispatcher.drain_messages() {
//!     match msg {
//!         CitizenMessage::Activated { id } => { /* update state, notify backend */ }
//!         CitizenMessage::Deactivated { id } => { /* cleanup */ }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Implementing the Citizen trait
//!
//! Each panel struct holds its own `CitizenId` and `CitizenState`:
//!
//! ```rust,no_run
//! use egui_citizen::{Citizen, CitizenId, CitizenState};
//!
//! struct SettingsPanel {
//!     citizen_id: CitizenId,
//!     citizen_state: CitizenState,
//!     // panel-specific fields...
//! }
//!
//! impl SettingsPanel {
//!     fn new(citizen_state: CitizenState) -> Self {
//!         Self {
//!             citizen_id: CitizenId::new("settings"),
//!             citizen_state,
//!         }
//!     }
//! }
//!
//! impl Citizen for SettingsPanel {
//!     fn id(&self) -> &CitizenId { &self.citizen_id }
//!     fn citizen_state(&self) -> &CitizenState { &self.citizen_state }
//!     fn citizen_state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
//! }
//! ```
//!
//! ## Threading example
//!
//! Route citizen messages to a backend thread via a channel:
//!
//! ```text
//! use crossbeam_channel::{unbounded, Sender};
//!
//! // At startup: spawn backend thread
//! let (tx, rx) = unbounded::<CitizenMessage>();
//! std::thread::spawn(move || {
//!     for msg in rx {
//!         match msg {
//!             CitizenMessage::Activated { id } if id.0 == "fetch" => {
//!                 // start an HTTP request, computation, serial read, etc.
//!             }
//!             CitizenMessage::Deactivated { id } if id.0 == "fetch" => {
//!                 // cancel or clean up
//!             }
//!             _ => {}
//!         }
//!     }
//! });
//!
//! // In the update loop, after drain_messages():
//! for msg in dispatcher.drain_messages() {
//!     let _ = tx.send(msg.clone()); // forward to backend
//! }
//! ```
//!
//! ## Design principles
//!
//! - **`activate()` is an encoded set/reset.** Exactly one citizen is active at a time.
//!   Activating one deactivates all others atomically.
//! - **No shared mutable state.** Panels read reactive `Dynamic<T>` fields.
//!   Backend threads receive immutable messages.
//! - **Frame-order independent.** Because messages are queued and drained once
//!   per frame, the order panels render in doesn't matter.
//! - **No dependency on `egui_dock`.** The core crate provides the trait and
//!   dispatcher — you wire it into whatever dock layout you use.

mod citizen;
pub mod dispatcher;
pub mod message;
mod state;

pub use citizen::Citizen;
pub use dispatcher::Dispatcher;
pub use message::{CitizenId, CitizenMessage};
pub use state::CitizenState;
