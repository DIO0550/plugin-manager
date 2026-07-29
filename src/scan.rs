//! コンポーネントスキャン共通関数
//!
//! ドメイン非依存のスキャンロジックを提供する。
//! Path と String に依存し、ドメイン型への変換はユースケース層で行う。
//!
//! ## 配置スキャン
//!
//! - [`list_placed_components`][]: `target.list_placed()` の戻り値から
//!   Instruction ファイルを除外した `flattened_name` 集合（`HashSet<String>`）を返す
//!
//! ## 低レベル関数
//!
//! - [`list_skill_names`][], [`list_agent_names`][], etc.: 個別コンポーネントのスキャン
//! - [`list_plugin_attached_resources`][]: プラグイン直下の付属リソース

mod attached;
mod components;
mod placement;

pub use attached::{list_plugin_attached_resources, AttachedEntry};
pub use components::{
    file_stem_name, list_agent_names, list_command_names, list_hook_names, list_markdown_names,
    list_skill_names,
};
pub use placement::{is_instruction_file, list_placed_components};
