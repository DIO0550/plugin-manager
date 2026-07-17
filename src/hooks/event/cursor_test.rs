//! Tests for CursorEventMap (also covered via converter/cursor_test.rs).

use super::cursor::CursorEventMap;
use crate::hooks::converter::EventMap;

#[test]
fn test_cursor_event_map_user_prompt_and_stop() {
    let map = CursorEventMap;
    assert_eq!(
        map.map_event("UserPromptSubmit"),
        Some("beforeSubmitPrompt")
    );
    assert_eq!(map.map_event("Stop"), Some("stop"));
    assert_eq!(map.map_event("SubagentStart"), Some("subagentStart"));
    assert_eq!(map.map_event("PreCompact"), Some("preCompact"));
    assert_eq!(
        map.map_event("PostToolUseFailure"),
        Some("postToolUseFailure")
    );
}
