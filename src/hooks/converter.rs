//! Hook configuration converter with polymorphic layer architecture.
//!
//! Provides 4 trait-based conversion layers (5 traits) per-target:
//! - Layer 1a `EventMap`: event name conversion
//! - Layer 1b `ToolMap`: tool name conversion (optional)
//! - Layer 2 `KeyMap`: key name/field conversion
//! - Layer 3 `StructureConverter`: top-level structure and format detection
//! - Layer 4 `ScriptGenerator`: script file generation
//!
//! The `convert()` orchestrator combines these layers to perform the full conversion.

use std::fmt;

use serde_json::Value;

use crate::error::PlmError;
use crate::hooks::hook_definition::{CommandHook, HttpHook, StubHook};
use crate::target::TargetKind;

// ============================================================================
// Shared constants
// ============================================================================

/// スクリプトの配置ディレクトリ prefix（値は後方互換のため "wrappers" のまま）
pub const SCRIPTS_DIR: &str = "wrappers";

/// スクリプトパスに名前空間を挿入する
///
/// `SCRIPTS_DIR` prefix を検出し、間に namespace を挿入する。
/// prefix がない場合はそのまま返す。
///
/// # Examples
/// ```ignore
/// namespace_script_path("wrappers/foo.sh", "my-hook") // => "wrappers/my-hook/foo.sh"
/// namespace_script_path("other/foo.sh", "my-hook")    // => "other/foo.sh"
/// ```
pub(crate) fn namespace_script_path(path: &str, namespace: &str) -> String {
    match path
        .strip_prefix(SCRIPTS_DIR)
        .and_then(|rest| rest.strip_prefix('/'))
    {
        Some(rest) => format!("{}/{}/{}", SCRIPTS_DIR, namespace, rest),
        None => path.to_string(),
    }
}

// ============================================================================
// Shared types
// ============================================================================

/// Conversion result containing the transformed JSON, warnings, and script info.
#[derive(Debug, Clone)]
pub struct ConvertResult {
    pub json: Value,
    pub warnings: Vec<ConversionWarning>,
    pub scripts: Vec<ScriptInfo>,
    pub source_format: SourceFormat,
}

/// Warnings emitted during conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionWarning {
    UnsupportedEvent { event: String },
    UnsupportedHookType { hook_type: String, event: String },
    RemovedField { field: String, reason: String },
    PromptAgentHookStub { event: String, hook_type: String },
    MissingVersion,
}

impl fmt::Display for ConversionWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionWarning::UnsupportedEvent { event } => {
                write!(
                    f,
                    "Event '{}' is not supported in Copilot CLI and was excluded",
                    event
                )
            }
            ConversionWarning::UnsupportedHookType { hook_type, event } => {
                write!(
                    f,
                    "Hook type '{}' for event '{}' is not supported and was excluded",
                    hook_type, event
                )
            }
            ConversionWarning::RemovedField { field, reason } => {
                write!(f, "Field '{}' was removed: {}", field, reason)
            }
            ConversionWarning::PromptAgentHookStub { event, hook_type } => {
                write!(
                    f,
                    "'{}' hook for event '{}' is a Claude Code-specific feature. A stub script was generated; please manually rewrite.",
                    hook_type, event
                )
            }
            ConversionWarning::MissingVersion => {
                write!(
                    f,
                    "Copilot CLI config is missing required 'version' field; it should be set to 1"
                )
            }
        }
    }
}

/// Information about a generated script that needs to be created.
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    pub path: String,
    pub content: String,
    pub original_config: Value,
    pub matcher: Option<String>,
}

/// Detected source format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    /// Input is in Claude Code format and needs conversion.
    ClaudeCode,
    /// Input is already in the target format (passthrough).
    TargetFormat,
}

// ============================================================================
// Conversion Layer Traits (4 layers, 5 traits)
// ============================================================================

/// Layer 1a: Event name mapping.
pub(crate) trait EventMap {
    /// Convert a Claude Code event name to the target event name.
    /// Returns `None` for unsupported/unknown events.
    fn map_event(&self, event: &str) -> Option<&'static str>;
}

/// Layer 1b: Tool name mapping (optional).
pub(crate) trait ToolMap {
    /// Convert a Claude Code tool name to the target tool name.
    /// Unknown tools are passed through.
    fn map_tool(&self, tool: &str) -> String;
}

/// Layer 2: Key name/field conversion within a hook definition.
pub(crate) trait KeyMap {
    /// Transform hook definition keys to the target format.
    /// Returns the mapped JSON object and any warnings for removed fields.
    fn map_keys(&self, hook: &Value, hook_type: &str) -> (Value, Vec<ConversionWarning>);
}

/// Layer 3: Top-level structure conversion and format detection.
pub(crate) trait StructureConverter {
    /// Detect whether the input is in the target format or Claude Code format.
    fn detect_format(&self, value: &Value) -> SourceFormat;

    /// Handle input that is already in the target format (passthrough).
    /// Returns an error if the target format is invalid (e.g., unsupported version).
    fn handle_target_format(
        &self,
        value: Value,
    ) -> Result<(Value, Vec<ConversionWarning>), PlmError>;

    /// Convert top-level structure from Claude Code to target format.
    /// Returns the full JSON with top-level fields transformed (hooks preserved as-is).
    fn convert_top_level(&self, value: &Value) -> (Value, Vec<ConversionWarning>);
}

/// Layer 4: Script generation for different hook types.
pub(crate) trait ScriptGenerator {
    /// Generate a script for a command-type hook.
    fn generate_command_script(
        &self,
        hook: &CommandHook<'_>,
        event: &str,
        matcher: Option<&str>,
        index: usize,
    ) -> ScriptInfo;

    /// Generate a script for an http-type hook.
    fn generate_http_script(
        &self,
        hook: &HttpHook<'_>,
        event: &str,
        matcher: Option<&str>,
        index: usize,
    ) -> Result<ScriptInfo, PlmError>;

    /// Generate a stub script for prompt/agent-type hooks.
    fn generate_stub_script(
        &self,
        hook: &StubHook<'_>,
        event: &str,
        matcher: Option<&str>,
        index: usize,
    ) -> ScriptInfo;
}

// ============================================================================
// Layers container + factory
// ============================================================================

/// Container for the conversion layers resolved for a specific target.
pub(crate) struct HookConversionLayers {
    pub event_map: Box<dyn EventMap>,
    pub tool_map: Option<Box<dyn ToolMap>>,
    pub key_map: Box<dyn KeyMap>,
    pub structure: Box<dyn StructureConverter>,
    pub script_gen: Box<dyn ScriptGenerator>,
}

/// Create conversion layers for the given target.
pub(crate) fn create_layers(target: TargetKind) -> Result<HookConversionLayers, PlmError> {
    match target {
        TargetKind::Copilot => Ok(HookConversionLayers {
            event_map: Box::new(super::copilot::CopilotEventMap),
            tool_map: Some(Box::new(super::tool::copilot::CopilotToolMap)),
            key_map: Box::new(super::copilot::CopilotKeyMap),
            structure: Box::new(super::copilot::CopilotStructureConverter),
            script_gen: Box::new(super::copilot::CopilotScriptGenerator),
        }),
        other => Err(PlmError::HookConversion(format!(
            "Hook conversion is not yet implemented for target: {}",
            other.as_str()
        ))),
    }
}

// ============================================================================
// Shared helpers
// ============================================================================

/// Escape a string for safe embedding in single-quoted shell strings.
/// Replaces `'` with `'\''` (end quote, escaped quote, start quote).
pub(crate) fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Generate a bash matcher filter snippet for scripts.
/// Uses anchored regex `^(pattern)$` for full-match per BL-003.
/// Sanitizes newlines to prevent shell injection via comment/grep breakout.
/// Returns empty string if matcher is None.
pub(crate) fn generate_matcher_filter(matcher: Option<&str>) -> String {
    match matcher {
        Some(pattern) => {
            let safe = pattern.replace('\n', "\\n").replace('\r', "\\r");
            let anchored = format!("^({})$", safe);
            format!(
                "\n# --- matcher filter: '{}' ---\nif command -v jq >/dev/null 2>&1; then\n  TOOL_NAME=$(printf '%s' \"$CLAUDE_INPUT\" | jq -r '.tool_name // empty' 2>/dev/null || true)\n  if [ -n \"$TOOL_NAME\" ] && ! echo \"$TOOL_NAME\" | grep -qE -e '{}'; then\n    exit 0\n  fi\nfi\n",
                shell_escape(&safe),
                shell_escape(&anchored)
            )
        }
        None => String::new(),
    }
}

// ============================================================================
// Orchestrator: convert()
// ============================================================================

/// Convert Claude Code hooks JSON to the target format.
///
/// If the input is already in the target format, it is returned as-is.
pub fn convert(input: &str, target: TargetKind) -> Result<ConvertResult, PlmError> {
    let layers = create_layers(target)?;

    let value: Value = serde_json::from_str(input)
        .map_err(|e| PlmError::HookConversion(format!("Invalid JSON: {}", e)))?;

    // Validate hooks field exists and is an object (borrow is dropped before match)
    match value.get("hooks") {
        Some(h) if h.is_object() => {}
        Some(_) => {
            return Err(PlmError::HookConversion(
                "'hooks' field must be an object".to_string(),
            ));
        }
        None => {
            return Err(PlmError::HookConversion(
                "Missing 'hooks' field".to_string(),
            ));
        }
    }

    match layers.structure.detect_format(&value) {
        SourceFormat::TargetFormat => {
            let (json, warnings) = layers.structure.handle_target_format(value)?;
            Ok(ConvertResult {
                json,
                warnings,
                scripts: vec![],
                source_format: SourceFormat::TargetFormat,
            })
        }
        SourceFormat::ClaudeCode => {
            let (mut result, mut warnings) = layers.structure.convert_top_level(&value);
            let mut scripts = Vec::new();
            let mut out = ConvertOutput {
                warnings: &mut warnings,
                scripts: &mut scripts,
            };

            // Re-access hooks from the original value (validation done above)
            let hooks_value = value.get("hooks").unwrap();
            let new_hooks = convert_event_hooks(hooks_value, &layers, &mut out)?;
            result
                .as_object_mut()
                .unwrap()
                .insert("hooks".to_string(), new_hooks);

            Ok(ConvertResult {
                json: result,
                warnings,
                scripts,
                source_format: SourceFormat::ClaudeCode,
            })
        }
    }
}

// ============================================================================
// Shared orchestration logic
// ============================================================================

/// Accumulator for warnings and scripts during conversion.
struct ConvertOutput<'a> {
    warnings: &'a mut Vec<ConversionWarning>,
    scripts: &'a mut Vec<ScriptInfo>,
}

/// Convert event hooks using the 4 layers.
fn convert_event_hooks(
    hooks: &Value,
    layers: &HookConversionLayers,
    out: &mut ConvertOutput<'_>,
) -> Result<Value, PlmError> {
    let hooks_obj = hooks.as_object().unwrap();
    let mut output = serde_json::Map::new();

    for (event_name, event_value) in hooks_obj {
        match layers.event_map.map_event(event_name) {
            Some(target_event) => {
                let converted_hooks = flatten_matchers(event_value, target_event, layers, out)?;
                if !converted_hooks.is_empty() {
                    output.insert(target_event.to_string(), Value::Array(converted_hooks));
                }
            }
            None => {
                out.warnings.push(ConversionWarning::UnsupportedEvent {
                    event: event_name.clone(),
                });
            }
        }
    }

    Ok(Value::Object(output))
}

/// Flatten matcher groups into a flat array of hook definitions.
fn flatten_matchers(
    groups: &Value,
    event: &str,
    layers: &HookConversionLayers,
    out: &mut ConvertOutput<'_>,
) -> Result<Vec<Value>, PlmError> {
    let mut result = Vec::new();

    let groups_arr = match groups.as_array() {
        Some(arr) => arr,
        None => {
            out.warnings.push(ConversionWarning::RemovedField {
                field: event.to_string(),
                reason: format!(
                    "Event '{}' value is not an array; expected matcher group structure",
                    event
                ),
            });
            return Ok(result);
        }
    };

    for group in groups_arr {
        let matcher = group
            .get("matcher")
            .and_then(|m| m.as_str())
            .filter(|s| !s.is_empty());

        let hooks = match group.get("hooks").and_then(|h| h.as_array()) {
            Some(arr) => arr,
            None => {
                out.warnings.push(ConversionWarning::RemovedField {
                    field: "hooks".to_string(),
                    reason: format!(
                        "Matcher group in event '{}' is missing 'hooks' array; skipped",
                        event
                    ),
                });
                continue;
            }
        };

        if let Some(m) = matcher {
            out.warnings.push(ConversionWarning::RemovedField {
                field: "matcher".to_string(),
                reason: format!("Matcher '{}' moved to script for event '{}'", m, event),
            });
        }

        for hook in hooks {
            if let Some(converted) = convert_hook_definition(hook, matcher, event, layers, out)? {
                result.push(converted);
            }
        }
    }

    Ok(result)
}

/// Insert `bash` script path and `type: "command"` into a mapped hook value.
fn insert_script_fields(mapped: &mut Value, script_path: String) -> Result<(), PlmError> {
    let obj = mapped.as_object_mut().ok_or_else(|| {
        PlmError::HookConversion("map_keys must return a JSON object".to_string())
    })?;
    obj.insert("bash".to_string(), Value::from(script_path));
    obj.insert("type".to_string(), Value::from("command"));
    Ok(())
}

/// Convert an individual hook definition using layers.
fn convert_hook_definition(
    hook: &Value,
    matcher: Option<&str>,
    event: &str,
    layers: &HookConversionLayers,
    out: &mut ConvertOutput<'_>,
) -> Result<Option<Value>, PlmError> {
    use crate::hooks::hook_definition::HookDefinition;

    let hook_obj = hook
        .as_object()
        .ok_or_else(|| PlmError::HookConversion("Hook definition must be an object".to_string()))?;
    let hook_type = hook_obj
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("command");

    let action = match HookDefinition::parse(hook_type, hook_obj, hook)? {
        Some(a) => a,
        None => {
            out.warnings.push(ConversionWarning::UnsupportedHookType {
                hook_type: hook_type.to_string(),
                event: event.to_string(),
            });
            return Ok(None);
        }
    };

    let (mut mapped, key_warnings) = layers.key_map.map_keys(hook, action.hook_type_str());
    out.warnings.extend(key_warnings);

    let index = out.scripts.len();
    let (mut script_info, extra_warnings) = match &action {
        HookDefinition::Command(h) => {
            let si = layers
                .script_gen
                .generate_command_script(h, event, matcher, index);
            (si, vec![])
        }
        HookDefinition::Http(h) => {
            let si = layers
                .script_gen
                .generate_http_script(h, event, matcher, index)?;
            (si, vec![])
        }
        HookDefinition::Stub(h) => {
            let si = layers
                .script_gen
                .generate_stub_script(h, event, matcher, index);
            (
                si,
                vec![ConversionWarning::PromptAgentHookStub {
                    event: event.to_string(),
                    hook_type: h.hook_type.to_string(),
                }],
            )
        }
    };

    if matches!(action, HookDefinition::Command(_)) && script_info.original_config.is_null() {
        script_info.original_config = hook.clone();
    }

    let script_path = format!("./{}", script_info.path);
    out.scripts.push(script_info);
    out.warnings.extend(extra_warnings);
    insert_script_fields(&mut mapped, script_path)?;

    Ok(Some(mapped))
}
