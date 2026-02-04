//! Installed タブのアクション実行
//!
//! Disable/Uninstall などのプラグイン操作を実行する。
//! Application層のユースケースに委譲する。

use super::model::UpdateStatusDisplay;
use crate::application;
use crate::plugin::{update_plugin, UpdateStatus};
use std::env;
use std::path::Path;

/// アクション実行結果
#[derive(Debug)]
pub enum ActionResult {
    /// 成功
    Success,
    /// エラー
    Error(String),
}

impl From<application::OperationResult> for ActionResult {
    fn from(result: application::OperationResult) -> Self {
        if result.success {
            ActionResult::Success
        } else {
            ActionResult::Error(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
pub fn disable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::disable_plugin(plugin_name, marketplace, &project_root, None).into()
}

/// プラグインを Uninstall（デプロイ先 + キャッシュ削除）
pub fn uninstall_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::uninstall_plugin(plugin_name, marketplace, &project_root).into()
}

/// プラグインを Enable（キャッシュからデプロイ先に配置）
pub fn enable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::enable_plugin(plugin_name, marketplace, &project_root, None).into()
}

/// stdout/stderr を一時的に抑制する RAII ガード
///
/// TUI の代替スクリーンが update_plugin の println!/eprintln! で乱れるのを防ぐ。
/// Drop で確実に復元される。
#[cfg(unix)]
struct OutputSuppressGuard {
    saved_stdout: i32,
    saved_stderr: i32,
}

#[cfg(unix)]
impl OutputSuppressGuard {
    fn new() -> Option<Self> {
        use std::ffi::CString;
        use std::os::unix::io::AsRawFd;

        let dev_null_path = CString::new("/dev/null").ok()?;
        let dev_null_fd = unsafe { libc::open(dev_null_path.as_ptr(), libc::O_WRONLY) };
        if dev_null_fd < 0 {
            return None;
        }

        let stdout_fd = std::io::stdout().as_raw_fd();
        let stderr_fd = std::io::stderr().as_raw_fd();

        let saved_stdout = unsafe { libc::dup(stdout_fd) };
        let saved_stderr = unsafe { libc::dup(stderr_fd) };

        if saved_stdout < 0 || saved_stderr < 0 {
            unsafe {
                libc::close(dev_null_fd);
                if saved_stdout >= 0 {
                    libc::close(saved_stdout);
                }
                if saved_stderr >= 0 {
                    libc::close(saved_stderr);
                }
            }
            return None;
        }

        unsafe {
            libc::dup2(dev_null_fd, stdout_fd);
            libc::dup2(dev_null_fd, stderr_fd);
            libc::close(dev_null_fd);
        }

        Some(Self {
            saved_stdout,
            saved_stderr,
        })
    }
}

#[cfg(unix)]
impl Drop for OutputSuppressGuard {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        let stdout_fd = std::io::stdout().as_raw_fd();
        let stderr_fd = std::io::stderr().as_raw_fd();

        unsafe {
            libc::dup2(self.saved_stdout, stdout_fd);
            libc::dup2(self.saved_stderr, stderr_fd);
            libc::close(self.saved_stdout);
            libc::close(self.saved_stderr);
        }
    }
}

/// Windows 用のスタブ実装
#[cfg(not(unix))]
struct OutputSuppressGuard;

#[cfg(not(unix))]
impl OutputSuppressGuard {
    fn new() -> Option<Self> {
        Some(Self)
    }
}

/// バッチ更新を実行
///
/// 各プラグインを順次 `update_plugin()` で更新し、結果を返す。
/// stdout/stderr をリダイレクトして TUI 画面の乱れを防ぐ。
pub fn batch_update_plugins(
    plugins: &[(String, Option<String>)],
) -> Vec<(String, UpdateStatusDisplay)> {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());

    // stdout/stderr をリダイレクト
    // Note: ガード作成失敗時は抑制なしで続行する（TUI 画面が乱れる可能性あり）。
    // TUI 代替スクリーン上では eprintln! が表示されないため、ログ出力は行わない。
    let _guard = OutputSuppressGuard::new();

    plugins
        .iter()
        .map(|(name, marketplace)| {
            let status = run_update_plugin(name, marketplace, &project_root);
            (name.clone(), status)
        })
        .collect()
}

/// 単一プラグインの更新を同期的に実行
fn run_update_plugin(
    plugin_name: &str,
    _marketplace: &Option<String>,
    project_root: &Path,
) -> UpdateStatusDisplay {
    // TODO: marketplace を update_plugin に渡す（現在は未対応）
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(update_plugin(plugin_name, project_root, None))
    });

    match result.status {
        UpdateStatus::Updated { .. } => UpdateStatusDisplay::Updated,
        UpdateStatus::AlreadyUpToDate => UpdateStatusDisplay::AlreadyUpToDate,
        UpdateStatus::Skipped { reason } => UpdateStatusDisplay::Skipped(reason),
        UpdateStatus::Failed => {
            UpdateStatusDisplay::Failed(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}
