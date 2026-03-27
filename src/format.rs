/// Name mapping direction for map_tool / map_model / map_event.
///
/// Used by both `parser::convert` and `hooks::event_map` for bidirectional
/// name conversions. Placed in a top-level module to avoid cross-layer
/// dependencies between parser and hooks.
///
/// This is distinct from:
/// - `TargetType`: used by `ClaudeCodeCommand::to_format()` (target-only, no ClaudeCode variant)
/// - `CommandFormat` / `AgentFormat` (in component/convert.rs): used for file-level format detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    ClaudeCode,
    Copilot,
    Codex,
}
