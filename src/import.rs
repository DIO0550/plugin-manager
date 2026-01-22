//! インポート機能モジュール
//!
//! Claude Code Plugin形式のリポジトリからコンポーネントをインポートする機能を提供する。

mod registry;

pub use registry::{ImportRecord, ImportRegistry, ImportsConfig};
