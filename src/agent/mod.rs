use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::ollama::client::OllamaClient;
use crate::ollama::OllamaConfig;
use crate::ollama::client::ChatMessage as OllamaChatMessage;

pub mod planner;

#[derive(Debug, Clone)]
pub enum AIProvider {
    OpenAI,
    Ollama,
}

#[derive(Debug, Clone)]
pub struct Agent {
    client: Client,
    config: crate::config::AgentConfig,
    api_key: Option<String>,
    provider: AIProvider,
    ollama_client: Option<OllamaClient>,
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

// Ollama API structures
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

impl Agent {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.agent.timeout_seconds))
            .build()?;
        
        // Determine provider based on config preference and API key availability
        let provider = match config.agent.preferred_provider.as_str() {
            "openai" if config.get_openai_api_key().is_some() => AIProvider::OpenAI,
            "ollama" => AIProvider::Ollama,
            _ => {
                // Default to Ollama (phi4) as it's more powerful and local
                AIProvider::Ollama
            }
        };
        
        // Initialize Ollama client with phi4 model
        let ollama_client = if matches!(provider, AIProvider::Ollama) {
            let ollama_config = OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model: "phi4:latest".to_string(), // Use phi4 model
                temperature: config.agent.temperature,
                max_tokens: Some(config.agent.max_tokens),
                timeout: Duration::from_secs(config.agent.timeout_seconds),
            };
            
            match OllamaClient::new(ollama_config) {
                Ok(client) => {
                    info!("✅ Ollama client initialized with phi4 model");
                    Some(client)
                }
                Err(e) => {
                    warn!("Failed to initialize Ollama client: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            client,
            config: config.agent.clone(),
            api_key: config.get_openai_api_key(),
            provider,
            ollama_client,
        })
    }
    
    pub async fn process_query(&self, query: &str) -> Result<String> {
        info!("Processing agent query: {}", query);
        
        match self.provider {
            AIProvider::OpenAI => self.process_openai_query(query).await,
            AIProvider::Ollama => self.process_ollama_query(query).await,
        }
    }
    
    async fn process_openai_query(&self, query: &str) -> Result<String> {
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
    
    async fn process_ollama_query(&self, query: &str) -> Result<String> {
        debug!("🤖 Sending request to Ollama phi4 model");
        
        // Check if we have an Ollama client
        if let Some(ollama_client) = &self.ollama_client {
            // First check if Ollama is healthy
            if !ollama_client.health_check().await.unwrap_or(false) {
                warn!("⚠️  Ollama service not available, using fallback");
                return Ok(self.generate_ollama_fallback_response(query));
            }
            
            // Create structured chat messages for phi4
            let system_prompt = self.create_system_prompt();
            let messages = vec![
                OllamaChatMessage::system(&system_prompt),
                OllamaChatMessage::user(query),
            ];
            
            match ollama_client.chat(&messages).await {
                Ok(response) => {
                    info!("🎯 phi4 model responded successfully");
                    Ok(response.trim().to_string())
                }
                Err(e) => {
                    warn!("❌ phi4 model error: {}", e);
                    Ok(self.generate_ollama_fallback_response(query))
                }
            }
        } else {
            info!("🔄 Ollama client not initialized, using enhanced fallback");
            Ok(self.generate_ollama_fallback_response(query))
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
    
    fn generate_ollama_fallback_response(&self, query: &str) -> String {
        // Enhanced keyword-based fallback for Ollama
        let query_lower = query.to_lowercase();
        
        // Study plan keywords
        if query_lower.contains("study") && query_lower.contains("plan") {
            return format!(
                "📚 **Study Plan Suggestion**\n\n\
                Based on your request for a study plan, here's a structured approach:\n\n\
                **This Week's Schedule:**\n\
                • Monday: Review fundamentals and create task list\n\
                • Tuesday-Thursday: Focus on core topics (2-3 hours daily)\n\
                • Friday: Practice problems and assessments\n\
                • Weekend: Review, summarize, and prepare for next week\n\n\
                **Suggested Commands:**\n\
                • Create tasks: `agentic task add --title 'Study Topic X' --priority high`\n\
                • Start prep session: `agentic prep start --exam [YOUR_EXAM] --duration 120`\n\
                • Track progress: `agentic prep stats --period week`\n\n\
                **💡 Tip:** Install Ollama (https://ollama.ai) for unlimited AI assistance!\n\
                Run: `ollama pull gemma3` to get started with free AI support."
            );
        }
        
        // Task management keywords
        if query_lower.contains("task") {
            if query_lower.contains("add") || query_lower.contains("create") {
                return "📝 To add a task: `agentic task add --title 'Your task' --priority [low|medium|high] --description 'Optional description'`".to_string();
            } else if query_lower.contains("list") || query_lower.contains("show") {
                return "📋 To list tasks: `agentic task list` (add --status todo/in-progress/done for filtering)".to_string();
            }
        }
        
        // Preparation keywords
        if query_lower.contains("prep") || query_lower.contains("exam") {
            return "🎯 For exam preparation: `agentic prep start --exam [EXAM_NAME] --schedule daily`\nThen add topics: `agentic prep add --topic 'Your Topic' --priority 5`".to_string();
        }
        
        // Blog keywords
        if query_lower.contains("blog") || query_lower.contains("write") {
            return "✍️ For blogging: `agentic blog new --title 'Your Title' --tags topic1,topic2`\nEdit: `agentic blog edit --post-id [ID]`".to_string();
        }
        
        // General productivity
        if query_lower.contains("productivity") || query_lower.contains("organize") {
            return format!(
                "🚀 **Productivity Boost**\n\n\
                Here's your productivity toolkit:\n\
                • `agentic task add --title 'Daily Goals' --priority high`\n\
                • `agentic prep start --exam 'Personal Development'`\n\
                • `agentic blog new --title 'Progress Journal'`\n\n\
                **🔥 Pro Tip:** For unlimited AI assistance, install Ollama!\n\
                Visit: https://ollama.ai and run `ollama pull gemma3`"
            );
        }
        
        // Default enhanced response
        format!(
            "🤖 **AI Assistant (Free Mode)**\n\n\
            I'm running in free mode with enhanced pattern matching. For unlimited AI responses:\n\n\
            **Option 1: Install Ollama (Recommended - Free & Unlimited)**\n\
            1. Visit https://ollama.ai and download Ollama\n\
            2. Run: `ollama pull gemma3`\n\
            3. Restart agentic CLI for automatic AI support\n\n\
            **Option 2: Use OpenAI**\n\
            Add your API key to the config file\n\n\
            **Your Query:** {}\n\n\
            **Quick Commands:**\n\
            • Tasks: `agentic task add --title 'Your task'`\n\
            • Study: `agentic prep start --exam CET`\n\
            • Blog: `agentic blog new --title 'Post title'`\n\
            • Run: `agentic run 'any command'`", 
            query
        )
    }
    
    #[allow(dead_code)]
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
