//! Hooks module for hook configuration conversion.
//!
//! Provides polymorphic conversion layers for Claude Code hooks
//! to various target formats (Copilot CLI, Codex, etc.).

pub mod converter;
pub(crate) mod event;
mod model;
pub(crate) mod tool;

// crate::hooks::name::HookName 経路据置のためのモジュール再エクスポート
// （rustc E0365 回避: model.rs 側で `pub(crate) mod name;` 宣言済み）
pub(crate) use model::name;
