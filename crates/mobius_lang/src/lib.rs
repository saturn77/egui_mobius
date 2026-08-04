//! # mobius_lang
//!
//! The application composition language for `egui_mobius`: describe *which*
//! citizens exist, *what state they share and in which direction*, *which
//! events they emit and who drains them*, and *where they sit* — as source.
//!
//! This crate is the language front end: lexer, span-carrying AST with
//! preserved comments, and recursive-descent parser. Elaboration (registry
//! resolution, the one-writer check, IR lowering) and the execute stage
//! build on top of it. See `develop/mobius_lang.typ` for the language
//! specification.
//!
//! ```rust
//! let source = r#"
//! app Minimal {
//!     interface State {
//!         level : f32 = 0.5
//!         modport ui (out level)
//!     }
//!     citizen Controls (s : State.ui) {
//!         column {
//!             slider "Level" 0.0..1.0 <-> s.level;
//!         }
//!     }
//!     @wiring {
//!         let state    = State();
//!         let controls = Controls(s = state.ui);
//!     }
//!     @layout {
//!         dock(controls, region = center);
//!     }
//! }
//! "#;
//! let (file, _comments) = mobius_lang::parse(source).unwrap();
//! assert_eq!(file.apps[0].name, "Minimal");
//! ```

pub mod ast;
pub mod error;
pub mod execute;
pub mod ir;
pub mod lexer;
pub mod lower;
pub mod parser;

pub use error::{ParseError, Span};
pub use execute::{Host, WiredApp, wire};
pub use ir::Ir;
pub use lower::{Diagnostic, Registry, lower};
pub use parser::parse;
