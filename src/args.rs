use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "claude-e",
    version,
    about = "PTY-backed exec wrapper for Claude Code"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Run a single-turn interactive Claude session
    #[command(visible_alias = "exec")]
    Run {
        /// Emit JSONL to stdout
        #[arg(long, default_value_t = true)]
        jsonl: bool,

        /// Output format: stream-json, json, or text
        #[arg(long, default_value = "stream-json")]
        output_format: String,

        /// Idle timeout in milliseconds — resets on transcript activity (default: 600000 = 10 min)
        #[arg(long, default_value_t = 600_000)]
        idle_timeout_ms: u64,

        /// Hard timeout in milliseconds — absolute max runtime (default: 3600000 = 1 hour)
        #[arg(long, default_value_t = 3_600_000)]
        hard_timeout_ms: u64,

        /// Alias for --idle-timeout-ms (backward compat)
        #[arg(long, default_value_t = 0)]
        timeout_ms: u64,

        /// Path to claude binary (default: "claude")
        #[arg(long, default_value = "claude")]
        claude_bin: String,

        /// Working directory for Claude
        #[arg(long)]
        cwd: Option<PathBuf>,

        /// PTY columns
        #[arg(long, default_value_t = 120)]
        cols: u16,

        /// PTY rows
        #[arg(long, default_value_t = 40)]
        rows: u16,

        /// Resume a persisted session
        #[arg(long)]
        resume: Option<String>,

        /// Auto-accept workspace trust prompt
        #[arg(long, default_value_t = true)]
        auto_accept_workspace_trust: bool,

        /// Show compact tool-use progress lines on stderr
        #[arg(
            short = 't',
            long = "tool",
            visible_alias = "t",
            default_value_t = false
        )]
        terminal_tools: bool,

        /// Extra args to forward to claude
        #[arg(last = true)]
        extra_args: Vec<String>,
    },
}
