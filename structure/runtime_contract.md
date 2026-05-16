# Runtime Contract

## Default Print-Compatible PTY Mode

When no `run` or `exec` subcommand is present, `claude-e` and `claude-exec`
parse a `claude -p`-style invocation and execute the existing PTY runtime:

```text
claude-e [claude -p style args] <prompt>
claude-e p [claude -p style args] <prompt>
claude-exec [claude -p style args] <prompt>
```

This mode still allocates a PTY and injects the prompt through Claude Code's
interactive UI. It suppresses wrapper runtime diagnostics from stdout so the
visible surface behaves like a print-mode command.

Accepted print-style controls:

- optional leading `p` or `print` alias, plus `-p` / `--print`;
- positional prompt text;
- piped stdin, appended after the positional prompt;
- `--input-format text|stream-json` for stdin parsing;
- `--output-format text|json|stream-json` for wrapper output normalization;
- `--session-id`, `--no-session-persistence`, `--resume`, and `-r` for session shape;
- `--json-schema`, which becomes an explicit JSON-only instruction appended to the prompt;
- `--verbose`, `--include-partial-messages`, `--include-hook-events`, `--replay-user-messages`, `--fallback-model`, and `--max-budget-usd`, accepted for compatibility and consumed when the PTY path cannot enforce the exact print-mode behavior;
- Claude runtime flags such as `--model`, `--effort`, `--permission-mode`, `--add-dir`, `--allowed-tools`, `--tools`, `--mcp-config`, `--settings`, and prompt/system/plugin flags, forwarded to the PTY Claude process.

Recognized value flags support `--flag=value`. Variadic Claude flags consume one
value per occurrence; repeat the flag or insert `--` before prompt text when the
boundary is ambiguous.

The Claude binary is resolved from `CLAUDE_EXEC_CLAUDE_BIN`, then `CLAUDE_BIN`,
then `claude`.

## PTY Run Input

- The prompt is read from stdin.
- Empty stdin is rejected with exit code `16`.
- Prompt input is capped at 10 MB.
- The sanitized prompt is injected into Claude through bracketed paste in a PTY.

## PTY Spawn

`claude-e` starts the underlying Claude CLI in a PTY. The Claude binary defaults to `claude`, but embedding runtimes should pass an explicit path with `--claude-bin` to avoid PATH snapshot drift.

Fresh runs add a generated `--session-id` unless print-compatible mode received
`--no-session-persistence`. Resume runs pass `--resume <session-id>`. A
print-compatible `--session-id` value overrides the generated id.

When `--auto-accept-workspace-trust` is set, the wrapper watches the PTY screen while waiting for the `SessionStart` hook. If Claude displays a workspace trust prompt, the wrapper submits the affirmative menu choice before prompt injection starts.

The wrapper writes a temporary Claude settings file with hook commands. Hooks relay SessionStart, Stop, and StopFailure payloads to files in an isolated temporary directory.

## PTY Output

Stdout is JSONL.

`claude-e` emits runtime events using the existing cli-jaw envelope:

```json
{"type":"jaw_runtime","event":"runtime_started","runId":"run_12345678"}
```

The envelope remains `jaw_runtime` during the extraction because cli-jaw already consumes it. A future protocol rename can add `claude_exec_runtime` as an additive alias before removing `jaw_runtime`.

Claude transcript records are tailed and normalized into Claude-like stream-json records. On completion, a synthetic result record may be emitted from the last assistant message.

After prompt injection, the wrapper verifies that the transcript advanced beyond
the pre-injection offset. A new `user` record or a new `assistant` record counts
as acceptance, because some Claude builds can start streaming the answer before
the user record is flushed.

## Exit Codes

| Code | Meaning |
|---:|---|
| `0` | Normal completion |
| `1` | Underlying Claude exited unsuccessfully without a more specific wrapper classification |
| `2` | Graceful interrupt; session metadata can be resumable |
| `4` | Claude spawn or PTY write failure |
| `5` | SessionStart hook failure, timeout, or early Claude exit before SessionStart. Timeout errors include a compact PTY screen snapshot when available. |
| `6` | Runtime timeout |
| `7` | Prompt injection transcript verification failure |
| `11` | Claude StopFailure hook |
| `13` | Hook temp dir or settings generation failure |
| `16` | stdin read, size, empty prompt, or prompt sanitization failure |

## Compatibility Guarantees

- `claude-e ...` and `claude-exec ...` without `run`/`exec` preserve the `claude -p` command shape while staying PTY-backed.
- `claude-e run` and `claude-exec run` remain stable for cli-jaw integration.
- `claude-e` is the preferred npm package and public command name.
- `claude-exec` remains a compatibility helper binary.
- `jaw-claude-i` remains a compatibility binary while cli-jaw migration is active.
- `claude-i` remains a compatibility binary while settings and saved cli-jaw provider ids still reference it.
- `jaw_runtime` remains emitted until cli-jaw supports an additive `claude_exec_runtime` event family.
