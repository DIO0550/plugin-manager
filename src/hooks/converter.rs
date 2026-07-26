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

/// Default named-hook key used by Antigravity conversion (before deploy rename).
pub fn antigravity_default_hook_name() -> &'static str {
    antigravity::ANTIGRAVITY_DEFAULT_HOOK_NAME
}

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
