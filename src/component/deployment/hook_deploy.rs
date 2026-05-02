//! Hook 変換デプロイ処理

use super::bash::{escape_for_bash_double_quote, write_executable_script};
use super::output::{DeploymentOutput, HookConvertOutput};
use super::ComponentDeployment;
use crate::error::{PlmError, Result};
use crate::hooks::converter::{self, SourceFormat, SCRIPTS_DIR};
use crate::hooks::name::HookName;
use crate::path_ext::PathExt;
use crate::target::TargetKind;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

impl ComponentDeployment {
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
    ///
    /// `target_kind` と `plugin_root` は呼び出し側 (executor の `deploy_hook`) が
    /// `ConversionConfig::Hook` から取り出した値を渡す。
    pub(super) fn deploy_hook_converted(
        &self,
        target_kind: TargetKind,
        plugin_root: Option<&Path>,
    ) -> Result<DeploymentOutput> {
        let input = fs::read_to_string(self.source_path())?;

        let mut convert_result = converter::convert(&input, target_kind)?;

        // ソース形式が既にターゲット形式（passthrough）で、script/warning が不要な場合はファイルコピー。
        // 変換後 JSON の version フィールドではなく、converter が検出した source_format で判定する。
        // これにより Claude Code 入力が誤って passthrough 扱いされることを防ぐ。
        if convert_result.source_format == SourceFormat::TargetFormat
            && convert_result.scripts.is_empty()
            && convert_result.warnings.is_empty()
        {
            self.source_path().copy_file_to(&self.target_path)?;
            return Ok(DeploymentOutput::Copied);
        }

        // Hook 名をサニタイズ（パスセグメント・シェルコマンドで安全に使えるようにする）
        let hook_name = HookName::new(self.name());
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
                source_format: convert_result.source_format,
            }));
        }

        let plugin_root = plugin_root.ok_or_else(|| {
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
            source_format: convert_result.source_format,
        }))
    }
}

#[cfg(test)]
#[path = "hook_deploy_test.rs"]
mod tests;
