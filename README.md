# Auto Commit

`auto-commit` is a CLI tool that uses a large language model to automatically generate commit messages for your staged changes. It helps you create well-formatted, conventional commit messages with ease.

## Features

-   **AI-Powered Messages**: Analyzes your staged diff and generates a commit message (title and body).
-   **Interactive Mode**: Preview, edit, or regenerate the message before committing.
-   **Customizable**: Use command-line options to add prompts, choose your editor, or skip interaction entirely.
-   **Conventional Commits**: Generates messages following the Conventional Commits specification.

## Installation

1.  **Prerequisites**:
    *   Rust toolchain (install from [rust-lang.org](https://www.rust-lang.org/tools/install))
    *   Python 3
    *   An API key for the language model (set as `QWENKEY` environment variable)

2.  **Build from source**:
    ```bash
    git clone https://github.com/your-username/auto-commit.git
    cd auto-commit
    cargo build --release
    ```
    The executable will be located at `target/release/auto_aommit`.

## Usage

1.  Stage your changes:
    ```bash
    git add <your-files>
    ```

2.  Run the tool:
    ```bash
    ./target/release/auto_aommit
    ```

3.  Follow the interactive prompts to edit, regenerate, or confirm the commit message.

### Command-Line Options

| Flag                | Description                                  |
| ------------------- | -------------------------------------------- |
| `--editor <EDITOR>` | Override the default `$EDITOR`.              |
| `--prompt "<TEXT>"` | Add extra instructions for the AI.           |
| `--dry-run`         | Generate the message without committing.     |
| `--yes`             | Skip the interactive menu and commit directly. |
| `--debug`           | Show debug information, like the AI prompt.  |

### Example

```bash
# Stage files
git add src/main.rs

# Run the tool
./target/release/auto_aommit

# Output:
# 正在分析已暂存的变更...
# 已调用大语言模型生成初稿：
# --------------------------------------------------
# feat: Implement interactive menu for user actions
#
# Adds a menu allowing the user to edit, regenerate,
# commit, or cancel the operation.
# --------------------------------------------------
# 请选择下一步操作：
# [1] 编辑 commit message
# [2] 重新生成
# [3] 确认并提交
# [4] 取消并退出
# > 3
# ✅ Commit 已提交: a1b2c3d
```
