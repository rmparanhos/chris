// No Windows, em release, esconde a janela preta de console que abriria junto.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // --- menu da bandeja (clique direito no ícone) ---
            let mostrar = MenuItem::with_id(app, "toggle", "Mostrar/ocultar blob", true, None::<&str>)?;
            let sair = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&mostrar, &sair])?;

            // --- ícone na bandeja do sistema ---
            // usa o mesmo ícone do app (o blob ciano que geramos).
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
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o CHRIS");
}
