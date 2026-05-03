//! アンインストール後のプラグインディレクトリ整理
//!
//! コンポーネント削除後に空になったディレクトリ
//! （`<base>/<kind_subdir>/<marketplace>/<plugin>` とその親）を再帰的に
//! 掃除する責務のみを持つ。マニフェスト読み込みなどのロード系処理は
//! `plugin::loader` を参照。

use crate::fs::{FileSystem, RealFs};
use crate::target::{PluginOrigin, TargetKind};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// プラグインディレクトリをクリーンアップ
///
/// コンポーネント削除後に空になったプラグインディレクトリを削除する。
///
/// # Arguments
///
/// * `kind` - target environment kind determining directory layout
/// * `origin` - plugin origin providing marketplace and plugin segments
/// * `project_root` - project root under which project-scope deploy directories live
///
/// `HOME` 環境変数が未設定の場合、personal scope のクリーンアップは
/// スキップされる（project scope のみ実行）。これにより `HOME` 欠落時に
/// literal `~` がカレント配下に解決され誤削除されるリスクを避ける。
pub(crate) fn cleanup_plugin_directories(
    kind: TargetKind,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    let fs = RealFs;
    let home = resolve_home_dir();
    cleanup_plugin_directories_impl(&fs, kind, home.as_deref(), origin, project_root);
}

/// `HOME` 環境変数を正規化する。
///
/// 以下を一箇所で扱い、`cleanup_plugin_directories` と
/// `cleanup_legacy_hierarchy` の両者で挙動が分岐しないようにする:
/// - 未設定 / 空文字 / 空白のみは `None`
/// - 非 UTF-8 値も `OsString` のまま受理（`var_os`）
/// - 相対パス（例: `.` や `tmp`）は `None`。CWD 配下で
///   `quarantine rename + remove_dir_all` が走るリスクを避けるため
fn resolve_home_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;

    let home = match home.to_str() {
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return None;
            }
            PathBuf::from(trimmed)
        }
        None => {
            let path = PathBuf::from(home);
            if path.as_os_str().is_empty() {
                return None;
            }
            path
        }
    };

    home.is_absolute().then_some(home)
}

/// 内部実装 — `home` と `fs` を注入可能にし、テストから直接呼ぶ。
///
/// `home` が `None` の場合、personal scope のクリーンアップはスキップされる。
pub(crate) fn cleanup_plugin_directories_impl(
    fs: &dyn FileSystem,
    kind: TargetKind,
    home: Option<&Path>,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    for (base, kind_subdir) in cleanup_specs(kind, home, project_root) {
        cleanup_one(fs, &base, kind_subdir, origin);
    }
}

/// TargetKind ごとに (base_dir, kind_subdir) のリストを返す。
///
/// - `home` が `Some` の場合: personal scope + project scope 両方のエントリを列挙
/// - `home` が `None` の場合: project scope のエントリのみ列挙（personal cleanup スキップ）
///
/// kind_subdir は `"agents"` / `"skills"` / `"prompts"` / `"hooks"` などの
/// コンポーネント種別配下ディレクトリ名。
fn cleanup_specs(
    kind: TargetKind,
    home: Option<&Path>,
    project_root: &Path,
) -> Vec<(PathBuf, &'static str)> {
    let mut specs: Vec<(PathBuf, &'static str)> = Vec::new();

    match kind {
        TargetKind::Codex => {
            if let Some(h) = home {
                specs.push((h.join(".codex"), "agents"));
                specs.push((h.join(".codex"), "skills"));
            }
            specs.push((project_root.join(".codex"), "agents"));
            specs.push((project_root.join(".codex"), "skills"));
        }
        TargetKind::Copilot => {
            if let Some(h) = home {
                // Personal scope: CopilotTarget::can_place は Agent / Hook のみサポート
                specs.push((h.join(".copilot"), "agents"));
                specs.push((h.join(".copilot"), "hooks"));
            }
            // Project scope: 全コンポーネント種別を受け付ける
            specs.push((project_root.join(".github"), "agents"));
            specs.push((project_root.join(".github"), "prompts"));
            specs.push((project_root.join(".github"), "skills"));
            specs.push((project_root.join(".github"), "hooks"));
        }
        TargetKind::Antigravity => {
            if let Some(h) = home {
                specs.push((h.join(".gemini").join("antigravity"), "skills"));
            }
            specs.push((project_root.join(".agent"), "skills"));
        }
        TargetKind::GeminiCli => {
            if let Some(h) = home {
                specs.push((h.join(".gemini"), "skills"));
            }
            specs.push((project_root.join(".gemini"), "skills"));
        }
    }

    specs
}

fn cleanup_one(fs: &dyn FileSystem, base: &Path, kind_subdir: &str, origin: &PluginOrigin) {
    // 防御的検証: 不正な marketplace / plugin セグメントが渡された場合、
    // base の外で remove_dir_all が走ってしまうのを防ぐため cleanup をスキップする。
    if !is_safe_path_segment(&origin.marketplace) || !is_safe_path_segment(&origin.plugin) {
        return;
    }

    let plugin_dir = base
        .join(kind_subdir)
        .join(&origin.marketplace)
        .join(&origin.plugin);
    remove_if_empty(fs, &plugin_dir);

    let marketplace_dir = base.join(kind_subdir).join(&origin.marketplace);
    remove_if_empty(fs, &marketplace_dir);

    let kind_root = base.join(kind_subdir);
    remove_if_empty(fs, &kind_root);
}

/// パスの 1 セグメントとして安全かを判定する。
///
/// `..` / パスセパレータ / 先頭ドット / 絶対パスを拒否し、`base` 外への
/// 書き込みや削除を防ぐ。`plugin/cache.rs::validate_source_path` と同じ方針。
fn is_safe_path_segment(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }
    if segment.contains("..") {
        return false;
    }
    if segment.contains('/') || segment.contains('\\') {
        return false;
    }
    if segment.starts_with('.') {
        return false;
    }
    if Path::new(segment).is_absolute() {
        return false;
    }
    true
}

/// 旧 3 階層構造 `<base>/<kind_subdir>/<marketplace>/<plugin>` を一掃する
///
/// install/uninstall/disable のタイミングで呼ばれ、新しいフラット 2 階層
/// 構造へ移行済みの環境から旧階層の残骸を除去する。
///
/// 安全性は「**誤削除防止のベストエフォート**」を方針とし、完全な TOCTOU
/// 耐性は目指さない。`is_safe_path_segment` / canonicalize / symlink 除外
/// （legacy + kind_root + mp_dir）/ depth=2 / file_name 一致 / 空親昇格削除
/// に加え、削除前に同一親配下の `<plugin>.plm-quarantine-<random>` 名へ
/// `rename` してから `remove_dir_all` する quarantine 方式で race を縮小する。
///
/// # I/O 抽象化方針
///
/// 同モジュールの `cleanup_plugin_directories` は `&dyn FileSystem` を受け取る
/// が、本関数（および下請けの `sweep_legacy_one` / `dir_is_empty` /
/// `path_is_symlink`）は `std::fs` を直接呼び出している。これは以下の API が
/// 12 ガードの安全性確保に必須だが現状 `FileSystem` trait 未提供のため:
/// - `std::fs::symlink_metadata` (シンボリックリンク非追従の type 判定)
/// - `Path::canonicalize` (kind_root 配下に厳密に含まれるかの判定)
/// - `std::fs::remove_dir` (空親ディレクトリ単独の昇格削除)
///
/// `FileSystem` 抽象に上記 3 点を追加して再注入する余地はあるが、本関数は
/// 旧階層からの一回限りのマイグレーション用途であり、MockFs 化の費用対効果が
/// 限定的なため当面 `std::fs` 直接呼び出しを維持する。
pub(crate) fn cleanup_legacy_hierarchy(
    kind: TargetKind,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    // HOME 正規化（空白/空文字・相対パス・非 UTF-8 の扱い）は
    // `cleanup_plugin_directories` と共通 helper `resolve_home_dir` に集約。
    let home = resolve_home_dir();
    cleanup_legacy_hierarchy_impl(kind, home.as_deref(), origin, project_root);
}

/// 内部実装 — `home` を注入可能にしテストから直接呼ぶ。
pub(crate) fn cleanup_legacy_hierarchy_impl(
    kind: TargetKind,
    home: Option<&Path>,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    for (base, kind_subdir) in cleanup_specs(kind, home, project_root) {
        sweep_legacy_one(&base, kind_subdir, origin);
    }
}

/// 単一 (base, kind_subdir) について、12 ガードを通過した場合のみ
/// `<base>/<kind_subdir>/<mp>/<plg>` を quarantine rename → remove_dir_all で除去し、
/// 空になった親 (mp_dir / kind_root) を昇格削除する。
fn sweep_legacy_one(base: &Path, kind_subdir: &str, origin: &PluginOrigin) {
    // ガード 1: marketplace / plugin が安全な path-segment か
    if !is_safe_path_segment(&origin.marketplace) || !is_safe_path_segment(&origin.plugin) {
        return;
    }

    let kind_root = base.join(kind_subdir);
    let mp_dir = kind_root.join(&origin.marketplace);
    let legacy = mp_dir.join(&origin.plugin);

    // ガード 2: legacy が存在
    if !legacy.exists() {
        return;
    }

    // ガード 3 / 11 / 12: legacy / kind_root / mp_dir のいずれかが symlink なら no-op
    if path_is_symlink(&legacy) || path_is_symlink(&kind_root) || path_is_symlink(&mp_dir) {
        return;
    }

    // ガード 10: legacy が is_dir であること
    if !legacy.is_dir() {
        return;
    }

    // ガード 4-5: canonicalize 後 legacy が kind_root 配下に厳密に含まれる
    let canonical_legacy = match legacy.canonicalize() {
        Ok(p) => p,
        Err(_) => return,
    };
    let canonical_kind_root = match kind_root.canonicalize() {
        Ok(p) => p,
        Err(_) => return,
    };
    if !canonical_legacy.starts_with(&canonical_kind_root) {
        return;
    }

    // ガード 6: legacy.parent() == Some(&mp_dir)
    if legacy.parent() != Some(mp_dir.as_path()) {
        return;
    }
    // ガード 7: mp_dir.parent() == Some(&kind_root)
    if mp_dir.parent() != Some(kind_root.as_path()) {
        return;
    }

    // ガード 8-9: file_name 一致
    if mp_dir.file_name() != Some(OsStr::new(&origin.marketplace)) {
        return;
    }
    if legacy.file_name() != Some(OsStr::new(&origin.plugin)) {
        return;
    }

    // 全ガード通過 — quarantine rename → remove_dir_all
    let suffix = quarantine_suffix();
    let quarantine = mp_dir.join(format!("{}.plm-quarantine-{}", origin.plugin, suffix));
    if std::fs::rename(&legacy, &quarantine).is_err() {
        // rename 失敗時は no-op で abort（race 中に削除されたなど誤削除を避ける）。
        return;
    }
    let _ = std::fs::remove_dir_all(&quarantine);

    // 空親昇格削除
    if dir_is_empty(&mp_dir) {
        let _ = std::fs::remove_dir(&mp_dir);
    }
    if dir_is_empty(&kind_root) {
        let _ = std::fs::remove_dir(&kind_root);
    }
}

fn path_is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn dir_is_empty(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    std::fs::read_dir(path)
        .map(|mut iter| iter.next().is_none())
        .unwrap_or(false)
}

/// quarantine 名に使うサフィックスを生成する。
///
/// 暗号学的乱数は不要（rename の race を縮小する目的のみ）。同一親配下での
/// 一意性確保が目的のため、`SystemTime` のナノ秒下位ビット + プロセス ID +
/// プロセス内単調カウンタを混ぜる。
///
/// **注意**: 以前はポインタアドレスも混ぜていたが、quarantine ディレクトリ名
/// としてアドレスがディスク上に露出するのを避けるため除外した。
fn quarantine_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let pid = std::process::id() as u64;
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mixed = nanos ^ pid.rotate_left(32) ^ counter.rotate_left(48);
    format!("{:016x}", mixed)
}

fn remove_if_empty(fs: &dyn FileSystem, path: &Path) {
    if !fs.is_dir(path) {
        return;
    }
    let Ok(entries) = fs.read_dir(path) else {
        return;
    };
    if !entries.is_empty() {
        return;
    }
    let _ = fs.remove_dir_all(path);
}

#[cfg(test)]
#[path = "cleanup_test.rs"]
mod tests;
