/// 環境変数ユーティリティ
pub struct EnvVar;

impl EnvVar {
    /// 環境変数を取得（空文字列はNoneとして扱う）
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
#[path = "env_test.rs"]
mod tests;
