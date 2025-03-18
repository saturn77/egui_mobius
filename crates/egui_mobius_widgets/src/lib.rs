//! # egui_mobius_widgets
//!
//! A collection of reusable widgets for the egui_mobius framework.
//!
//! ## Features
//!
//! - **StyledButton**: A customizable button with enhanced styling options
//! - **StatefulButton**: A button that maintains its state between frames with customizable colors and behavior
//!
//! ## Example
//!
//! ```rust,no_run
//! use egui_mobius_widgets::{StyledButton, StatefulButton};
//! use eframe::egui;
//!
//! fn ui_example(ui: &mut egui::Ui) {
//!     // Create a styled button
//!     let styled_btn = StyledButton::new("Click me")
//!         .margin(egui::Vec2::new(8.0, 4.0))
//!         .rounding(4.0);
//!
//!     // Create a stateful button
//!     let mut stateful_btn = StatefulButton::new()
//!         .margin(egui::Vec2::new(8.0, 4.0))
//!         .rounding(4.0);
//!
//!     // Use the buttons in your UI
//!     if styled_btn.show(ui).clicked() {
//!         println!("Styled button clicked!");
//!     }
//!
//!     if stateful_btn.show(ui, "Toggle").clicked() {
//!         println!("Stateful button toggled!");
//!     }
//! }
//! ```

pub mod styled_button;
pub use styled_button::StyledButton;

pub mod stateful_button;
pub use stateful_button::StatefulButton;