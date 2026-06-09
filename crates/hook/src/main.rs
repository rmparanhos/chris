//! CHRIS CLI (`chris`).
//!
//! Two modes:
//!   * `chris hook --agent copilot`    -> called by the agent on `preToolUse`.
//!         Reads the payload from stdin, asks the daemon and returns the decision.
//!   * `chris install --agent copilot` -> writes the agent's hook config.

use std::io::{Read, Write};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chris_adapters::{format_claude, format_copilot, parse_claude, parse_copilot};
use chris_core::{Decision, Msg, ReqId};
use chris_transport_ipc as transport;

/// Our internal timeout (deny if the user doesn't respond). It must be SMALLER
/// than the `timeoutSec` configured in the agent, so we respond before it gives
/// up and ignores the hook. Adjustable via the `CHRIS_TIMEOUT_SECS` env var.
const DEFAULT_TIMEOUT_SECS: u64 = 120;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str);
    match cmd {
        Some("hook") => run_hook(&args),
        Some("install") => run_install(&args),
        Some("--help") | Some("-h") | None => print_help(),
        Some(other) => {
            eprintln!("unknown subcommand: {other}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    eprintln!(
        "CHRIS — Coding-agent Hook Review Interactive Sidekick\n\n\
         Usage:\n  \
         chris hook    --agent <copilot|claude>   (called by the agent on PreToolUse)\n  \
         chris install --agent <copilot|claude>   (installs the agent's hook config)\n"
    );
}

/// Reads `--agent <x>` from the arguments (default: copilot).
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
    // "let it pass" response in each agent's format (used on errors)
    let passthrough = if agent == "claude" { "" } else { "{}" };
    if agent != "copilot" && agent != "claude" {
        print!("{passthrough}");
        return;
    }

    // 1) read the agent's payload from stdin
    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        print!("{passthrough}");
        return;
    }

    // 2) unique request id (clock nanos, truncated)
    let id = ReqId(now_nanos() as u32);

    // 3) translate to the neutral format (parsing is the same for both agents)
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

    // 4) decide.
    //    Read-only tools (read/grep/list/search/get…) are deferred to the agent
    //    so the companion only interrupts you for actions with side effects
    //    (shell, file writes, MCP, unknown tools). Set CHRIS_ASK_ALL=1 to ask
    //    for everything (the previous behavior). Fail-safe: unknown -> ask.
    let decision = if !ask_all() && is_read_only(&req.tool) {
        Decision::Defer
    } else {
        ask_daemon(req)
    };

    // 5) respond in the agent's format
    let reason = match decision {
        Decision::Allow => "approved by the user in CHRIS",
        Decision::Deny => "denied / no response in CHRIS",
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

/// `CHRIS_ASK_ALL=1` (or `true`) brings back the "ask for everything" behavior,
/// bypassing the read-only filter below.
fn ask_all() -> bool {
    matches!(
        std::env::var("CHRIS_ASK_ALL").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

/// Splits a tool name into lowercase word tokens, breaking on non-alphanumerics
/// **and** on camelCase boundaries, so `get_file_contents`, `WebSearch` and
/// `NotebookRead` all tokenize into the words we match on.
fn tokenize(tool: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for c in tool.chars() {
        if c.is_alphanumeric() {
            if c.is_uppercase() && prev_lower && !cur.is_empty() {
                out.push(cur.to_lowercase());
                cur = String::new();
            }
            cur.push(c);
            prev_lower = c.is_lowercase();
        } else {
            if !cur.is_empty() {
                out.push(cur.to_lowercase());
                cur = String::new();
            }
            prev_lower = false;
        }
    }
    if !cur.is_empty() {
        out.push(cur.to_lowercase());
    }
    out
}

/// Heuristic: is this tool side-effect-free (so we can let the agent handle it
/// without interrupting the user)?
///
/// We classify by the tool name, which is shared across agents and MCP servers
/// (`Read`, `Grep`, `list_issues`, `get_file_contents`, …). It is **fail-safe**:
/// an effectful verb always wins, and anything we don't recognize falls through
/// to `false` (ask). So new/unknown/MCP tools are never silently skipped.
fn is_read_only(tool: &str) -> bool {
    let tokens = tokenize(tool);
    let tokens: Vec<&str> = tokens.iter().map(String::as_str).collect();

    // Verbs that mean "this changes something" -> always ask.
    const EFFECTFUL: [&str; 26] = [
        "write", "edit", "create", "update", "delete", "remove", "rm", "push",
        "merge", "run", "exec", "shell", "bash", "cmd", "powershell", "install",
        "apply", "move", "rename", "chmod", "kill", "deploy", "publish", "post",
        "put", "patch",
    ];
    if tokens.iter().any(|t| EFFECTFUL.contains(t)) {
        return false;
    }

    // Verbs that mean "this only reads" -> safe to defer.
    const READ_ONLY: [&str; 18] = [
        "read", "glob", "grep", "list", "ls", "search", "view", "find", "cat",
        "head", "tail", "stat", "get", "fetch", "show", "describe", "diff",
        "status",
    ];
    tokens.iter().any(|t| READ_ONLY.contains(t))
}

/// Talks to the daemon on a thread and applies the timeout policy.
///
/// - daemon absent / I/O error -> `Defer` (falls back to the agent's native prompt)
/// - daemon present but no response in time -> `Deny` (fail-safe)
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
        Ok(Ok(d)) => d,             // the daemon responded
        Ok(Err(_)) => Decision::Defer, // couldn't talk to the daemon
        Err(_) => Decision::Deny,   // timeout: nobody responded in time
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

    // absolute path of the binary itself (quoted, because of spaces on Windows)
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "chris".to_string());
    let exe_q = if exe.contains(' ') { format!("\"{exe}\"") } else { exe };

    match agent.as_str() {
        "copilot" => install_copilot(&format!("{exe_q} hook --agent copilot")),
        "claude" => install_claude(&format!("{exe_q} hook --agent claude")),
        _ => {
            eprintln!("install: use  --agent copilot  or  --agent claude.");
            std::process::exit(2);
        }
    }
}

/// Copilot: writes `.github/hooks/chris.json`.
fn install_copilot(invoke: &str) {
    // `timeoutSec` MUCH larger than our internal timeout, so our inactivity
    // `Deny` arrives before Copilot gives up on the hook.
    let config = serde_json::json!({
        "version": 1,
        "hooks": {
            "preToolUse": [
                {
                    "type": "command",
                    "bash": invoke,
                    "powershell": invoke,
                    "timeoutSec": 600,
                    "comment": "CHRIS — approval via the companion"
                }
            ]
        }
    });

    let dir = std::path::Path::new(".github").join("hooks");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("install: couldn't create {}: {e}", dir.display());
        std::process::exit(1);
    }
    let path = dir.join("chris.json");
    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&config).unwrap()) {
        eprintln!("install: couldn't write {}: {e}", path.display());
        std::process::exit(1);
    }
    println!("Copilot hook installed at {}", path.display());
    println!("Hook command: {invoke}");
    println!("Reminder: the daemon (companiond) must be running for the blob to react.");
}

/// Claude Code: MERGES the hook into `.claude/settings.json` (doesn't erase the rest).
fn install_claude(invoke: &str) {
    let dir = std::path::Path::new(".claude");
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("install: couldn't create {}: {e}", dir.display());
        std::process::exit(1);
    }
    let path = dir.join("settings.json");

    // read the existing settings (or start with an empty object)
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

    // ensure hooks.PreToolUse is an array
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

    // avoid duplicating if already installed
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
        eprintln!("install: couldn't write {}: {e}", path.display());
        std::process::exit(1);
    }
    println!("Claude Code hook installed/updated at {}", path.display());
    println!("Hook command: {invoke}");
    println!("Reminder: the daemon (companiond) must be running for the blob to react.");
}

#[cfg(test)]
mod tests {
    use super::is_read_only;

    #[test]
    fn read_only_tools_are_deferred() {
        // Claude-style and MCP-style read tools
        for t in [
            "Read", "Grep", "Glob", "LS", "WebSearch", "list_issues",
            "get_file_contents", "search_code", "show", "git status",
        ] {
            assert!(is_read_only(t), "{t} should be read-only");
        }
    }

    #[test]
    fn effectful_tools_are_asked() {
        // shell + file writes + GitHub MCP writes
        for t in [
            "shell", "Bash", "Write", "Edit", "create_pull_request",
            "merge_pull_request", "delete_file", "update_issue", "rm -rf",
        ] {
            assert!(!is_read_only(t), "{t} should require approval");
        }
    }

    #[test]
    fn unknown_tools_fall_through_to_ask() {
        // fail-safe: nothing recognized -> not read-only -> ask
        assert!(!is_read_only("frobnicate"));
        assert!(!is_read_only("mcp_tool"));
        assert!(!is_read_only(""));
    }
}
