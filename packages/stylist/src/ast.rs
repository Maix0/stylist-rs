//! This module contains the semantic representation of a CSS StyleSheet.
//!
//! ```text
//! struct Sheet
//! └── Vec<enum ScopeContent>
//!     ├── struct Block
//!     │   ├── condition: Vec<Selector>
//!     │   └── Vec<struct StyleAttribute>
//!     │       ├── key: String
//!     │       └── value: Vec<StringFragment>
//!     └── struct Rule
//!         ├── condition: Vec<StringFragment>
//!         └── Vec<enum RuleContent>
//!             ├── Block (*)
//!             └── Rule (*)
//! ```
//!
//! # Note
//!
//! This module is not stable at the moment and is exposed to be used by procedural macros.
//! Its API may change at anytime.

#[doc(inline)]
pub use stylist_core::ast::*;

#[doc(inline)]
pub use stylist_core::bow::Bow;
