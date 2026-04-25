//! コンポーネントのデプロイ処理

mod output;

pub use output::{DeploymentOutput, HookConvertOutput};

use super::convert::{self, AgentFormat, CommandFormat};
use crate::component::{Component, ComponentKind, Scope};
use crate::error::{PlmError, Result};
use crate::hooks::converter::{self, SourceFormat, SCRIPTS_DIR};
use crate::hooks::name::HookName;
use crate::path_ext::PathExt;
use crate::target::TargetKind;
use std::collections::HashSet;
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
    pub(crate) fn builder() -> ComponentDeploymentBuilder {
        ComponentDeploymentBuilder::new()
    }

    /// 配置先パスを取得
    pub fn path(&self) -> &Path {
        &self.target_path
    }

    /// JSON 内の hooks[].bash パスを名前空間付きに書き換える
    ///
    /// # Arguments
    ///
    /// * `json` - Hook definition JSON whose `hooks[].bash` paths should be rewritten in place.
    /// * `original_paths` - Set of original `./<SCRIPTS_DIR>/...` paths eligible for rewriting.
    /// * `safe_name` - Sanitized hook name used as the namespace segment in the new path.
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
    fn deploy_hook_converted(&self) -> Result<DeploymentOutput> {
        let input = fs::read_to_string(&self.source_path)?;

        // target_kind は build() で検証済み
        let target_kind = self.target_kind.unwrap();
        let mut convert_result = converter::convert(&input, target_kind)?;

        // ソース形式が既にターゲット形式（passthrough）で、script/warning が不要な場合はファイルコピー。
        // 変換後 JSON の version フィールドではなく、converter が検出した source_format で判定する。
        // これにより Claude Code 入力が誤って passthrough 扱いされることを防ぐ。
        if convert_result.source_format == SourceFormat::TargetFormat
            && convert_result.scripts.is_empty()
            && convert_result.warnings.is_empty()
        {
            self.source_path.copy_file_to(&self.target_path)?;
            return Ok(DeploymentOutput::Copied);
        }

        // Hook 名をサニタイズ（パスセグメント・シェルコマンドで安全に使えるようにする）
        let hook_name = HookName::new(&self.name);
        let safe_name = hook_name.as_safe();

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

        let json_str = serde_json::to_string_pretty(&convert_result.json)
            .map_err(|e| PlmError::HookConversion(format!("Failed to serialize JSON: {}", e)))?;

        if let Some(parent) = self.target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.target_path, &json_str)?;

        let script_count = convert_result.scripts.len();
        if script_count == 0 {
            return Ok(DeploymentOutput::HookConverted(HookConvertOutput {
                warnings: convert_result.warnings,
                script_count: 0,
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

        let escaped = escape_for_bash_double_quote(&plugin_root_str);

        convert_result.scripts.iter().try_for_each(|script| {
            let filename = Path::new(&script.path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| script.path.clone());

            let script_path = script_dir.join(&filename);
            let content = script.content.replace("@@PLUGIN_ROOT@@", &escaped);

            write_executable_script(&script_path, &content)
        })?;

        Ok(DeploymentOutput::HookConverted(HookConvertOutput {
            warnings: convert_result.warnings,
            script_count,
        }))
    }

    /// 配置を実行（ファイルコピー）
    ///
    /// Command コンポーネントで `source_format` と `dest_format` が設定されている場合、
    /// Command フォーマット変換を行う。
    /// Agent コンポーネントで `source_agent_format` と `dest_agent_format` が設定されている場合、
    /// Agent フォーマット変換を行う。
    pub fn execute(&self) -> Result<DeploymentOutput> {
        match self.kind {
            ComponentKind::Skill => {
                // Skills are directories
                self.source_path.copy_dir_to(&self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
            ComponentKind::Command => {
                if let (Some(src_fmt), Some(dest_fmt)) = (self.source_format, self.dest_format) {
                    let result = convert::convert_and_write(
                        &self.source_path,
                        &self.target_path,
                        src_fmt,
                        dest_fmt,
                    )?;
                    Ok(DeploymentOutput::CommandConverted(result))
                } else {
                    self.source_path.copy_file_to(&self.target_path)?;
                    Ok(DeploymentOutput::Copied)
                }
            }
            ComponentKind::Agent => {
                if let (Some(src_fmt), Some(dest_fmt)) =
                    (self.source_agent_format, self.dest_agent_format)
                {
                    let result = convert::convert_agent_and_write(
                        &self.source_path,
                        &self.target_path,
                        src_fmt,
                        dest_fmt,
                    )?;
                    Ok(DeploymentOutput::AgentConverted(result))
                } else {
                    self.source_path.copy_file_to(&self.target_path)?;
                    Ok(DeploymentOutput::Copied)
                }
            }
            ComponentKind::Instruction => {
                self.source_path.copy_file_to(&self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
            ComponentKind::Hook => {
                if self.hook_convert {
                    self.deploy_hook_converted()
                } else {
                    self.source_path.copy_file_to(&self.target_path)?;
                    Ok(DeploymentOutput::Copied)
                }
            }
        }
    }
}

/// ComponentDeployment のビルダー
#[derive(Debug, Default)]
pub(crate) struct ComponentDeploymentBuilder {
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
    ///
    /// # Arguments
    ///
    /// * `component` - Component whose `kind`, `name`, and `path` are copied into the builder.
    pub fn component(mut self, component: &Component) -> Self {
        self.kind = Some(component.kind);
        self.name = Some(component.name.clone());
        self.source_path = Some(component.path.clone());
        self
    }

    /// コンポーネント種別を設定
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind (`Skill`, `Agent`, `Command`, `Instruction`, `Hook`).
    pub fn kind(mut self, kind: ComponentKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// コンポーネント名を設定
    ///
    /// # Arguments
    ///
    /// * `name` - Component name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// スコープを設定
    ///
    /// # Arguments
    ///
    /// * `scope` - Deployment scope (`Personal` or `Project`).
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// ターゲットパスを設定
    ///
    /// # Arguments
    ///
    /// * `path` - Target path to deploy the component to.
    pub fn target_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// ソースの Command フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Source command format.
    pub fn source_format(mut self, format: CommandFormat) -> Self {
        self.source_format = Some(format);
        self
    }

    /// ターゲットの Command フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Destination command format.
    pub fn dest_format(mut self, format: CommandFormat) -> Self {
        self.dest_format = Some(format);
        self
    }

    /// ソースの Agent フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Source agent format.
    pub fn source_agent_format(mut self, format: AgentFormat) -> Self {
        self.source_agent_format = Some(format);
        self
    }

    /// ターゲットの Agent フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Destination agent format.
    pub fn dest_agent_format(mut self, format: AgentFormat) -> Self {
        self.dest_agent_format = Some(format);
        self
    }

    /// Hook 変換を有効化
    ///
    /// # Arguments
    ///
    /// * `convert` - Whether to run the hook converter during deployment.
    pub fn hook_convert(mut self, convert: bool) -> Self {
        self.hook_convert = Some(convert);
        self
    }

    /// Hook 変換のターゲット種別を設定
    ///
    /// # Arguments
    ///
    /// * `kind` - Target kind that drives hook conversion rules.
    pub fn target_kind(mut self, kind: TargetKind) -> Self {
        self.target_kind = Some(kind);
        self
    }

    /// プラグインルートパスを設定（@@PLUGIN_ROOT@@ 置換用）
    ///
    /// # Arguments
    ///
    /// * `path` - Plugin cache root substituted for the `@@PLUGIN_ROOT@@` placeholder in scripts.
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
///
/// # Arguments
///
/// * `path` - File path to write the script to.
/// * `content` - Script contents to write.
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
///
/// # Arguments
///
/// * `s` - Raw string to escape for safe interpolation inside bash double quotes.
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
