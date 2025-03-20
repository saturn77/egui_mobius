//! Thread-safe reactive state management system.
//! 
//! See the `reactive` module for details.

pub mod reactive;

// Re-export commonly used types
pub use reactive::{Value, Derived, SignalRegistry};
