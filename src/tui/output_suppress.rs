//! stdout/stderr を一時的に抑制する RAII ガード
//!
//! TUI の代替スクリーンが println!/eprintln! で乱れるのを防ぐ。

/// stdout/stderr を一時的に抑制する RAII ガード
///
/// dup2 によるプロセスグローバルな fd リダイレクションを行うため、
/// グローバル Mutex で排他制御し、リダイレクト前に flush する。
/// Drop で確実に復元される。
#[cfg(unix)]
pub(crate) struct OutputSuppressGuard {
    saved_stdout: i32,
    saved_stderr: i32,
    _lock: std::sync::MutexGuard<'static, ()>,
}

/// OutputSuppressGuard の排他制御用 Mutex
#[cfg(unix)]
static OUTPUT_SUPPRESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(unix)]
impl OutputSuppressGuard {
    pub(crate) fn new() -> Option<Self> {
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
                // dup2 実行前のエラーなので、dev_null_fd はまだ stdout/stderr に
                // リダイレクトされておらず、常に close してよい
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
pub(crate) struct OutputSuppressGuard;

#[cfg(not(unix))]
impl OutputSuppressGuard {
    pub(crate) fn new() -> Option<Self> {
        Some(Self)
    }
}
