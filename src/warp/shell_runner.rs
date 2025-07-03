use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success {
        stdout: String,
        stderr: String,
        duration: Duration,
    },
    Error {
        stderr: String,
        exit_code: i32,
        duration: Duration,
    },
}

/// Shell runner that executes commands with streaming output
#[derive(Debug, Clone)]
pub struct ShellRunner {
    streaming: bool,
}

impl ShellRunner {
    /// Create a new shell runner
    pub fn new(streaming: bool) -> Self {
        Self { streaming }
    }

    /// Execute a shell command with optional streaming output
    pub async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        debug!("Executing command: {}", command);

        // Parse the command for cross-platform execution
        let (shell, args) = self.get_shell_command(command);

        let mut cmd = Command::new(&shell);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        let mut child = cmd.spawn().map_err(|e| {
            anyhow!("Failed to spawn command '{}': {}", command, e)
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            anyhow!("Failed to capture stdout")
        })?;

        let stderr = child.stderr.take().ok_or_else(|| {
            anyhow!("Failed to capture stderr")
        })?;

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        if self.streaming {
            // Stream output in real-time
            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            let stdout_handle = tokio::spawn(async move {
                let mut lines = stdout_reader.lines();
                let mut collected = Vec::new();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    println!("{}", line);
                    collected.push(line);
                }
                collected
            });

            let stderr_handle = tokio::spawn(async move {
                let mut lines = stderr_reader.lines();
                let mut collected = Vec::new();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("{}", line.yellow());
                    collected.push(line);
                }
                collected
            });

            // Wait for both streams and the process
            let (stdout_result, stderr_result, exit_status) = tokio::join!(
                stdout_handle,
                stderr_handle,
                child.wait()
            );

            stdout_lines = stdout_result.unwrap_or_default();
            stderr_lines = stderr_result.unwrap_or_default();

            let duration = start_time.elapsed();

            match exit_status {
                Ok(status) => {
                    if status.success() {
                        Ok(ExecutionResult::Success {
                            stdout: stdout_lines.join("\n"),
                            stderr: stderr_lines.join("\n"),
                            duration,
                        })
                    } else {
                        let exit_code = status.code().unwrap_or(-1);
                        Ok(ExecutionResult::Error {
                            stderr: stderr_lines.join("\n"),
                            exit_code,
                            duration,
                        })
                    }
                }
                Err(e) => Err(anyhow!("Failed to wait for command: {}", e)),
            }
        } else {
            // Collect all output at once
            let output = child.wait_with_output().await.map_err(|e| {
                anyhow!("Failed to execute command: {}", e)
            })?;

            let duration = start_time.elapsed();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok(ExecutionResult::Success {
                    stdout,
                    stderr,
                    duration,
                })
            } else {
                let exit_code = output.status.code().unwrap_or(-1);
                Ok(ExecutionResult::Error {
                    stderr,
                    exit_code,
                    duration,
                })
            }
        }
    }

    /// Get the appropriate shell command for the current platform
    fn get_shell_command(&self, command: &str) -> (String, Vec<String>) {
        if cfg!(target_os = "windows") {
            // Use PowerShell on Windows for better command support
            ("powershell".to_string(), vec!["-Command".to_string(), command.to_string()])
        } else {
            // Use bash on Unix-like systems
            ("bash".to_string(), vec!["-c".to_string(), command.to_string()])
        }
    }

    /// Execute a command with a timeout
    pub async fn execute_with_timeout(&self, command: &str, timeout: Duration) -> Result<ExecutionResult> {
        match tokio::time::timeout(timeout, self.execute(command)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!("Command timed out after {:.2}s", timeout.as_secs_f64())),
        }
    }

    /// Execute a command in a specific directory
    pub async fn execute_in_dir(&self, command: &str, dir: &str) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        debug!("Executing command in {}: {}", dir, command);

        let (shell, mut args) = self.get_shell_command(command);
        
        // Modify command to change directory first
        let full_command = if cfg!(target_os = "windows") {
            format!("cd '{}'; {}", dir, command)
        } else {
            format!("cd '{}' && {}", dir, command)
        };
        
        args[1] = full_command;

        let mut cmd = Command::new(&shell);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        let output = cmd.output().await.map_err(|e| {
            anyhow!("Failed to execute command in directory '{}': {}", dir, e)
        })?;

        let duration = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(ExecutionResult::Success {
                stdout,
                stderr,
                duration,
            })
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            Ok(ExecutionResult::Error {
                stderr,
                exit_code,
                duration,
            })
        }
    }

    /// Check if a command is potentially dangerous
    pub fn is_dangerous_command(&self, command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf /",
            "del /s /q c:",
            "format c:",
            "shutdown",
            "reboot",
            "dd if=",
            "mkfs.",
            "> /dev/",
            "chmod 777 /",
            "chown root /",
        ];

        let command_lower = command.to_lowercase();
        dangerous_patterns.iter().any(|pattern| command_lower.contains(pattern))
    }

    /// Execute a command with safety checks
    pub async fn execute_safely(&self, command: &str) -> Result<ExecutionResult> {
        if self.is_dangerous_command(command) {
            return Err(anyhow!(
                "Refusing to execute potentially dangerous command: {}",
                command
            ));
        }

        self.execute(command).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_command_detection() {
        let runner = ShellRunner::new(false);
        
        assert!(runner.is_dangerous_command("rm -rf /"));
        assert!(runner.is_dangerous_command("del /s /q c:"));
        assert!(runner.is_dangerous_command("shutdown -h now"));
        assert!(!runner.is_dangerous_command("ls -la"));
        assert!(!runner.is_dangerous_command("npm install"));
    }

    #[test]
    fn test_shell_command_parsing() {
        let runner = ShellRunner::new(false);
        
        let (shell, args) = runner.get_shell_command("echo hello");
        
        if cfg!(target_os = "windows") {
            assert_eq!(shell, "powershell");
            assert_eq!(args[0], "-Command");
        } else {
            assert_eq!(shell, "bash");
            assert_eq!(args[0], "-c");
        }
        
        assert_eq!(args[1], "echo hello");
    }
}
