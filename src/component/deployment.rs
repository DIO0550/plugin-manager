//! コンポーネントのデプロイ処理

use super::convert::{self, AgentConversionResult, AgentFormat, CommandFormat, ConversionResult};
use crate::component::{Component, ComponentKind, Scope};
use crate::error::{PlmError, Result};
use crate::hooks::converter;
use crate::path_ext::PathExt;
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

    /// Hook 名をパスセグメントとして安全な文字列にサニタイズ
    ///
    /// - `[A-Za-z0-9_-]` 以外をハイフンに置換（`.` も除去してトラバーサル防止）
    /// - 先頭・末尾のハイフンを除去
    /// - 空文字列、`.`、`..` の場合はフォールバック名 `_hook` を返す
    pub(crate) fn sanitize_hook_name(name: &str) -> String {
        let sanitized: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let trimmed = sanitized.trim_matches('-');
        if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
            "_hook".to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Hook 変換デプロイを実行
    fn deploy_hook_converted(&self) -> Result<DeploymentResult> {
        // 1. ソース JSON を読み込み
        let input = fs::read_to_string(&self.source_path)?;

        // 2. converter で変換
        let mut convert_result = converter::convert(&input)?;

        // 3. Copilot CLI 形式（wrapper 不要・変換なし）の場合はファイルコピーにフォールバック
        if convert_result.wrapper_scripts.is_empty() && convert_result.warnings.is_empty() {
            self.source_path.copy_file_to(&self.target_path)?;
            return Ok(DeploymentResult::Copied);
        }

        // 4. plugin_root を取得
        let plugin_root = self.plugin_root.as_ref().ok_or_else(|| {
            PlmError::Validation("plugin_root is required for hook conversion".to_string())
        })?;

        // Hook 名をサニタイズ（パスセグメント・シェルコマンドで安全に使えるようにする）
        let safe_name = Self::sanitize_hook_name(&self.name);

        // 4. wrapper パスを名前空間付きに書き換え（生成された wrapper がある場合のみ）
        if !convert_result.wrapper_scripts.is_empty() {
            // JSON 内の bash パスを構造的に書き換え: 生成された wrapper パスのみ対象
            let original_paths: Vec<String> = convert_result
                .wrapper_scripts
                .iter()
                .map(|s| format!("./{}", s.path))
                .collect();

            if let Some(obj) = convert_result.json.as_object_mut() {
                if let Some(hooks) = obj.get_mut("hooks").and_then(|h| h.as_object_mut()) {
                    for (_event, hook_list) in hooks.iter_mut() {
                        if let Some(arr) = hook_list.as_array_mut() {
                            for hook in arr.iter_mut() {
                                if let Some(bash) = hook.get("bash").and_then(|b| b.as_str()) {
                                    if original_paths.contains(&bash.to_string()) {
                                        // ./wrappers/xxx.sh → ./wrappers/{hook-name}/xxx.sh
                                        let new_path = bash.replacen(
                                            "./wrappers/",
                                            &format!("./wrappers/{}/", safe_name),
                                            1,
                                        );
                                        hook.as_object_mut().unwrap().insert(
                                            "bash".to_string(),
                                            serde_json::Value::String(new_path),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // WrapperScriptInfo のパスも更新
            for script in &mut convert_result.wrapper_scripts {
                if let Some(filename) = script.path.strip_prefix("wrappers/") {
                    script.path = format!("wrappers/{}/{}", safe_name, filename);
                }
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

        // 8. wrapper スクリプトを配置
        let wrapper_count = convert_result.wrapper_scripts.len();
        for script in &convert_result.wrapper_scripts {
            let filename = Path::new(&script.path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| script.path.clone());

            let wrapper_dir = self
                .target_path
                .parent()
                .unwrap()
                .join("wrappers")
                .join(&safe_name);
            fs::create_dir_all(&wrapper_dir)?;

            let wrapper_path = wrapper_dir.join(&filename);

            // @@PLUGIN_ROOT@@ を実パスに置換（ダブルクオート内に埋め込まれるためエスケープ）
            let plugin_root_str = plugin_root.display().to_string();
            let escaped = plugin_root_str
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('$', "\\$")
                .replace('`', "\\`");
            let content = script.content.replace("@@PLUGIN_ROOT@@", &escaped);

            fs::write(&wrapper_path, &content)?;

            // Unix: 実行権限を設定
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&wrapper_path, fs::Permissions::from_mode(0o755))?;
            }
        }

        Ok(DeploymentResult::HookConverted(HookConvertResult {
            warnings: convert_result.warnings,
            wrapper_count,
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
    pub wrapper_count: usize,
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
        if kind == ComponentKind::Hook && hook_convert && self.plugin_root.is_none() {
            return Err(PlmError::Validation(
                "plugin_root is required when hook_convert is true".to_string(),
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
            plugin_root: self.plugin_root,
        })
    }
}

#[cfg(test)]
#[path = "deployment_test.rs"]
mod tests;
