//! Hook conversion sub-parent.
//!
//! Groups the polymorphic conversion engine (`converter`) with target-specific
//! adapters (`antigravity`, `codex`, `copilot`, `cursor`). Lifts the leaf
//! `converter` symbols into the sub-parent's namespace so the external path
//! `crate::hooks::converter::*` continues to resolve unchanged.

mod antigravity;
mod codex;
#[allow(clippy::module_inception)]
mod converter;
mod copilot;
mod cursor;

pub use self::converter::*;

#[cfg(test)]
mod antigravity_test;
#[cfg(test)]
mod codex_test;
#[cfg(test)]
mod converter_test;
#[cfg(test)]
mod copilot_test;
#[cfg(test)]
mod cursor_test;
