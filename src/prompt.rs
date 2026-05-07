use crate::llm::Message;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CommitMessageJson {
    title: String,
    body: String,
}

pub trait PromptStrategy {
    fn build_messages(
        &self,
        diff: &str,
        user_prompt: &Option<String>,
        project_context: &Option<String>,
    ) -> Vec<Message>;

    fn parse_response(&self, response: &str) -> Result<String>;
}

// --- Shared prompt helpers ---

fn context_section(context: &Option<String>) -> Option<String> {
    context
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|c| format!("## 项目背景\n\n{}", c))
}

fn diff_section(diff: &str) -> String {
    format!("## Diff\n\n```\n{}\n```", diff)
}

fn extra_section(extra: &str) -> Option<String> {
    if extra.is_empty() {
        return None;
    }
    Some(format!("## 附加要求\n\n{}", extra))
}

fn assemble_user_content(parts: &[Option<String>]) -> String {
    parts
        .iter()
        .filter_map(|p| p.clone())
        .collect::<Vec<_>>()
        .join("\n\n")
}

// --- Simple Strategy ---

pub struct SimplePrompt;

impl SimplePrompt {
    fn system_content() -> String {
        "你是一个帮助生成 git commit message 的助手。\
         请根据 git diff 生成简洁、符合 Conventional Commits 规范的中文 commit message。\
         输出仅包含 commit message，不要有任何额外说明。"
            .to_string()
    }

    fn instruction() -> String {
        "基于以下信息，生成符合 Conventional Commits 规范的中文 commit message（包括标题和正文），\
         不要有任何额外说明。"
            .to_string()
    }

    fn user_extra(user_prompt: &Option<String>) -> Option<String> {
        user_prompt
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(extra_section)
    }
}

impl PromptStrategy for SimplePrompt {
    fn build_messages(
        &self,
        diff: &str,
        user_prompt: &Option<String>,
        project_context: &Option<String>,
    ) -> Vec<Message> {
        let system_msg = Message {
            role: "system".to_string(),
            content: Self::system_content(),
        };

        let user_content = assemble_user_content(&[
            Some(Self::instruction()),
            context_section(project_context),
            Some(diff_section(diff)),
            Self::user_extra(user_prompt),
        ]);

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

pub struct JsonPrompt;

impl JsonPrompt {
    fn system_content() -> String {
        "你是一个帮助生成 git commit message 的助手。\
         请根据 git diff 生成符合 Conventional Commits 规范的中文 commit message。\
         你必须输出有效的 JSON 格式，包含 \"title\" 和 \"body\" 两个字段。"
            .to_string()
    }

    fn instruction() -> String {
        "基于以下信息，生成符合 Conventional Commits 规范的中文 commit message。\n\
         应当使用中文说明信息，不能使用 markdown 标记，不能使用符号\"`\"除了\"-\"用于分点作答。\n\
         必须使用中文说明信息。\n\n\
         你的输出必须是有效的 JSON 对象，包含两个字段：\n\
         \"title\"（标题行字符串）\n\
         \"body\"（详细描述字符串）\n\
         不要包含 JSON 之外的任何文字或解释。\n\n\
         示例格式：\n\
         ```\n\
         {\n\
           \"title\": \"feat: 实现用户认证功能\",\n\
           \"body\": \"- 添加登录和登出接口\\n- 使用 JWT 进行会话管理\"\n\
         }\n\
         ```"
        .to_string()
    }

    fn user_extra(user_prompt: &Option<String>) -> Option<String> {
        user_prompt
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(extra_section)
    }
}

impl PromptStrategy for JsonPrompt {
    fn build_messages(
        &self,
        diff: &str,
        user_prompt: &Option<String>,
        project_context: &Option<String>,
    ) -> Vec<Message> {
        let system_msg = Message {
            role: "system".to_string(),
            content: Self::system_content(),
        };

        let user_content = assemble_user_content(&[
            Some(Self::instruction()),
            context_section(project_context),
            Some(diff_section(diff)),
            Self::user_extra(user_prompt),
        ]);

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

pub enum PromptMode {
    Simple,
    Json,
}

impl PromptMode {
    pub fn get_strategy(&self) -> Box<dyn PromptStrategy> {
        match self {
            PromptMode::Simple => Box::new(SimplePrompt),
            PromptMode::Json => Box::new(JsonPrompt),
        }
    }
}
