// Shell execution is the highest-privilege action Arbiter can perform.
// Every command must pass The Baton toggle before it runs, basically an explicit user allowance.
use tracing::{info, warn};

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("The Baton: '{0}' is not allowed — grant it in the Signet first")]
    BatonNotGranted(String),
    #[error("Shell: spawn failed: {0}")]
    SpawnFailed(String),
    #[error("Shell: exit {status} — {stderr}")]
    NonZeroExit { status: i32, stderr: String },
}

#[derive(Debug)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub async fn run(
    target_key: &str,
    command: &str,
    args: &[&str],
    allowed_targets: &std::collections::HashSet<String>,
) -> Result<ShellOutput, ShellError> {
    if !allowed_targets.contains(target_key) {
        warn!(%target_key, "The Baton: execution blocked — not in allowed set");
        return Err(ShellError::BatonNotGranted(target_key.to_string()));
    }

    info!(%command, ?args, "The Baton: executing allowed command");

    let output = tokio::process::Command::new(command)
        .args(args)
        .output()
        .await
        .map_err(|e| ShellError::SpawnFailed(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    if !output.status.success() {
        warn!(%exit_code, %stderr, "Shell: command exited with error");
        return Err(ShellError::NonZeroExit {
            status: exit_code,
            stderr,
        });
    }

    info!(%exit_code, "Shell: command completed successfully");
    Ok(ShellOutput {
        stdout,
        stderr,
        exit_code,
    })
}

pub async fn spawn_detached(
    target_key: &str,
    command: &str,
    args: &[&str],
    allowed_targets: &std::collections::HashSet<String>,
) -> Result<(), ShellError> {
    if !allowed_targets.contains(target_key) {
        warn!(%target_key, "The Baton: detached spawn blocked");
        return Err(ShellError::BatonNotGranted(target_key.to_string()));
    }

    tokio::process::Command::new(command)
        .args(args)
        .spawn()
        .map_err(|e| ShellError::SpawnFailed(e.to_string()))?;

    info!(%command, "The Baton: process spawned detached");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_baton_granted() {
        let mut allowed = HashSet::new();
        allowed.insert("safe_echo".to_string());

        let cmd = if cfg!(target_os = "windows") {
            "cmd.exe"
        } else {
            "sh"
        };
        let args = if cfg!(target_os = "windows") {
            vec!["/c", "echo Hello"]
        } else {
            vec!["-c", "echo Hello"]
        };

        let result = run("safe_echo", cmd, &args, &allowed).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout.contains("Hello"));
        assert_eq!(output.exit_code, 0);
    }

    #[tokio::test]
    async fn test_baton_blocked() {
        let mut allowed = HashSet::new();
        allowed.insert("safe_echo".to_string());

        let cmd = if cfg!(target_os = "windows") {
            "cmd.exe"
        } else {
            "sh"
        };
        let args = if cfg!(target_os = "windows") {
            vec!["/c", "echo Malicious"]
        } else {
            vec!["-c", "echo Malicious"]
        };

        let result = run("malicious_script", cmd, &args, &allowed).await;

        match result {
            Err(ShellError::BatonNotGranted(req)) => {
                assert_eq!(req, "malicious_script");
            }
            _ => panic!("Expected BatonNotGranted error"),
        }
    }
}
