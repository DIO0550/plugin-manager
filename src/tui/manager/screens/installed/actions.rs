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
///
/// dup2 によるプロセスグローバルな fd リダイレクションを行うため、
/// グローバル Mutex で排他制御し、リダイレクト前に flush する。
#[cfg(unix)]
struct OutputSuppressGuard {
    saved_stdout: i32,
    saved_stderr: i32,
    _lock: std::sync::MutexGuard<'static, ()>,
}

/// OutputSuppressGuard の排他制御用 Mutex
#[cfg(unix)]
static OUTPUT_SUPPRESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(unix)]
impl OutputSuppressGuard {
    fn new() -> Option<Self> {
        use std::ffi::CString;
        use std::io::Write;
        use std::os::unix::io::AsRawFd;

        // グローバル Mutex を取得して排他制御
        // poison されていてもガードを取得して処理を継続する
        let lock = match OUTPUT_SUPPRESS_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // リダイレクト前に pending な出力を flush
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();

        let dev_null_path = CString::new("/dev/null").ok()?;
        let dev_null_fd = unsafe { libc::open(dev_null_path.as_ptr(), libc::O_WRONLY) };
        if dev_null_fd < 0 {
            return None;
        }

        let stdout_fd = std::io::stdout().as_raw_fd();
        let stderr_fd = std::io::stderr().as_raw_fd();

        let saved_stdout = unsafe { libc::dup(stdout_fd) };
        let saved_stderr = unsafe { libc::dup(stderr_fd) };

        // Ensure all allocated file descriptors are properly cleaned up on error.
        // Both partial success cases are handled:
        // - saved_stdout >= 0 && saved_stderr < 0: close saved_stdout
        // - saved_stdout < 0 && saved_stderr >= 0: close saved_stderr
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
            let r_stdout = libc::dup2(dev_null_fd, stdout_fd);
            let r_stderr = libc::dup2(dev_null_fd, stderr_fd);

            if r_stdout < 0 || r_stderr < 0 {
                // dup2 が成功した fd は /dev/null にリダイレクト済みなので、
                // 元の fd に復元する必要がある。失敗した fd は変更されていないため復元不要。
                if r_stdout >= 0 {
                    libc::dup2(saved_stdout, stdout_fd);
                }
                if r_stderr >= 0 {
                    libc::dup2(saved_stderr, stderr_fd);
                }
                // dev_null_fd が stdout/stderr と同じ場合（元々閉じられていた場合）は
                // close すると dup2 済みの fd を壊すため、異なる場合のみ close する
                if dev_null_fd != stdout_fd && dev_null_fd != stderr_fd {
                    libc::close(dev_null_fd);
                }
                libc::close(saved_stdout);
                libc::close(saved_stderr);
                return None;
            }

            if dev_null_fd != stdout_fd && dev_null_fd != stderr_fd {
                libc::close(dev_null_fd);
            }
        }

        Some(Self {
            saved_stdout,
            saved_stderr,
            _lock: lock,
        })
    }
}

#[cfg(unix)]
impl Drop for OutputSuppressGuard {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        let stdout_fd = std::io::stdout().as_raw_fd();
        let stderr_fd = std::io::stderr().as_raw_fd();

        // Best-effort restoration of stdout/stderr.
        // Drop cannot report errors, so failures are intentionally ignored.
        // Note: eprintln! cannot be used here because stderr may still be redirected.
        unsafe {
            let stdout_result = libc::dup2(self.saved_stdout, stdout_fd);
            let stderr_result = libc::dup2(self.saved_stderr, stderr_fd);

            // Suppress unused variable warnings; errors are intentionally ignored.
            let _ = stdout_result;
            let _ = stderr_result;

            libc::close(self.saved_stdout);
            libc::close(self.saved_stderr);
        }
        // _lock は Drop 順序により dup2 復元後に自動解放される
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
pub fn batch_update_plugins(plugin_names: &[String]) -> Vec<(String, UpdateStatusDisplay)> {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());

    // stdout/stderr をリダイレクト
    // Note: ガード作成失敗時は抑制なしで続行する（TUI 画面が乱れる可能性あり）。
    // TUI 代替スクリーン上では eprintln! が表示されないため、ログ出力は行わない。
    let _guard = OutputSuppressGuard::new();

    plugin_names
        .iter()
        .map(|name| {
            let status = run_update_plugin(name, &project_root);
            (name.clone(), status)
        })
        .collect()
}

/// 単一プラグインの更新を同期的に実行
fn run_update_plugin(plugin_name: &str, project_root: &Path) -> UpdateStatusDisplay {
    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            return UpdateStatusDisplay::Failed("No Tokio runtime available".to_string());
        }
    };

    let result = tokio::task::block_in_place(|| {
        handle.block_on(update_plugin(plugin_name, project_root, None))
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
