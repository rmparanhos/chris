//! Core (brain) of CHRIS.
//!
//! This is where the PURE logic lives: the data types, the format of the
//! messages that travel over the wire, and the decision (risk) rules. No I/O,
//! networking, Tauri or async — that stays at the "edges". Being `no_std`, this
//! same code compiles on the PC and on the ESP32 microcontroller.

// `#![no_std]` = "don't depend on the standard library (std)". std assumes an
// operating system (files, threads, networking). The ESP32 doesn't have that.
#![no_std]

// But we still want types that allocate memory (String, Vec). They live in the
// `alloc` crate, which works without an operating system. So we pull it in:
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Wire protocol version. Firmware (ESP32) and daemon (PC) may update at
/// different paces; this byte makes the mismatch detectable.
pub const PROTOCOL_VERSION: u8 = 1;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Which coding agent originated the request.
// `derive(...)` makes the compiler generate code automatically: compare (`Eq`),
// copy, print for debug, and (de)serialize (`Serialize`/`Deserialize`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agent {
    Copilot,
    Claude,
    Codex,
}

/// The response to an approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// May execute.
    Allow,
    /// May not (e.g.: user denied, or timeout — fail-safe).
    Deny,
    /// CHRIS doesn't decide; the agent falls back to its native prompt
    /// (e.g.: when the daemon isn't running).
    Defer,
}

/// How risky the action appears. Shown in the popup to help the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Risk {
    Low,
    Medium,
    High,
}

/// Identifier for a request. A simple `u32` — lightweight and works on the
/// ESP32 (a UUID would be overkill to correlate question and answer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReqId(pub u32);

/// Visual state of the blob, driven by the brain and rendered by the body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlobState {
    Idle,
    Alert,
    Approved,
    Denied,
}

/// An already-normalized approval request (the adapter converted the agent's
/// payload into this). The brain only knows this format — hence "agnostic".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: ReqId,
    pub agent: Agent,
    /// Tool name (e.g.: "shell", "write_file").
    pub tool: String,
    /// Human-readable summary (e.g.: the command that will be executed).
    pub summary: String,
    /// The agent's working directory.
    pub cwd: String,
    pub risk: Risk,
}

/// The decision sent back to the hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionMsg {
    pub id: ReqId,
    pub decision: Decision,
    pub reason: String,
}

/// Everything that travels over the wire between hook ⇄ daemon (and, in the future, PC ⇄ ESP32).
// `enum` here is a "union": a `Msg` is EITHER a request OR a decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    Request(ApprovalRequest),
    Decision(DecisionMsg),
}

// ---------------------------------------------------------------------------
// Wire protocol (serialization)
// ---------------------------------------------------------------------------

/// Possible errors when decoding received bytes.
// No `thiserror`/`anyhow` (which need std) — a simple enum is enough.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// No bytes came in.
    Empty,
    /// The version byte doesn't match ours.
    UnsupportedVersion(u8),
    /// postcard couldn't interpret the content.
    Decode,
}

/// Turns a `Msg` into bytes ready to send.
/// The first byte is always the protocol version.
pub fn encode(msg: &Msg) -> Result<Vec<u8>, postcard::Error> {
    let mut buf = postcard::to_allocvec(msg)?;
    buf.insert(0, PROTOCOL_VERSION); // prefix the version
    Ok(buf)
}

/// Reads received bytes back into a `Msg`, validating the version.
pub fn decode(bytes: &[u8]) -> Result<Msg, WireError> {
    // split the first byte (version) from the rest (content)
    let (&version, rest) = bytes.split_first().ok_or(WireError::Empty)?;
    if version != PROTOCOL_VERSION {
        return Err(WireError::UnsupportedVersion(version));
    }
    postcard::from_bytes(rest).map_err(|_| WireError::Decode)
}

// ---------------------------------------------------------------------------
// Decision rules: risk assessment
// ---------------------------------------------------------------------------

/// Simple risk heuristic based on the command/action.
/// (MVP default — will be refined later.)
pub fn assess_risk(command: &str) -> Risk {
    // We always compare in lowercase so it doesn't depend on case.
    let c = to_lower(command);

    // High-risk signs: delete, privilege, format, download-and-execute.
    const HIGH: [&str; 8] = [
        "rm -rf", "rm -r", " del ", "rmdir", "sudo ", "mkfs", "dd if=", ":(){",
    ];
    let baixa_e_executa =
        (c.contains("curl") || c.contains("wget")) && (c.contains("| sh") || c.contains("|sh"));
    if baixa_e_executa || HIGH.iter().any(|p| c.contains(p)) {
        return Risk::High;
    }

    // Medium risk: writes/moves/installs things.
    const MEDIUM: [&str; 6] = ["git push", "npm install", "pip install", " mv ", " cp ", " > "];
    if MEDIUM.iter().any(|p| c.contains(p)) {
        return Risk::Medium;
    }

    Risk::Low
}

/// Lowercase without depending on `std` (`str::to_lowercase` needs alloc/std
/// for Unicode; here ASCII suffices, which is the case for shell commands).
fn to_lower(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        out.push(ch.to_ascii_lowercase());
    }
    out
}

// ---------------------------------------------------------------------------
// Edge abstractions (implemented on the PC and, later, on the ESP32)
// ---------------------------------------------------------------------------

/// How messages travel. On the PC = named pipe; on the ESP32 = Wi-Fi/BLE/serial.
/// The brain talks to this without knowing what the medium is.
pub trait Transport {
    /// Error type specific to each implementation.
    type Error;
    fn send(&mut self, msg: &Msg) -> Result<(), Self::Error>;
    /// Returns `Ok(None)` when nothing has arrived yet.
    fn recv(&mut self) -> Result<Option<Msg>, Self::Error>;
}

/// How the "body" presents itself. On the PC = blob webview; on the ESP32 = screen + buttons.
pub trait Presentation {
    /// Changes the blob's animation.
    fn react(&mut self, state: BlobState);
    /// Shows the details of a request.
    fn show(&mut self, req: &ApprovalRequest);
    /// Reads user input, if any (button click / physical button).
    fn poll_input(&mut self) -> Option<Decision>;
}

// ---------------------------------------------------------------------------
// Tests (run with `cargo test`)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn roundtrip_request() {
        // Create a request, encode, decode and check that it came back identical.
        let original = Msg::Request(ApprovalRequest {
            id: ReqId(42),
            agent: Agent::Copilot,
            tool: "shell".to_string(),
            summary: "rm -rf build/".to_string(),
            cwd: "/proj".to_string(),
            risk: Risk::High,
        });
        let bytes = encode(&original).unwrap();
        assert_eq!(bytes[0], PROTOCOL_VERSION); // version at the front
        assert_eq!(decode(&bytes).unwrap(), original);
    }

    #[test]
    fn rejects_wrong_version() {
        let bytes = [99u8, 0, 0]; // version 99 doesn't exist
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
