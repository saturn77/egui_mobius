//! Platform module — system info gathering for the lens
//! "System Info" button.
//!
//! Native uses `sysinfo` + `local-ip-address` for the full system
//! probe (OS, kernel, CPU, RAM, IP). WASM falls back to browser-side
//! info from `web_sys::Navigator` and friends — userAgent, hardware
//! concurrency, screen size, language. Both expose the same
//! `format_os()` -> String entry point so the consuming app doesn't
//! have to cfg-gate.

#[cfg(not(target_arch = "wasm32"))]
pub mod details;

#[cfg(target_arch = "wasm32")]
pub mod details_wasm;

#[cfg(target_arch = "wasm32")]
pub use details_wasm as details;
