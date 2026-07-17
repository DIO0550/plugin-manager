# Issue #361: Cursor hooks.json — 実装コピー用パターン集

> **目的**: Cursor Hooks 変換・配置を、既存 Copilot/Codex 実装のアナロジーで実装するためのコードマップ。
> **作成日**: 2026-07-17
> **前提**: Cursor はまだ `TargetKind::Cursor` の Hook 未対応（`create_layers` は Err、`CursorTarget` は Hook 非サポート）。

---

## 実装方針（要約）

| 関心事 | コピー元 | Cursor での差分 |
|--------|----------|-----------------|
| EventBridge | Copilot `event/copilot.rs` | イベント名が異なる（下表） |
| Structure (`version: 1`, camelCase 検出) | Copilot `StructureConverter` | ほぼそのまま流用可 |
| KeyMap (`command` / `timeout` 維持) | Codex `CodexKeyMap` | Copilot の `bash`/`timeoutSec` 変換はしない |
| matcher 構造 | Copilot flatten + **新挙動** | フラット化しつつ **entry に matcher を残す**（現状 flatten は matcher を script に移して削除） |
| ScriptGenerator | Codex（command は path 空）+ 薄いラッパー | Copilot の exit-code/JSON 変換は不要（Claude 互換） |
| 単一 `hooks.json` 配置 | Codex `placement_location` / conflict / overwrite | パスは `.cursor/hooks.json` / `~/.cursor/hooks.json` |
| managedFiles | `install.rs` の Codex 専用分岐 | `"cursor"` キーでも同様に記録 |
| bash パス書き換え | `hook_deploy.rs` | Cursor が wrapper を使うなら `command` キーも書き換え対象に |

### Cursor EventBridge（公式 docs + `docs/concepts/targets.md`）

| Claude Code | Cursor |
|-------------|--------|
| `SessionStart` | `sessionStart` |
| `SessionEnd` | `sessionEnd` |
| `PreToolUse` | `preToolUse` |
| `PostToolUse` | `postToolUse` |
| `UserPromptSubmit` | `beforeSubmitPrompt` |
| `Stop` | `stop` |
| `SubagentStop` | `subagentStop` |
| （任意）`SubagentStart` | `subagentStart` |
| （任意）`PreCompact` | `preCompact` |

Cursor 固有イベント（`beforeShellExecution` 等）は変換対象外。

Cursor 公式例の形:

```json
{
  "version": 1,
  "hooks": {
    "preToolUse": [
      { "command": "./hooks/validate-tool.sh", "matcher": "Shell|Read|Write", "timeout": 30 }
    ],
    "beforeSubmitPrompt": [{ "command": "./hooks/audit.sh" }],
    "stop": [{ "command": "./hooks/audit.sh" }]
  }
}
```

---

## 1. Orchestrator — `src/hooks/converter/converter.rs`

### `HookConversionLayers` / `create_layers`

```247:285:src/hooks/converter/converter.rs
/// Container for the conversion layers resolved for a specific target.
pub(crate) struct HookConversionLayers {
    pub event_map: Box<dyn EventMap>,
    pub tool_map: Option<Box<dyn ToolMap>>,
    pub key_map: Box<dyn KeyMap>,
    pub structure: Box<dyn StructureConverter>,
    pub script_gen: Box<dyn ScriptGenerator>,
    pub preserve_matcher_groups: bool,
}

/// Create conversion layers for the given target.
pub(crate) fn create_layers(target: TargetKind) -> Result<HookConversionLayers, PlmError> {
    match target {
        TargetKind::Copilot => Ok(HookConversionLayers {
            event_map: Box::new(super::copilot::CopilotEventMap),
            tool_map: Some(Box::new(super::super::tool::copilot::CopilotToolMap)),
            key_map: Box::new(super::copilot::CopilotKeyMap),
            structure: Box::new(super::copilot::CopilotStructureConverter),
            script_gen: Box::new(super::copilot::CopilotScriptGenerator),
            preserve_matcher_groups: false,
        }),
        TargetKind::Codex => Ok(HookConversionLayers {
            event_map: Box::new(super::codex::CodexEventMap),
            tool_map: Some(Box::new(super::super::tool::codex::CodexToolMap)),
            key_map: Box::new(super::codex::CodexKeyMap),
            structure: Box::new(super::codex::CodexStructureConverter),
            script_gen: Box::new(super::codex::CodexScriptGenerator),
            preserve_matcher_groups: true,
        }),
        other => Err(PlmError::HookConversion(format!(
            "Hook conversion is not yet implemented for target: {}",
            other.as_str()
        ))),
    }
}
```

**Cursor 追加案**: `TargetKind::Cursor` アームを追加。

- `preserve_matcher_groups: false`（フラット化）
- ただし現行 `flatten_matchers` は matcher を JSON から消すため、**entry に matcher を残す第 3 モードが必要**（後述）

### `convert` orchestrator

```328:383:src/hooks/converter/converter.rs
pub fn convert(input: &str, target: TargetKind) -> Result<ConvertOutcome, PlmError> {
    let layers = create_layers(target)?;
    // ... JSON parse + hooks field validation ...
    match layers.structure.detect_format(&value) {
        SourceFormat::TargetFormat => { /* passthrough */ }
        SourceFormat::ClaudeCode => {
            let (mut result, mut warnings) = layers.structure.convert_top_level(&value);
            // convert_event_hooks → insert "hooks"
        }
    }
}
```

### preserve vs flatten 分岐

```406:416:src/hooks/converter/converter.rs
                let converted_hooks = if layers.preserve_matcher_groups {
                    preserve_matcher_groups(event_value, target_event, layers, out)?
                } else {
                    flatten_matchers(event_value, target_event, layers, out)?
                };
                if !converted_hooks.is_empty() {
                    output.insert(target_event.to_string(), Value::Array(converted_hooks));
                }
```

### `preserve_matcher_groups`（Codex 用・ネスト維持）

```433:503:src/hooks/converter/converter.rs
fn preserve_matcher_groups(
    groups: &Value,
    event: &str,
    layers: &HookConversionLayers,
    out: &mut ConvertOutput<'_>,
) -> Result<Vec<Value>, PlmError> {
    // groups[] → { matcher?, hooks: [converted...] } を維持
    // convert_hook_definition で各 hook を変換
}
```

### `flatten_matchers`（Copilot 用・matcher を script へ移動）

```513:570:src/hooks/converter/converter.rs
fn flatten_matchers(...) -> Result<Vec<Value>, PlmError> {
    // ...
    if let Some(m) = matcher {
        out.warnings.push(ConversionWarning::RemovedField {
            field: "matcher".to_string(),
            reason: format!("Matcher '{}' moved to script for event '{}'", m, event),
        });
    }
    for hook in hooks {
        if let Some(converted) = convert_hook_definition(hook, matcher, event, layers, out)? {
            result.push(converted);  // ← matcher キーは entry に付かない
        }
    }
}
```

**Cursor 向けギャップ**: フラット配列にしつつ、各 entry に `"matcher": "..."` を残す必要がある。案:

1. `HookConversionLayers` に `preserve_matcher_on_entries: bool` を追加し、`flatten_matchers` 内で `converted` に matcher を insert、かつ RemovedField 警告を出さない
2. または専用 `flatten_matchers_preserving_field`

### `insert_script_fields`（現状は常に `bash`）

```572:585:src/hooks/converter/converter.rs
fn insert_script_fields(mapped: &mut Value, script_path: String) -> Result<(), PlmError> {
    let obj = mapped.as_object_mut().ok_or_else(|| {
        PlmError::HookConversion("map_keys must return a JSON object".to_string())
    })?;
    obj.insert("bash".to_string(), Value::from(script_path));
    obj.insert("type".to_string(), Value::from("command"));
    Ok(())
}
```

```655:685:src/hooks/converter/converter.rs
    if script_info.path.is_empty() {
        // Codex path: keep mapped keys (command/timeout) inline
        ...
        return Ok(Some(mapped));
    }
    let script_path = format!("./{}", script_info.path);
    out.scripts.push(script_info);
    insert_script_fields(&mut mapped, script_path)?;  // always "bash"
```

**Cursor**:

- command を inline 維持するなら Codex 同様 `path: ""`（推奨・最小）
- wrapper を生成するなら `command` キーへパスを書くよう `insert_script_fields` をパラメータ化（`"bash"` vs `"command"`）

### トレイト定義（Layer 1–4）

```122:245:src/hooks/converter/converter.rs
pub(crate) trait EventMap { fn map_event(&self, event: &str) -> Option<&'static str>; }
pub(crate) trait ToolMap { fn map_tool(&self, tool: &str) -> String; }
pub(crate) trait KeyMap {
    fn map_keys(&self, hook: &Value, hook_type: &str) -> (Value, Vec<ConversionWarning>);
}
pub(crate) trait StructureConverter {
    fn detect_format(&self, value: &Value) -> SourceFormat;
    fn handle_target_format(&self, value: Value) -> Result<(Value, Vec<ConversionWarning>), PlmError>;
    fn convert_top_level(&self, value: &Value) -> (Value, Vec<ConversionWarning>);
}
pub(crate) trait ScriptGenerator {
    fn generate_command_script(...) -> ScriptInfo;
    fn generate_http_script(...) -> Result<ScriptInfo, PlmError>;
    fn generate_stub_script(...) -> ScriptInfo;
    fn preserves_stub_inline(&self) -> bool { false }
}
```

---

## 2. Copilot layers — `src/hooks/converter/copilot.rs`

### KeyMap（`timeout`→`timeoutSec`, `command` 除去）— Cursor では逆側

```55:107:src/hooks/converter/copilot.rs
impl KeyMap for CopilotKeyMap {
    fn map_keys(&self, hook: &Value, hook_type: &str) -> (Value, Vec<ConversionWarning>) {
        // type-specific keys skipped (command/type for command hooks)
        // "timeout" => "timeoutSec"
        // "statusMessage" => "comment"
        // "async"/"once" => RemovedField warning
    }
}
```

### StructureConverter（version + PascalCase 検出）— Cursor で流用

```109:184:src/hooks/converter/copilot.rs
impl StructureConverter for CopilotStructureConverter {
    fn detect_format(&self, value: &Value) -> SourceFormat {
        if value.get("version").is_some() { return SourceFormat::TargetFormat; }
        // PascalCase event keys => ClaudeCode, else TargetFormat
    }
    fn handle_target_format(...) { /* validate version==1 or inject + MissingVersion */ }
    fn convert_top_level(...) {
        obj.insert("version".to_string(), Value::from(1));
        // remove disableAllHooks
    }
}
```

### ScriptGenerator — Cursor では重い EXIT_CODE_HANDLER は不要

```17:53:src/hooks/converter/copilot.rs
const EXIT_CODE_HANDLER: &str = r#"# --- execute original command and capture result ---
...
# Copilot: exit 2 → permissionDecision deny JSON; always exit 0
"#;
```

```188:214:src/hooks/converter/copilot.rs
    fn generate_command_script(...) -> ScriptInfo {
        // builds: env bridge + matcher filter + ORIGINAL_CMD + EXIT_CODE_HANDLER
        // path: wrappers/cmd-{event}-{index}.sh
    }
```

Cursor は Claude 互換 exit code のため、command は Codex 同様 inline、または最小ラッパー（`CLAUDE_PLUGIN_ROOT` 埋め込み程度）でよい。

---

## 3. Codex layers — `src/hooks/converter/codex.rs`（identity-ish）

### KeyMap — Cursor のキー方針に近い

```19:89:src/hooks/converter/codex.rs
impl KeyMap for CodexKeyMap {
    fn map_keys(&self, hook: &Value, hook_type: &str) -> (Value, Vec<ConversionWarning>) {
        // keep command/timeout/type
        // drop async/once/bash
        // timeoutSec → timeout (if timeout absent)
        // comment → statusMessage
    }
}
```

### StructureConverter — Codex は常に ClaudeCode 扱い、version 削除

```94:129:src/hooks/converter/codex.rs
impl StructureConverter for CodexStructureConverter {
    fn detect_format(&self, _value: &Value) -> SourceFormat {
        SourceFormat::ClaudeCode
    }
    fn convert_top_level(&self, value: &Value) -> (Value, Vec<ConversionWarning>) {
        // remove version, disableAllHooks
    }
}
```

Cursor は Copilot 寄り（`version: 1` 必須）なので Structure は Copilot 側をコピー。

### ScriptGenerator — path 空 = inline 維持

```134:183:src/hooks/converter/codex.rs
impl ScriptGenerator for CodexScriptGenerator {
    fn generate_command_script(...) -> ScriptInfo {
        ScriptInfo { path: String::new(), content: String::new(), ... }
    }
    fn preserves_stub_inline(&self) -> bool { true }
}
```

統合テスト例（ネスト維持 + command 維持）:

```236:265:src/hooks/converter/codex_test.rs
fn test_codex_convert_keeps_command_hook_inline() {
    // convert(..., TargetKind::Codex)
    // scripts empty; PreToolUse[0].matcher == "Bash"; hooks[0].command == "echo hi"
}
```

---

## 4. Event / Tool / module wiring

### `src/hooks/event.rs`

```1:10:src/hooks/event.rs
pub(crate) mod claude_code;
pub(crate) mod codex;
pub(crate) mod copilot;
```

→ `pub(crate) mod cursor;` を追加。

### EventBridge 基盤 — `src/hooks/event/claude_code.rs`

```39:54:src/hooks/event/claude_code.rs
pub(crate) struct EventBridge {
    pub event: HookEvent,
    pub target: &'static str,
}
pub(crate) fn to_target_event(table: &[EventBridge], event: &HookEvent) -> Option<&'static str> {
    table.iter().find(|e| e.event == *event).map(|e| e.target)
}
```

### Copilot EventMap（コピー元テーブル形）

```3:42:src/hooks/event/copilot.rs
const COPILOT_EVENT_ENTRIES: &[EventBridge] = &[
    EventBridge { event: HookEvent::SessionStart, target: "sessionStart" },
    EventBridge { event: HookEvent::SessionEnd, target: "sessionEnd" },
    EventBridge { event: HookEvent::PreToolUse, target: "preToolUse" },
    EventBridge { event: HookEvent::PostToolUse, target: "postToolUse" },
    EventBridge { event: HookEvent::UserPromptSubmit, target: "userPromptSubmitted" },
    EventBridge { event: HookEvent::Stop, target: "agentStop" },
    EventBridge { event: HookEvent::SubagentStop, target: "subagentStop" },
];

impl EventMap for CopilotEventMap {
    fn map_event(&self, event: &str) -> Option<&'static str> {
        let hook_event = HookEvent::from_str(event.trim());
        to_target_event(COPILOT_EVENT_ENTRIES, &hook_event)
    }
}
```

**Cursor テーブル案**（同ファイル構成で `event/cursor.rs`）:

```rust
const CURSOR_EVENT_ENTRIES: &[EventBridge] = &[
    EventBridge { event: HookEvent::SessionStart, target: "sessionStart" },
    EventBridge { event: HookEvent::SessionEnd, target: "sessionEnd" },
    EventBridge { event: HookEvent::PreToolUse, target: "preToolUse" },
    EventBridge { event: HookEvent::PostToolUse, target: "postToolUse" },
    EventBridge { event: HookEvent::UserPromptSubmit, target: "beforeSubmitPrompt" },
    EventBridge { event: HookEvent::Stop, target: "stop" },
    EventBridge { event: HookEvent::SubagentStop, target: "subagentStop" },
];
```

### Codex EventMap（PascalCase 恒等）— 比較用

```7:23:src/hooks/event/codex.rs
impl EventMap for CodexEventMap {
    fn map_event(&self, event: &str) -> Option<&'static str> {
        match event {
            "SessionStart" => Some("SessionStart"),
            // ... PermissionRequest, PreCompact, PostCompact, SubagentStart/Stop
            _ => None,
        }
    }
}
```

### ToolMap — `src/hooks/tool/copilot.rs`

```3:65:src/hooks/tool/copilot.rs
pub(crate) const COPILOT_TOOL_ENTRIES: &[ToolBridge] = &[
    ToolBridge { claude_code_tools: &[HookTool::Bash], target_name: "bash", ... },
    // Read→view, Write→create, Edit→edit, ...
];
impl ToolMap for CopilotToolMap {
    fn map_tool(&self, tool: &str) -> String { /* forward map or passthrough */ }
}
```

Cursor 公式 matcher 例は `"Shell|Read|Write"`。ツール名マッピング要否は別途検証。初期は Codex 同様 passthrough / identity で十分な可能性が高い（matcher を entry に残すなら script 内フィルタ不要）。

### converter モジュール配線 — `src/hooks/converter.rs`

```8:20:src/hooks/converter.rs
mod codex;
#[allow(clippy::module_inception)]
mod converter;
mod copilot;

pub use self::converter::*;

#[cfg(test)]
mod codex_test;
#[cfg(test)]
mod converter_test;
#[cfg(test)]
mod copilot_test;
```

→ `mod cursor;` + `cursor_test` を追加。`create_layers` から `super::cursor::*` を参照。

### hooks ルート — `src/hooks.rs`

```6:9:src/hooks.rs
pub mod converter;
pub(crate) mod event;
mod model;
pub(crate) mod tool;
```

---

## 5. Cursor target 現状 — フル状態

### `src/target/env/cursor.rs`（Hook 未対応）

- コメント L2–3: `Hooks は後続 Issue（#361）で対応する。`
- `supported_components`: Skill / Agent / Command / Instruction のみ（Hook なし）
- `can_place`: Hook なし
- `placement_location`: Hook 分岐なし → `test_cursor_placement_location_hook_returns_none`
- `list_placed`: Hook なし

Codex からコピーすべき Hook 配置:

```184:184:src/target/env/codex.rs
            ComponentKind::Hook => PlacementLocation::file(base.join("hooks.json")),
```

```132:132:src/target/env/codex.rs
            ComponentKind::Hook if !c.is_dir && c.name == "hooks.json" => Some("hooks".to_string()),
```

Cursor 版パス:

| Scope | Path |
|-------|------|
| Project | `{project}/.cursor/hooks.json` |
| Personal | `~/.cursor/hooks.json` |

`cursor_test.rs` は現状 481 行。Hook 用に Codex 相当の placement / list_placed / conflict / overwrite テストを追加する。

---

## 6. Codex conflict / overwrite / list_placed

### conflict（複数 Hook コンポーネント拒否）

```58:72:src/target/env/codex.rs
    pub fn hook_component_conflict_error(components: &[Component]) -> Option<String> {
        let hook_count = components
            .iter()
            .filter(|component| component.kind == ComponentKind::Hook)
            .count();
        (hook_count > 1).then(|| {
            format!(
                "Codex target supports a single hooks.json per scope; {} Hook components would overwrite each other. ...",
                hook_count
            )
        })
    }
```

### overwrite（managedFiles 所有権）

```94:112:src/target/env/codex.rs
    pub fn hook_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !target_path.exists() { return None; }
        let already_owned = crate::plugin::meta::load_meta(plugin_root)
            .map(|meta| meta.manages_file("codex", target_path))
            .unwrap_or(false);
        if already_owned { return None; }
        Some(format!("{} already exists and is not managed by this plugin. ...", ...))
    }
```

Cursor では `manages_file("cursor", target_path)` に変更した同等メソッドを `CursorTarget` に置くか、ターゲット名を引数化した共通ヘルパーにする。

### list_placed for hooks

```208:221:src/target/env/codex.rs
        let dir_path = match kind {
            ComponentKind::Skill => base.join("skills"),
            ComponentKind::Agent => base.join("agents"),
            ComponentKind::Hook => base.clone(),
            _ => return Ok(vec![]),
        };
        // filter: hooks.json → name "hooks"
```

---

## 7. conflict / overwrite 呼び出し箇所

### `src/install.rs` — place_plugin

```202:251:src/install.rs
        let codex_hook_conflict = if target.kind() == TargetKind::Codex {
            CodexTarget::hook_component_conflict_error(&request.scanned.components)
        } else { None };

        // ... for each Hook component: push failure if conflict ...

        if component.kind == ComponentKind::Hook && target.kind() == TargetKind::Codex {
            if let Some(error) =
                CodexTarget::hook_overwrite_error(&target_path, request.scanned.plugin_root())
            { /* PlaceFailure */ }
        }
```

### ConversionConfig::Hook 発動条件

```262:269:src/install.rs
                ComponentKind::Hook
                    if matches!(target.kind(), TargetKind::Codex | TargetKind::Copilot) =>
                {
                    ConversionConfig::Hook {
                        target_kind: target.kind(),
                        plugin_root: Some(request.scanned.plugin_root().to_path_buf()),
                    }
                }
```

→ `TargetKind::Cursor` を追加。

### `src/commands/deploy/import.rs`

```243:258:src/commands/deploy/import.rs
    if component.kind == ComponentKind::Hook && target.kind() == TargetKind::Codex {
        if let Some(error) = CodexTarget::hook_overwrite_error(&target_path, ctx.plugin_root) {
            return Err(error);
        }
    }
    // ConversionConfig::Hook if Codex | Copilot
```

```380:386:src/commands/deploy/import.rs
        let codex_hook_conflict = if target.kind() == TargetKind::Codex {
            CodexTarget::hook_component_conflict_error(components)
        } else { None };
```

```299:300:src/commands/deploy/import.rs
            if target_kind == TargetKind::Codex && deployment.kind() == ComponentKind::Hook {
                crate::install::record_codex_hook_ownership(ctx.plugin_root, deployment.path());
```

---

## 8. Hook deploy — `src/component/deployment/hook_deploy.rs`（フル要点）

### bash パス書き換え（namespace）

```44:73:src/component/deployment/hook_deploy.rs
    fn rewrite_script_paths_in_json(
        json: &mut serde_json::Value,
        original_paths: &HashSet<String>,
        safe_name: &str,
    ) {
        // フラット hooks.<event>[] の "bash" のみ書き換え
        // ./wrappers/foo.sh → ./wrappers/{safe_name}/foo.sh
        // ※ Codex matcher-group ネストや Cursor の "command" キーは未対応
    }
```

### deploy_hook_converted

```79:179:src/component/deployment/hook_deploy.rs
    pub(super) fn deploy_hook_converted(...) -> Result<DeploymentOutput> {
        let mut convert_result = converter::convert(&input, target_kind)?;
        // TargetFormat + empty scripts/warnings → copy_file
        // else: namespace scripts, write JSON, write wrappers with @@PLUGIN_ROOT@@ replace
    }
```

`count_hooks_in_json` はフラット（Copilot）とネスト（Codex）両方を数える（L19–35）。Cursor フラットは Copilot 側の数え方で OK。

**Cursor で wrapper を使う場合**: `rewrite_script_paths_in_json` を `command`（と必要ならネスト）に拡張。inline command のみならスクリプト生成 0 で書き換え不要。

---

## 9. managedFiles 記録

### place 後 — `update_meta_after_place`

```412:433:src/install.rs
    for success in &result.successes {
        if success.component_kind == ComponentKind::Hook
            && success.target_kind == TargetKind::Codex
            && plugin_meta.add_managed_file(&success.target, &success.target_path)
        {
            updated = true;
        }
        // statusByTarget enabled if target had no failures
    }
```

### import 専用 — `record_codex_hook_ownership`

```468:483:src/install.rs
pub fn record_codex_hook_ownership(plugin_path: &Path, hook_path: &Path) {
    // manages_file("codex", ...) / set_status("codex") / add_managed_file("codex", ...)
}
```

### PluginMeta API

```194:215:src/plugin/meta/meta.rs
    pub fn add_managed_file(&mut self, target: &str, path: &Path) -> bool { ... }
    pub fn manages_file(&self, target: &str, path: &Path) -> bool { ... }
```

Cursor では `"cursor"` を target キーにして同じパターンを呼ぶ（関数を汎用化するか `record_cursor_hook_ownership` を追加）。

---

## 10. テストパターン

### Layer 単体 — `src/hooks/converter/copilot_test.rs`

```15:67:src/hooks/converter/copilot_test.rs
fn test_event_map_supported_events() { /* map_event assertions */ }
fn test_key_map_command_hook() {
    // timeout→timeoutSec, command/type removed, async/once warnings
}
fn test_detect_format_with_version / _pascal_case / _camel_case
```

### 統合 — `src/hooks/converter/converter_test.rs`

```244:272:src/hooks/converter/converter_test.rs
fn test_flatten_single_matcher() {
    // Copilot: bash ends with .sh; matcher moved to script; RemovedField warning
}
```

```177:199:src/hooks/converter/converter_test.rs
fn test_convert_supported_events() {
    // SessionStart→sessionStart, UserPromptSubmit→userPromptSubmitted, Stop→agentStop
}
```

### Codex 統合 — `src/hooks/converter/codex_test.rs`

- `test_codex_convert_keeps_command_hook_inline`（上掲）
- KeyMap / Structure / ScriptGenerator の単体テストが層ごとに揃っている

**Cursor テストで断言すべきこと**:

1. Event: `UserPromptSubmit` → `beforeSubmitPrompt`, `Stop` → `stop`
2. Top-level: `"version": 1`
3. Keys: `command` / `timeout` が残り、`bash` / `timeoutSec` が無い
4. Structure: フラット配列 + entry 上の `matcher`
5. Scripts: command は空（または最小）、http/prompt は方針どおり
6. `create_layers(Cursor)` が Ok
7. `CursorTarget` placement → `.cursor/hooks.json`
8. conflict / overwrite / managedFiles("cursor")

---

## 実装チェックリスト（ファイル単位）

| # | ファイル | 作業 |
|---|----------|------|
| 1 | `src/hooks/event/cursor.rs` (+ test) | EventBridge テーブル |
| 2 | `src/hooks/event.rs` | `mod cursor` |
| 3 | `src/hooks/tool/cursor.rs`（任意） | identity ToolMap |
| 4 | `src/hooks/converter/cursor.rs` (+ test) | KeyMap≈Codex, Structure≈Copilot, Script≈Codex/minimal |
| 5 | `src/hooks/converter.rs` | `mod cursor` |
| 6 | `src/hooks/converter/converter.rs` | `create_layers(Cursor)` + matcher-on-entry flatten + 必要なら `insert_script_fields` 汎用化 |
| 7 | `src/target/env/cursor.rs` (+ test) | Hook support, placement, conflict/overwrite, list_placed |
| 8 | `src/install.rs` | conflict/overwrite/ConversionConfig/managedFiles に Cursor |
| 9 | `src/commands/deploy/import.rs` | 同上 + ownership 記録 |
| 10 | `src/component/deployment/hook_deploy.rs` | `command` パス書き換え（wrapper 使う場合のみ） |

---

## 参照

- Cursor Hooks 公式: <https://cursor.com/docs/agent/hooks>
- 配置パス: `docs/concepts/targets.md`（Hooks User/Project）
- Copilot 変換仕様（歴史的）: `docs/hooks-conversion/`
- スキーマ対応表: `docs/reference/hooks-schema-mapping.md`
- managedFiles: `docs/architecture/cache.md`
