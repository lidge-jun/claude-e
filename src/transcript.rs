use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use crate::normalize;

fn touch_activity(tracker: Option<&Arc<AtomicU64>>) {
    if let Some(t) = tracker {
        #[allow(clippy::cast_possible_truncation)]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        t.store(now, Ordering::Relaxed);
    }
}

fn update_tool_state(event: &serde_json::Value, counter: Option<&Arc<AtomicUsize>>) {
    let Some(counter) = counter else { return };
    let content = event
        .get("message")
        .or(Some(event))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array());
    let Some(blocks) = content else { return };

    for block in blocks {
        match block.get("type").and_then(|t| t.as_str()) {
            Some("tool_use") => {
                counter.fetch_add(1, Ordering::Relaxed);
            }
            Some("tool_result") => {
                let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                    if v > 0 { Some(v - 1) } else { None }
                });
            }
            _ => {}
        }
    }
}

pub fn tail_transcript(
    transcript_path: &Path,
    stop: Arc<AtomicBool>,
    output_format: &str,
    initial_offset: u64,
    terminal_tools: bool,
    activity_tracker: Option<Arc<AtomicU64>>,
    active_tools: Option<Arc<AtomicUsize>>,
) -> Result<Option<serde_json::Value>, String> {
    let mut file = wait_for_file(transcript_path, &stop, 20_000)?;
    let mut offset = clamped_initial_offset(&file, initial_offset);
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("failed to seek transcript to {offset}: {e}"))?;
    let mut last_assistant: Option<serde_json::Value> = None;

    loop {
        let reader = BufReader::new(&file);
        let mut any_line = false;

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let line_bytes = line.len() as u64 + 1; // +1 for newline

                    if line.trim().is_empty() {
                        offset += line_bytes;
                        any_line = true;
                        continue;
                    }

                    // Only advance offset if JSON parses — partial writes retry next poll
                    if serde_json::from_str::<serde_json::Value>(&line).is_err() {
                        log::debug!(
                            "transcript: incomplete JSON, will retry: {}...",
                            &line[..line.len().min(80)]
                        );
                        break;
                    }

                    any_line = true;
                    offset += line_bytes;
                    touch_activity(activity_tracker.as_ref());
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                        update_tool_state(&v, active_tools.as_ref());
                    }

                    if let Some(normalized) = normalize::normalize_transcript_line(&line) {
                        emit_line(&normalized, output_format, terminal_tools);

                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                            if v.get("type").and_then(|t| t.as_str()) == Some("assistant") {
                                last_assistant = Some(v);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::debug!("transcript read error: {e}");
                    break;
                }
            }
        }

        if stop.load(Ordering::Relaxed) {
            if any_line {
                // One more drain pass after stop signal
                std::thread::sleep(std::time::Duration::from_millis(300));
                if let Ok(mut f) = File::open(transcript_path) {
                    let _ = f.seek(SeekFrom::Start(offset));
                    let r = BufReader::new(f);
                    for line in r.lines().map_while(Result::ok) {
                        if let Some(normalized) = normalize::normalize_transcript_line(&line) {
                            touch_activity(activity_tracker.as_ref());
                            emit_line(&normalized, output_format, terminal_tools);
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                                if v.get("type").and_then(|t| t.as_str()) == Some("assistant") {
                                    last_assistant = Some(v);
                                }
                            }
                        }
                    }
                }
            }
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
        // Seek to current offset for next read pass
        let _ = file.seek(SeekFrom::Start(offset));
    }

    Ok(last_assistant)
}

pub fn current_file_len(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|metadata| metadata.len())
}

pub fn wait_for_prompt_activity_after_offset(
    transcript_path: &Path,
    initial_offset: u64,
    timeout_ms: u64,
    stop: &AtomicBool,
) -> Result<bool, String> {
    let mut file = wait_for_file(transcript_path, stop, timeout_ms)?;
    let mut offset = clamped_initial_offset(&file, initial_offset);
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("failed to seek transcript to {offset}: {e}"))?;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    loop {
        let reader = BufReader::new(&file);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let line_bytes = line.len() as u64 + 1;
                    if line.trim().is_empty() {
                        offset += line_bytes;
                        continue;
                    }

                    let value = match serde_json::from_str::<serde_json::Value>(&line) {
                        Ok(value) => value,
                        Err(_) => break,
                    };

                    offset += line_bytes;
                    if is_prompt_acceptance_activity(&value) {
                        return Ok(true);
                    }
                }
                Err(e) => {
                    log::debug!("transcript verification read error: {e}");
                    break;
                }
            }
        }

        if stop.load(Ordering::Relaxed) {
            return Ok(false);
        }
        if start.elapsed() > timeout {
            return Ok(false);
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = file.seek(SeekFrom::Start(offset));
    }
}

fn is_prompt_acceptance_activity(value: &serde_json::Value) -> bool {
    matches!(
        value.get("type").and_then(|t| t.as_str()),
        Some("user" | "assistant")
    )
}

fn clamped_initial_offset(file: &File, requested_offset: u64) -> u64 {
    file.metadata()
        .map(|metadata| requested_offset.min(metadata.len()))
        .unwrap_or(0)
}

fn emit_line(normalized: &str, output_format: &str, terminal_tools: bool) {
    match output_format {
        "stream-json" => {
            println!("{normalized}");
            if terminal_tools {
                emit_terminal_tool_events(normalized);
            }
        }
        "json" => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(normalized) {
                let t = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if t == "result" {
                    println!("{normalized}");
                }
            }
            if terminal_tools {
                emit_terminal_tool_events(normalized);
            }
        }
        "text" => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(normalized) {
                if v.get("type").and_then(|t| t.as_str()) == Some("assistant") {
                    extract_and_print_text(&v);
                }
            }
            if terminal_tools {
                emit_terminal_tool_events(normalized);
            }
        }
        _ => println!("{normalized}"),
    }
}

fn emit_terminal_tool_events(normalized: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(normalized) else {
        return;
    };
    let Some(message) = value.get("message") else {
        return;
    };
    let Some(content) = message
        .get("content")
        .and_then(|content| content.as_array())
    else {
        return;
    };

    for block in content {
        match block.get("type").and_then(|kind| kind.as_str()) {
            Some("tool_use") => emit_tool_use(block),
            Some("tool_result") => emit_tool_result(block),
            _ => {}
        }
    }
}

fn emit_tool_use(block: &serde_json::Value) {
    let name = block
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("tool");
    let summary = block
        .get("input")
        .map(compact_tool_input)
        .unwrap_or_else(|| String::from("(no input)"));
    eprintln!("[claude-e:tool] {name}: {summary}");
}

fn emit_tool_result(block: &serde_json::Value) {
    let status = if block
        .get("is_error")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        "error"
    } else {
        "ok"
    };
    let summary = block
        .get("content")
        .and_then(serde_json::Value::as_str)
        .map(compact_text)
        .unwrap_or_else(|| String::from("(no text result)"));
    eprintln!("[claude-e:tool-result] {status}: {summary}");
}

fn compact_tool_input(input: &serde_json::Value) -> String {
    if let Some(command) = input.get("command").and_then(serde_json::Value::as_str) {
        return compact_text(command);
    }
    if let Some(description) = input.get("description").and_then(serde_json::Value::as_str) {
        return compact_text(description);
    }
    if let Some(path) = input
        .get("file_path")
        .or_else(|| input.get("path"))
        .and_then(serde_json::Value::as_str)
    {
        return compact_text(path);
    }
    compact_text(&input.to_string())
}

fn compact_text(text: &str) -> String {
    const LIMIT: usize = 180;
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= LIMIT {
        return compact;
    }
    let mut shortened = compact.chars().take(LIMIT).collect::<String>();
    shortened.push_str("...");
    shortened
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn current_file_len_reports_existing_file_size() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        write!(file, "first\nsecond\n").expect("write fixture");

        assert_eq!(current_file_len(file.path()), Some(13));
    }

    #[test]
    fn clamped_initial_offset_caps_at_file_size() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        write!(file, "line\n").expect("write fixture");

        assert_eq!(clamped_initial_offset(file.as_file(), 2), 2);
        assert_eq!(clamped_initial_offset(file.as_file(), 999), 5);
    }

    #[test]
    fn wait_for_prompt_activity_after_offset_detects_user_after_offset() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"OLD_RESPONSE"}}]}}}}"#
        )
        .expect("write old assistant");
        let initial_offset = current_file_len(file.path()).expect("old offset");
        writeln!(
            file,
            r#"{{"type":"user","message":{{"role":"user","content":"NEW_PROMPT"}}}}"#
        )
        .expect("write new user");

        assert!(
            wait_for_prompt_activity_after_offset(
                file.path(),
                initial_offset,
                500,
                &AtomicBool::new(false),
            )
            .expect("wait for prompt activity")
        );
    }

    #[test]
    fn wait_for_prompt_activity_after_offset_ignores_user_before_offset() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        writeln!(
            file,
            r#"{{"type":"user","message":{{"role":"user","content":"OLD_PROMPT"}}}}"#
        )
        .expect("write old user");
        let initial_offset = current_file_len(file.path()).expect("old offset");

        assert!(
            !wait_for_prompt_activity_after_offset(
                file.path(),
                initial_offset,
                150,
                &AtomicBool::new(false),
            )
            .expect("wait for prompt activity")
        );
    }

    #[test]
    fn wait_for_prompt_activity_after_offset_accepts_assistant_after_offset() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        writeln!(
            file,
            r#"{{"type":"user","message":{{"role":"user","content":"OLD_PROMPT"}}}}"#
        )
        .expect("write old user");
        let initial_offset = current_file_len(file.path()).expect("old offset");
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"NEW_RESPONSE"}}]}}}}"#
        )
        .expect("write new assistant");

        assert!(
            wait_for_prompt_activity_after_offset(
                file.path(),
                initial_offset,
                500,
                &AtomicBool::new(false),
            )
            .expect("wait for prompt activity")
        );
    }

    #[test]
    fn tail_transcript_skips_assistant_before_initial_offset() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"OLD_RESPONSE"}}],"model":"old-model"}},"sessionId":"sid-old"}}"#
        )
        .expect("write old assistant");
        let initial_offset = current_file_len(file.path()).expect("old offset");
        writeln!(
            file,
            r#"{{"type":"user","message":{{"role":"user","content":"NEW_PROMPT"}},"sessionId":"sid-new"}}"#
        )
        .expect("write new user");
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"NEW_RESPONSE"}}],"model":"new-model"}},"sessionId":"sid-new"}}"#
        )
        .expect("write new assistant");

        let last_assistant = tail_transcript(
            file.path(),
            Arc::new(AtomicBool::new(true)),
            "json",
            initial_offset,
            false,
            None,
            None,
        )
        .expect("tail transcript")
        .expect("new assistant");

        assert_eq!(
            last_assistant["message"]["content"][0]["text"],
            "NEW_RESPONSE"
        );
        assert_eq!(last_assistant["sessionId"], "sid-new");
    }

    #[test]
    fn tail_transcript_skips_synthetic_no_response_placeholder() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"No response requested."}}],"model":"<synthetic>"}},"sessionId":"sid-new"}}"#
        )
        .expect("write placeholder assistant");
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"REAL_RESPONSE"}}],"model":"claude-opus-4-7"}},"sessionId":"sid-new"}}"#
        )
        .expect("write real assistant");

        let last_assistant = tail_transcript(
            file.path(),
            Arc::new(AtomicBool::new(true)),
            "json",
            0,
            false,
            None,
            None,
        )
        .expect("tail transcript")
        .expect("real assistant");

        assert_eq!(
            last_assistant["message"]["content"][0]["text"],
            "REAL_RESPONSE"
        );
    }

    #[test]
    fn update_tool_state_increments_on_tool_use() {
        let counter = Arc::new(AtomicUsize::new(0));
        let event: serde_json::Value = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "tool_use", "name": "bash", "input": {}},
                    {"type": "tool_use", "name": "read", "input": {}}
                ]
            }
        });
        update_tool_state(&event, Some(&counter));
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn update_tool_state_decrements_on_tool_result() {
        let counter = Arc::new(AtomicUsize::new(2));
        let event: serde_json::Value = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [{"type": "tool_result", "tool_use_id": "abc", "content": "ok"}]
            }
        });
        update_tool_state(&event, Some(&counter));
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn update_tool_state_saturates_at_zero() {
        let counter = Arc::new(AtomicUsize::new(0));
        let event: serde_json::Value = serde_json::json!({
            "type": "user",
            "message": {
                "content": [{"type": "tool_result", "content": "ok"}]
            }
        });
        update_tool_state(&event, Some(&counter));
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn update_tool_state_noop_without_flag() {
        let event: serde_json::Value = serde_json::json!({
            "type": "assistant",
            "message": { "content": [{"type": "tool_use", "name": "x", "input": {}}] }
        });
        update_tool_state(&event, None);
    }
}

fn wait_for_file(path: &Path, stop: &AtomicBool, timeout_ms: u64) -> Result<File, String> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    loop {
        if let Ok(f) = File::open(path) {
            return Ok(f);
        }
        if start.elapsed() > timeout {
            return Err(format!(
                "transcript not found after {timeout_ms}ms: {}",
                path.display()
            ));
        }
        if stop.load(Ordering::Relaxed) {
            return Err("stopped before transcript appeared".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn extract_and_print_text(value: &serde_json::Value) {
    let mut printed = false;
    if let Some(message) = value.get("message") {
        if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
            for block in content {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        print!("{text}");
                        printed = true;
                    }
                }
            }
        }
    }
    if printed {
        let _ = std::io::stdout().flush();
    }
}
