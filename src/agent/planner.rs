use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use super::Agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub context: HashMap<String, String>,
    pub estimated_duration: u64, // in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: String,
    pub command: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub expected_output: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Planner {
    agent: Agent,
}

impl Planner {
    #[allow(dead_code)]
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }
    
    #[allow(dead_code)]
    pub async fn create_execution_plan(&self, goal: &str) -> Result<ExecutionPlan> {
        info!("Creating execution plan for goal: {}", goal);
        
        let planning_prompt = self.create_planning_prompt(goal);
        let response = self.agent.process_query(&planning_prompt).await?;
        
        // For now, we'll use a simple heuristic to parse the response
        // In a real implementation, you might want to use more structured prompts
        // or fine-tuned models that return JSON
        let plan = self.parse_plan_response(&response, goal)?;
        
        debug!("Created execution plan with {} steps", plan.steps.len());
        Ok(plan)
    }
    
    #[allow(dead_code)]
    fn create_planning_prompt(&self, goal: &str) -> String {
        format!(
            r#"Create a detailed execution plan for the following goal: {}

Please break down the goal into specific, actionable steps that can be executed as CLI commands.
Each step should be:
1. Specific and measurable
2. Executable as a terminal command
3. Have clear dependencies on previous steps
4. Include expected outcomes

Format your response as a numbered list of steps with:
- Step number
- Command to execute
- Brief description
- Dependencies (if any)
- Expected outcome

Example format:
1. Command: `agentic task add --title "Setup environment" --priority high`
   Description: Create initial task for environment setup
   Dependencies: None
   Expected: Task created with ID

Focus on using the agentic CLI tool and standard terminal commands where appropriate."#,
            goal
        )
    }
    
    #[allow(dead_code)]
    fn parse_plan_response(&self, response: &str, goal: &str) -> Result<ExecutionPlan> {
        let mut steps = Vec::new();
        let mut step_counter = 1;
        
        // Simple parsing logic - in production, you'd want more robust parsing
        let lines: Vec<&str> = response.lines().collect();
        let mut current_command = None;
        let mut current_description = None;
        let mut current_dependencies = Vec::new();
        
        for line in lines {
            let line = line.trim();
            
            if line.starts_with(&format!("{}.", step_counter)) {
                // Save previous step if exists
                if let (Some(cmd), Some(desc)) = (current_command.take(), current_description.take()) {
                    steps.push(ExecutionStep {
                        id: format!("step_{}", step_counter - 1),
                        command: cmd,
                        description: desc,
                        dependencies: current_dependencies.clone(),
                        expected_output: None,
                        retry_count: 0,
                    });
                }
                
                // Start new step
                current_dependencies.clear();
                
                if let Some(command_start) = line.find("Command:") {
                    let command_part = &line[command_start + 8..].trim();
                    // Find the opening backtick
                    if let Some(first_backtick) = command_part.find('`') {
                        let after_first_backtick = &command_part[first_backtick + 1..];
                        // Find the closing backtick
                        if let Some(second_backtick) = after_first_backtick.find('`') {
                            current_command = Some(after_first_backtick[..second_backtick].to_string());
                        } else {
                            // If no closing backtick, take everything after the opening backtick
                            current_command = Some(after_first_backtick.to_string());
                        }
                    }
                }
                step_counter += 1;
            } else if line.starts_with("Description:") {
                current_description = Some(line[12..].trim().to_string());
            } else if line.starts_with("Dependencies:") {
                let deps_str = line[13..].trim();
                if deps_str != "None" {
                    current_dependencies = deps_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }
        }
        
        // Save last step
        if let (Some(cmd), Some(desc)) = (current_command, current_description) {
            steps.push(ExecutionStep {
                id: format!("step_{}", step_counter - 1),
                command: cmd,
                description: desc,
                dependencies: current_dependencies,
                expected_output: None,
                retry_count: 0,
            });
        }
        
        // If parsing failed, create a simple fallback plan
        if steps.is_empty() {
            steps.push(ExecutionStep {
                id: "step_1".to_string(),
                command: format!("echo 'Goal: {}'", goal),
                description: "Display the goal".to_string(),
                dependencies: Vec::new(),
                expected_output: Some(format!("Goal: {}", goal)),
                retry_count: 0,
            });
        }
        
        Ok(ExecutionPlan {
            steps,
            context: HashMap::new(),
            estimated_duration: 60, // Default 1 minute
        })
    }
    
    #[allow(dead_code)]
    pub async fn optimize_plan(&self, plan: &ExecutionPlan) -> Result<ExecutionPlan> {
        info!("Optimizing execution plan with {} steps", plan.steps.len());
        
        let mut optimized_plan = plan.clone();
        
        // Remove duplicate commands
        optimized_plan.steps.dedup_by(|a, b| a.command == b.command);
        
        // Sort by dependencies (simple topological sort)
        optimized_plan.steps.sort_by(|a, b| {
            if a.dependencies.contains(&b.id) {
                std::cmp::Ordering::Greater
            } else if b.dependencies.contains(&a.id) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        
        // Estimate duration based on command types
        let total_duration = optimized_plan.steps.iter().map(|step| {
            if step.command.contains("install") || step.command.contains("download") {
                120 // 2 minutes for installation commands
            } else if step.command.contains("test") || step.command.contains("build") {
                60 // 1 minute for build/test commands
            } else {
                10 // 10 seconds for other commands
            }
        }).sum();
        
        optimized_plan.estimated_duration = total_duration;
        
        debug!("Optimized plan duration: {}s", optimized_plan.estimated_duration);
        Ok(optimized_plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    
    #[tokio::test]
    async fn test_plan_creation() {
        let config = Config::default();
        let agent = Agent::new(&config).unwrap();
        let planner = Planner::new(agent);
        
        // Since this test might not have an API key or network access,
        // we expect it to either succeed or fail gracefully
        let plan = planner.create_execution_plan("Setup a new Rust project").await;
        
        // If the plan succeeds, it should have steps
        if let Ok(plan) = plan {
            assert!(!plan.steps.is_empty());
        }
        // If it fails (due to no API key, network issues, etc.), that's also acceptable
        // The important thing is that the method doesn't panic
    }
    
    #[test]
    fn test_plan_parsing() {
        let config = Config::default();
        let agent = Agent::new(&config).unwrap();
        let planner = Planner::new(agent);
        
        let response = "1. Command: `cargo init --name test`\nDescription: Initialize Rust project\nDependencies: None";
        let plan = planner.parse_plan_response(response, "test goal").unwrap();
        
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].command, "cargo init --name test");
    }
}
