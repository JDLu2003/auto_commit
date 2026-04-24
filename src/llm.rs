use anyhow::{anyhow, Context, Result};
use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Debug)]
struct RequestBody {
    stream: bool,
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ResponseBody {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize, Debug)]
struct ResponseMessage {
    content: String,
}

pub async fn call_llm_api(prompt: &str) -> Result<String> {
    let api_key = env::var("DEEPSEEK_API_KEY")
        .context("Error: DEEPSEEK_API_KEY environment variable not set.")?;
    let client = Client::new();

    let request_body = RequestBody {
        stream: false,
        model: "deepseek-v4-flash".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
    };

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to LLM API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await?;
        return Err(anyhow!(
            "LLM API request failed with status code: {}\nResponse: {}",
            status,
            error_body
        ));
    }

    let response_body = response
        .json::<ResponseBody>()
        .await
        .context("Failed to parse JSON response from LLM API")?;

    if let Some(choice) = response_body.choices.get(0) {
        Ok(choice.message.content.clone())
    } else {
        Err(anyhow!(
            "{} {}",
            "Error:".red(),
            "No content received from LLM API."
        ))
    }
}
