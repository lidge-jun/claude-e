use std::ffi::OsString;
use std::io::{IsTerminal, Read as _};
use std::path::PathBuf;

use serde_json::Value;

use crate::config::RunConfig;

const DEFAULT_TIMEOUT_MS: u64 = 600_000;
const DEFAULT_COLS: u16 = 120;
const DEFAULT_ROWS: u16 = 40;

#[derive(Debug, PartialEq)]
pub struct PrintModeOptions {
    pub prompt: String,
    pub output_format: String,
    pub timeout_ms: u64,
    pub idle_timeout_ms: u64,
    pub hard_timeout_ms: u64,
    pub claude_bin: String,
    pub cwd: Option<PathBuf>,
    pub cols: u16,
    pub rows: u16,
    pub resume: Option<String>,
    pub session_id: Option<String>,
    pub no_session_persistence: bool,
    pub auto_accept_workspace_trust: bool,
    pub terminal_tools: bool,
    pub show_session_footer: bool,
    pub claude_args: Vec<String>,
}

pub fn parse_print_mode_args(
    raw_args: Vec<OsString>,
    stdin_prompt: Option<String>,
) -> Result<PrintModeOptions, String> {
    let mut input_format = "text".to_string();
    let mut output_format = "text".to_string();
    let mut timeout_ms = DEFAULT_TIMEOUT_MS;
    let mut idle_timeout_ms: Option<u64> = None;
    let mut hard_timeout_ms: u64 = 3_600_000;
    let mut claude_bin = resolve_claude_bin();
    let mut cwd: Option<PathBuf> = None;
    let mut cols = DEFAULT_COLS;
    let mut rows = DEFAULT_ROWS;
    let mut resume: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut no_session_persistence = false;
    let mut auto_accept_workspace_trust = true;
    let mut terminal_tools = false;
    let mut show_session_footer = true;
    let mut claude_args = Vec::new();
    let mut prompt_parts = Vec::new();
    let mut json_schema: Option<String> = None;

    let args = raw_args
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|_| "non-UTF-8 arguments are not supported in print-compatible mode")
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut index = 0;
    while index < args.len() {
        let raw = args[index].clone();
        let (flag, inline_value) = split_inline_option(&raw);

        match flag.as_str() {
            "p" | "print" if index == 0 => {
                index += 1;
            }
            "-p" | "--print" => {
                index += 1;
            }
            "--input-format" => {
                input_format = take_value(&args, &mut index, "--input-format", inline_value)?;
                validate_choice(&input_format, &["text", "stream-json"], "--input-format")?;
            }
            "--output-format" => {
                output_format = take_value(&args, &mut index, "--output-format", inline_value)?;
                validate_choice(
                    &output_format,
                    &["text", "json", "stream-json"],
                    "--output-format",
                )?;
            }
            "--timeout-ms" => {
                let raw = take_value(&args, &mut index, "--timeout-ms", inline_value)?;
                timeout_ms = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --timeout-ms value: {raw}"))?;
            }
            "--idle-timeout-ms" => {
                let raw = take_value(&args, &mut index, "--idle-timeout-ms", inline_value)?;
                idle_timeout_ms = Some(
                    raw.parse::<u64>()
                        .map_err(|_| format!("invalid --idle-timeout-ms value: {raw}"))?,
                );
            }
            "--hard-timeout-ms" => {
                let raw = take_value(&args, &mut index, "--hard-timeout-ms", inline_value)?;
                hard_timeout_ms = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --hard-timeout-ms value: {raw}"))?;
            }
            "--claude-bin" => {
                claude_bin = take_value(&args, &mut index, "--claude-bin", inline_value)?;
            }
            "--cwd" => {
                cwd = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--cwd",
                    inline_value,
                )?));
            }
            "--cols" => {
                let raw = take_value(&args, &mut index, "--cols", inline_value)?;
                cols = raw
                    .parse::<u16>()
                    .map_err(|_| format!("invalid --cols value: {raw}"))?;
            }
            "--rows" => {
                let raw = take_value(&args, &mut index, "--rows", inline_value)?;
                rows = raw
                    .parse::<u16>()
                    .map_err(|_| format!("invalid --rows value: {raw}"))?;
            }
            "-r" | "--resume" => {
                resume = take_optional_value(&args, &mut index, &flag, inline_value);
                if resume.is_none() {
                    return Err(format!(
                        "{flag} requires a session id in PTY-backed print mode"
                    ));
                }
            }
            "--session-id" => {
                session_id = Some(take_value(&args, &mut index, "--session-id", inline_value)?);
            }
            "--no-session-persistence" => {
                no_session_persistence = true;
                index += 1;
            }
            "--auto-accept-workspace-trust" => {
                auto_accept_workspace_trust = true;
                index += 1;
            }
            "--no-auto-accept-workspace-trust" => {
                auto_accept_workspace_trust = false;
                index += 1;
            }
            "-t" | "--t" | "--tool" | "--tool-events" | "--show-tools" => {
                terminal_tools = true;
                index += 1;
            }
            "--no-session-footer" => {
                show_session_footer = false;
                index += 1;
            }
            "--include-partial-messages" | "--include-hook-events" | "--verbose" => {
                // These are accepted for claude -p command-shape compatibility.
                // Transcript replay owns output timing in the PTY-backed path.
                index += 1;
            }
            "--replay-user-messages" => {
                // Accepted for stream-json input compatibility; the PTY path has no
                // separate input-ack channel, so this is intentionally a no-op.
                index += 1;
            }
            "--json-schema" => {
                json_schema = Some(take_value(
                    &args,
                    &mut index,
                    "--json-schema",
                    inline_value,
                )?);
            }
            "--fallback-model" | "--max-budget-usd" => {
                // Print-only Claude flags. Accept and consume them so the command
                // surface is stable; the interactive PTY path cannot enforce them.
                let _ = take_value(&args, &mut index, &flag, inline_value)?;
            }
            "--" => {
                prompt_parts.extend(args[index + 1..].iter().cloned());
                break;
            }
            _ if is_forward_bool_flag(&flag) => {
                claude_args.push(flag);
                index += 1;
            }
            _ if is_forward_single_value_flag(&flag) => {
                let value = take_value(&args, &mut index, &flag, inline_value)?;
                claude_args.push(flag);
                claude_args.push(value);
            }
            _ if is_forward_optional_value_flag(&flag) => {
                claude_args.push(flag.clone());
                if let Some(value) = take_inline_optional_value(&mut index, inline_value) {
                    claude_args.push(value);
                }
            }
            _ if is_forward_variadic_flag(&flag) => {
                let value = take_value(&args, &mut index, &flag, inline_value)?;
                claude_args.push(flag);
                claude_args.push(value);
            }
            _ if flag.starts_with('-') => {
                // Unknown options are treated as boolean Claude flags instead of
                // becoming prompt text. This is safer for forward compatibility.
                claude_args.push(raw);
                index += 1;
            }
            _ => {
                prompt_parts.push(raw);
                index += 1;
            }
        }
    }

    let stdin_prompt = normalize_stdin_prompt(&input_format, stdin_prompt)?;
    let mut prompt = prompt_parts.join(" ");
    if let Some(stdin) = stdin_prompt {
        if prompt.trim().is_empty() {
            prompt = stdin;
        } else {
            prompt.push_str("\n\n");
            prompt.push_str(&stdin);
        }
    }
    if let Some(schema) = json_schema {
        append_json_schema_instruction(&mut prompt, &schema);
    }

    if prompt.trim().is_empty() {
        return Err("prompt is empty".to_string());
    }

    let effective_idle = idle_timeout_ms.unwrap_or(timeout_ms);

    Ok(PrintModeOptions {
        prompt: prompt.trim().to_string(),
        output_format,
        timeout_ms,
        idle_timeout_ms: effective_idle,
        hard_timeout_ms,
        claude_bin,
        cwd,
        cols,
        rows,
        resume,
        session_id,
        no_session_persistence,
        auto_accept_workspace_trust,
        terminal_tools,
        show_session_footer,
        claude_args,
    })
}

pub fn read_stdin_if_piped() -> Result<Option<String>, String> {
    if std::io::stdin().is_terminal() {
        return Ok(None);
    }

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| format!("stdin read failed: {e}"))?;
    Ok(Some(input))
}

pub fn config_from_options(options: PrintModeOptions) -> RunConfig {
    RunConfig::new_with_timeouts(
        options.claude_bin,
        options.cwd,
        options.cols,
        options.rows,
        options.idle_timeout_ms,
        options.hard_timeout_ms,
        options.output_format,
        options.resume,
        options.session_id,
        options.no_session_persistence,
        options.auto_accept_workspace_trust,
        options.claude_args,
        false,
        options.terminal_tools,
        options.show_session_footer,
    )
}

fn resolve_claude_bin() -> String {
    std::env::var("CLAUDE_EXEC_CLAUDE_BIN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("CLAUDE_BIN")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "claude".to_string())
}

fn split_inline_option(raw: &str) -> (String, Option<String>) {
    if !raw.starts_with("--") {
        return (raw.to_string(), None);
    }

    raw.split_once('=')
        .map(|(flag, value)| (flag.to_string(), Some(value.to_string())))
        .unwrap_or_else(|| (raw.to_string(), None))
}

fn take_value(
    args: &[String],
    index: &mut usize,
    flag: &str,
    inline_value: Option<String>,
) -> Result<String, String> {
    if let Some(value) = inline_value {
        *index += 1;
        return Ok(value);
    }

    let value_index = *index + 1;
    let value = args
        .get(value_index)
        .ok_or_else(|| format!("missing value for {flag}"))?
        .clone();
    *index = value_index + 1;
    Ok(value)
}

fn take_optional_value(
    args: &[String],
    index: &mut usize,
    _flag: &str,
    inline_value: Option<String>,
) -> Option<String> {
    if let Some(value) = inline_value {
        *index += 1;
        return Some(value);
    }

    let value_index = *index + 1;
    let Some(value) = args.get(value_index) else {
        *index += 1;
        return None;
    };
    if looks_like_option(value) {
        *index += 1;
        None
    } else {
        *index = value_index + 1;
        Some(value.clone())
    }
}

fn take_inline_optional_value(index: &mut usize, inline_value: Option<String>) -> Option<String> {
    *index += 1;
    inline_value
}

fn looks_like_option(value: &str) -> bool {
    value.starts_with('-') && value != "-"
}

fn validate_choice(value: &str, choices: &[&str], flag: &str) -> Result<(), String> {
    if choices.contains(&value) {
        return Ok(());
    }
    Err(format!(
        "invalid {flag} value: {value} (expected one of {})",
        choices.join(", ")
    ))
}

fn normalize_stdin_prompt(
    input_format: &str,
    stdin_prompt: Option<String>,
) -> Result<Option<String>, String> {
    let Some(stdin) = stdin_prompt.map(|value| value.trim().to_string()) else {
        return Ok(None);
    };
    if stdin.is_empty() {
        return Ok(None);
    }
    if input_format == "text" {
        return Ok(Some(stdin));
    }

    let mut messages = Vec::new();
    for (line_index, line) in stdin.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(trimmed).map_err(|e| {
            format!(
                "invalid --input-format stream-json line {}: {e}",
                line_index + 1
            )
        })?;
        if let Some(text) = extract_user_text(&value) {
            messages.push(text);
        }
    }

    Ok((!messages.is_empty()).then(|| messages.join("\n\n")))
}

fn extract_user_text(value: &Value) -> Option<String> {
    let record_type = value.get("type").and_then(|v| v.as_str());
    let message = value.get("message").or_else(|| value.get("delta"))?;
    let role = message.get("role").and_then(|v| v.as_str());
    if record_type != Some("user") && role != Some("user") {
        return None;
    }

    extract_content_text(message.get("content")?)
}

fn extract_content_text(content: &Value) -> Option<String> {
    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }

    let blocks = content.as_array()?;
    let mut text = String::new();
    for block in blocks {
        if block.get("type").and_then(|v| v.as_str()) == Some("text") {
            text.push_str(block.get("text").and_then(|v| v.as_str()).unwrap_or(""));
        }
    }
    (!text.is_empty()).then_some(text)
}

fn append_json_schema_instruction(prompt: &mut String, schema: &str) {
    if !prompt.trim().is_empty() {
        prompt.push_str("\n\n");
    }
    prompt.push_str("Return only JSON that validates against this JSON Schema:\n");
    prompt.push_str(schema);
}

fn is_forward_bool_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--allow-dangerously-skip-permissions"
            | "--bare"
            | "--brief"
            | "--chrome"
            | "--continue"
            | "-c"
            | "--dangerously-skip-permissions"
            | "--disable-slash-commands"
            | "--exclude-dynamic-system-prompt-sections"
            | "--fork-session"
            | "--ide"
            | "--mcp-debug"
            | "--no-chrome"
            | "--strict-mcp-config"
    )
}

fn is_forward_single_value_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--agent"
            | "--agents"
            | "--append-system-prompt"
            | "--append-system-prompt-file"
            | "--debug-file"
            | "--effort"
            | "--mcp-config"
            | "--model"
            | "--name"
            | "-n"
            | "--permission-mode"
            | "--plugin-dir"
            | "--plugin-url"
            | "--remote-control-session-name-prefix"
            | "--setting-sources"
            | "--settings"
            | "--system-prompt"
            | "--system-prompt-file"
    )
}

fn is_forward_optional_value_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--debug" | "-d" | "--from-pr" | "--remote-control" | "--tmux" | "--worktree" | "-w"
    )
}

fn is_forward_variadic_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--add-dir"
            | "--allowedTools"
            | "--allowed-tools"
            | "--betas"
            | "--disallowedTools"
            | "--disallowed-tools"
            | "--file"
            | "--tools"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_json_output_and_prompt_without_forwarding_output_format() {
        let options = parse_print_mode_args(
            os_args(&["--output-format", "json", "summarize this commit"]),
            None,
        )
        .expect("parse print mode");

        assert_eq!(options.output_format, "json");
        assert_eq!(options.prompt, "summarize this commit");
        assert!(options.claude_args.is_empty());
    }

    #[test]
    fn forwards_model_but_keeps_final_positional_as_prompt() {
        let options =
            parse_print_mode_args(os_args(&["--model", "opus", "explain quicksort"]), None)
                .expect("parse print mode");

        assert_eq!(options.claude_args, vec!["--model", "opus"]);
        assert_eq!(options.prompt, "explain quicksort");
        assert_eq!(options.output_format, "text");
    }

    #[test]
    fn accepts_verbose_stream_json_without_forwarding_verbose() {
        let options = parse_print_mode_args(
            os_args(&["--output-format", "stream-json", "audit src/", "--verbose"]),
            None,
        )
        .expect("parse print mode");

        assert_eq!(options.output_format, "stream-json");
        assert_eq!(options.prompt, "audit src/");
        assert!(options.claude_args.is_empty());
    }

    #[test]
    fn accepts_leading_p_alias_without_including_it_in_prompt() {
        let options = parse_print_mode_args(os_args(&["p", "--model", "opus", "hello"]), None)
            .expect("parse print mode");

        assert_eq!(options.prompt, "hello");
        assert_eq!(options.claude_args, vec!["--model", "opus"]);
    }

    #[test]
    fn supports_equals_value_options() {
        let options = parse_print_mode_args(
            os_args(&["--output-format=json", "--model=opus", "hello"]),
            None,
        )
        .expect("parse print mode");

        assert_eq!(options.output_format, "json");
        assert_eq!(options.claude_args, vec!["--model", "opus"]);
        assert_eq!(options.prompt, "hello");
    }

    #[test]
    fn forwards_help_surface_flags_with_values() {
        let options = parse_print_mode_args(
            os_args(&[
                "--add-dir",
                "/tmp/repo",
                "--allowed-tools",
                "Bash(git *)",
                "--permission-mode",
                "auto",
                "audit",
            ]),
            None,
        )
        .expect("parse print mode");

        assert_eq!(
            options.claude_args,
            vec![
                "--add-dir",
                "/tmp/repo",
                "--allowed-tools",
                "Bash(git *)",
                "--permission-mode",
                "auto"
            ]
        );
        assert_eq!(options.prompt, "audit");
    }

    #[test]
    fn maps_session_flags_to_wrapper_config() {
        let options = parse_print_mode_args(
            os_args(&[
                "--session-id",
                "3b241101-e2bb-4255-8caf-4136c566a962",
                "--no-session-persistence",
                "hello",
            ]),
            None,
        )
        .expect("parse print mode");
        let config = config_from_options(options);

        assert!(config.no_session_persistence);
        assert!(config.session_id.is_empty());
        assert!(config.extra_args.is_empty());
    }

    #[test]
    fn maps_tool_progress_and_session_footer_flags() {
        let options =
            parse_print_mode_args(os_args(&["--tool", "--no-session-footer", "inspect"]), None)
                .expect("parse print mode");
        let config = config_from_options(options);

        assert!(config.terminal_tools);
        assert!(!config.show_session_footer);
        assert_eq!(config.extra_args, Vec::<String>::new());
    }

    #[test]
    fn maps_t_alias_to_tool_progress() {
        let options =
            parse_print_mode_args(os_args(&["--t", "inspect"]), None).expect("parse print mode");

        assert!(options.terminal_tools);
        assert!(options.show_session_footer);
        assert_eq!(options.prompt, "inspect");
    }

    #[test]
    fn accepts_optional_from_pr_without_stealing_prompt() {
        let options = parse_print_mode_args(os_args(&["--from-pr", "summarize"]), None)
            .expect("parse print mode");

        assert_eq!(options.claude_args, vec!["--from-pr"]);
        assert_eq!(options.prompt, "summarize");
    }

    #[test]
    fn preserves_unknown_inline_options_for_forward_compatibility() {
        let options = parse_print_mode_args(os_args(&["--future-flag=value", "hello"]), None)
            .expect("parse print mode");

        assert_eq!(options.claude_args, vec!["--future-flag=value"]);
        assert_eq!(options.prompt, "hello");
    }

    #[test]
    fn parses_stream_json_input_into_prompt() {
        let stdin = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"hello"}]}}"#;
        let options = parse_print_mode_args(
            os_args(&["--input-format", "stream-json"]),
            Some(stdin.to_string()),
        )
        .expect("parse print mode");

        assert_eq!(options.prompt, "hello");
    }

    #[test]
    fn appends_json_schema_instruction() {
        let options = parse_print_mode_args(
            os_args(&["--json-schema", r#"{"type":"object"}"#, "summarize"]),
            None,
        )
        .expect("parse print mode");

        assert!(options.prompt.contains("summarize"));
        assert!(options.prompt.contains("Return only JSON"));
        assert!(options.prompt.contains(r#"{"type":"object"}"#));
    }

    #[test]
    fn combines_prompt_argument_and_piped_stdin() {
        let options = parse_print_mode_args(
            os_args(&["summarize this commit"]),
            Some("diff --git a/file b/file\n".to_string()),
        )
        .expect("parse print mode");

        assert_eq!(
            options.prompt,
            "summarize this commit\n\ndiff --git a/file b/file"
        );
    }

    #[test]
    fn builds_non_runtime_config_for_pty_backed_print_mode() {
        let options = parse_print_mode_args(
            os_args(&["--claude-bin", "/tmp/claude", "--model", "opus", "hello"]),
            None,
        )
        .expect("parse print mode");
        let config = config_from_options(options);

        assert_eq!(config.claude_bin, "/tmp/claude");
        assert_eq!(config.extra_args, vec!["--model", "opus"]);
        assert!(!config.emit_runtime_events);
        assert!(config.auto_accept_trust);
    }
}
