use std::io;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// 本地 Git 命令超时（30 秒）
pub const GIT_LOCAL_TIMEOUT: Duration = Duration::from_secs(30);

/// 网络 Git 命令超时（120 秒）
pub const GIT_NETWORK_TIMEOUT: Duration = Duration::from_secs(120);

/// 带超时的命令执行，替代 `Command::output()`
///
/// 通过 `try_wait` 轮询实现超时检测，超时后 kill 子进程。
pub fn output_with_timeout(cmd: &mut Command, timeout: Duration) -> io::Result<Output> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(_) => return child.wait_with_output(),
            None if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!(
                        "命令执行超时（已等待 {} 秒）",
                        timeout.as_secs()
                    ),
                ));
            }
            None => std::thread::sleep(Duration::from_millis(200)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_with_timeout_success() {
        let output = output_with_timeout(
            Command::new("git").arg("--version"),
            Duration::from_secs(5),
        );
        assert!(output.is_ok());
        let out = output.unwrap();
        assert!(out.status.success());
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("git version"));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_output_with_timeout_expires() {
        let result = output_with_timeout(
            Command::new("sleep").arg("10"),
            Duration::from_secs(1),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
    }

    #[cfg(windows)]
    #[test]
    fn test_output_with_timeout_expires() {
        let result = output_with_timeout(
            Command::new("ping").args(["-n", "10", "127.0.0.1"]),
            Duration::from_secs(1),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
    }
}
