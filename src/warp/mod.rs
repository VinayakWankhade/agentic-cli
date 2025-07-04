use anyhow::Result;
use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod agents;
pub mod config;
pub mod pipeline;
pub mod shell_runner;

use crate::config::Config;

/// Core Warp pipeline that orchestrates the three-agent system
#[derive(Debug, Clone)]
pub struct WarpPipeline {
    planner: agents::PlannerAgent,
    coder: agents::CoderAgent,
    shell_runner: shell_runner::ShellRunner,
    config: WarpConfig,
}

/// Configuration for the Warp pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpConfig {
    pub planner_model: String,
    pub coder_model: String,
    pub fallback_model: String,
    pub ollama_host: String,
    pub timeout_seconds: u64,
    pub streaming: bool,
}

impl Default for WarpConfig {
    fn default() -> Self {
        Self {
            planner_model: "phi4".to_string(),
            coder_model: "codellama".to_string(),
            fallback_model: "gemma3".to_string(),
            ollama_host: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            streaming: true,
        }
    }
}

impl WarpPipeline {
    /// Create a new Warp pipeline instance
    pub fn new(_config: &Config) -> Result<Self> {
        let warp_config = WarpConfig::default(); // TODO: Load from .agentic.toml
        
        let client = Client::builder()
            .timeout(Duration::from_secs(warp_config.timeout_seconds))
            .build()?;

        let planner = agents::PlannerAgent::new(
            client.clone(),
            warp_config.ollama_host.clone(),
            warp_config.planner_model.clone(),
            warp_config.fallback_model.clone(),
        );

        let coder = agents::CoderAgent::new(
            client.clone(),
            warp_config.ollama_host.clone(),
            warp_config.coder_model.clone(),
            warp_config.fallback_model.clone(),
        );

        let shell_runner = shell_runner::ShellRunner::new(warp_config.streaming);

        Ok(Self {
            planner,
            coder,
            shell_runner,
            config: warp_config,
        })
    }

    /// Execute the full pipeline: natural language -> plan -> command -> execution
    pub async fn execute(&self, input: &str) -> Result<pipeline::PipelineResult> {
        println!("{} {}", "🧠".blue(), "Planning...".cyan());
        
        // Step 1: Planning Agent
        let plan = self.planner.generate_plan(input).await?;
        println!("{} {}: {}", "📝".green(), "Plan".green().bold(), plan.cyan());
        
        println!("\n{} {}", "💻".blue(), "Translating to shell...".cyan());
        
        // Step 2: Coder Agent
        let command = self.coder.generate_command(&plan).await?;
        println!("{} {}: {}", "🔧".green(), "Suggested Command".green().bold(), command.yellow());
        
        // Ask for confirmation
        println!("\n{} Execute this command? (y/N): ", "❓".yellow());
        let mut input_line = String::new();
        std::io::stdin().read_line(&mut input_line)?;
        
        if !input_line.trim().to_lowercase().starts_with('y') {
            return Ok(pipeline::PipelineResult {
                original_input: input.to_string(),
                plan: plan.clone(),
                command: command.clone(),
                execution_result: None,
                cancelled: true,
            });
        }

        println!("\n{} {}", "🚀".blue(), "Running Command...".cyan());
        
        // Step 3: Shell Runner
        let execution_result = self.shell_runner.execute(&command).await?;
        
        // Display results
        match &execution_result {
            shell_runner::ExecutionResult::Success { stdout, stderr, duration } => {
                println!("{} {}:", "✅".green(), "Output".green().bold());
                if !stdout.is_empty() {
                    println!("{}", stdout);
                }
                if !stderr.is_empty() {
                    println!("{} {}:", "⚠️".yellow(), "Warnings".yellow());
                    println!("{}", stderr.yellow());
                }
                println!("\n{} Completed in {:.2}s", "⚡".green(), duration.as_secs_f64());
            }
            shell_runner::ExecutionResult::Error { stderr, exit_code, duration } => {
                println!("{} {} (exit code: {}):", "❌".red(), "Error".red().bold(), exit_code);
                println!("{}", stderr.red());
                println!("\n{} Failed after {:.2}s", "💥".red(), duration.as_secs_f64());
            }
        }

        Ok(pipeline::PipelineResult {
            original_input: input.to_string(),
            plan,
            command,
            execution_result: Some(execution_result),
            cancelled: false,
        })
    }

    /// Execute only the planning and coding steps (no execution)
    pub async fn dry_run(&self, input: &str) -> Result<(String, String)> {
        println!("{} {} (dry run)", "🧠".blue(), "Planning...".cyan());
        let plan = self.planner.generate_plan(input).await?;
        println!("{} {}: {}", "📝".green(), "Plan".green().bold(), plan.cyan());
        
        println!("\n{} {} (dry run)", "💻".blue(), "Translating to shell...".cyan());
        let command = self.coder.generate_command(&plan).await?;
        println!("{} {}: {}", "🔧".green(), "Suggested Command".green().bold(), command.yellow());
        
        Ok((plan, command))
    }
}
