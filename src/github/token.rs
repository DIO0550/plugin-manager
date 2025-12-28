use crate::env::EnvVar;
use std::process::Command;

/// GitHub認証トークン
#[derive(Debug, Clone)]
pub struct Token(String);

impl Token {
    /// 新しいTokenを作成
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// トークン文字列への参照を取得
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// GitHubトークンを取得
    /// 優先順位: 1. GITHUB_TOKEN環境変数, 2. gh CLI認証
    pub fn from_env() -> Option<Self> {
        Self::from_env_var().or_else(Self::from_gh_cli)
    }

    /// 環境変数から取得（CI/CD用）
    fn from_env_var() -> Option<Self> {
        EnvVar::get("GITHUB_TOKEN").map(Self::new)
    }

    /// gh CLIから取得（ローカル開発用）
    fn from_gh_cli() -> Option<Self> {
        Command::new("gh")
            .args(["auth", "token"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
            .map(Self::new)
    }

    /// Bearer認証ヘッダー値を生成
    pub fn to_bearer(&self) -> String {
        format!("Bearer {}", self.0)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_new() {
        let token = Token::new("test_token");
        assert_eq!(token.as_str(), "test_token");
    }

    #[test]
    fn test_token_to_bearer() {
        let token = Token::new("abc123");
        assert_eq!(token.to_bearer(), "Bearer abc123");
    }

    #[test]
    fn test_token_display_hides_value() {
        let token = Token::new("secret");
        assert_eq!(format!("{}", token), "Token(***)");
    }
}
