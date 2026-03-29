//! コンポーネントのデプロイ処理

use super::convert::{self, AgentConversionResult, AgentFormat, CommandFormat, ConversionResult};
use crate::component::{Component, ComponentKind, Scope};
use crate::error::{PlmError, Result};
use crate::hooks::converter::{self, SourceFormat, SCRIPTS_DIR};
use crate::hooks::name::HookName;
use crate::path_ext::PathExt;
use crate::target::TargetKind;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// コンポーネントのデプロイ情報
///
/// 配置の実行（コピー/削除など）を担当する。
/// 配置先の決定は `PlacementLocation` が担当する。
#[derive(Debug, Clone)]
pub struct ComponentDeployment {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Scope,
    source_path: PathBuf,
    target_path: PathBuf,
    /// ソースの Command フォーマット（Command の場合のみ有効）
    source_format: Option<CommandFormat>,
    /// ターゲットの Command フォーマット（Command の場合のみ有効）
    dest_format: Option<CommandFormat>,
    /// ソースの Agent フォーマット（Agent の場合のみ有効）
    source_agent_format: Option<AgentFormat>,
    /// ターゲットの Agent フォーマット（Agent の場合のみ有効）
    dest_agent_format: Option<AgentFormat>,
    /// Hook 変換を実行するかどうか
    hook_convert: bool,
    /// Hook 変換のターゲット種別
    target_kind: Option<TargetKind>,
    /// @@PLUGIN_ROOT@@ 置換用のプラグインキャッシュルートパス
    plugin_root: Option<PathBuf>,
}

impl ComponentDeployment {
    /// Builderを生成
    pub fn builder() -> ComponentDeploymentBuilder {
        ComponentDeploymentBuilder::new()
    }

    /// 配置先パスを取得
    pub fn path(&self) -> &Path {
        &self.target_path
    }

    /// ソースパスを取得
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// JSON 内の hooks[].bash パスを名前空間付きに書き換える
    fn rewrite_script_paths_in_json(
        json: &mut serde_json::Value,
        original_paths: &HashSet<String>,
        safe_name: &str,
    ) {
        let Some(hooks) = json.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
            return;
        };

        let original_prefix = format!("./{}/", SCRIPTS_DIR);
        let namespaced_prefix = format!("./{}/{}/", SCRIPTS_DIR, safe_name);

        let hook_defs = hooks
            .values_mut()
            .filter_map(|v| v.as_array_mut())
            .flatten();

        for hook in hook_defs {
            let Some(bash) = hook.get("bash").and_then(|b| b.as_str()) else {
                continue;
            };
            if !original_paths.contains(bash) {
                continue;
            }
            let new_path = bash.replacen(&original_prefix, &namespaced_prefix, 1);
            hook.as_object_mut()
                .unwrap()
                .insert("bash".to_string(), serde_json::Value::String(new_path));
        }
    }

    /// Hook 変換デプロイを実行
    fn deploy_hook_converted(&self) -> Result<DeploymentResult> {
        // 1. ソース JSON を読み込み
        let input = fs::read_to_string(&self.source_path)?;

        // 2. converter で変換（target_kind は build() で検証済み）
        let target_kind = self.target_kind.unwrap();
        let mut convert_result = converter::convert(&input, target_kind)?;

        // 3. ソース形式が既にターゲット形式（passthrough）で、script/warning が不要な場合はファイルコピー
        //    変換後 JSON の version フィールドではなく、converter が検出した source_format で判定する。
        //    これにより Claude Code 入力が誤って passthrough 扱いされることを防ぐ。
        if convert_result.source_format == SourceFormat::TargetFormat
            && convert_result.scripts.is_empty()
            && convert_result.warnings.is_empty()
        {
            self.source_path.copy_file_to(&self.target_path)?;
            return Ok(DeploymentResult::Copied);
        }

        // Hook 名をサニタイズ（パスセグメント・シェルコマンドで安全に使えるようにする）
        let hook_name = HookName::new(&self.name);
        let safe_name = hook_name.as_safe();

        // 4. script パスを名前空間付きに書き換え（生成された script がある場合のみ）
        if !convert_result.scripts.is_empty() {
            let original_paths: HashSet<String> = convert_result
                .scripts
                .iter()
                .map(|s| format!("./{}", s.path))
                .collect();

            Self::rewrite_script_paths_in_json(
                &mut convert_result.json,
                &original_paths,
                safe_name,
            );

            for script in &mut convert_result.scripts {
                script.path = converter::namespace_script_path(&script.path, safe_name);
            }
        }

        // 5. JSON をシリアライズ
        let json_str = serde_json::to_string_pretty(&convert_result.json)
            .map_err(|e| PlmError::HookConversion(format!("Failed to serialize JSON: {}", e)))?;

        // 6. ターゲットディレクトリを作成
        if let Some(parent) = self.target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 7. 変換済み JSON を書き出し
        fs::write(&self.target_path, &json_str)?;

        // 8. スクリプトを配置
        let script_count = convert_result.scripts.len();
        if script_count == 0 {
            return Ok(DeploymentResult::HookConverted(HookConvertResult {
                warnings: convert_result.warnings,
                script_count: 0,
                summary: None,
            }));
        }

        let plugin_root = self.plugin_root.as_ref().ok_or_else(|| {
            PlmError::Validation("plugin_root is required when scripts are generated".to_string())
        })?;
        let plugin_root_str = plugin_root.display().to_string();

        let script_dir = self
            .target_path
            .parent()
            .ok_or_else(|| {
                PlmError::Validation(
                    "target_path must have a parent directory for scripts".to_string(),
                )
            })?
            .join(SCRIPTS_DIR)
            .join(safe_name);
        fs::create_dir_all(&script_dir)?;

        for script in &convert_result.scripts {
            let filename = Path::new(&script.path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| script.path.clone());

            let script_path = script_dir.join(&filename);

            // @@PLUGIN_ROOT@@ を実パスに置換
            // bash ダブルクオート内で特別な意味を持つ文字のみをエスケープする
            let escaped = {
                let mut out = String::with_capacity(plugin_root_str.len());
                for ch in plugin_root_str.chars() {
                    match ch {
                        '\\' | '"' | '$' | '`' => {
                            out.push('\\');
                            out.push(ch);
                        }
                        '\n' => {
                            out.push('\\');
                            out.push('n');
                        }
                        _ => out.push(ch),
                    }
                }
                out
            };
            let content = script.content.replace("@@PLUGIN_ROOT@@", &escaped);

            fs::write(&script_path, &content)?;

            // Unix: 実行権限を設定
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
            }
        }

        Ok(DeploymentResult::HookConverted(HookConvertResult {
            warnings: convert_result.warnings,
            script_count,
            summary: None,
        }))
    }

    /// 配置を実行（ファイルコピー）
    ///
    /// Command コンポーネントで `source_format` と `dest_format` が設定されている場合、
    /// Command フォーマット変換を行う。
    /// Agent コンポーネントで `source_agent_format` と `dest_agent_format` が設定されている場合、
    /// Agent フォーマット変換を行う。
    pub fn execute(&self) -> Result<DeploymentResult> {
        match self.kind {
            ComponentKind::Skill => {
                // Skills are directories
                self.source_path.copy_dir_to(&self.target_path)?;
                Ok(DeploymentResult::Copied)
            }
            ComponentKind::Command => {
                // Command は変換の可能性がある
                if let (Some(src_fmt), Some(dest_fmt)) = (self.source_format, self.dest_format) {
                    let result = convert::convert_and_write(
                        &self.source_path,
                        &self.target_path,
                        src_fmt,
                        dest_fmt,
                    )?;
                    Ok(DeploymentResult::Converted(result))
                } else {
                    self.source_path.copy_file_to(&self.target_path)?;
                    Ok(DeploymentResult::Copied)
                }
            }
            ComponentKind::Agent => {
                // Agent は変換の可能性がある
                if let (Some(src_fmt), Some(dest_fmt)) =
                    (self.source_agent_format, self.dest_agent_format)
                {
                    let result = convert::convert_agent_and_write(
                        &self.source_path,
                        &self.target_path,
                        src_fmt,
                        dest_fmt,
                    )?;
                    Ok(DeploymentResult::AgentConverted(result))
                } else {
                    self.source_path.copy_file_to(&self.target_path)?;
                    Ok(DeploymentResult::Copied)
                }
            }
            ComponentKind::Instruction => {
                self.source_path.copy_file_to(&self.target_path)?;
                Ok(DeploymentResult::Copied)
            }
            ComponentKind::Hook => {
                if self.hook_convert {
                    self.deploy_hook_converted()
                } else {
                    self.source_path.copy_file_to(&self.target_path)?;
                    Ok(DeploymentResult::Copied)
                }
            }
        }
    }
}

/// 変換時に除外された理由
#[derive(Debug)]
pub enum ExcludeReason {
    /// サポートされていないイベント
    UnsupportedEvent,
    /// マッピングが存在しない
    NoMapping,
    /// サポートされていないフック種別
    UnsupportedHookType { hook_type: String },
}

impl std::fmt::Display for ExcludeReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExcludeReason::UnsupportedEvent => write!(f, "Unsupported event"),
            ExcludeReason::NoMapping => write!(f, "No mapping available"),
            ExcludeReason::UnsupportedHookType { hook_type } => {
                write!(f, "Unsupported hook type: {}", hook_type)
            }
        }
    }
}

/// 変換時に除外されたアイテム
#[derive(Debug)]
pub struct ExcludedItem {
    /// 除外されたイベント名
    pub event_name: String,
    /// 除外理由
    pub reason: ExcludeReason,
}

/// 変換サマリー結果
#[derive(Debug)]
pub struct ConvertedSummaryResult {
    /// 変換マッピング（変換元イベント名 -> 変換先イベント名）
    pub mappings: BTreeMap<String, String>,
    /// 除外されたアイテムのリスト
    pub excluded: Vec<ExcludedItem>,
}

/// デプロイ結果
#[derive(Debug)]
pub enum DeploymentResult {
    /// ファイルコピーのみ
    Copied,
    /// Command フォーマット変換が行われた
    Converted(ConversionResult),
    /// Agent フォーマット変換が行われた
    AgentConverted(AgentConversionResult),
    /// Hook 変換が行われた
    HookConverted(HookConvertResult),
}

/// Hook 変換結果
#[derive(Debug)]
pub struct HookConvertResult {
    pub warnings: Vec<crate::hooks::converter::ConversionWarning>,
    pub script_count: usize,
    /// 変換サマリー（変換マッピング + 除外リスト）
    pub summary: Option<ConvertedSummaryResult>,
}

impl std::fmt::Display for DeploymentResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentResult::Copied => write!(f, "Copied"),
            DeploymentResult::Converted(conv) => {
                if conv.converted {
                    write!(
                        f,
                        "Converted: {} → {}",
                        conv.source_format, conv.dest_format
                    )
                } else {
                    write!(f, "Copied (no conversion needed)")
                }
            }
            DeploymentResult::AgentConverted(conv) => {
                if conv.converted {
                    write!(
                        f,
                        "Agent converted: {} → {}",
                        conv.source_format, conv.dest_format
                    )
                } else {
                    write!(f, "Copied (no agent conversion needed)")
                }
            }
            DeploymentResult::HookConverted(hr) => {
                write!(
                    f,
                    "Hook converted ({} script{}, {} warning{})",
                    hr.script_count,
                    if hr.script_count == 1 { "" } else { "s" },
                    hr.warnings.len(),
                    if hr.warnings.len() == 1 { "" } else { "s" }
                )
            }
        }
    }
}

/// ComponentDeployment のビルダー
#[derive(Debug, Default)]
pub struct ComponentDeploymentBuilder {
    kind: Option<ComponentKind>,
    name: Option<String>,
    scope: Option<Scope>,
    source_path: Option<PathBuf>,
    target_path: Option<PathBuf>,
    source_format: Option<CommandFormat>,
    dest_format: Option<CommandFormat>,
    source_agent_format: Option<AgentFormat>,
    dest_agent_format: Option<AgentFormat>,
    hook_convert: Option<bool>,
    target_kind: Option<TargetKind>,
    plugin_root: Option<PathBuf>,
}

impl ComponentDeploymentBuilder {
    /// 新しいビルダーを生成
    pub fn new() -> Self {
        Self::default()
    }

    /// Component から kind, name, source_path を設定
    pub fn component(mut self, component: &Component) -> Self {
        self.kind = Some(component.kind);
        self.name = Some(component.name.clone());
        self.source_path = Some(component.path.clone());
        self
    }

    /// コンポーネント種別を設定
    pub fn kind(mut self, kind: ComponentKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// コンポーネント名を設定
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// スコープを設定
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// ソースパスを設定
    pub fn source_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_path = Some(path.into());
        self
    }

    /// ターゲットパスを設定
    pub fn target_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// ソースの Command フォーマットを設定
    pub fn source_format(mut self, format: CommandFormat) -> Self {
        self.source_format = Some(format);
        self
    }

    /// ターゲットの Command フォーマットを設定
    pub fn dest_format(mut self, format: CommandFormat) -> Self {
        self.dest_format = Some(format);
        self
    }

    /// ソースの Agent フォーマットを設定
    pub fn source_agent_format(mut self, format: AgentFormat) -> Self {
        self.source_agent_format = Some(format);
        self
    }

    /// ターゲットの Agent フォーマットを設定
    pub fn dest_agent_format(mut self, format: AgentFormat) -> Self {
        self.dest_agent_format = Some(format);
        self
    }

    /// Hook 変換を有効化
    pub fn hook_convert(mut self, convert: bool) -> Self {
        self.hook_convert = Some(convert);
        self
    }

    /// Hook 変換のターゲット種別を設定
    pub fn target_kind(mut self, kind: TargetKind) -> Self {
        self.target_kind = Some(kind);
        self
    }

    /// プラグインルートパスを設定（@@PLUGIN_ROOT@@ 置換用）
    pub fn plugin_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.plugin_root = Some(path.into());
        self
    }

    /// ComponentDeployment を構築
    pub fn build(self) -> Result<ComponentDeployment> {
        let kind = self
            .kind
            .ok_or_else(|| PlmError::Validation("kind is required".to_string()))?;
        let name = self
            .name
            .ok_or_else(|| PlmError::Validation("name is required".to_string()))?;
        let scope = self
            .scope
            .ok_or_else(|| PlmError::Validation("scope is required".to_string()))?;
        let source_path = self
            .source_path
            .ok_or_else(|| PlmError::Validation("source_path is required".to_string()))?;
        let target_path = self
            .target_path
            .ok_or_else(|| PlmError::Validation("target_path is required".to_string()))?;

        let hook_convert = self.hook_convert.unwrap_or(false);

        if hook_convert && self.target_kind.is_none() {
            return Err(PlmError::Validation(
                "target_kind is required when hook_convert is enabled".to_string(),
            ));
        }

        Ok(ComponentDeployment {
            kind,
            name,
            scope,
            source_path,
            target_path,
            source_format: self.source_format,
            dest_format: self.dest_format,
            source_agent_format: self.source_agent_format,
            dest_agent_format: self.dest_agent_format,
            hook_convert,
            target_kind: self.target_kind,
            plugin_root: self.plugin_root,
        })
    }
}

/// スクリプトファイルを書き出し、Unix では実行権限 (0o755) を設定する。
fn write_executable_script(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

/// bash ダブルクォート内で特別な意味を持つ文字をエスケープする。
///
/// 対象文字: `\`, `"`, `$`, `` ` ``, `\n`
/// 用途: `@@PLUGIN_ROOT@@` プレースホルダーの置換値として使用
fn escape_for_bash_double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' | '"' | '$' | '`' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => {
                out.push('\\');
                out.push('n');
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
#[path = "deployment_test.rs"]
mod tests;
