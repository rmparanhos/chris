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
    let risk = assess_risk(&summary);
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

/// Readable summary of the action, with as much context as is useful to decide.
/// Never returns "null": falls back to friendly text.
fn summarize(tool_name: &str, input: &serde_json::Value) -> String {
    let get = |k: &str| input.get(k).and_then(|v| v.as_str()).filter(|s| !s.is_empty());

    // 1) shell command (run a command)
    if let Some(cmd) = get("command") {
        return cmd.to_string();
    }

    // 2) file write/edit: show the path AND a preview of what will be written.
    //    Covers Write/Edit across agents (content / new_string / contents, and
    //    file_path / path).
    let path = get("file_path").or_else(|| get("path"));
    let body = get("content").or_else(|| get("new_string")).or_else(|| get("contents"));
    if let Some(p) = path {
        let mut out = format!("file: {p}");
        if let Some(old) = get("old_string") {
            out.push_str(&format!("\n\n— replacing —\n{}", preview(old)));
            if let Some(b) = body {
                out.push_str(&format!("\n\n— with —\n{}", preview(b)));
            }
        } else if let Some(b) = body {
            out.push_str(&format!("\n\n{}", preview(b)));
        }
        return out;
    }

    // 3) other common single-value fields
    for key in ["url", "query", "pattern", "content"] {
        if let Some(v) = get(key) {
            return format!("{key}: {v}");
        }
    }
    if let Some(s) = input.as_str() {
        if !s.is_empty() {
            return s.to_string();
        }
    }

    // 4) no useful details: use the tool name instead of "null"
    if input.is_null() || input.as_object().map(|o| o.is_empty()).unwrap_or(false) {
        return if tool_name.is_empty() {
            "(sem detalhes)".to_string()
        } else {
            format!("{tool_name} (sem detalhes)")
        };
    }

    // 5) object with unknown fields: show it pretty (never "null")
    serde_json::to_string_pretty(input).unwrap_or_else(|_| input.to_string())
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
        assert!(req.summary.starts_with("file: src/main.rs"));
        assert!(req.summary.contains("fn main() {}"));
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
