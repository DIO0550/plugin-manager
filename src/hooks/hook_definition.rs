//! Hook type domain types with validation.
//!
//! Provides `HookType` enum for type-safe hook type dispatch,
//! validated hook structs (`CommandHook`, `HttpHook`, `StubHook`),
//! and `HookDefinition` enum as a factory for parsing raw JSON.

use serde_json::Value;

use crate::error::PlmError;

/// Allowed HTTP methods for hook scripts.
pub(crate) const ALLOWED_HTTP_METHODS: &[&str] =
    &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

/// Claude Code hook type enumeration.
pub(crate) enum HookType {
    Command,
    Http,
    Prompt,
    Agent,
}

impl HookType {
    /// Parse a hook type string into a `HookType`.
    ///
    /// # Arguments
    ///
    /// * `s` - raw hook type string from hook configuration
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "command" => Some(Self::Command),
            "http" => Some(Self::Http),
            "prompt" => Some(Self::Prompt),
            "agent" => Some(Self::Agent),
            _ => None,
        }
    }
}

/// Validated command hook.
pub(crate) struct CommandHook<'a> {
    pub command: &'a str,
    pub raw: &'a Value,
}

impl<'a> CommandHook<'a> {
    /// Validate and construct a `CommandHook` from a raw hook object.
    ///
    /// # Arguments
    ///
    /// * `hook_obj` - raw hook object map holding the `command` field
    /// * `hook` - original hook JSON value retained for later access
    pub fn new(
        hook_obj: &'a serde_json::Map<String, Value>,
        hook: &'a Value,
    ) -> Result<Self, PlmError> {
        let cmd = hook_obj
            .get("command")
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                PlmError::HookConversion(
                    "command hook missing required 'command' field".to_string(),
                )
            })?;
        if cmd.contains('\n') || cmd.contains('\r') {
            return Err(PlmError::HookConversion(
                "command must not contain newline or carriage return characters".to_string(),
            ));
        }
        Ok(Self {
            command: cmd,
            raw: hook,
        })
    }
}

/// Validated http hook.
pub(crate) struct HttpHook<'a> {
    pub url: &'a str,
    pub method: String,
    /// Validated header (name, raw value) pairs.
    /// Shell escaping and Content-Type completion are done in ScriptGenerator.
    pub headers: Vec<(&'a str, &'a str)>,
    pub raw: &'a Value,
}

impl<'a> HttpHook<'a> {
    /// Validate and construct an `HttpHook` from a raw hook object.
    ///
    /// # Arguments
    ///
    /// * `hook_obj` - raw hook object map holding `url`, `method`, and `headers`
    /// * `hook` - original hook JSON value retained for later access
    pub fn new(
        hook_obj: &'a serde_json::Map<String, Value>,
        hook: &'a Value,
    ) -> Result<Self, PlmError> {
        let url = hook_obj
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| {
                PlmError::HookConversion("http hook missing required 'url' field".to_string())
            })?;

        if url.contains('\n') || url.contains('\r') {
            return Err(PlmError::HookConversion(
                "http hook 'url' value contains newline characters".to_string(),
            ));
        }

        let method_raw = hook_obj
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("POST");
        let method = method_raw.to_uppercase();
        if !ALLOWED_HTTP_METHODS.contains(&method.as_str()) {
            return Err(PlmError::HookConversion(format!(
                "http hook has unsupported method '{}'; allowed: {}",
                method_raw,
                ALLOWED_HTTP_METHODS.join(", ")
            )));
        }

        let headers = Self::validate_headers(hook_obj)?;

        Ok(Self {
            url,
            method,
            headers,
            raw: hook,
        })
    }

    /// Validate the optional `headers` map and return its entries.
    ///
    /// # Arguments
    ///
    /// * `hook_obj` - raw hook object map possibly containing a `headers` field
    fn validate_headers(
        hook_obj: &'a serde_json::Map<String, Value>,
    ) -> Result<Vec<(&'a str, &'a str)>, PlmError> {
        let Some(headers_obj) = hook_obj.get("headers").and_then(|h| h.as_object()) else {
            return Ok(vec![]);
        };
        let mut result = Vec::new();
        for (k, v) in headers_obj {
            let v_str = v.as_str().ok_or_else(|| {
                PlmError::HookConversion(format!(
                    "http hook header '{}' has non-string value; only string values are supported",
                    k
                ))
            })?;
            if !k.bytes().all(|b| {
                b.is_ascii_alphanumeric()
                    || matches!(
                        b,
                        b'!' | b'#'
                            | b'$'
                            | b'%'
                            | b'&'
                            | b'\''
                            | b'*'
                            | b'+'
                            | b'-'
                            | b'.'
                            | b'^'
                            | b'_'
                            | b'|'
                            | b'~'
                    )
            }) {
                return Err(PlmError::HookConversion(format!(
                    "http hook header name '{}' contains invalid characters",
                    k
                )));
            }
            if v_str.contains('\n') || v_str.contains('\r') {
                return Err(PlmError::HookConversion(format!(
                    "http hook header '{}' value contains newline characters",
                    k
                )));
            }
            if v_str.contains("$(") || v_str.contains('`') {
                return Err(PlmError::HookConversion(format!(
                    "http hook header '{}' contains unsupported command substitution syntax",
                    k
                )));
            }
            result.push((k.as_str(), v_str));
        }
        Ok(result)
    }
}

/// Validated prompt/agent hook (stub generation).
pub(crate) struct StubHook<'a> {
    pub hook_type: &'a str,
    pub raw: &'a Value,
}

impl<'a> StubHook<'a> {
    /// Construct a `StubHook` for prompt/agent hook types.
    ///
    /// # Arguments
    ///
    /// * `hook_type` - hook type string to record on the stub
    /// * `hook` - original hook JSON value retained for later access
    pub fn new(hook_type: &'a str, hook: &'a Value) -> Self {
        Self {
            hook_type,
            raw: hook,
        }
    }
}

/// Parsed hook definition. Holds validated data.
pub(crate) enum HookDefinition<'a> {
    Command(CommandHook<'a>),
    Http(HttpHook<'a>),
    Stub(StubHook<'a>),
}

impl<'a> HookDefinition<'a> {
    /// Parse a raw JSON hook object into a HookDefinition.
    /// Returns `Ok(None)` for unknown hook types.
    ///
    /// # Arguments
    ///
    /// * `hook_type` - hook type string identifying the variant to build
    /// * `hook_obj` - raw hook object map carrying the hook fields
    /// * `hook` - original hook JSON value retained for later access
    pub fn parse(
        hook_type: &'a str,
        hook_obj: &'a serde_json::Map<String, Value>,
        hook: &'a Value,
    ) -> Result<Option<Self>, PlmError> {
        let Some(ht) = HookType::parse(hook_type) else {
            return Ok(None);
        };
        match ht {
            HookType::Command => Ok(Some(Self::Command(CommandHook::new(hook_obj, hook)?))),
            HookType::Http => Ok(Some(Self::Http(HttpHook::new(hook_obj, hook)?))),
            HookType::Prompt => Ok(Some(Self::Stub(StubHook::new(hook_type, hook)))),
            HookType::Agent => Ok(Some(Self::Stub(StubHook::new(hook_type, hook)))),
        }
    }

    /// Hook type string for KeyMap.
    pub fn hook_type_str(&self) -> &str {
        match self {
            Self::Command(_) => "command",
            Self::Http(_) => "http",
            Self::Stub(s) => s.hook_type,
        }
    }
}
