use crate::hooks::converter::EventMap;
use crate::hooks::event::claude_code::{to_target_event, EventBridge, HookEvent};

/// Antigravity 公式イベント（PascalCase）への対応表。
///
/// 直接対応: `PreToolUse` / `PostToolUse` / `Stop`
/// 近似: `SessionStart` / `UserPromptSubmit` → `PreInvocation`、`SessionEnd` → `Stop`
/// （出典: https://antigravity.google/docs/hooks 、docs/reference/hooks-schema-mapping.md §10）
const ANTIGRAVITY_EVENT_ENTRIES: &[EventBridge] = &[
    EventBridge {
        event: HookEvent::PreToolUse,
        target: "PreToolUse",
    },
    EventBridge {
        event: HookEvent::PostToolUse,
        target: "PostToolUse",
    },
    EventBridge {
        event: HookEvent::Stop,
        target: "Stop",
    },
    EventBridge {
        event: HookEvent::SessionStart,
        target: "PreInvocation",
    },
    EventBridge {
        event: HookEvent::UserPromptSubmit,
        target: "PreInvocation",
    },
    EventBridge {
        event: HookEvent::SessionEnd,
        target: "Stop",
    },
];

/// Events that must use flat handler arrays (not matcher groups).
pub(crate) const ANTIGRAVITY_FLAT_HANDLER_EVENTS: &[&str] =
    &["PreInvocation", "PostInvocation", "Stop"];

pub(crate) struct AntigravityEventMap;

impl EventMap for AntigravityEventMap {
    fn map_event(&self, event: &str) -> Option<&'static str> {
        let hook_event = HookEvent::from_str(event.trim());
        to_target_event(ANTIGRAVITY_EVENT_ENTRIES, &hook_event)
    }
}
