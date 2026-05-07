use crate::llm::Message;

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
    fn build_messages(&self, diff: &str, user_prompt: &Option<String>) -> Vec<Message>;

    fn parse_response(&self, response: &str) -> Result<String>;
}

// --- Simple Strategy ---

/// The simple strategy: plain text prompt, direct response.
pub struct SimplePrompt;

impl PromptStrategy for SimplePrompt {
    fn build_messages(&self, diff: &str, user_prompt: &Option<String>) -> Vec<Message> {
        let system_msg = Message {
            role: "system".to_string(),
            content: "你是一个帮助生成 git commit message 的助手。请根据 git diff 生成简洁、符合 Conventional Commits 规范的中文 commit message。输出仅包含 commit message，不要有任何额外说明。".to_string(),
        };

        let mut user_content = format!(
            "基于以下 git diff，生成符合 Conventional Commits 规范的中文 commit message（包括标题和正文），不要有任何额外说明。\n\nDiff:\n```\n{}\n```",
            diff
        );

        if let Some(p) = user_prompt.as_deref().filter(|s| !s.is_empty()) {
            user_content.push_str(&format!("\n\n附加要求: {}", p));
        }

        let user_msg = Message {
            role: "user".to_string(),
            content: user_content,
        };

        vec![system_msg, user_msg]
    }

    fn parse_response(&self, response: &str) -> Result<String> {
        Ok(response.trim().to_string())
    }
}

// --- JSON Strategy ---

/// The JSON strategy: asks for a JSON object and parses it.
pub struct JsonPrompt;

impl PromptStrategy for JsonPrompt {
    fn build_messages(&self, diff: &str, user_prompt: &Option<String>) -> Vec<Message> {
        let system_msg = Message {
            role: "system".to_string(),
            content: "你是一个帮助生成 git commit message 的助手。请根据 git diff 生成符合 Conventional Commits 规范的中文 commit message。你必须输出有效的 JSON 格式，包含 \"title\" 和 \"body\" 两个字段。".to_string(),
        };

        let mut user_content = format!(
            "基于以下 git diff，生成符合 Conventional Commits 规范的中文 commit message。\n应当使用中文说明信息，不能使用 markdown 标记，不能使用符号\"`\"除了\"-\"用于分点作答。\n必须使用中文说明信息。\n\n你的输出必须是有效的 JSON 对象，包含两个字段：\n\"title\"（标题行字符串）\n\"body\"（详细描述字符串）\n不要包含 JSON 之外的任何文字或解释。\n\n示例格式：\n{{\n  \"title\": \"feat: 实现用户认证功能\",\n  \"body\": \"- 添加登录和登出接口\\n- 使用 JWT 进行会话管理\"\n}}\n\nDiff:\n```\n{}\n```",
            diff
        );

        if let Some(p) = user_prompt.as_deref().filter(|s| !s.is_empty()) {
            user_content.push_str(&format!(
                "\n\n内容附加要求: {}",
                p
            ));
        }

        let user_msg = Message {
            role: "user".to_string(),
            content: user_content,
        };

        vec![system_msg, user_msg]
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
