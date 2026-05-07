//! install コマンド出力の整形ユーティリティ
//!
//! Hook 変換結果を CLI に表示するための副作用なし pure function 群。
//! `commands/install.rs` の出力ループは本モジュールの戻り値を `println!` /
//! `eprintln!` に流すだけのラッパに専念する。

use crate::component::ComponentKind;
use crate::hooks::converter::{ConversionWarning, SourceFormat};
use crate::target::TargetKind;
use owo_colors::OwoColorize;
use std::collections::BTreeSet;

/// `(converted from Claude Code format)` サフィックスを表示すべきかを判定する pure function。
///
/// 仕様（必須 3 分岐）:
/// - `Some(SourceFormat::ClaudeCode)` → `true`（Claude Code 形式から変換された Hook のみ）
/// - `Some(SourceFormat::TargetFormat)` → `false`（Copilot 形式 passthrough の
///   `HookConverted` ルートで warning を伴うケース。false positive 防止）
/// - `None` → `false`（suffix を表示しない）
///
/// `None` は次の 2 ケースで発生する。どちらでも suffix は表示しない:
///
/// 1. **Hook 以外**（Skill / Agent / Command / Instruction）。`PlaceSuccess` 構築時に
///    `hook_source_format` が `None` で初期化される
/// 2. **Hook だが `DeploymentOutput::Copied` 経路**を通ったケース（version 付き
///    Copilot 形式の完全 passthrough。`HookConvertOutput` を経由しないため
///    `source_format` を保持しない）
///
/// この関数は **stdout の suffix 表示専用**。stderr の Hook 警告セクション
/// （skipped events 集約 / Manual rewrite / All events skipped / 個別 Warning）の
/// 表示条件はこの関数を経由しない。warning 群の表示条件は
/// `component_kind == ComponentKind::Hook` かつ各カテゴリの非空判定のみで決まり、
/// `source_format` には依存しない。
pub fn should_show_converted_suffix(source: Option<SourceFormat>) -> bool {
    matches!(source, Some(SourceFormat::ClaudeCode))
}

/// `&[ConversionWarning]` を出力カテゴリへ分類した結果。
///
/// 将来 `ConversionWarning` に新 variant が追加されても、既知の集約対象に
/// 該当しないものは全て `others` に流れる（`match` 末尾の `_ =>` で網羅）。
pub struct ClassifiedWarnings<'a> {
    /// `UnsupportedEvent` から抽出した event 名（一意化・整列済み）。
    ///
    /// **注意**: ここに入るのは「イベント全体が除外された」ケースのみ。
    /// `UnsupportedHookType` は同じイベント内の他のフックが残ることがあるため
    /// `others` 側へ流し、個別の Warning として `Display` の正確な文言で表示する。
    pub skipped: BTreeSet<String>,
    /// `PromptAgentHookStub` の `(hook_type, event)` ペア（入力順保持）
    pub stubs: Vec<(String, String)>,
    /// 上記カテゴリ以外の warning（`UnsupportedHookType` / `RemovedField` /
    /// `MissingVersion` / 将来 variant）。CLI では `format_individual_warning`
    /// 経由で 1 件ずつ出力する。
    pub others: Vec<&'a ConversionWarning>,
}

/// `&[ConversionWarning]` を `ClassifiedWarnings` に分類する pure function。
pub fn classify_hook_warnings(warnings: &[ConversionWarning]) -> ClassifiedWarnings<'_> {
    let mut skipped: BTreeSet<String> = BTreeSet::new();
    let mut stubs: Vec<(String, String)> = Vec::new();
    let mut others: Vec<&ConversionWarning> = Vec::new();

    for w in warnings {
        match w {
            ConversionWarning::UnsupportedEvent { event } => {
                skipped.insert(event.clone());
            }
            ConversionWarning::PromptAgentHookStub { hook_type, event } => {
                stubs.push((hook_type.clone(), event.clone()));
            }
            // `UnsupportedHookType` は「イベント内の特定フックのみ除外」を意味するため
            // 「イベント全体がスキップされた」を示す skipped バケットには入れない。
            // `RemovedField` / `MissingVersion` / 将来追加 variant も含めて others へ流し、
            // `Display` の正確な文言を保ったまま 1 件ずつ出力する。
            _ => {
                others.push(w);
            }
        }
    }

    ClassifiedWarnings {
        skipped,
        stubs,
        others,
    }
}

/// stdout の "+" 行末に付与する変換済みサフィックス（cyan）。
///
/// 例: ` (converted from Claude Code format)`
///
/// 呼び出し側は `should_show_converted_suffix(success.hook_source_format)` が
/// `true` のときのみ呼ぶこと。
pub fn format_converted_hook_suffix() -> String {
    format!(" {}", "(converted from Claude Code format)".cyan())
}

/// 除外イベントの集約警告（stderr / yellow）。
///
/// 集約済み event 集合（`BTreeSet` で一意化・整列済み）を受け取る。
/// 0 件のときは `None`。
///
/// 文言: 件数に応じて単数形 `1 event skipped` / 複数形 `N events skipped` を
/// 切り替える（user-facing CLI なので英語の単複は正確に出す）。
/// `target_kind` でターゲットを示す文言（`Copilot CLI` / `Codex CLI`）を切り替える。
pub fn format_skipped_events_warning(
    events: &BTreeSet<String>,
    target_kind: TargetKind,
) -> Option<String> {
    if events.is_empty() {
        return None;
    }
    let count = events.len();
    let noun = if count == 1 { "event" } else { "events" };
    let list = events.iter().cloned().collect::<Vec<_>>().join(", ");
    Some(format!(
        "  {} {} {} skipped (not supported in {}): {}",
        "Warning:".yellow(),
        count,
        noun,
        target_display_name(target_kind),
        list
    ))
}

/// ターゲット向け表示名（user-facing CLI の警告文言用）。
fn target_display_name(target_kind: TargetKind) -> &'static str {
    match target_kind {
        TargetKind::Copilot => "Copilot CLI",
        TargetKind::Codex => "Codex CLI",
        TargetKind::Antigravity => "Antigravity",
        TargetKind::GeminiCli => "Gemini CLI",
    }
}

/// prompt/agent フックの手動書き換え案内（stderr / magenta + bold 見出し）。
///
/// `(hook_type, event)` のリストを受け取る。0 件のときは `None`。
/// 出力は見出し + 各 stub 行 + Note 行を `\n` で結合した 1 個の `String`。
pub fn format_manual_rewrite_section(stubs: &[(String, String)]) -> Option<String> {
    if stubs.is_empty() {
        return None;
    }
    let count = stubs.len();
    let noun = if count == 1 { "hook" } else { "hooks" };
    let header = format!("Manual rewrite required ({} {}):", count, noun);
    let mut lines: Vec<String> = Vec::with_capacity(stubs.len() + 2);
    lines.push(format!("  {}", header.magenta().bold()));
    for (hook_type, event) in stubs {
        lines.push(format!("    - '{}' hook for event '{}'", hook_type, event));
    }
    lines.push(
        "  Note: stub scripts have been generated; please rewrite them manually.".to_string(),
    );
    Some(lines.join("\n"))
}

/// 変換結果として hook が 1 件も残らなかった場合の追加警告（stderr / yellow）。
///
/// 「空 `hooks.json` を放置したまま気づかれない」状況を防ぐためのセーフティネット。
/// `source_format == Some(SourceFormat::ClaudeCode) && hook_count == 0` のとき返す。
/// それ以外は `None`。
///
/// **判定方針**: 変換後 JSON に残った hook 定義数で判定する。Copilot 変換では
/// script_count と一致するが、Codex 変換では command hook を inline のまま残すため
/// script_count が 0 でも hook_count は 1 以上になり得る。
///
/// **`source_format == Some(TargetFormat)` の扱い**: passthrough でも
/// `deploy_hook_converted()` 側で `hook_count` は常に算出されるが、入力 JSON が
/// そのまま配置されるため変換による消失は起こり得ない。`hook_count == 0` でも
/// それは入力時点の状態（例: `MissingVersion` 警告つきの Copilot 形式で
/// `hooks` が空）であり「変換で消えた」ことを意味しないため、誤検知を避けて
/// ここでは警告を出さない。
///
/// **`source_format == None` の扱い**: Hook 以外（Skill / Agent / ...）または
/// `DeploymentOutput::Copied` 経路の Hook（version 付き Copilot 完全 passthrough）。
/// どちらも警告対象外。
pub fn format_empty_hooks_warning(
    hook_count: usize,
    source_format: Option<SourceFormat>,
) -> Option<String> {
    if hook_count == 0 && source_format == Some(SourceFormat::ClaudeCode) {
        Some(format!(
            "  {} An empty hooks.json was placed; no hooks remained after conversion.",
            "Warning:".yellow()
        ))
    } else {
        None
    }
}

/// 個別 Warning（others カテゴリ全般）を 1 行にフォーマット。
///
/// 既存 `ConversionWarning::Display` をそのまま使い、yellow の `Warning:` を付与。
pub fn format_individual_warning(warning: &ConversionWarning) -> String {
    format!("  {} {}", "Warning:".yellow(), warning)
}

/// Hook ターゲットの success を CLI に表示するための副作用なしレンダリング結果。
pub struct HookRenderOutput {
    /// `+` 行末に追加する suffix。`None` なら追加しない（Copilot passthrough や Hook 以外）。
    pub stdout_suffix: Option<String>,
    /// stderr に順に出力するブロック群。各要素は `format_*` 関数の戻り値で
    /// 複数行を含む可能性がある（例: Manual rewrite セクションは見出し + 行 +
    /// Note を `\n` で結合した 1 個の `String`）。CLI 側は各要素を
    /// `eprintln!("{}", block)` で出力すればよい。
    pub stderr_blocks: Vec<String>,
}

/// Hook の success データから表示用の `HookRenderOutput` を組み立てる pure function。
///
/// 分岐仕様:
/// - `stdout_suffix`: `should_show_converted_suffix(hook_source_format)` が
///   `true` のときのみ `Some(format_converted_hook_suffix())`、それ以外は `None`
/// - `stderr_blocks`: `component_kind == Hook` かつ各カテゴリ非空のときのみ追加。
///   `source_format` 非依存（Copilot 形式 + `MissingVersion` でも warning は出る）
/// - Hook 以外の `component_kind` では空の `HookRenderOutput` を返す
pub fn render_hook_success(
    component_kind: ComponentKind,
    target_kind: TargetKind,
    hook_source_format: Option<SourceFormat>,
    hook_warnings: &[ConversionWarning],
    _script_count: usize,
    hook_count: usize,
) -> HookRenderOutput {
    if component_kind != ComponentKind::Hook {
        return HookRenderOutput {
            stdout_suffix: None,
            stderr_blocks: Vec::new(),
        };
    }

    let stdout_suffix = if should_show_converted_suffix(hook_source_format) {
        Some(format_converted_hook_suffix())
    } else {
        None
    };

    let classified = classify_hook_warnings(hook_warnings);
    let mut stderr_blocks: Vec<String> = Vec::new();

    if let Some(line) = format_skipped_events_warning(&classified.skipped, target_kind) {
        stderr_blocks.push(line);
    }
    if let Some(section) = format_manual_rewrite_section(&classified.stubs) {
        stderr_blocks.push(section);
    }
    // 空 `hooks.json` 警告は変換後に残った hook 定義数で判定する。
    if let Some(line) = format_empty_hooks_warning(hook_count, hook_source_format) {
        stderr_blocks.push(line);
    }
    for w in &classified.others {
        stderr_blocks.push(format_individual_warning(w));
    }

    HookRenderOutput {
        stdout_suffix,
        stderr_blocks,
    }
}

#[cfg(test)]
#[path = "format_test.rs"]
mod tests;
