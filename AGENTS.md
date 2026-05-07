# AGENTS.md

## Build, Install & Run

```bash
cargo build --release          # binary: target/release/auto_aommit
cargo install --path .         # install to ~/.cargo/bin/acommit
cargo run -- --help            # debug run
```

No test suite exists. No CI config.

## Environment

- `DEEPSEEK_API_KEY` must be set; the Rust binary calls the DeepSeek API directly at runtime (no Python dependency despite README claiming one).

## Architecture

- `src/main.rs` — CLI entry: arg parsing, interactive menu, git commit via `git commit -F`
- `src/llm.rs` — DeepSeek API client (model: `deepseek-v4-flash`, endpoint: `https://api.deepseek.com/chat/completions`)
- `src/prompt.rs` — Two prompt strategies: `Simple` (plain text) and `Json` (structured `{title, body}`), selected via `--json` flag

`ask.py` is a standalone script not used by the Rust binary. It references `deepseek-v4-pro` and Ollama — separate from main app logic.

## Key Gotchas

- Binary name in `Cargo.toml` is `acommit`; the built binary is `auto_aommit`
- All user-facing UI strings are in Chinese
- `git2` crate is used for repo checks and staged-diff extraction (not `git diff` CLI)
- Committing uses `Command::new("git")` subprocess, not `git2`
- `.gitignore` excludes `Cargo.lock` and the `acommit` binary

## Conventions

- Error handling: `anyhow` throughout
- CLI args: `clap` derive mode
- Async: `tokio` runtime (`#[tokio::main]`)