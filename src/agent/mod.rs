use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::config::Config;

pub mod planner;

#[derive(Debug, Clone)]
pub struct Agent {
    client: Client,
    config: crate::config::AgentConfig,
    api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

impl Agent {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.agent.timeout_seconds))
            .build()?;
        
        Ok(Self {
            client,
            config: config.agent.clone(),
            api_key: config.get_openai_api_key(),
        })
    }
    
    pub async fn process_query(&self, query: &str) -> Result<String> {
        info!("Processing agent query: {}", query);
        
        // Check if we have an API key
        if self.api_key.is_none() {
            return Ok(self.generate_fallback_response(query));
        }
        
        // Create system prompt for command interpretation
        let system_prompt = self.create_system_prompt();
        
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: query.to_string(),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };
        
        debug!("Sending request to OpenAI API");
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            warn!("OpenAI API error: {}", error_text);
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }
        
        let chat_response: ChatResponse = response.json().await?;
        
        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response from OpenAI API"))
        }
    }
    
    fn create_system_prompt(&self) -> String {
        r#"You are an intelligent CLI assistant that helps users with terminal commands and task management.

Your primary responsibilities:
1. Convert natural language queries into specific CLI commands
2. Provide helpful explanations for complex commands
3. Suggest best practices and alternatives
4. Help with task management, study preparation, and productivity

Available command categories:
- task: Task management (add, list, complete, priority)
- prep: Study and exam preparation
- blog: Content creation and blogging
- run: Execute arbitrary commands
- agent: AI-powered assistance

When responding:
- Be concise and practical
- Provide the exact command to run when possible
- Include brief explanations for complex commands
- Suggest safer alternatives for potentially dangerous operations
- Use modern CLI tools and best practices

Examples:
User: "prep for cet exam"
Response: "agentic prep start --exam CET --schedule daily"

User: "add a high priority task to build dashboard"
Response: "agentic task add --title 'Build dashboard' --priority high"

User: "show me recent tasks"
Response: "agentic task list --recent"
"#.to_string()
    }
    
    fn generate_fallback_response(&self, query: &str) -> String {
        // Simple keyword-based fallback when no API key is available
        let query_lower = query.to_lowercase();
        
        if query_lower.contains("task") && query_lower.contains("add") {
            "To add a task, use: agentic task add --title 'Your task title' --priority [low|medium|high]".to_string()
        } else if query_lower.contains("task") && query_lower.contains("list") {
            "To list tasks, use: agentic task list".to_string()
        } else if query_lower.contains("prep") && query_lower.contains("start") {
            "To start a preparation session, use: agentic prep start --exam [exam_name]".to_string()
        } else if query_lower.contains("blog") {
            "For blog commands, use: agentic blog --help".to_string()
        } else {
            format!(
                "I'd love to help, but I need an OpenAI API key to provide intelligent responses.\n\
                Please set the OPENAI_API_KEY environment variable or add it to your config.\n\
                \n\
                For now, here are some basic commands you can try:\n\
                - agentic task add --title 'Your task'\n\
                - agentic prep start --exam CET\n\
                - agentic blog new --title 'Your blog post'\n\
                - agentic run 'your command here'\n\
                \n\
                Your query: {}", query
            )
        }
    }
    
    pub async fn interpret_command(&self, query: &str) -> Result<String> {
        // This method specifically focuses on converting natural language to CLI commands
        let enhanced_query = format!(
            "Convert this natural language request into a specific CLI command using the agentic CLI tool: {}",
            query
        );
        
        self.process_query(&enhanced_query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    
    #[test]
    fn test_fallback_responses() {
        let config = Config::default();
        let agent = Agent::new(&config).unwrap();
        
        let response = agent.generate_fallback_response("add a task to study");
        assert!(response.contains("agentic task add"));
        
        let response = agent.generate_fallback_response("start prep for exam");
        assert!(response.contains("agentic prep start"));
    }
}
