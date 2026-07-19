use super::*;
use std::sync::{Mutex, OnceLock};

/// 環境変数を触るテストの直列化用ロック
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// テスト用の有効な自己署名 CA 証明書 (PEM)
const VALID_CA_PEM: &str = "-----BEGIN CERTIFICATE-----\n\
MIIDDTCCAfWgAwIBAgIUVDM8UEjQ2eRTQSVIoTVYtHGWXe4wDQYJKoZIhvcNAQEL\n\
BQAwFjEUMBIGA1UEAwwLcGxtLXRlc3QtY2EwHhcNMjYwNzE5MDIyNjI5WhcNMjcw\n\
NzE5MDIyNjI5WjAWMRQwEgYDVQQDDAtwbG0tdGVzdC1jYTCCASIwDQYJKoZIhvcN\n\
AQEBBQADggEPADCCAQoCggEBAJ+k7zsbR48Hm21ijEd5HOrnrax/jmC8340W0Ov8\n\
y+cfn9Y0GxhMdp2ahd3F32W51LUf4XqtK3s+epBXNBN7GmiKWacftFdlI+V1lHYo\n\
FaNwk1wAdMRy3dqCLiliVpsJD3MkD95Cv8zSZyOO/4sz2mbVk4zqyEnjVIxSmLDs\n\
2hlLpgRrg/0m3W7dXgDO8qCZNWWtVYyszUkDcJVNOaGdzSqkbHUxSPqnlmVC7KUv\n\
zc48I8vuxmGX1OJndYSttZcv+MFkoFN7y7Y11Dh0DX7asxhgjT4p18GOxaLQI3G5\n\
4tgakUe9dH9QTC90bcTN03+2TrMr68bOipje9PVokTgWxKECAwEAAaNTMFEwHQYD\n\
VR0OBBYEFNBwLddqahv6qAOYs57qy0EzMmSTMB8GA1UdIwQYMBaAFNBwLddqahv6\n\
qAOYs57qy0EzMmSTMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQELBQADggEB\n\
AD9cmoRMvnLUt9Uhi8srjYa1PwKicG3Um+zeC64N4yyYDz/3bV58kIp+ihCxNhoV\n\
1b6ahNy0OVGKPTLViSoj6RzBKJxK7I/u63YGwtW+M38WVhl4j3Xd24ZYrg/HPTwr\n\
ypdZVF2JRFoSO22r4Ah39LDGGJzKFkEcocaotWH937LbmmV3PvzvKY/f36YIeCf+\n\
1S5xIGlWOvMit9GSgXDWX3uC5wCIuv33M4BALGZGPXLNY+cWnPnwVR8XdCe/KmFu\n\
2mq9h2rRgWuqPn+7SvhCtjrmubxi1CbfxyCnnJIibatyzcAkPXAH0QKL7FTpDmFP\n\
0HAcpL120k8Oo8XmiPgeDYk=\n\
-----END CERTIFICATE-----\n";

struct EnvGuard {
    keys: Vec<&'static str>,
}

impl EnvGuard {
    fn clear(keys: &[&'static str]) -> Self {
        for key in keys {
            // Tests that mutate these env vars are serialized via env_lock().
            std::env::remove_var(key);
        }
        Self {
            keys: keys.to_vec(),
        }
    }

    fn set(&self, key: &'static str, value: &str) {
        // Tests that mutate these env vars are serialized via env_lock().
        std::env::set_var(key, value);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for key in &self.keys {
            // Tests that mutate these env vars are serialized via env_lock().
            std::env::remove_var(key);
        }
    }
}

fn write_temp_pem(contents: &str) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new().expect("temp file");
    file.write_all(contents.as_bytes()).expect("write pem");
    file
}

#[test]
fn test_http_config_default() {
    let config = HttpConfig::default();
    assert_eq!(config.user_agent, "plm-cli");
    assert!(config.timeout.is_some());
}

#[test]
fn test_auth_provider_builder() {
    let auth = AuthProvider::new()
        .with_github_token("gh_token")
        .with_gitlab_token("gl_token");

    assert_eq!(auth.github_token(), Some("gh_token"));
    assert_eq!(auth.gitlab_token(), Some("gl_token"));
    assert_eq!(auth.bitbucket_token(), None);
}

#[test]
fn build_client_without_cert_env_vars() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);

    let client = HttpConfig::default().build_client();
    // Client は Debug で構築成功を確認する（パニックしないこと）
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_with_valid_ssl_cert_file() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    let pem = write_temp_pem(VALID_CA_PEM);
    _guard.set("SSL_CERT_FILE", pem.path().to_str().unwrap());

    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_with_valid_codex_proxy_cert_only() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    let pem = write_temp_pem(VALID_CA_PEM);
    _guard.set("CODEX_PROXY_CERT", pem.path().to_str().unwrap());

    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_with_distinct_ssl_and_codex_cert_paths() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    let pem1 = write_temp_pem(VALID_CA_PEM);
    let pem2 = write_temp_pem(VALID_CA_PEM);
    _guard.set("SSL_CERT_FILE", pem1.path().to_str().unwrap());
    _guard.set("CODEX_PROXY_CERT", pem2.path().to_str().unwrap());

    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_skips_duplicate_cert_paths() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    let pem = write_temp_pem(VALID_CA_PEM);
    let path = pem.path().to_str().unwrap().to_string();
    _guard.set("SSL_CERT_FILE", &path);
    _guard.set("CODEX_PROXY_CERT", &path);

    // 同一パスを二重追加しても Client 構築が成功する
    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_with_empty_ssl_cert_file_is_skipped() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    _guard.set("SSL_CERT_FILE", "");

    // EnvVar::get は空文字を None 扱い → スキップ
    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_with_missing_ssl_cert_file_warns_and_continues() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    _guard.set(
        "SSL_CERT_FILE",
        "/nonexistent/plm-test-ca-does-not-exist.pem",
    );

    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_with_invalid_pem_warns_and_continues() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    let pem = write_temp_pem("this is not a PEM certificate\n");
    _guard.set("SSL_CERT_FILE", pem.path().to_str().unwrap());

    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_keeps_valid_ssl_when_codex_cert_is_invalid() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    let valid = write_temp_pem(VALID_CA_PEM);
    let invalid = write_temp_pem("not-a-cert");
    _guard.set("SSL_CERT_FILE", valid.path().to_str().unwrap());
    _guard.set("CODEX_PROXY_CERT", invalid.path().to_str().unwrap());

    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}

#[test]
fn build_client_with_pem_bundle() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["SSL_CERT_FILE", "CODEX_PROXY_CERT"]);
    // 同一証明書を2回並べたバンドルでも from_pem_bundle 経由で構築できる
    let bundle = format!("{VALID_CA_PEM}{VALID_CA_PEM}");
    let pem = write_temp_pem(&bundle);
    _guard.set("SSL_CERT_FILE", pem.path().to_str().unwrap());

    let client = HttpConfig::default().build_client();
    let _ = format!("{:?}", client);
}
