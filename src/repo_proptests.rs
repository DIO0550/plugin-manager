use super::*;
use proptest::prelude::*;

/// owner/repo に使える文字列（英数字、ハイフン、アンダースコア）
fn valid_name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,19}".prop_map(|s| s)
}

/// git ref に使える文字列
fn valid_ref_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9][a-zA-Z0-9._/-]{0,19}".prop_map(|s| s)
}

proptest! {
    /// 異なる形式で同じ owner/repo を指定した場合、同じ結果が得られる
    #[test]
    fn prop_all_formats_produce_same_owner_name(
        owner in valid_name_strategy(),
        repo in valid_name_strategy()
    ) {
        // 短縮形式
        let shorthand = format!("{}/{}", owner, repo);
        let result_short = from_url(&shorthand).unwrap();

        // HTTPS形式
        let https = format!("https://github.com/{}/{}", owner, repo);
        let result_https = from_url(&https).unwrap();

        // SCP形式
        let scp = format!("git@github.com:{}/{}", owner, repo);
        let result_scp = from_url(&scp).unwrap();

        // SSH形式
        let ssh = format!("ssh://git@github.com/{}/{}", owner, repo);
        let result_ssh = from_url(&ssh).unwrap();

        // すべて同じ owner/name になる
        prop_assert_eq!(result_short.owner(), result_https.owner());
        prop_assert_eq!(result_short.owner(), result_scp.owner());
        prop_assert_eq!(result_short.owner(), result_ssh.owner());

        prop_assert_eq!(result_short.name(), result_https.name());
        prop_assert_eq!(result_short.name(), result_scp.name());
        prop_assert_eq!(result_short.name(), result_ssh.name());

        // ホストはすべて GitHub
        prop_assert_eq!(result_short.host(), HostKind::GitHub);
        prop_assert_eq!(result_https.host(), HostKind::GitHub);
        prop_assert_eq!(result_scp.host(), HostKind::GitHub);
        prop_assert_eq!(result_ssh.host(), HostKind::GitHub);
    }

    /// ref 指定時に ref_or_default が正しい値を返す
    #[test]
    fn prop_ref_or_default_returns_ref_when_specified(
        owner in valid_name_strategy(),
        repo in valid_name_strategy(),
        git_ref in valid_ref_strategy()
    ) {
        let input = format!("{}/{}@{}", owner, repo, git_ref);
        let result = from_url(&input).unwrap();

        prop_assert_eq!(result.git_ref(), Some(git_ref.as_str()));
        prop_assert_eq!(result.ref_or_default(), &git_ref);
    }

    /// ref 未指定時に ref_or_default が "HEAD" を返す
    #[test]
    fn prop_ref_or_default_returns_head_when_unspecified(
        owner in valid_name_strategy(),
        repo in valid_name_strategy()
    ) {
        let input = format!("{}/{}", owner, repo);
        let result = from_url(&input).unwrap();

        prop_assert!(result.git_ref().is_none());
        prop_assert_eq!(result.ref_or_default(), "HEAD");
    }

    /// .git サフィックスは除去される
    #[test]
    fn prop_git_suffix_is_removed(
        owner in valid_name_strategy(),
        repo in valid_name_strategy()
    ) {
        let with_git = format!("{}/{}.git", owner, repo);
        let without_git = format!("{}/{}", owner, repo);

        let result_with = from_url(&with_git).unwrap();
        let result_without = from_url(&without_git).unwrap();

        prop_assert_eq!(result_with.name(), result_without.name());
    }

    /// full_name は owner/repo 形式を返す
    #[test]
    fn prop_full_name_format(
        owner in valid_name_strategy(),
        repo in valid_name_strategy()
    ) {
        let input = format!("{}/{}", owner, repo);
        let result = from_url(&input).unwrap();

        let expected_full_name = format!("{}/{}", owner, repo);
        prop_assert_eq!(result.full_name(), expected_full_name);
    }
}
