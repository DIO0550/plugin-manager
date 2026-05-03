//! Hook スクリプトのパス解決を行う純粋関数モジュール。
//!
//! 上位仕様 BL-006 (`docs/hooks-conversion/install-integration-spec.md`) に基づき、
//! - 絶対パス       → そのまま使用
//! - 相対パス       → `cache_root` 配下に解決
//! - コマンド名のみ → そのまま使用 (PATH から探索される想定)
//!
//! の 3 種別を判定する。
//!
//! # 入力契約
//! - IN-1: 入力は **実行ファイル部分のみ (program トークン)** とする。引数を含めてはならない。
//!   `./script.sh --flag` や `npm test` のようなコマンドライン全体、`~/bin/hook` や
//!   `$HOME/bin/hook` のようなシェル展開トークンを含む文字列は契約違反である。
//!   resolver を呼ぶ場合は program 部のみを渡すこと。シェル構文を含む command 文字列を
//!   どう扱うか (変換対象外として警告するか、shell-aware に分離するか等) は本関数の責務外で、
//!   後続 Issue (#173 / #189) で決定する。
//! - IN-1a: 判定セパレータは `'/'` のみ。`'\\'` は POSIX では通常のファイル名構成文字として
//!   扱う。`'/'` を含まない裸のファイル名 (`hook.sh`, `dir\\sub.sh` 等) はコマンド名として
//!   素通しされ PATH 検索扱いになる。プラグイン同梱スクリプトを `cache_root` 配下から
//!   実行したい場合は `./hook.sh` のように相対パス記法に正規化すること。
//! - IN-2: `@@PLUGIN_ROOT@@/...` のようなプレースホルダ未解決文字列を渡してはならない。
//!   `'/'` を含む相対パスとして誤変換される。プレースホルダ解決は別経路で行う。
//! - IN-3: `cache_root` は UTF-8 で表現可能であること。
//!   `Path::display()` は非 UTF-8 バイト列を `U+FFFD` で置換する lossy 変換を行う。
//!
//! # 意図的に行わないこと
//! - `../` を含む相対パスのフォルダトラバーサル防御
//!   (`cache_root.join(input)` でそのまま返す。防御は呼び出し側の責務)
//! - `./scripts/x.sh` の `./` 正規化
//!   (例: `/cache/./scripts/x.sh` をそのまま返す)
//! - 空白を含む入力のトークン分割やシェルエスケープ
//!   (シェルエスケープは呼び出し側責務。`hooks` 配下からは crate 公開の
//!   `crate::hooks::converter::shell_escape` を利用する。
//!   `component::deployment::bash::escape_for_bash_double_quote` は
//!   `pub(super)` で `component::deployment` 配下からのみ参照可能)

use std::path::Path;

/// スクリプト指定文字列をプラグインキャッシュ基準で解決する。
///
/// 判定順序 (上から評価):
/// 1. 空文字列 → 空文字列を返す
/// 2. `Path::is_absolute()` が true → そのまま返す
/// 3. `'/'` を含む → `cache_root.join(input).display().to_string()`
/// 4. それ以外 → そのまま返す (コマンド名扱い)
///
/// POSIX 前提のため `'\\'` は通常のファイル名構成文字として扱い、
/// セパレータ判定には用いない (`dir\\sub.sh` はコマンド名分岐に流れる)。
///
/// # Arguments
/// * `input` - 呼び出し側で分離済みの program トークンのみ。
///   コマンドライン全体・シェル展開・プレースホルダ未解決文字列を含めないこと
///   (詳細はモジュール冒頭の「入力契約」を参照、IN-1 / IN-2)。
/// * `cache_root` - プラグインキャッシュルート
///   (例: `~/.plm/cache/<marketplace>/<plugin>`、UTF-8 表現可能であること、IN-3)。
pub(crate) fn resolve_script_path(input: &str, cache_root: &Path) -> String {
    if input.is_empty() {
        return String::new();
    }
    if Path::new(input).is_absolute() {
        return input.to_string();
    }
    if input.contains('/') {
        return cache_root.join(input).display().to_string();
    }
    input.to_string()
}
