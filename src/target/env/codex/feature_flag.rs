//! Codex `config.toml` の `[features] codex_hooks` 自動追記ロジック。
//!
//! 公式仕様: <https://developers.openai.com/codex/config-advanced>
//! 確認日付: 2026-06-29
//!
//! 注: `features.codex_hooks` は公式ドキュメント上 deprecated alias と
//! 明記されているため、将来 1 箇所差し替えで済むよう定数化する。

use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{value, DocumentMut, Item};

use crate::component::convert::atomic_write;
use crate::error::{PlmError, Result};

/// `[features]` テーブル名（deprecated alias 移行時にここを差し替える）。
const FEATURES_TABLE: &str = "features";
/// `codex_hooks` キー名（同上）。
const CODEX_HOOKS_KEY: &str = "codex_hooks";

/// feature flag 適用結果を表す値オブジェクト。
///
/// - `applied` が `true` のとき、`config.toml` への書き込みが行われた
/// - `applied` が `false` で `skipped_reason` が `Some` のとき、スキップされた
///   （冪等 / 明示的 false / `--no-enable-flag` 等）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureFlagOutcome {
    pub applied: bool,
    pub skipped_reason: Option<String>,
    pub target_path: PathBuf,
}

/// pure helper の戻り値。文字列→文字列の編集結果を表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EditResult {
    /// 編集後の TOML 文字列（書き込みが必要）。
    Changed(String),
    /// すでに `codex_hooks = true` のため変更不要。
    Unchanged,
    /// `codex_hooks = false` が明示設定されているためスキップ。
    SkippedFalse,
}

/// 与えられた TOML 文字列に `[features] codex_hooks = true` を追記する pure 関数。
///
/// - 空文字列 → `[features]\ncodex_hooks = true\n` を返す
/// - `[features]` セクション無し → 末尾に追記
/// - `[features]` あり `codex_hooks` 無し → セクション内に追記
/// - `codex_hooks = true` 既設定 → `Unchanged`
/// - `codex_hooks = false` 既設定 → `SkippedFalse`
/// - TOML パースエラー → `Err`
pub(crate) fn edit_toml_str(input: &str) -> Result<EditResult> {
    let mut doc: DocumentMut = input
        .parse::<DocumentMut>()
        .map_err(|e| PlmError::Parse(format!("config.toml: {e}")))?;

    if let Some(current) = current_codex_hooks_value(&doc) {
        return Ok(match current {
            true => EditResult::Unchanged,
            false => EditResult::SkippedFalse,
        });
    }

    insert_codex_hooks_true(&mut doc);
    Ok(EditResult::Changed(doc.to_string()))
}

/// 現在の `[features].codex_hooks` の bool 値を返す（存在しない / bool でない場合 `None`）。
fn current_codex_hooks_value(doc: &DocumentMut) -> Option<bool> {
    doc.get(FEATURES_TABLE)
        .and_then(Item::as_table_like)
        .and_then(|t| t.get(CODEX_HOOKS_KEY))
        .and_then(Item::as_bool)
}

/// `[features]` テーブルを取得 or 新規挿入し、`codex_hooks = true` を入れる。
fn insert_codex_hooks_true(doc: &mut DocumentMut) {
    let features = doc
        .entry(FEATURES_TABLE)
        .or_insert_with(|| Item::Table(toml_edit::Table::new()));
    if let Some(table) = features.as_table_like_mut() {
        table.insert(CODEX_HOOKS_KEY, value(true));
    }
}

/// `config.toml` に `[features] codex_hooks = true` を適用する。
///
/// - ファイルが存在しない場合は親ディレクトリを作成して新規作成
/// - 既存の他キー・コメント・改行・キー順を保持（toml_edit）
/// - アトミック書き込みは `crate::component::convert::atomic_write` を再利用
pub fn apply_codex_hooks_flag(config_path: &Path) -> Result<FeatureFlagOutcome> {
    let input = match fs::read_to_string(config_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(PlmError::Io(e)),
    };

    match edit_toml_str(&input)? {
        EditResult::Changed(content) => {
            atomic_write(config_path, &content)?;
            Ok(FeatureFlagOutcome {
                applied: true,
                skipped_reason: None,
                target_path: config_path.to_path_buf(),
            })
        }
        EditResult::Unchanged => Ok(FeatureFlagOutcome {
            applied: false,
            skipped_reason: Some("already enabled".to_string()),
            target_path: config_path.to_path_buf(),
        }),
        EditResult::SkippedFalse => Ok(FeatureFlagOutcome {
            applied: false,
            skipped_reason: Some(
                "codex_hooks = false is explicitly set; change manually to enable hooks"
                    .to_string(),
            ),
            target_path: config_path.to_path_buf(),
        }),
    }
}

#[cfg(test)]
#[path = "feature_flag_test.rs"]
mod tests;
