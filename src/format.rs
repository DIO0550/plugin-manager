/// Name mapping format for `map_tool` / `map_model` / `map_event`.
///
/// Represents the assistant / model family involved in a mapping. The
/// direction of a mapping is expressed by the `(from, to)` pair used in
/// the calling code, not by this enum itself.
///
/// Used by both `parser::convert` and `hooks::event_map` for bidirectional
/// name conversions. Placed in a top-level module to avoid cross-layer
/// dependencies between parser and hooks.
///
/// This is distinct from:
/// - `TargetType`: used by `ClaudeCodeCommand::to_format()` (target-only, no ClaudeCode variant)
/// - `CommandFormat` / `AgentFormat` (in component/convert.rs): used for file-level
///   format detection (file-level, not name-mapping-level)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    ClaudeCode,
    Copilot,
    Codex,
}

/// Forward lookup: find value by key in a `(key, value)` pair table.
pub(crate) fn lookup_forward<'a>(map: &'a [(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    map.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// Reverse lookup: find key by value in a `(key, value)` pair table.
pub(crate) fn lookup_reverse<'a>(map: &'a [(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    map.iter().find(|(_, v)| *v == key).map(|(k, _)| *k)
}
