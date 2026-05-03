//! Tests for `resolve_script_path` (BL-006).
//!
//! 前提: POSIX 環境 (Linux CI)。判定セパレータは `'/'` のみ。
//! Windows ドライブレター付きパス (`C:\\foo\\bar`) は `Path::is_absolute()` が false を返し、
//! `'/'` も含まないため、本テストでは「コマンド名分岐」に流れて素通しされる挙動を固定する。
//! Windows 対応は将来 Issue。

use std::path::Path;

use super::script_path::resolve_script_path;

// --- 絶対パス分岐 -------------------------------------------------

#[test]
fn absolute_posix_path_returned_as_is() {
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("/usr/local/bin/hook", cache),
        "/usr/local/bin/hook"
    );
}

#[test]
fn absolute_path_with_spaces_returned_as_is() {
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("/abs/with spaces/file.sh", cache),
        "/abs/with spaces/file.sh"
    );
}

// --- 相対パス分岐 (セパレータ '/' を含む) ------------------------

#[test]
fn relative_with_dot_slash_is_joined_without_normalization() {
    // `./` は正規化しない (ヒアリング 6.2)
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("./scripts/validate.sh", cache),
        "/cache/./scripts/validate.sh"
    );
}

#[test]
fn relative_without_dot_slash_is_joined() {
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("scripts/validate.sh", cache),
        "/cache/scripts/validate.sh"
    );
}

#[test]
fn relative_path_with_spaces_is_joined_without_escape() {
    // 相対パス + ファイル名に空白 (シェルエスケープは呼び出し側責務)
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("scripts/with space.sh", cache),
        "/cache/scripts/with space.sh"
    );
}

#[test]
fn relative_with_parent_traversal_is_joined_without_guard() {
    // `../` ガードは行わない (ヒアリング 4.6 / 探索 3-3)
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path("../foo", cache), "/cache/../foo");
}

// --- コマンド名分岐 (セパレータなし) -----------------------------

#[test]
fn bare_command_name_returned_as_is() {
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path("npm", cache), "npm");
}

#[test]
fn bare_filename_with_extension_treated_as_command() {
    // 契約 IN-1a: セパレータを含まない裸のファイル名はコマンド名として扱う。
    // プラグイン同梱スクリプトを cache_root 配下から実行したい場合は、
    // 呼び出し側で `./hook.sh` のように相対パス記法に正規化すること。
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path("hook.sh", cache), "hook.sh");
}

// --- 観察的固定テスト (契約違反入力・POSIX 専用挙動の凍結) -----------
// 以下は契約違反入力および POSIX 専用挙動に対する不定義動作を観察的に凍結したリグレッション
// 検知用テスト。仕様化された挙動ではないため、後続 Issue (#173 / #189) で入力検証を導入する場合は
// 方針変更に応じて更新・削除する。呼び出し側がこの挙動に依存することは禁止。

#[test]
fn contract_violation_empty_input_returns_empty() {
    // 契約違反: 空文字列はスクリプト指定子として意味を持たない。
    // 防御的に空文字列を返す現挙動を凍結。
    // 本来は呼び出し側 (`hook_definition.rs` 等) のバリデーション層で排除すべき。
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path("", cache), "");
}

#[test]
fn contract_violation_command_with_arguments_passed_through() {
    // IN-1 違反: 空白で区切られた引数付きコマンドラインを直接渡されると、
    // セパレータ未含有のためコマンド名分岐に流れて素通しされる。
    // 呼び出し側 (#173 / #189) で program と args を分離する責務がある。
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path("npm test", cache), "npm test");
}

#[test]
fn contract_violation_relative_with_arguments_misjoined() {
    // IN-1 違反: '/' を含む引数付き入力 `./script.sh --flag` は相対パス分岐に流れて、
    // `--flag` まで含めた文字列が cache_root.join される現挙動を凍結。
    // 呼び出し側 (#173 / #189) で program と args を分離する責務がある。
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("./script.sh --flag", cache),
        "/cache/./script.sh --flag"
    );
}

#[test]
fn contract_violation_tilde_shell_expansion_misresolved() {
    // `~/bin/hook` はシェル展開を含むため契約 IN-1 違反。
    // 本関数は `~` を解釈せず、'/' 含有として相対分岐で誤解決する現挙動を観察的に固定。
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("~/bin/hook", cache),
        "/cache/~/bin/hook"
    );
}

#[test]
fn posix_only_backslash_path_treated_as_command_name() {
    // POSIX 前提のため `'\\'` はセパレータ判定に用いない。
    // `dir\\sub\\x.sh` はコマンド名分岐に流れて素通しされる現挙動を凍結。
    // Windows 対応は将来 Issue。
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("dir\\sub\\x.sh", cache),
        "dir\\sub\\x.sh"
    );
}

#[test]
fn posix_only_windows_drive_letter_treated_as_command_name() {
    // POSIX 上では `Path::new("C:\\foo\\bar").is_absolute()` は false。
    // セパレータ判定も `'/'` のみのため、コマンド名分岐に流れる現挙動を凍結。
    // Windows 対応は将来 Issue。
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path("C:\\foo\\bar", cache), "C:\\foo\\bar");
}

#[test]
fn contract_violation_unresolved_plugin_root_placeholder_misjoined() {
    // `@@PLUGIN_ROOT@@/scripts/x.sh` はプレースホルダ未解決のため契約 IN-2 違反。
    // 本関数はプレースホルダを認識せず、'/' 含有として相対分岐で誤解決する現挙動を凍結。
    // プレースホルダ解決は別経路 (`hook_deploy.rs:143` の `replace("@@PLUGIN_ROOT@@", ...)`) で行われる。
    let cache = Path::new("/cache");
    assert_eq!(
        resolve_script_path("@@PLUGIN_ROOT@@/scripts/x.sh", cache),
        "/cache/@@PLUGIN_ROOT@@/scripts/x.sh"
    );
}

#[test]
fn contract_violation_lone_dot_treated_as_command_name() {
    // IN-1a: 単独の `.` は POSIX ではカレントディレクトリだが、本関数では '/' を含まないため
    // コマンド名分岐に流れて素通しされる現挙動を凍結。スクリプト指定子としては意味を持たないため
    // 呼び出し側で排除すべき。
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path(".", cache), ".");
}

#[test]
fn contract_violation_lone_dotdot_treated_as_command_name() {
    // IN-1a: 単独の `..` は POSIX では親ディレクトリだが、本関数では '/' を含まないため
    // コマンド名分岐に流れて素通しされる現挙動を凍結。スクリプト指定子としては意味を持たないため
    // 呼び出し側で排除すべき。
    let cache = Path::new("/cache");
    assert_eq!(resolve_script_path("..", cache), "..");
}
