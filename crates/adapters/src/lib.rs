//! Adapters por agente.
//!
//! Cada agente de codificação manda o pedido de aprovação no SEU formato e
//! espera a resposta no SEU formato. Estas funções traduzem esses dois lados
//! para/dos tipos neutros do `core`. É isto que mantém o cérebro agnóstico:
//! toda a diferença entre agentes mora aqui.
//!
//! MVP = **Copilot CLI**. Claude e Codex entram na fase 2 (são quase iguais).

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
// Copilot CLI
// ===========================================================================

/// Payload do evento `preToolUse` do Copilot (campos em snake_case).
/// Só pegamos o que interessa; `#[serde(default)]` evita erro se faltar algo.
#[derive(Deserialize)]
struct CopilotPreToolUse {
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    tool_input: serde_json::Value,
    #[serde(default)]
    cwd: String,
}

/// Converte o payload do Copilot em um `ApprovalRequest` neutro.
pub fn parse_copilot(payload: &str, id: ReqId) -> Result<ApprovalRequest, AdapterError> {
    let p: CopilotPreToolUse = serde_json::from_str(payload)?;
    let summary = summarize(&p.tool_input);
    let risk = assess_risk(&summary);
    Ok(ApprovalRequest {
        id,
        agent: Agent::Copilot,
        tool: p.tool_name,
        summary,
        cwd: p.cwd,
        risk,
    })
}

/// Resumo legível a partir da entrada da ferramenta. Para um shell, isso é o
/// comando; senão, o JSON compacto da entrada.
fn summarize(input: &serde_json::Value) -> String {
    if let Some(cmd) = input.get("command").and_then(|v| v.as_str()) {
        return cmd.to_string();
    }
    if let Some(s) = input.as_str() {
        return s.to_string();
    }
    input.to_string()
}

/// Formata a decisão do CHRIS no formato que o Copilot espera.
///
/// - `Allow`  -> `{"permissionDecision":"allow", ...}`
/// - `Deny`   -> `{"permissionDecision":"deny", ...}`
/// - `Defer`  -> `{}` (saída vazia = cai no prompt nativo do Copilot)
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
        // Defer: saída vazia faz o Copilot usar o fluxo de permissão dele.
        Decision::Defer => "{}".to_string(),
    };
    AgentResponse { stdout, exit_code: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chris_core::Risk;

    const SAMPLE: &str = r#"{
        "hook_event_name": "preToolUse",
        "session_id": "abc",
        "cwd": "/home/dev/proj",
        "tool_name": "shell",
        "tool_input": { "command": "rm -rf build/" }
    }"#;

    #[test]
    fn parse_copilot_shell() {
        let req = parse_copilot(SAMPLE, ReqId(1)).unwrap();
        assert_eq!(req.agent, Agent::Copilot);
        assert_eq!(req.tool, "shell");
        assert_eq!(req.summary, "rm -rf build/");
        assert_eq!(req.cwd, "/home/dev/proj");
        assert_eq!(req.risk, Risk::High); // rm -rf => alto risco
    }

    #[test]
    fn format_allow_deny_defer() {
        let allow = format_copilot(Decision::Allow, "ok");
        assert!(allow.stdout.contains("\"permissionDecision\":\"allow\""));
        assert_eq!(allow.exit_code, 0);

        let deny = format_copilot(Decision::Deny, "negado");
        assert!(deny.stdout.contains("\"permissionDecision\":\"deny\""));

        let defer = format_copilot(Decision::Defer, "");
        assert_eq!(defer.stdout, "{}");
    }
}
