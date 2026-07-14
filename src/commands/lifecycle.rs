//! plm のライフサイクル系コマンド集約モジュール。
//!
//! `enable` / `disable` / `uninstall` / `update` を束ねる。

pub mod disable;
pub mod enable;
mod toggle;
pub mod uninstall;
pub mod update;
