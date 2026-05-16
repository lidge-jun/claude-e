use std::ffi::OsString;
use std::io::{IsTerminal, Read as _};
use std::path::PathBuf;

use crate::config::RunConfig;

const DEFAULT_TIMEOUT_MS: u64 = 600_000;
const DEFAULT_COLS: u16 = 120;
const DEFAULT_ROWS: u16 = 40;

#[derive(Debug, PartialEq)]
pub struct PrintModeOptions {
    pub prompt: String,
    pub output_format: String,
    pub timeout_ms: u64,
    pub claude_bin: String,
    pub cwd: Option<PathBuf>,
    pub cols: u16,
    pub rows: u16,
    pub resume: Option<String>,
    pub auto_accept_workspace_trust: bool,
    pub claude_args: Vec<String>,
}

pub fn parse_print_mode_args(
    raw_args: Vec<OsString>,
    stdin_prompt: Option<String>,
) -> Result<PrintModeOptions, String> {
    let mut output_format = "text".to_string();
    let mut timeout_ms = DEFAULT_TIMEOUT_MS;
    let mut claude_bin = resolve_claude_bin();
    let mut cwd: Option<PathBuf> = None;
    let mut cols = DEFAULT_COLS;
    let mut rows = DEFAULT_ROWS;
    let mut resume: Option<String> = None;
    let mut auto_accept_workspace_trust = true;
    let mut claude_args = Vec::new();
    let mut prompt_parts = Vec::new();

    let args = raw_args
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|_| "non-UTF-8 arguments are not supported in print-compatible mode")
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "p" | "print" if index == 0 => {
                index += 1;
            }
            "-p" | "--print" => {
                index += 1;
            }
            "--output-format" => {
                output_format = take_value(&args, &mut index, "--output-format")?;
            }
            "--timeout-ms" => {
                let raw = take_value(&args, &mut index, "--timeout-ms")?;
                timeout_ms = raw
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --timeout-ms value: {raw}"))?;
            }
            "--claude-bin" => {
                claude_bin = take_value(&args, &mut index, "--claude-bin")?;
            }
            "--cwd" => {
                cwd = Some(PathBuf::from(take_value(&args, &mut index, "--cwd")?));
            }
            "--cols" => {
                let raw = take_value(&args, &mut index, "--cols")?;
                cols = raw
                    .parse::<u16>()
                    .map_err(|_| format!("invalid --cols value: {raw}"))?;
            }
            "--rows" => {
                let raw = take_value(&args, &mut index, "--rows")?;
                rows = raw
                    .parse::<u16>()
                    .map_err(|_| format!("invalid --rows value: {raw}"))?;
            }
            "--resume" => {
                resume = Some(take_value(&args, &mut index, "--resume")?);
            }
            "--auto-accept-workspace-trust" => {
                auto_accept_workspace_trust = true;
                index += 1;
            }
            "--no-auto-accept-workspace-trust" => {
                auto_accept_workspace_trust = false;
                index += 1;
            }
            "--verbose" => {
                // Claude -p requires this for stream-json partials. The PTY path owns
                // streaming from the transcript, so accepting and swallowing it keeps
                // the print-mode surface compatible without confusing interactive Claude.
                index += 1;
            }
            "--" => {
                prompt_parts.extend(args[index + 1..].iter().cloned());
                break;
            }
            _ if arg.starts_with('-') => {
                claude_args.push(arg.clone());
                index += 1;
                if claude_flag_consumes_value(arg) {
                    let value = args
                        .get(index)
                        .ok_or_else(|| format!("missing value for {arg}"))?
                        .clone();
                    claude_args.push(value);
                    index += 1;
                }
            }
            _ => {
                prompt_parts.push(arg.clone());
                index += 1;
            }
        }
    }

    let mut prompt = prompt_parts.join(" ");
    if let Some(stdin) = stdin_prompt
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        if prompt.trim().is_empty() {
            prompt = stdin;
        } else {
            prompt.push_str("\n\n");
            prompt.push_str(&stdin);
        }
    }

    if prompt.trim().is_empty() {
        return Err("prompt is empty".to_string());
    }

    Ok(PrintModeOptions {
        prompt: prompt.trim().to_string(),
        output_format,
        timeout_ms,
        claude_bin,
        cwd,
        cols,
        rows,
        resume,
        auto_accept_workspace_trust,
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
    RunConfig::new(
        options.claude_bin,
        options.cwd,
        options.cols,
        options.rows,
        options.timeout_ms,
        options.output_format,
        options.resume,
        options.auto_accept_workspace_trust,
        options.claude_args,
        false,
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

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    let value_index = *index + 1;
    let value = args
        .get(value_index)
        .ok_or_else(|| format!("missing value for {flag}"))?
        .clone();
    *index = value_index + 1;
    Ok(value)
}

fn claude_flag_consumes_value(flag: &str) -> bool {
    matches!(
        flag,
        "--model"
            | "--fallback-model"
            | "--permission-mode"
            | "--append-system-prompt"
            | "--add-dir"
            | "--mcp-config"
            | "--settings"
            | "--session-id"
            | "--max-turns"
            | "--allowedTools"
            | "--allowed-tools"
            | "--disallowedTools"
            | "--disallowed-tools"
            | "--permission-prompt-tool"
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
