use crate::hooks::converter::EventMap;
use crate::hooks::event::claude_code::{to_target_event, EventBridge, HookEvent};

const CURSOR_EVENT_ENTRIES: &[EventBridge] = &[
    EventBridge {
        event: HookEvent::SessionStart,
        target: "sessionStart",
    },
    EventBridge {
        event: HookEvent::SessionEnd,
        target: "sessionEnd",
    },
    EventBridge {
        event: HookEvent::PreToolUse,
        target: "preToolUse",
    },
    EventBridge {
        event: HookEvent::PostToolUse,
        target: "postToolUse",
    },
    EventBridge {
        event: HookEvent::PostToolUseFailure,
        target: "postToolUseFailure",
    },
    EventBridge {
        event: HookEvent::UserPromptSubmit,
        target: "beforeSubmitPrompt",
    },
    EventBridge {
        event: HookEvent::Stop,
        target: "stop",
    },
    EventBridge {
        event: HookEvent::SubagentStart,
        target: "subagentStart",
    },
    EventBridge {
        event: HookEvent::SubagentStop,
        target: "subagentStop",
    },
    EventBridge {
        event: HookEvent::PreCompact,
        target: "preCompact",
    },
];

pub(crate) struct CursorEventMap;

impl EventMap for CursorEventMap {
    fn map_event(&self, event: &str) -> Option<&'static str> {
        let hook_event = HookEvent::from_str(event.trim());
        to_target_event(CURSOR_EVENT_ENTRIES, &hook_event)
    }
}
