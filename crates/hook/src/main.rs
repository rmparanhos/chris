//! CLI do CHRIS (`chris`).
//!
//! Dois modos:
//!   * `chris hook --agent copilot`    -> chamado pelo agente no `preToolUse`.
//!         Lê o payload do stdin, pergunta ao daemon e devolve a decisão.
//!   * `chris install --agent copilot` -> escreve a config de hook do agente.

use std::io::{Read, Write};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chris_adapters::{format_copilot, parse_copilot};
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
         chris hook    --agent <copilot>   (chamado pelo agente no preToolUse)\n  \
         chris install --agent <copilot>   (instala a config de hook do agente)\n"
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
    if agent != "copilot" {
        // MVP só fala Copilot. Não bloqueia: defere ao agente.
        print!("{{}}");
        return;
    }

    // 1) lê o payload do agente no stdin
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        print!("{{}}"); // sem payload legível -> defere
        return;
    }

    // 2) id único do pedido (nanos do relógio, truncado)
    let id = ReqId(now_nanos() as u32);

    // 3) traduz para o formato neutro
    let req = match parse_copilot(&payload, id) {
        Ok(r) => r,
        Err(_) => {
            print!("{{}}"); // não entendi o payload -> defere
            return;
        }
    };

    // 4) pergunta ao daemon, com timeout
    let decision = ask_daemon(req);

    // 5) responde ao agente no formato do Copilot
    let reason = match decision {
        Decision::Allow => "aprovado pelo usuário no CHRIS",
        Decision::Deny => "negado/sem resposta no CHRIS",
        Decision::Defer => "",
    };
    let resp = format_copilot(decision, reason);
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
    if agent != "copilot" {
        eprintln!("install: por enquanto só `--agent copilot` é suportado.");
        std::process::exit(2);
    }

    // caminho absoluto do próprio binário, para o agente achar o `chris`
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "chris".to_string());
    let invoke = format!("{exe} hook --agent copilot");

    // Config do Copilot: .github/hooks/chris.json
    // `timeoutSec` BEM maior que o nosso timeout interno, para o nosso `Deny`
    // por inatividade chegar antes de o Copilot desistir.
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
    let pretty = serde_json::to_string_pretty(&config).unwrap();
    if let Err(e) = std::fs::write(&path, pretty) {
        eprintln!("install: não consegui escrever {}: {e}", path.display());
        std::process::exit(1);
    }

    println!("Hook do Copilot instalado em {}", path.display());
    println!("Comando do hook: {invoke}");
    println!("Lembre: o daemon (companiond) precisa estar rodando para o blob reagir.");
}
