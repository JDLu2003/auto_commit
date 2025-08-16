use anyhow::{anyhow, Context, Result};
use atty::Stream;
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use git2::Repository;
use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

mod llm;
mod prompt;

/// A CLI tool to generate commit messages using AI.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Override the default editor
    #[arg(long)]
    editor: Option<String>,

    /// Add a prompt to the AI
    #[arg(long)]
    prompt: Option<String>,

    /// Dry run, generate message but do not commit
    #[arg(long)]
    dry_run: bool,

    /// Skip interactive menu and commit directly
    #[arg(long)]
    yes: bool,

    /// Show debug information
    #[arg(long)]
    debug: bool,

    /// Use JSON mode for structured LLM output
    #[arg(long)]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let prompt_mode = if args.json {
        prompt::PromptMode::Json
    } else {
        prompt::PromptMode::Simple
    };

    let repo = check_git_repository()?;
    let diff = get_staged_diff(&repo)?;

    if args.debug {
        println!("[DEBUG] Staged diff:\n{}", diff);
    }

    println!("{}", "正在分析已暂存的变更...".yellow());

    let mut current_prompt = args.prompt.clone();
    let mut commit_msg =
        generate_commit_message(&diff, &prompt_mode, &current_prompt, args.debug).await?;

    if args.yes || !atty::is(Stream::Stdin) {
        println!("{}", "已生成 Commit Message:".green());
        println!("{}", commit_msg);
        if !atty::is(Stream::Stdin) {
            println!("{}", "非交互式环境，直接输出结果。".yellow());
        }
        return commit(&repo, &commit_msg, args.dry_run);
    }

    loop {
        println!("{}", "已调用大语言模型生成初稿：".green());
        println!("--------------------------------------------------");
        println!("{}", commit_msg);
        println!("--------------------------------------------------");

        let items = &[
            "编辑 commit message",
            "重新生成",
            "确认并提交",
            "取消并退出",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择下一步操作：")
            .items(items)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                commit_msg = edit_message(&commit_msg, &args.editor)?;
            }
            1 => {
                println!("输入附加生成要求（直接回车跳过）：");
                let mut new_prompt = String::new();
                std::io::stdin().read_line(&mut new_prompt)?;
                current_prompt = Some(new_prompt.trim().to_string());
                println!("{}", "正在重新生成...".yellow());
                commit_msg =
                    generate_commit_message(&diff, &prompt_mode, &current_prompt, args.debug)
                        .await?;
            }
            2 => {
                return commit(&repo, &commit_msg, args.dry_run);
            }
            3 => {
                println!("{}", "❌ 操作已取消，未提交任何变更".red());
                return Ok(())
            }
            _ => unreachable!(),
        }
    }
}

fn commit(_repo: &Repository, message: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "{}",
            "Dry run: Commit message generated but not committed.".yellow()
        );
        return Ok(())
    }

    let mut file = NamedTempFile::new()?;
    file.write_all(message.as_bytes())?;
    let path = file.path();

    let output = Command::new("git")
        .arg("commit")
        .arg("-F")
        .arg(path)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // A more robust way to get the commit hash
        if let Some(line) = stdout.lines().next() {
            if let Some(hash) = line
                .split(|c| c == '[' || c == ']')
                .nth(1)
                .and_then(|s| s.split_whitespace().last())
            {
                println!("✅ Commit 已提交: {}", hash.green());
                return Ok(())
            }
        }
        // Fallback for unexpected output
        println!("✅ Commit 已提交");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{} git commit 执行失败", "Error:".red());
        eprintln!("{}", stderr);
    }
    Ok(())
}

fn edit_message(message: &str, editor: &Option<String>) -> Result<String> {
    let editor_from_env = std::env::var("EDITOR").ok();
    let editor_cmd = editor
        .as_deref()
        .or(editor_from_env.as_deref())
        .unwrap_or("vim");

    let mut file = NamedTempFile::new()?;
    file.write_all(message.as_bytes())?;
    let path = file.path();

    let status = Command::new(editor_cmd).arg(path).status()?;

    if !status.success() {
        return Err(anyhow!("Editor exited with a non-zero status."));
    }

    let mut new_message = String::new();
    // Re-open the file for reading
    let mut file = std::fs::File::open(path)?;
    file.read_to_string(&mut new_message)?;

    Ok(new_message.trim().to_string())
}

/// Calls the LLM API to generate a commit message using a selected strategy.
async fn generate_commit_message(
    diff: &str,
    prompt_mode: &prompt::PromptMode,
    user_prompt: &Option<String>,
    debug: bool,
) -> Result<String> {
    let strategy = prompt_mode.get_strategy();
    let full_prompt = strategy.build_prompt(diff, user_prompt);

    if debug {
        println!("[DEBUG] LLM input:\n{}", full_prompt);
    }

    let llm_response = llm::call_llm_api(&full_prompt).await?;

    if debug {
        println!("[DEBUG] LLM output:\n{}", llm_response);
    }

    strategy.parse_response(&llm_response)
}

/// Check if the current directory is a Git repository.
fn check_git_repository() -> Result<Repository> {
    Repository::open(".").context("Error: Current directory is not a Git repository.")
}

/// Get the staged diff.
fn get_staged_diff(repo: &Repository) -> Result<String> {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
        Err(e) => return Err(e.into()),
    };

    let head_tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

    let diff = repo.diff_tree_to_index(head_tree.as_ref(), None, None)?;

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        let content = String::from_utf8_lossy(line.content());
        diff_text.push_str(&content);
        true
    })?;

    if diff_text.is_empty() {
        Err(anyhow!(
            "{} {}",
            "Error:".red(),
            "No staged changes found. Please run `git add` first."
        ))
    } else {
        Ok(diff_text)
    }
}
