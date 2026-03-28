use crate::hooks::converter::EventMap;
use crate::hooks::event::claude_code::{to_target_event, EventBridge, HookEvent};

const COPILOT_EVENT_ENTRIES: &[EventBridge] = &[
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
        event: HookEvent::UserPromptSubmit,
        target: "userPromptSubmitted",
    },
    EventBridge {
        event: HookEvent::Stop,
        target: "agentStop",
    },
    EventBridge {
        event: HookEvent::SubagentStop,
        target: "subagentStop",
    },
];

pub(crate) struct CopilotEventMap;

impl EventMap for CopilotEventMap {
    fn map_event(&self, event: &str) -> Option<&'static str> {
        let hook_event = HookEvent::from_str(event.trim());
        to_target_event(COPILOT_EVENT_ENTRIES, &hook_event)
    }
}
