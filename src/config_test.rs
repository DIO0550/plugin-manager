use super::*;

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
