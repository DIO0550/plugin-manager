use crate::hooks::converter::EventMap;
use crate::hooks::event::entry::{to_target_event, HookEvent, HookEventEntry};

const COPILOT_EVENT_ENTRIES: &[HookEventEntry] = &[
    HookEventEntry {
        event: HookEvent::SessionStart,
        target: "sessionStart",
    },
    HookEventEntry {
        event: HookEvent::SessionEnd,
        target: "sessionEnd",
    },
    HookEventEntry {
        event: HookEvent::PreToolUse,
        target: "preToolUse",
    },
    HookEventEntry {
        event: HookEvent::PostToolUse,
        target: "postToolUse",
    },
    HookEventEntry {
        event: HookEvent::UserPromptSubmit,
        target: "userPromptSubmitted",
    },
    HookEventEntry {
        event: HookEvent::Stop,
        target: "agentStop",
    },
    HookEventEntry {
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
