//! CHRIS local IPC transport.
//!
//! This is one of the "edges": it takes the `core`'s `Msg`s and moves them
//! between two processes on the same machine — the `hook` (client) and the
//! `daemon` (server). `interprocess` handles the difference between a named
//! pipe (Windows) and a unix socket (Linux/macOS), so the same code works on
//! all three.
//!
//! Framing: each message is sent with a 4-byte length prefix, followed by the
//! `core`'s bytes (1 version byte + postcard).

use std::io::{self, Read, Write};

use chris_core::{decode, encode, Msg};
use interprocess::local_socket::{
    prelude::*, GenericNamespaced, ListenerOptions, Stream,
};

/// Name of the "pipe". On Windows it becomes `\\.\pipe\chris-companion`; on
/// Linux, a socket in the abstract namespace. The same identifier on both sides.
pub const SOCKET_NAME: &str = "chris-companion.sock";

fn socket_name() -> io::Result<interprocess::local_socket::Name<'static>> {
    SOCKET_NAME.to_ns_name::<GenericNamespaced>()
}

/// Server side (daemon): opens the pipe and listens for connections.
pub fn listen() -> io::Result<interprocess::local_socket::Listener> {
    ListenerOptions::new().name(socket_name()?).create_sync()
}

/// Accepts the next connection. A wrapper so the caller doesn't need to import
/// the `interprocess` trait.
pub fn accept(listener: &interprocess::local_socket::Listener) -> io::Result<Stream> {
    listener.accept()
}

/// Client side (hook): connects to the daemon's pipe.
pub fn connect() -> io::Result<Stream> {
    Stream::connect(socket_name()?)
}

/// Sends a message (with a length prefix).
pub fn write_msg<W: Write>(w: &mut W, msg: &Msg) -> io::Result<()> {
    let bytes = encode(msg).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "encode"))?;
    let len = (bytes.len() as u32).to_be_bytes();
    w.write_all(&len)?;
    w.write_all(&bytes)?;
    w.flush()
}

/// Reads a message (reads the length, then the content).
pub fn read_msg<R: Read>(r: &mut R) -> io::Result<Msg> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let n = u32::from_be_bytes(len) as usize;
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf)?;
    decode(&buf).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "decode"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chris_core::{Agent, ApprovalRequest, Decision, DecisionMsg, ReqId, Risk};
    use std::thread;

    #[test]
    fn roundtrip_over_socket() {
        let listener = listen().expect("abrir o cano");

        // server: accepts 1 connection, reads the request, responds Allow
        let server = thread::spawn(move || {
            let mut conn = listener.accept().expect("aceitar conexão");
            let got = read_msg(&mut conn).expect("ler pedido");
            let id = match got {
                Msg::Request(r) => r.id,
                _ => panic!("esperava um Request"),
            };
            let resp = Msg::Decision(DecisionMsg {
                id,
                decision: Decision::Allow,
                reason: "teste".into(),
            });
            write_msg(&mut conn, &resp).expect("responder");
        });

        // client: connects, sends the request, reads the decision
        let mut client = connect().expect("conectar");
        let req = Msg::Request(ApprovalRequest {
            id: ReqId(7),
            agent: Agent::Copilot,
            tool: "shell".into(),
            summary: "ls -la".into(),
            cwd: "/proj".into(),
            risk: Risk::Low,
        });
        write_msg(&mut client, &req).expect("enviar");
        let resp = read_msg(&mut client).expect("ler resposta");

        match resp {
            Msg::Decision(d) => {
                assert_eq!(d.id, ReqId(7));
                assert_eq!(d.decision, Decision::Allow);
            }
            _ => panic!("esperava uma Decision"),
        }
        server.join().unwrap();
    }
}
