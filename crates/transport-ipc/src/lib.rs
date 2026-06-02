//! Transporte IPC local do CHRIS.
//!
//! É uma das "bordas": pega as `Msg` do `core` e as faz trafegar entre dois
//! processos na mesma máquina — o `hook` (cliente) e o `daemon` (servidor).
//! `interprocess` cuida da diferença entre named pipe (Windows) e unix
//! socket (Linux/macOS), então o mesmo código serve nos três.
//!
//! Enquadramento ("framing"): cada mensagem vai com um prefixo de 4 bytes
//! (tamanho), seguido dos bytes do `core` (1 byte de versão + postcard).

use std::io::{self, Read, Write};

use chris_core::{decode, encode, Msg};
use interprocess::local_socket::{
    prelude::*, GenericNamespaced, ListenerOptions, Stream,
};

/// Nome do "cano". No Windows vira `\\.\pipe\chris-companion`; no Linux, um
/// socket no namespace abstrato. O mesmo identificador nos dois lados.
pub const SOCKET_NAME: &str = "chris-companion.sock";

fn socket_name() -> io::Result<interprocess::local_socket::Name<'static>> {
    SOCKET_NAME.to_ns_name::<GenericNamespaced>()
}

/// Lado servidor (daemon): abre o cano e fica escutando conexões.
pub fn listen() -> io::Result<interprocess::local_socket::Listener> {
    ListenerOptions::new().name(socket_name()?).create_sync()
}

/// Aceita a próxima conexão. Wrapper para o chamador não precisar importar o
/// trait do `interprocess`.
pub fn accept(listener: &interprocess::local_socket::Listener) -> io::Result<Stream> {
    listener.accept()
}

/// Lado cliente (hook): conecta no cano do daemon.
pub fn connect() -> io::Result<Stream> {
    Stream::connect(socket_name()?)
}

/// Envia uma mensagem (com prefixo de tamanho).
pub fn write_msg<W: Write>(w: &mut W, msg: &Msg) -> io::Result<()> {
    let bytes = encode(msg).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "encode"))?;
    let len = (bytes.len() as u32).to_be_bytes();
    w.write_all(&len)?;
    w.write_all(&bytes)?;
    w.flush()
}

/// Lê uma mensagem (lê o tamanho, depois o conteúdo).
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

        // servidor: aceita 1 conexão, lê o pedido, responde Allow
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

        // cliente: conecta, manda o pedido, lê a decisão
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
