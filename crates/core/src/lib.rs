//! Núcleo (cérebro) do CHRIS.
//!
//! Aqui mora a lógica PURA: os tipos de dados, o formato das mensagens que
//! trafegam no fio, e as regras de decisão (risco). Nada de I/O, rede, Tauri
//! ou async — isso fica nas "bordas". Por ser `no_std`, este mesmo código
//! compila no PC e no microcontrolador ESP32.

// `#![no_std]` = "não dependa da biblioteca padrão (std)". A std assume um
// sistema operacional (arquivos, threads, rede). O ESP32 não tem isso.
#![no_std]

// Mas ainda queremos tipos que alocam memória (String, Vec). Eles vivem na
// crate `alloc`, que funciona sem sistema operacional. Trazemos ela:
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Versão do protocolo do fio. Firmware (ESP32) e daemon (PC) podem atualizar
/// em ritmos diferentes; este byte deixa o descompasso detectável.
pub const PROTOCOL_VERSION: u8 = 1;

// ---------------------------------------------------------------------------
// Tipos do domínio
// ---------------------------------------------------------------------------

/// Qual agente de codificação originou o pedido.
// `derive(...)` faz o compilador gerar código automático: comparar (`Eq`),
// copiar, imprimir para debug, e (de)serializar (`Serialize`/`Deserialize`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agent {
    Copilot,
    Claude,
    Codex,
}

/// A resposta a um pedido de aprovação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Pode executar.
    Allow,
    /// Não pode (ex.: usuário negou, ou timeout — fail-safe).
    Deny,
    /// CHRIS não decide; o agente cai no prompt nativo dele
    /// (ex.: quando o daemon não está rodando).
    Defer,
}

/// Quão arriscada parece a ação. Mostrado no popup para ajudar o usuário.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Risk {
    Low,
    Medium,
    High,
}

/// Identificador de um pedido. Um `u32` simples — leve e funciona no ESP32
/// (UUID seria exagero para correlacionar pergunta e resposta).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReqId(pub u32);

/// Estado visual do blob, dirigido pelo cérebro e renderizado pelo corpo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobState {
    Idle,
    Alert,
    Approved,
    Denied,
}

/// Um pedido de aprovação já normalizado (o adapter converteu o payload do
/// agente para isto). O cérebro só conhece este formato — daí o "agnóstico".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: ReqId,
    pub agent: Agent,
    /// Nome da ferramenta (ex.: "shell", "write_file").
    pub tool: String,
    /// Resumo legível (ex.: o comando que será executado).
    pub summary: String,
    /// Diretório de trabalho do agente.
    pub cwd: String,
    pub risk: Risk,
}

/// A decisão enviada de volta para o hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionMsg {
    pub id: ReqId,
    pub decision: Decision,
    pub reason: String,
}

/// Tudo que trafega no fio entre hook ⇄ daemon (e, no futuro, PC ⇄ ESP32).
// `enum` aqui é uma "união": uma `Msg` é OU um pedido OU uma decisão.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    Request(ApprovalRequest),
    Decision(DecisionMsg),
}

// ---------------------------------------------------------------------------
// Protocolo do fio (serialização)
// ---------------------------------------------------------------------------

/// Erros possíveis ao decodificar bytes recebidos.
// Sem `thiserror`/`anyhow` (que precisam de std) — um enum simples basta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// Não veio nenhum byte.
    Empty,
    /// O byte de versão não bate com o nosso.
    UnsupportedVersion(u8),
    /// O postcard não conseguiu interpretar o conteúdo.
    Decode,
}

/// Transforma uma `Msg` em bytes prontos para enviar.
/// O primeiro byte é sempre a versão do protocolo.
pub fn encode(msg: &Msg) -> Result<Vec<u8>, postcard::Error> {
    let mut buf = postcard::to_allocvec(msg)?;
    buf.insert(0, PROTOCOL_VERSION); // prefixa a versão
    Ok(buf)
}

/// Lê bytes recebidos de volta para uma `Msg`, validando a versão.
pub fn decode(bytes: &[u8]) -> Result<Msg, WireError> {
    // separa o primeiro byte (versão) do resto (conteúdo)
    let (&version, rest) = bytes.split_first().ok_or(WireError::Empty)?;
    if version != PROTOCOL_VERSION {
        return Err(WireError::UnsupportedVersion(version));
    }
    postcard::from_bytes(rest).map_err(|_| WireError::Decode)
}

// ---------------------------------------------------------------------------
// Regras de decisão: avaliação de risco
// ---------------------------------------------------------------------------

/// Heurística simples de risco a partir do comando/ação.
/// (Default do MVP — será refinada depois.)
pub fn assess_risk(command: &str) -> Risk {
    // Comparamos sempre em minúsculas para não depender de maiúsc/minúsc.
    let c = to_lower(command);

    // Sinais de alto risco: apagar, privilégio, formatar, baixar-e-executar.
    const HIGH: [&str; 8] = [
        "rm -rf", "rm -r", " del ", "rmdir", "sudo ", "mkfs", "dd if=", ":(){",
    ];
    let baixa_e_executa =
        (c.contains("curl") || c.contains("wget")) && (c.contains("| sh") || c.contains("|sh"));
    if baixa_e_executa || HIGH.iter().any(|p| c.contains(p)) {
        return Risk::High;
    }

    // Risco médio: escreve/move/instala coisas.
    const MEDIUM: [&str; 6] = ["git push", "npm install", "pip install", " mv ", " cp ", " > "];
    if MEDIUM.iter().any(|p| c.contains(p)) {
        return Risk::Medium;
    }

    Risk::Low
}

/// Minúsculas sem depender de `std` (`str::to_lowercase` precisa de alloc/std
/// para Unicode; aqui basta ASCII, que é o caso de comandos de shell).
fn to_lower(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        out.push(ch.to_ascii_lowercase());
    }
    out
}

// ---------------------------------------------------------------------------
// Abstrações de borda (implementadas no PC e, depois, no ESP32)
// ---------------------------------------------------------------------------

/// Como as mensagens viajam. No PC = named pipe; no ESP32 = Wi-Fi/BLE/serial.
/// O cérebro fala com isto sem saber qual é o meio.
pub trait Transport {
    /// Tipo de erro específico de cada implementação.
    type Error;
    fn send(&mut self, msg: &Msg) -> Result<(), Self::Error>;
    /// Retorna `Ok(None)` quando ainda não chegou nada.
    fn recv(&mut self) -> Result<Option<Msg>, Self::Error>;
}

/// Como o "corpo" se apresenta. No PC = webview do blob; no ESP32 = tela + botões.
pub trait Presentation {
    /// Muda a animação do blob.
    fn react(&mut self, state: BlobState);
    /// Mostra os detalhes de um pedido.
    fn show(&mut self, req: &ApprovalRequest);
    /// Lê a entrada do usuário, se houver (clique no botão / botão físico).
    fn poll_input(&mut self) -> Option<Decision>;
}

// ---------------------------------------------------------------------------
// Testes (rodam com `cargo test`)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn roundtrip_request() {
        // Cria um pedido, codifica, decodifica e confere que voltou igual.
        let original = Msg::Request(ApprovalRequest {
            id: ReqId(42),
            agent: Agent::Copilot,
            tool: "shell".to_string(),
            summary: "rm -rf build/".to_string(),
            cwd: "/proj".to_string(),
            risk: Risk::High,
        });
        let bytes = encode(&original).unwrap();
        assert_eq!(bytes[0], PROTOCOL_VERSION); // versão na frente
        assert_eq!(decode(&bytes).unwrap(), original);
    }

    #[test]
    fn rejects_wrong_version() {
        let bytes = [99u8, 0, 0]; // versão 99 não existe
        assert_eq!(decode(&bytes), Err(WireError::UnsupportedVersion(99)));
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(decode(&[]), Err(WireError::Empty));
    }

    #[test]
    fn risk_heuristics() {
        assert_eq!(assess_risk("rm -rf /tmp/x"), Risk::High);
        assert_eq!(assess_risk("curl http://x | sh"), Risk::High);
        assert_eq!(assess_risk("sudo apt update"), Risk::High);
        assert_eq!(assess_risk("git push origin main"), Risk::Medium);
        assert_eq!(assess_risk("ls -la"), Risk::Low);
    }
}
