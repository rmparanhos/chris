// No Windows, em release, esconde a janela preta de console que abriria junto.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Daemon do CHRIS (Tauri).
//!
//! - Mostra o blob (janela transparente) e um ícone na bandeja.
//! - Escuta o cano IPC. Quando um pedido de aprovação chega:
//!     1. o blob vai para "alerta";
//!     2. abre o popup com os detalhes;
//!     3. espera o clique (Allow/Deny) ou o timeout (-> Deny);
//!     4. responde a decisão de volta para o hook.

use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::time::Duration;

use chris_core::{Decision, DecisionMsg, Msg};
use chris_transport_ipc as transport;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};

/// Mesmo timeout do hook (deny se ninguém responder a tempo).
const TIMEOUT_SECS: u64 = 120;

/// Estado compartilhado: o "canal" para entregar a decisão do pedido atual.
#[derive(Default)]
struct AppState {
    /// (id do pedido atual, remetente para acordar quem está esperando)
    current: Mutex<Option<(u32, Sender<Decision>)>>,
}

/// Comando chamado pelos botões do popup (via JS).
#[tauri::command]
fn decide(state: State<AppState>, id: u32, allow: bool) {
    if let Some((cur_id, tx)) = state.current.lock().unwrap().as_ref() {
        if *cur_id == id {
            let _ = tx.send(if allow { Decision::Allow } else { Decision::Deny });
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![decide])
        .setup(|app| {
            setup_tray(app.handle())?;

            // sobe o servidor IPC numa thread de fundo
            let handle = app.handle().clone();
            std::thread::spawn(move || ipc_loop(handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o CHRIS");
}

/// Monta o ícone e o menu da bandeja.
fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let mostrar = MenuItem::with_id(app, "toggle", "Mostrar/ocultar blob", true, None::<&str>)?;
    let sair = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&mostrar, &sair])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("CHRIS — idle")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "toggle" => {
                if let Some(win) = app.get_webview_window("blob") {
                    let visivel = win.is_visible().unwrap_or(false);
                    let _ = if visivel { win.hide() } else { win.show() };
                }
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

/// Laço do servidor IPC: aceita conexões e trata um pedido por vez (fila).
fn ipc_loop(app: AppHandle) {
    let listener = match transport::listen() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("CHRIS: não consegui abrir o cano IPC: {e}");
            return;
        }
    };
    loop {
        match transport::accept(&listener) {
            Ok(mut conn) => {
                // lê o pedido
                let req = match transport::read_msg(&mut conn) {
                    Ok(Msg::Request(r)) => r,
                    _ => continue, // mensagem inesperada: ignora
                };
                let decision = handle_request(&app, &req);
                // responde de volta para o hook (ignora se o hook já saiu)
                let _ = transport::write_msg(
                    &mut conn,
                    &Msg::Decision(DecisionMsg {
                        id: req.id,
                        decision,
                        reason: String::new(),
                    }),
                );
            }
            Err(_) => continue,
        }
    }
}

/// Mostra o pedido (blob + popup) e espera a decisão (ou timeout = Deny).
fn handle_request(app: &AppHandle, req: &chris_core::ApprovalRequest) -> Decision {
    // blob -> alerta
    let _ = app.emit_to("blob", "blob-state", serde_json::json!({"state":"alert","count":1}));

    // popup -> mostra com os detalhes
    let _ = app.emit_to(
        "popup",
        "approval",
        serde_json::json!({
            "id": req.id.0,
            "agent": format!("{:?}", req.agent),
            "tool": req.tool,
            "summary": req.summary,
            "cwd": req.cwd,
            "risk": format!("{:?}", req.risk).to_lowercase(),
        }),
    );
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.show();
        let _ = win.set_focus();
    }

    // registra o canal de decisão e espera
    let (tx, rx) = mpsc::channel();
    {
        let state = app.state::<AppState>();
        *state.current.lock().unwrap() = Some((req.id.0, tx));
    }
    let decision = rx
        .recv_timeout(Duration::from_secs(TIMEOUT_SECS))
        .unwrap_or(Decision::Deny);
    {
        let state = app.state::<AppState>();
        *state.current.lock().unwrap() = None;
    }

    // feedback visual e fecha o popup
    let visual = match decision {
        Decision::Allow => "approved",
        _ => "denied",
    };
    let _ = app.emit_to("blob", "blob-state", serde_json::json!({"state":visual,"count":0}));
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.hide();
    }
    std::thread::sleep(Duration::from_millis(1200));
    let _ = app.emit_to("blob", "blob-state", serde_json::json!({"state":"idle","count":0}));

    decision
}
