//! Hook conversion sub-parent.
//!
//! Groups the polymorphic conversion engine (`converter`) with target-specific
//! adapters (`codex`, `copilot`). Lifts the leaf `converter` symbols into the
//! sub-parent's namespace so the external path
//! `crate::hooks::converter::*` continues to resolve unchanged.

mod codex;
#[allow(clippy::module_inception)]
mod converter;
mod copilot;

pub use self::converter::*;

#[cfg(test)]
mod codex_test;
#[cfg(test)]
mod converter_test;
#[cfg(test)]
mod copilot_test;
