use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::shell_runner::ExecutionResult;

/// Result of a complete Warp pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub original_input: String,
    pub plan: String,
    pub command: String,
    pub execution_result: Option<ExecutionResult>,
    pub cancelled: bool,
}

impl PipelineResult {
    /// Check if the pipeline execution was successful
    pub fn is_success(&self) -> bool {
        if self.cancelled {
            return false;
        }
        
        match &self.execution_result {
            Some(ExecutionResult::Success { .. }) => true,
            Some(ExecutionResult::Error { .. }) => false,
            None => false,
        }
    }

    /// Get the total execution time if available
    pub fn execution_duration(&self) -> Option<Duration> {
        match &self.execution_result {
            Some(ExecutionResult::Success { duration, .. }) => Some(*duration),
            Some(ExecutionResult::Error { duration, .. }) => Some(*duration),
            None => None,
        }
    }

    /// Get the output text (stdout) if successful
    pub fn output(&self) -> Option<&str> {
        match &self.execution_result {
            Some(ExecutionResult::Success { stdout, .. }) => Some(stdout),
            _ => None,
        }
    }

    /// Get the error text (stderr) if failed
    pub fn error(&self) -> Option<&str> {
        match &self.execution_result {
            Some(ExecutionResult::Error { stderr, .. }) => Some(stderr),
            Some(ExecutionResult::Success { stderr, .. }) if !stderr.is_empty() => Some(stderr),
            _ => None,
        }
    }

    /// Get the exit code if command was executed
    pub fn exit_code(&self) -> Option<i32> {
        match &self.execution_result {
            Some(ExecutionResult::Success { .. }) => Some(0),
            Some(ExecutionResult::Error { exit_code, .. }) => Some(*exit_code),
            None => None,
        }
    }

    /// Generate a summary of the pipeline execution
    pub fn summary(&self) -> String {
        if self.cancelled {
            return "Pipeline cancelled by user".to_string();
        }

        match &self.execution_result {
            Some(ExecutionResult::Success { duration, .. }) => {
                format!("✅ Command executed successfully in {:.2}s", duration.as_secs_f64())
            }
            Some(ExecutionResult::Error { exit_code, duration, .. }) => {
                format!("❌ Command failed with exit code {} after {:.2}s", exit_code, duration.as_secs_f64())
            }
            None => "⚠️ Command was not executed".to_string(),
        }
    }
}

/// Pipeline execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub cancelled_executions: usize,
    pub average_duration: Duration,
    pub most_common_commands: Vec<(String, usize)>,
}

impl PipelineStats {
    pub fn new() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            cancelled_executions: 0,
            average_duration: Duration::from_secs(0),
            most_common_commands: Vec::new(),
        }
    }

    /// Update statistics with a new pipeline result
    pub fn update(&mut self, result: &PipelineResult) {
        self.total_executions += 1;

        if result.cancelled {
            self.cancelled_executions += 1;
        } else if result.is_success() {
            self.successful_executions += 1;
        } else {
            self.failed_executions += 1;
        }

        // Update average duration
        if let Some(duration) = result.execution_duration() {
            let total_time = self.average_duration.as_secs_f64() * (self.total_executions - 1) as f64;
            let new_average = (total_time + duration.as_secs_f64()) / self.total_executions as f64;
            self.average_duration = Duration::from_secs_f64(new_average);
        }
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }
        (self.successful_executions as f64 / self.total_executions as f64) * 100.0
    }
}
