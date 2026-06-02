//! Adapters por agente.
//!
//! Cada agente de codificação manda o pedido de aprovação no SEU formato e
//! espera a resposta no SEU formato. Estas funções traduzem esses dois lados
//! para/dos tipos neutros do `core`. É isto que mantém o cérebro agnóstico.
//!
//! Suportados: **Copilot CLI** e **Claude Code** (os payloads de `PreToolUse`
//! são praticamente iguais; o que muda é o formato da resposta).

use chris_core::{assess_risk, Agent, ApprovalRequest, Decision, ReqId};
use serde::Deserialize;

/// Erro ao interpretar o payload do agente.
#[derive(Debug)]
pub enum AdapterError {
    Json(serde_json::Error),
}

impl From<serde_json::Error> for AdapterError {
    fn from(e: serde_json::Error) -> Self {
        AdapterError::Json(e)
    }
}

/// O que devolver ao agente: texto pro stdout + código de saída do processo.
#[derive(Debug, PartialEq, Eq)]
pub struct AgentResponse {
    pub stdout: String,
    pub exit_code: i32,
}

// ===========================================================================
// Parsing do evento PreToolUse (comum a Copilot e Claude)
// ===========================================================================

/// Payload do evento `PreToolUse` (campos compartilhados por Copilot e Claude).
/// `#[serde(default)]` evita erro se algum campo faltar.
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

/// Resumo legível da ação. Nunca devolve "null": cai num texto amigável.
fn summarize(tool_name: &str, input: &serde_json::Value) -> String {
    // caso comum: um comando de shell
    if let Some(cmd) = input.get("command").and_then(|v| v.as_str()) {
        if !cmd.is_empty() {
            return cmd.to_string();
        }
    }
    // alguns tools usam outros campos de texto
    for key in ["content", "path", "file_path", "url", "query"] {
        if let Some(v) = input.get(key).and_then(|v| v.as_str()) {
            if !v.is_empty() {
                return format!("{key}: {v}");
            }
        }
    }
    if let Some(s) = input.as_str() {
        if !s.is_empty() {
            return s.to_string();
        }
    }
    // sem detalhes úteis: usa o nome da ferramenta em vez de "null"
    if input.is_null()
        || input
            .as_object()
            .map(|o| o.is_empty())
            .unwrap_or(false)
    {
        return if tool_name.is_empty() {
            "(sem detalhes)".to_string()
        } else {
            format!("{tool_name} (sem detalhes)")
        };
    }
    // objeto com campos desconhecidos: mostra compacto (mas nunca será "null")
    input.to_string()
}

// ===========================================================================
// Copilot CLI
// ===========================================================================

pub fn parse_copilot(payload: &str, id: ReqId) -> Result<ApprovalRequest, AdapterError> {
    parse_pretooluse(payload, id, Agent::Copilot)
}

/// Resposta no formato do Copilot.
/// - `Allow`/`Deny` -> `{"permissionDecision": "...", ...}`
/// - `Defer`        -> `{}` (cai no prompt nativo do Copilot)
pub fn format_copilot(decision: Decision, reason: &str) -> AgentResponse {
    let stdout = match decision {
        Decision::Allow => serde_json::json!({
            "permissionDecision": "allow",
            "permissionDecisionReason": reason
        })
        .to_string(),
        Decision::Deny => serde_json::json!({
            "permissionDecision": "deny",
            "permissionDecisionReason": reason
        })
        .to_string(),
        Decision::Defer => "{}".to_string(),
    };
    AgentResponse { stdout, exit_code: 0 }
}

// ===========================================================================
// Claude Code
// ===========================================================================

pub fn parse_claude(payload: &str, id: ReqId) -> Result<ApprovalRequest, AdapterError> {
    parse_pretooluse(payload, id, Agent::Claude)
}

/// Resposta no formato do Claude Code (`hookSpecificOutput.permissionDecision`).
/// - `Defer` -> saída vazia (Claude usa o fluxo de permissão normal dele).
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
    fn summary_never_null() {
        // tool_input ausente -> nada de "null"
        let payload = r#"{ "tool_name": "mcp_tool" }"#;
        let req = parse_copilot(payload, ReqId(1)).unwrap();
        assert_eq!(req.summary, "mcp_tool (sem detalhes)");
        assert_ne!(req.summary, "null");
    }

    #[test]
    fn format_copilot_variants() {
        assert!(format_copilot(Decision::Allow, "ok")
            .stdout
            .contains("\"permissionDecision\":\"allow\""));
        assert_eq!(format_copilot(Decision::Defer, "").stdout, "{}");
    }

    #[test]
    fn format_claude_variants() {
        let allow = format_claude(Decision::Allow, "ok");
        assert!(allow.stdout.contains("\"hookSpecificOutput\""));
        assert!(allow.stdout.contains("\"permissionDecision\":\"allow\""));
        // Defer no Claude = saída vazia
        assert_eq!(format_claude(Decision::Defer, "").stdout, "");
    }
}
