//! # egui_mobius_widgets
//!
//! A collection of reusable widgets for the egui_mobius framework, designed to enhance your UI with
//! stateful and styled components.
//!
//! ## Features
//!
//! - **StyledButton**: A customizable button with enhanced styling options, perfect for creating
//!   visually consistent and appealing UIs.
//! - **StatefulButton**: A button that maintains its state between frames with customizable colors
//!   and behavior, ideal for toggle switches and start/stop controls.
//!
//! ## Basic Example
//!
//! ```rust,no_run
//! use egui_mobius_widgets::{StyledButton, StatefulButton};
//!
//! fn ui_example(ui: &mut egui::Ui) {
//!     // Create a styled button with custom appearance
//!     let styled_btn = StyledButton::new("Click me")
//!         .margin(egui::Vec2::new(8.0, 4.0))
//!         .rounding(4.0)
//!         .hover_color(egui::Color32::LIGHT_BLUE);
//!
//!     // Create a stateful button (e.g., for a start/stop control)
//!     let mut stateful_btn = StatefulButton::new()
//!         .margin(egui::Vec2::new(8.0, 4.0))
//!         .rounding(4.0)
//!         .run_color(egui::Color32::GREEN)
//!         .stop_color(egui::Color32::RED);
//!
//!     // Handle button interactions
//!     if styled_btn.show(ui).clicked() {
//!         println!("Styled button clicked!");
//!     }
//!
//!     if stateful_btn.show(ui).clicked() {
//!         stateful_btn.set_started(!stateful_btn.is_started());
//!         println!("Process is now {}", 
//!             if stateful_btn.is_started() { "running" } else { "stopped" });
//!     }
//! }
//! ```
//!
//! ### Custom Styling
//!
//! ```rust,no_run
//! use egui_mobius_widgets::StyledButton;
//! use eframe::egui;
//!
//! fn create_themed_button(theme: &str) -> StyledButton {
//!     match theme {
//!         "dark" => StyledButton::new("Dark Theme")
//!             .normal_color(egui::Color32::from_rgb(48, 48, 48))
//!             .hover_color(egui::Color32::from_rgb(64, 64, 64))
//!             .text_color(egui::Color32::WHITE)
//!             .rounding(8.0),
//!         "light" => StyledButton::new("Light Theme")
//!             .normal_color(egui::Color32::from_rgb(240, 240, 240))
//!             .hover_color(egui::Color32::from_rgb(220, 220, 220))
//!             .text_color(egui::Color32::BLACK)
//!             .rounding(8.0),
//!         _ => StyledButton::default(),
//!     }
//! }
//! ```

pub mod styled_button;
pub use styled_button::StyledButton;

pub mod stateful_button;
pub use stateful_button::StatefulButton;