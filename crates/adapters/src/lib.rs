//! Per-agent adapters.
//!
//! Each coding agent sends the approval request in ITS OWN format and expects
//! the response in ITS OWN format. These functions translate both sides
//! to/from the neutral types in `core`. This is what keeps the brain agnostic.
//!
//! Supported: **Copilot CLI** and **Claude Code** (the `PreToolUse` payloads
//! are practically identical; what differs is the response format).

use chris_core::{assess_risk, Agent, ApprovalRequest, Decision, ReqId};
use serde::Deserialize;

/// Error while parsing the agent payload.
#[derive(Debug)]
pub enum AdapterError {
    Json(serde_json::Error),
}

impl From<serde_json::Error> for AdapterError {
    fn from(e: serde_json::Error) -> Self {
        AdapterError::Json(e)
    }
}

/// What to return to the agent: text for stdout + the process exit code.
#[derive(Debug, PartialEq, Eq)]
pub struct AgentResponse {
    pub stdout: String,
    pub exit_code: i32,
}

// ===========================================================================
// Parsing of the PreToolUse event (common to Copilot and Claude)
// ===========================================================================

/// Payload of the `PreToolUse` event (fields shared by Copilot and Claude).
/// `#[serde(default)]` avoids an error if some field is missing.
#[derive(Deserialize, Default)]
struct PreToolUse {
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    tool_input: serde_json::Value,
    #[serde(default)]
    cwd: String,
}

fn parse_pretooluse(payload: &str, id: ReqId, agent: Agent) -> Result<ApprovalRequest, AdapterError> {
    let p: PreToolUse = serde_json::from_str(payload)?;
    let summary = summarize(&p.tool_name, &p.tool_input);
    // Assess risk from the actual command when there is one (a file's content
    // shouldn't trip the shell heuristics); otherwise fall back to the summary.
    let cmd = p.tool_input.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let risk = if cmd.is_empty() { assess_risk(&summary) } else { assess_risk(cmd) };
    Ok(ApprovalRequest {
        id,
        agent,
        tool: p.tool_name,
        summary,
        cwd: p.cwd,
        risk,
    })
}

/// How many characters of a long field (file content, etc.) we keep in the
/// summary. Enough to give context without flooding the popup.
const PREVIEW_LIMIT: usize = 1500;

/// Truncates long text, adding an ellipsis note so it's clear it was cut.
fn preview(s: &str) -> String {
    if s.chars().count() <= PREVIEW_LIMIT {
        return s.to_string();
    }
    let cut: String = s.chars().take(PREVIEW_LIMIT).collect();
    format!("{cut}\n… (truncated)")
}

/// Friendly fallback text when there is nothing useful to show.
fn no_details(tool_name: &str) -> String {
    if tool_name.is_empty() {
        "(sem detalhes)".to_string()
    } else {
        format!("{tool_name} (sem detalhes)")
    }
}

/// Fields that are typically long/multi-line — rendered as a labelled block on
/// their own lines instead of inline.
fn is_block_key(k: &str) -> bool {
    matches!(
        k,
        "command" | "content" | "contents" | "new_string" | "old_string" | "body" | "text" | "diff"
    )
}

/// Renders one `key: value` field. Long/multi-line values go on their own lines.
fn field_line(key: &str, v: &serde_json::Value) -> Option<String> {
    if v.is_null() {
        return None;
    }
    let text = match v.as_str() {
        Some(s) => s.to_string(),
        None => v.to_string(),
    };
    if text.is_empty() {
        return None;
    }
    Some(if is_block_key(key) || text.contains('\n') || text.len() > 80 {
        format!("{key}:\n{}", preview(&text))
    } else {
        format!("{key}: {text}")
    })
}

/// Readable summary of the action. Renders EVERY field of the tool input so the
/// popup carries the same information the CLI would show — nothing hidden.
/// Never returns "null": falls back to friendly text.
fn summarize(tool_name: &str, input: &serde_json::Value) -> String {
    // a plain string input
    if let Some(s) = input.as_str() {
        return if s.is_empty() { no_details(tool_name) } else { preview(s) };
    }

    let obj = match input.as_object() {
        Some(o) if !o.is_empty() => o,
        _ => return no_details(tool_name),
    };

    // common shortcut: a lone shell command -> show it bare
    if obj.len() == 1 {
        if let Some(cmd) = obj.get("command").and_then(|v| v.as_str()) {
            if !cmd.is_empty() {
                return preview(cmd);
            }
        }
    }

    // otherwise render every field; put the important ones first, then the rest
    let mut lines: Vec<String> = Vec::new();
    let mut done: Vec<&str> = Vec::new();
    for key in ["command", "file_path", "path", "url", "old_string", "new_string"] {
        if let Some(v) = obj.get(key) {
            if let Some(line) = field_line(key, v) {
                lines.push(line);
            }
            done.push(key);
        }
    }
    for (key, v) in obj {
        if done.contains(&key.as_str()) {
            continue;
        }
        if let Some(line) = field_line(key, v) {
            lines.push(line);
        }
    }

    if lines.is_empty() {
        no_details(tool_name)
    } else {
        lines.join("\n")
    }
}

// ===========================================================================
// Copilot CLI
// ===========================================================================

pub fn parse_copilot(payload: &str, id: ReqId) -> Result<ApprovalRequest, AdapterError> {
    parse_pretooluse(payload, id, Agent::Copilot)
}

/// Response in Copilot's format.
/// - `Allow`/`Deny` -> `{"permissionDecision": "...", ...}`
/// - `Defer`        -> `{}` (falls back to Copilot's native prompt)
pub fn format_copilot(decision: Decision, reason: &str) -> AgentResponse {
    match decision {
        // includes both `permissionDecision` (VS Code style) and `behavior`
        // (Copilot's internal field), so that "allow" suppresses the native
        // prompt across all versions.
        Decision::Allow => AgentResponse {
            stdout: serde_json::json!({
                "permissionDecision": "allow",
                "permissionDecisionReason": reason,
                "behavior": "allow"
            })
            .to_string(),
            exit_code: 0,
        },
        // exit code 2 is Copilot's most reliable "deny" signal; the JSON reinforces it.
        Decision::Deny => AgentResponse {
            stdout: serde_json::json!({
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
                "behavior": "deny"
            })
            .to_string(),
            exit_code: 2,
        },
        // Defer: empty output -> Copilot uses its own permission flow.
        Decision::Defer => AgentResponse {
            stdout: "{}".to_string(),
            exit_code: 0,
        },
    }
}

// ===========================================================================
// Claude Code
// ===========================================================================

pub fn parse_claude(payload: &str, id: ReqId) -> Result<ApprovalRequest, AdapterError> {
    parse_pretooluse(payload, id, Agent::Claude)
}

/// Response in Claude Code's format (`hookSpecificOutput.permissionDecision`).
/// - `Defer` -> empty output (Claude uses its own normal permission flow).
pub fn format_claude(decision: Decision, reason: &str) -> AgentResponse {
    let value = match decision {
        Decision::Allow => Some("allow"),
        Decision::Deny => Some("deny"),
        Decision::Defer => None,
    };
    let stdout = match value {
        Some(v) => serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": v,
                "permissionDecisionReason": reason
            }
        })
        .to_string(),
        None => String::new(),
    };
    AgentResponse { stdout, exit_code: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chris_core::Risk;

    const SHELL: &str = r#"{
        "hook_event_name": "PreToolUse",
        "cwd": "/home/dev/proj",
        "tool_name": "shell",
        "tool_input": { "command": "rm -rf build/" }
    }"#;

    #[test]
    fn parse_shell_copilot() {
        let req = parse_copilot(SHELL, ReqId(1)).unwrap();
        assert_eq!(req.agent, Agent::Copilot);
        assert_eq!(req.summary, "rm -rf build/");
        assert_eq!(req.risk, Risk::High);
    }

    #[test]
    fn parse_shell_claude() {
        let req = parse_claude(SHELL, ReqId(1)).unwrap();
        assert_eq!(req.agent, Agent::Claude);
        assert_eq!(req.summary, "rm -rf build/");
    }

    #[test]
    fn summary_write_shows_path_and_content() {
        let payload = r#"{
            "tool_name": "Write",
            "tool_input": { "file_path": "src/main.rs", "content": "fn main() {}" }
        }"#;
        let req = parse_claude(payload, ReqId(1)).unwrap();
        assert!(req.summary.contains("src/main.rs"));
        assert!(req.summary.contains("fn main() {}"));
    }

    #[test]
    fn summary_renders_all_fields() {
        // every field present so the popup carries full context
        let payload = r#"{
            "tool_name": "Bash",
            "tool_input": { "command": "git push", "description": "push to origin", "timeout": 5000 }
        }"#;
        let req = parse_copilot(payload, ReqId(1)).unwrap();
        assert!(req.summary.contains("git push"));
        assert!(req.summary.contains("description: push to origin"));
        assert!(req.summary.contains("timeout: 5000"));
        assert_eq!(req.risk, chris_core::Risk::Medium); // from the command, not the content
    }

    #[test]
    fn summary_never_null() {
        // tool_input missing -> no "null"
        let payload = r#"{ "tool_name": "mcp_tool" }"#;
        let req = parse_copilot(payload, ReqId(1)).unwrap();
        assert_eq!(req.summary, "mcp_tool (sem detalhes)");
        assert_ne!(req.summary, "null");
    }

    #[test]
    fn format_copilot_variants() {
        let allow = format_copilot(Decision::Allow, "ok");
        assert!(allow.stdout.contains("\"permissionDecision\":\"allow\""));
        assert!(allow.stdout.contains("\"behavior\":\"allow\""));
        assert_eq!(allow.exit_code, 0);

        let deny = format_copilot(Decision::Deny, "no");
        assert!(deny.stdout.contains("\"behavior\":\"deny\""));
        assert_eq!(deny.exit_code, 2); // exit 2 = reliable deny

        assert_eq!(format_copilot(Decision::Defer, "").stdout, "{}");
    }

    #[test]
    fn format_claude_variants() {
        let allow = format_claude(Decision::Allow, "ok");
        assert!(allow.stdout.contains("\"hookSpecificOutput\""));
        assert!(allow.stdout.contains("\"permissionDecision\":\"allow\""));
        // Defer in Claude = empty output
        assert_eq!(format_claude(Decision::Defer, "").stdout, "");
    }
}
