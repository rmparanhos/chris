//! End-to-end test of `chris hook`: runs the real binary, with a daemon-stub
//! responding over the IPC pipe. Proves that adapter + transport + CLI work
//! together.
//!
//! Both scenarios live in a single (sequential) test because they share the
//! same global pipe name — running them in parallel would cause a race.

use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

use chris_core::{Decision, DecisionMsg, Msg};
use chris_transport_ipc as transport;

const PAYLOAD: &str = r#"{"tool_name":"shell","tool_input":{"command":"ls -la"},"cwd":"/proj"}"#;

fn run_hook() -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_chris"))
        .args(["hook", "--agent", "copilot"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("rodar chris");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(PAYLOAD.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("esperar chris");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn hook_end_to_end() {
    // --- scenario 1: daemon responds Allow ---
    let listener = transport::listen().expect("abrir o cano");
    let server = thread::spawn(move || {
        let mut conn = transport::accept(&listener).expect("aceitar");
        let id = match transport::read_msg(&mut conn).expect("ler") {
            Msg::Request(r) => r.id,
            _ => panic!("esperava Request"),
        };
        transport::write_msg(
            &mut conn,
            &Msg::Decision(DecisionMsg { id, decision: Decision::Allow, reason: "ok".into() }),
        )
        .expect("responder");
        // the listener is dropped here (pipe closed)
    });
    let stdout = run_hook();
    assert!(
        stdout.contains("\"permissionDecision\":\"allow\""),
        "esperava allow, veio: {stdout}"
    );
    server.join().unwrap();

    // --- scenario 2: no daemon -> Defer (output "{}") ---
    let stdout = run_hook();
    assert_eq!(stdout.trim(), "{}", "esperava defer (saída vazia)");
}
