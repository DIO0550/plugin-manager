use crate::hooks::converter::EventMap;

/// Codex skeleton: returns `None` for all events.
pub(crate) struct CodexEventMap;

impl EventMap for CodexEventMap {
    fn map_event(&self, _event: &str) -> Option<&'static str> {
        None
    }
}
