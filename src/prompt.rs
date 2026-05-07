use anyhow::{Context, Result};
use serde::Deserialize;

/// Represents the expected JSON structure for a commit message from the LLM.
#[derive(Deserialize, Debug)]
struct CommitMessageJson {
    title: String,
    body: String,
}

/// A trait defining the interface for different prompt generation and parsing strategies.
pub trait PromptStrategy {
    /// Builds the full prompt to be sent to the LLM.
    fn build_prompt(&self, diff: &str, user_prompt: &Option<String>) -> String;

    /// Parses the raw response from the LLM into a formatted commit message string.
    fn parse_response(&self, response: &str) -> Result<String>;
}

// --- Simple Strategy ---

/// The simple strategy: plain text prompt, direct response.
pub struct SimplePrompt;

impl PromptStrategy for SimplePrompt {
    fn build_prompt(&self, diff: &str, user_prompt: &Option<String>) -> String {
        let mut full_prompt = format!(
            "Based on the following git diff, generate a concise and descriptive commit message in Chinese, following the Conventional Commits specification. The output should be only the commit message (title and body), without any extra text or explanation.\n\nDiff:\n```\n{}\n```",
            diff
        );

        if let Some(p) = user_prompt.as_deref().filter(|s| !s.is_empty()) {
            full_prompt.push_str(&format!("\n\nAdditional instructions: {}\n", p));
        }
        full_prompt
    }

    fn parse_response(&self, response: &str) -> Result<String> {
        Ok(response.trim().to_string())
    }
}

// --- JSON Strategy ---

/// The JSON strategy: asks for a JSON object and parses it.
pub struct JsonPrompt;

impl PromptStrategy for JsonPrompt {
    fn build_prompt(&self, diff: &str, user_prompt: &Option<String>) -> String {
        let mut full_prompt = format!(
            "Based on the following git diff, generate a commit message in Chinese, 应当使用中文说明信息,不能使用 markdown 标记，不能使用符号\"`\"除了\"-\"用于分点作答
                应当使用中文说明信息应当使用中文说明信息应当使用中文说明信息应当使用中文说明信息
                following the Conventional Commits specification. 
                Your output MUST be a valid JSON object with two keys: 
                \"title\" (a string for the subject line) 
                and \"body\" (a string for the detailed description). 
                Do not include any other text or explanations outside of the JSON object.
                \n\nExample format:\n{{\n  \"title\": \"feat: Implement user authentication\",\n  \"body\": \"- Add login and logout endpoints.\\n- Use JWT for session management.\"\n}}\n\nDiff:\n```\n{}\n```",
            diff
        );

        if let Some(p) = user_prompt.as_deref().filter(|s| !s.is_empty()) {
            full_prompt.push_str(&format!(
                "\n\nAdditional instructions for the content: {}\n",
                p
            ));
        }
        full_prompt
    }

    fn parse_response(&self, response: &str) -> Result<String> {
        let parsed_json: CommitMessageJson = serde_json::from_str(response).with_context(|| {
            format!(
                "Failed to parse LLM response as JSON. Response: '{}'",
                response
            )
        })?;
        Ok(format!("{}\n\n{}", parsed_json.title, parsed_json.body))
    }
}

/// Enum to select the desired prompt mode.
pub enum PromptMode {
    Simple,
    Json,
}

impl PromptMode {
    /// Returns a boxed trait object for the selected strategy.
    pub fn get_strategy(&self) -> Box<dyn PromptStrategy> {
        match self {
            PromptMode::Simple => Box::new(SimplePrompt),
            PromptMode::Json => Box::new(JsonPrompt),
        }
    }
}
