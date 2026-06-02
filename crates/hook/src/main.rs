//! CLI do CHRIS (`chris`).
//!
//! Dois modos:
//!   * `chris hook --agent copilot`    -> chamado pelo agente no `preToolUse`.
//!         Lê o payload do stdin, pergunta ao daemon e devolve a decisão.
//!   * `chris install --agent copilot` -> escreve a config de hook do agente.

use std::io::{Read, Write};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chris_adapters::{format_claude, format_copilot, parse_claude, parse_copilot};
use chris_core::{Decision, Msg, ReqId};
use chris_transport_ipc as transport;

/// Nosso timeout interno (deny se o usuário não responder). Tem que ser MENOR
/// que o `timeoutSec` configurado no agente, para respondermos antes de ele
/// desistir e ignorar o hook. Ajustável via env `CHRIS_TIMEOUT_SECS`.
const DEFAULT_TIMEOUT_SECS: u64 = 120;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str);
    match cmd {
        Some("hook") => run_hook(&args),
        Some("install") => run_install(&args),
        Some("--help") | Some("-h") | None => print_help(),
        Some(other) => {
            eprintln!("subcomando desconhecido: {other}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    eprintln!(
        "CHRIS — Coding-agent Hook Review Interactive Sidekick\n\n\
         Uso:\n  \
         chris hook    --agent <copilot|claude>   (chamado pelo agente no PreToolUse)\n  \
         chris install --agent <copilot|claude>   (instala a config de hook do agente)\n"
    );
}

/// Lê `--agent <x>` dos argumentos (default: copilot).
fn agent_arg(args: &[String]) -> String {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--agent" {
            if let Some(v) = it.next() {
                return v.clone();
            }
        }
    }
    "copilot".to_string()
}

// ---------------------------------------------------------------------------
// chris hook
// ---------------------------------------------------------------------------

fn run_hook(args: &[String]) {
    let agent = agent_arg(args);
    // resposta de "deixa passar" no formato de cada agente (usada em erros)
    let passthrough = if agent == "claude" { "" } else { "{}" };
    if agent != "copilot" && agent != "claude" {
        print!("{passthrough}");
        return;
    }

    // 1) lê o payload do agente no stdin
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        print!("{passthrough}");
        return;
    }

    // 2) id único do pedido (nanos do relógio, truncado)
    let id = ReqId(now_nanos() as u32);

    // 3) traduz para o formato neutro (parsing é igual nos dois agentes)
    let parsed = if agent == "claude" {
        parse_claude(&payload, id)
    } else {
        parse_copilot(&payload, id)
    };
    let req = match parsed {
        Ok(r) => r,
        Err(_) => {
            print!("{passthrough}");
            return;
        }
    };

    // 4) pergunta ao daemon, com timeout
    let decision = ask_daemon(req);

    // 5) responde no formato do agente
    let reason = match decision {
        Decision::Allow => "aprovado pelo usuário no CHRIS",
        Decision::Deny => "negado/sem resposta no CHRIS",
        Decision::Defer => "",
    };
    let resp = if agent == "claude" {
        format_claude(decision, reason)
    } else {
        format_copilot(decision, reason)
    };
    print!("{}", resp.stdout);
    let _ = std::io::stdout().flush();
    std::process::exit(resp.exit_code);
}

/// Conversa com o daemon numa thread e aplica a política de timeout.
///
/// - daemon ausente / erro de I/O -> `Defer` (cai no prompt nativo do agente)
/// - daemon presente mas sem resposta a tempo -> `Deny` (fail-safe)
fn ask_daemon(req: chris_core::ApprovalRequest) -> Decision {
    let timeout = std::env::var("CHRIS_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| -> std::io::Result<Decision> {
            let mut conn = transport::connect()?;
            transport::write_msg(&mut conn, &Msg::Request(req))?;
            match transport::read_msg(&mut conn)? {
                Msg::Decision(d) => Ok(d.decision),
                _ => Ok(Decision::Defer),
            }
        })();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(timeout)) {
        Ok(Ok(d)) => d,             // o daemon respondeu
        Ok(Err(_)) => Decision::Defer, // não deu pra falar com o daemon
        Err(_) => Decision::Deny,   // timeout: ninguém respondeu a tempo
    }
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// chris install
// ---------------------------------------------------------------------------

fn run_install(args: &[String]) {
    let agent = agent_arg(args);

    // caminho absoluto do próprio binário (citado, por causa de espaços no Windows)
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "chris".to_string());
    let exe_q = if exe.contains(' ') { format!("\"{exe}\"") } else { exe };

    match agent.as_str() {
        "copilot" => install_copilot(&format!("{exe_q} hook --agent copilot")),
        "claude" => install_claude(&format!("{exe_q} hook --agent claude")),
        _ => {
            eprintln!("install: use  --agent copilot  ou  --agent claude.");
            std::process::exit(2);
        }
    }
}

/// Copilot: escreve `.github/hooks/chris.json`.
fn install_copilot(invoke: &str) {
    // `timeoutSec` BEM maior que o nosso timeout interno, para o nosso `Deny`
    // por inatividade chegar antes de o Copilot desistir do hook.
    let config = serde_json::json!({
        "version": 1,
        "hooks": {
            "preToolUse": [
                {
                    "type": "command",
                    "bash": invoke,
                    "powershell": invoke,
                    "timeoutSec": 600,
                    "comment": "CHRIS — aprovação via companion"
                }
            ]
        }
    });

    let dir = std::path::Path::new(".github").join("hooks");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("install: não consegui criar {}: {e}", dir.display());
        std::process::exit(1);
    }
    let path = dir.join("chris.json");
    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&config).unwrap()) {
        eprintln!("install: não consegui escrever {}: {e}", path.display());
        std::process::exit(1);
    }
    println!("Hook do Copilot instalado em {}", path.display());
    println!("Comando do hook: {invoke}");
    println!("Lembre: o daemon (companiond) precisa estar rodando para o blob reagir.");
}

/// Claude Code: MESCLA o hook em `.claude/settings.json` (não apaga o resto).
fn install_claude(invoke: &str) {
    let dir = std::path::Path::new(".claude");
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("install: não consegui criar {}: {e}", dir.display());
        std::process::exit(1);
    }
    let path = dir.join("settings.json");

    // lê o settings existente (ou começa um objeto vazio)
    let mut root: serde_json::Value = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if !root.is_object() {
        root = serde_json::json!({});
    }

    // garante hooks.PreToolUse como array
    let obj = root.as_object_mut().unwrap();
    let hooks = obj.entry("hooks").or_insert_with(|| serde_json::json!({}));
    if !hooks.is_object() {
        *hooks = serde_json::json!({});
    }
    let pre = hooks
        .as_object_mut()
        .unwrap()
        .entry("PreToolUse")
        .or_insert_with(|| serde_json::json!([]));
    if !pre.is_array() {
        *pre = serde_json::json!([]);
    }
    let arr = pre.as_array_mut().unwrap();

    // evita duplicar se já instalado
    let exists = arr.iter().any(|e| {
        e.get("hooks").and_then(|h| h.as_array()).map_or(false, |hs| {
            hs.iter()
                .any(|c| c.get("command").and_then(|x| x.as_str()) == Some(invoke))
        })
    });
    if !exists {
        arr.push(serde_json::json!({
            "matcher": "*",
            "hooks": [ { "type": "command", "command": invoke, "timeout": 600 } ]
        }));
    }

    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&root).unwrap()) {
        eprintln!("install: não consegui escrever {}: {e}", path.display());
        std::process::exit(1);
    }
    println!("Hook do Claude Code instalado/atualizado em {}", path.display());
    println!("Comando do hook: {invoke}");
    println!("Lembre: o daemon (companiond) precisa estar rodando para o blob reagir.");
}
