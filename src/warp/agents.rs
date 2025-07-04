use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Ollama API request structure
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama API response structure
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Planning Agent - converts natural language to structured plans
#[derive(Debug, Clone)]
pub struct PlannerAgent {
    client: Client,
    ollama_host: String,
    model: String,
    fallback_model: String,
}

impl PlannerAgent {
    pub fn new(client: Client, ollama_host: String, model: String, fallback_model: String) -> Self {
        Self {
            client,
            ollama_host,
            model,
            fallback_model,
        }
    }

    /// Generate a structured plan from natural language input
    pub async fn generate_plan(&self, input: &str) -> Result<String> {
        let system_prompt = r#"You are a planning agent that converts natural language requests into clear, structured plans.

Your role:
1. Analyze the user's request and understand their intent
2. Break down complex requests into logical steps
3. Identify the tools, technologies, and actions needed
4. Output a concise plan in plain English

Guidelines:
- Be specific about what needs to be done
- Include key details like project names, technologies, or configurations
- Keep plans actionable and clear
- Focus on the "what" rather than the "how"

Examples:
Input: "create a new React app and start the dev server"
Output: "Create a new React project using Vite, install dependencies, and start the development server"

Input: "show me all running Docker containers and their status"
Output: "List all currently running Docker containers with their status information"

Input: "backup my database and compress it"
Output: "Create a database backup, compress the backup file, and save it to a secure location"
"#;

        let prompt = format!("{}\n\nUser Request: {}\nPlan:", system_prompt, input);

        // Try primary model first
        match self.query_model(&self.model, &prompt).await {
            Ok(response) => Ok(response.trim().to_string()),
            Err(_) => {
                warn!("Primary model {} failed, trying fallback {}", self.model, self.fallback_model);
                // Try fallback model
                match self.query_model(&self.fallback_model, &prompt).await {
                    Ok(response) => Ok(response.trim().to_string()),
                    Err(_) => {
                        // Use pattern-based fallback
                        Ok(self.generate_fallback_plan(input))
                    }
                }
            }
        }
    }

    async fn query_model(&self, model: &str, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/api/generate", self.ollama_host))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama API error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }

    fn generate_fallback_plan(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();

        if input_lower.contains("react") && input_lower.contains("app") {
            "Create a new React application with modern tooling and start development server".to_string()
        } else if input_lower.contains("docker") && input_lower.contains("container") {
            "List and inspect Docker containers with their current status".to_string()
        } else if input_lower.contains("git") && input_lower.contains("repo") {
            "Initialize or manage Git repository with version control operations".to_string()
        } else if input_lower.contains("install") {
            "Install the specified software package or dependency".to_string()
        } else if input_lower.contains("backup") {
            "Create a backup of the specified data or files".to_string()
        } else if input_lower.contains("test") {
            "Run tests for the current project or specified component".to_string()
        } else {
            format!("Execute the requested operation: {}", input)
        }
    }
}

/// Coding Agent - converts structured plans to shell commands
#[derive(Debug, Clone)]
pub struct CoderAgent {
    client: Client,
    ollama_host: String,
    model: String,
    fallback_model: String,
}

impl CoderAgent {
    pub fn new(client: Client, ollama_host: String, model: String, fallback_model: String) -> Self {
        Self {
            client,
            ollama_host,
            model,
            fallback_model,
        }
    }

    /// Generate shell commands from a structured plan
    pub async fn generate_command(&self, plan: &str) -> Result<String> {
        let system_prompt = r#"You are a coding agent that converts structured plans into precise shell commands.

Your role:
1. Translate plans into executable shell commands
2. Use modern, cross-platform tools when possible
3. Chain commands efficiently with && or ;
4. Ensure commands are safe and follow best practices
5. Output ONLY the command(s), no explanations

Guidelines:
- Use modern tools (npm/yarn, git, docker, etc.)
- Prefer single-line command chains when logical
- Include necessary flags and options
- Use safe defaults and common conventions
- Work on Windows (PowerShell/CMD), macOS, and Linux

Examples:
Plan: "Create a new React project using Vite, install dependencies, and start the development server"
Command: npm create vite@latest my-react-app --template react && cd my-react-app && npm install && npm run dev

Plan: "List all currently running Docker containers with their status information"
Command: docker ps -a --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

Plan: "Create a database backup, compress the backup file, and save it to a secure location"
Command: mysqldump -u root -p mydb > backup.sql && gzip backup.sql && mv backup.sql.gz ~/backups/
"#;

        let prompt = format!("{}\n\nPlan: {}\nCommand:", system_prompt, plan);

        // Try primary model first
        match self.query_model(&self.model, &prompt).await {
            Ok(response) => Ok(response.trim().to_string()),
            Err(_) => {
                warn!("Primary model {} failed, trying fallback {}", self.model, self.fallback_model);
                // Try fallback model
                match self.query_model(&self.fallback_model, &prompt).await {
                    Ok(response) => Ok(response.trim().to_string()),
                    Err(_) => {
                        // Use pattern-based fallback
                        Ok(self.generate_fallback_command(plan))
                    }
                }
            }
        }
    }

    async fn query_model(&self, model: &str, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/api/generate", self.ollama_host))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama API error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }

    fn generate_fallback_command(&self, plan: &str) -> String {
        let plan_lower = plan.to_lowercase();

        if plan_lower.contains("react") && plan_lower.contains("vite") {
            "npm create vite@latest my-app --template react && cd my-app && npm install && npm run dev".to_string()
        } else if plan_lower.contains("docker") && plan_lower.contains("container") {
            "docker ps -a".to_string()
        } else if plan_lower.contains("git") && plan_lower.contains("repo") {
            "git init && git add . && git commit -m 'Initial commit'".to_string()
        } else if plan_lower.contains("install") && plan_lower.contains("npm") {
            "npm install".to_string()
        } else if plan_lower.contains("backup") && plan_lower.contains("database") {
            "mysqldump -u root -p mydb > backup.sql".to_string()
        } else if plan_lower.contains("test") {
            "npm test".to_string()
        } else if plan_lower.contains("build") {
            "npm run build".to_string()
        } else {
            format!("echo 'Executing: {}'", plan)
        }
    }
}
