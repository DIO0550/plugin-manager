//! コンポーネントスキャン共通関数
//!
//! ドメイン非依存のスキャンロジックを提供する。
//! Path と String に依存し、ドメイン型への変換はユースケース層で行う。
//!
//! ## 配置スキャン
//!
//! - [`list_placed_components`]: `target.list_placed()` の戻り値から
//!   Instruction ファイルを除外した `flattened_name` 集合（`HashSet<String>`）を返す
//!
//! ## 低レベル関数
//!
//! - [`list_skill_names`], [`list_agent_names`], etc.: 個別コンポーネントのスキャン

mod components;
mod constants;
mod placement;

pub use components::{
    file_stem_name, list_agent_names, list_command_names, list_hook_names, list_markdown_names,
    list_skill_names,
};
pub use constants::{
    DEFAULT_AGENTS_DIR, DEFAULT_COMMANDS_DIR, DEFAULT_HOOKS_DIR, DEFAULT_INSTRUCTIONS_DIR,
    DEFAULT_INSTRUCTIONS_FILE, DEFAULT_SKILLS_DIR,
};
pub use placement::list_placed_components;
