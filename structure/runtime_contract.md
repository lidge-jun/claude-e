# Runtime Contract

## Input

- The prompt is read from stdin.
- Empty stdin is rejected with exit code `16`.
- Prompt input is capped at 10 MB.
- The sanitized prompt is injected into Claude through bracketed paste in a PTY.

## Spawn

`claude-exec` starts the underlying Claude CLI in a PTY. The Claude binary defaults to `claude`, but embedding runtimes should pass an explicit path with `--claude-bin` to avoid PATH snapshot drift.

Fresh runs add a generated `--session-id`. Resume runs pass `--resume <session-id>`.

The wrapper writes a temporary Claude settings file with hook commands. Hooks relay SessionStart, Stop, and StopFailure payloads to files in an isolated temporary directory.

## Output

Stdout is JSONL.

`claude-exec` emits runtime events using the existing cli-jaw envelope:

```json
{"type":"jaw_runtime","event":"runtime_started","runId":"run_12345678"}
```

The envelope remains `jaw_runtime` during the extraction because cli-jaw already consumes it. A future protocol rename can add `claude_exec_runtime` as an additive alias before removing `jaw_runtime`.

Claude transcript records are tailed and normalized into Claude-like stream-json records. On completion, a synthetic result record may be emitted from the last assistant message.

## Exit Codes

| Code | Meaning |
|---:|---|
| `0` | Normal completion |
| `1` | Underlying Claude exited unsuccessfully without a more specific wrapper classification |
| `2` | Graceful interrupt; session metadata can be resumable |
| `4` | Claude spawn or PTY write failure |
| `5` | SessionStart hook failure, timeout, or early Claude exit before SessionStart |
| `6` | Runtime timeout |
| `7` | Prompt injection transcript verification failure |
| `11` | Claude StopFailure hook |
| `13` | Hook temp dir or settings generation failure |
| `16` | stdin read, size, empty prompt, or prompt sanitization failure |

## Compatibility Guarantees

- `claude-exec run` remains stable for cli-jaw integration.
- `jaw-claude-i` remains a compatibility binary while cli-jaw migration is active.
- `claude-i` remains a compatibility binary while settings and saved cli-jaw provider ids still reference it.
- `jaw_runtime` remains emitted until cli-jaw supports an additive `claude_exec_runtime` event family.
