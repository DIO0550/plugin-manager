/// 環境変数ユーティリティ
pub struct EnvVar;

impl EnvVar {
    /// 環境変数を取得（空文字列はNoneとして扱う）
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_existing_var() {
        std::env::set_var("TEST_ENV_VAR", "test_value");
        assert_eq!(EnvVar::get("TEST_ENV_VAR"), Some("test_value".to_string()));
        std::env::remove_var("TEST_ENV_VAR");
    }

    #[test]
    fn test_get_empty_var() {
        std::env::set_var("TEST_EMPTY_VAR", "");
        assert_eq!(EnvVar::get("TEST_EMPTY_VAR"), None);
        std::env::remove_var("TEST_EMPTY_VAR");
    }

    #[test]
    fn test_get_nonexistent_var() {
        assert_eq!(EnvVar::get("NONEXISTENT_VAR_12345"), None);
    }
}
