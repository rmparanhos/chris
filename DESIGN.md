# CHRIS — Design

> **CHRIS** = **C**oding-agent **H**ook **R**eview **I**nteractive **S**idekick.
> The agent-agnostic, hook-based companion that reacts to approvals.

## 1. Concept

A **desktop companion** (Rust + Tauri, cross-platform) that **reacts to and
mediates approvals** from coding agents. When an agent (Copilot CLI, Claude
Code, Codex, …) needs the user's approval to run a tool, the companion — an
**animated blob** — reacts, and the user approves/denies through it, without
having to go back to the terminal.

The companion is **agent-agnostic**: integration happens through **hooks** (a
stable, structured mechanism), not through terminal interception (PTY) nor
through an MCP server.

**Future vision (concrete target):** the companion's "body" can migrate to a
**physical buddy on ESP32** — a little screen with the sprite + physical
Approve/Deny buttons on your desk. That's why the **brain** is designed to be
portable (`no_std`) from the start.

## 2. Why hooks (and not PTY or MCP)

| Mechanism | Verdict | Reason |
|-----------|----------|--------|
| **Hooks** | ✅ chosen | Structured payload, clean decision sent back, synchronous. Convergent: all 3 agents have `PreToolUse`. |
| PTY wrapper | ❌ | Fragile (parsing TUI/ANSI), intrusive (sits in the middle of the terminal). |
| MCP permission tool | ❌ | Depends on the agent delegating permission via MCP; less universal than hooks. |

### Support per agent (verified)

| Agent | Approval event | How it returns the decision | Where it's configured |
|--------|--------------------|------------------------|----------------|
| **Copilot CLI** (MVP) | `preToolUse` (snake_case payload: `tool_name`, `tool_input`) | returning `deny` blocks; synchronous — the session waits for the hook | `.github/hooks/NAME.json` |
| **Claude Code** | `PreToolUse` | JSON on stdout with `permissionDecision: allow/deny/ask` (or exit 2) | `.claude/settings.json` |
| **Codex CLI** | `PreToolUse` / `PermissionRequest` | hook response + `/hooks` to trust | config in `~/.codex` |

All three also expose **non-blocking** events ("agent finished / needs
attention": `agentStop`, `Stop`/`Notification`, `notify`) — used for the blob to
react outside of an approval. (Phase 2.)

## 3. Architecture

A strict separation between **brain** (pure logic, portable) and **body** (I/O,
presentation).

```
        PC (always present)                           Body (choice)
┌─────────────────────────────────┐
│ hook + adapters + CORE + bridge │ ──transport──▶  ┌─ Tauri blob (PC screen)     [MVP]
│  (intercepts the agent, decides)│  ◀──decision──  └─ ESP32 buddy (screen+buttons)[future]
└─────────────────────────────────┘
```

The ESP32 **does not intercept** the agent (the agent runs on the PC) — it is
purely a **remote body**: presentation + approval input. There is always a piece
on the PC.

### Flow of an approval

```
agent (copilot) fires preToolUse
        │  (passes JSON payload on stdin, and WAITS)
        ▼
┌──────────────────┐   named pipe (ACL)   ┌────────────────────┐
│ companion-hook   │ ──ApprovalRequest──▶ │   companiond        │
│ (short binary)   │ ◀──Decision──────────│ (Tauri: blob,       │
│ normalizes→core  │                      │  popup, tray, state) │
└────────┬─────────┘                      └────────────────────┘
         │ writes the decision in the agent's format (stdout / exit code)
         ▼
   agent proceeds or blocks
```

### Components

- **`companiond`** — daemon = Tauri app. Lives in the tray, owner of the blob,
  the popup and the state. Autostart (`Run` registry key on Windows). It is also
  the *host* of the transport and the *bridge* to the body.
- **`companion-hook`** — short binary called on `PreToolUse`. Reads the payload,
  normalizes it via an adapter, talks to the daemon over a named pipe, returns
  the decision in the agent's format. (`companion-hook.exe` on Windows; no
  extension on Unix.)
- **`companion install`** — writes each agent's hook config pointing to
  `companion-hook`, and sets the agent's timeout high enough.

## 4. Layers / workspace

```
├── Cargo.toml                 # workspace
├── crates/
│   ├── core/        # #![no_std] + alloc. Types, protocol (postcard), risk,
│   │                #   state machine, Transport and Presentation traits.
│   │                #   ZERO I/O, ZERO async. Compiles on the PC and the ESP32.
│   ├── adapters/    # std. Parses the agents' payload (serde_json) → core. PC side.
│   ├── transport-ipc/ # std. named pipe (desktop).
│   └── hook/        # std bin. companion-hook + install subcommand.
├── companiond/      # std. Tauri = desktop presentation + bridge + transport host.
└── firmware/        # #![no_std] bin. ESP32 (esp-hal). Target xtensa/riscv, separate build.
```

**Golden rule:** all pure logic lives in the `no_std` `core`; all I/O (Tauri,
Tokio, named pipe, serde_json, Wi-Fi/BLE) sits at the edges. Desktop and ESP32
share the same brain; they only swap bodies.

### Central traits (sketch)

```rust
// core (no_std + alloc)
pub enum Decision { Allow, Deny, Defer }

pub trait Transport {            // named pipe on the PC; Wi-Fi/BLE/serial on the ESP32
    fn send(&mut self, msg: &Msg) -> Result<(), Error>;
    fn recv(&mut self) -> Result<Option<Msg>, Error>;
}

pub trait Presentation {         // webview on the PC; display + GPIO on the ESP32
    fn react(&mut self, state: BlobState);
    fn show(&mut self, req: &ApprovalRequest);
    fn poll_input(&mut self) -> Option<Decision>;
}
```

### Protocol (wire)

- **`Msg` serialized with `postcard`** (binary, no_std, compact) — lightweight
  for the MCU. **JSON only exists in `adapters`**, on the PC side (parsing the
  agent's payload).
- **Version byte** at the start of `Msg`: firmware (ESP32) and daemon update at
  different paces — the version skew needs to be detectable.

Message sketch:
```
hook → daemon:  ApprovalRequest { id, agent, tool, summary, detail, cwd, risk }
daemon → hook:  Decision { id, decision: Allow|Deny|Defer, reason }
```

### `no_std` discipline in `core`

- `no_std + alloc` (both targets have an allocator — avoids the pain of pure `heapless`).
- No `anyhow`/`thiserror` (std) → errors are simple enums.
- No `std::time` → timestamp comes in as a parameter.
- `uuid` in no_std mode, or a simple `ReqId(u32)`.
- No Tokio/async in the core.

## 5. Decision policy

| Situation | Decision | Rationale |
|----------|---------|----------|
| User clicks **Allow** / **Deny** | as clicked | — |
| **Timeout** (user saw it and didn't respond) | **Deny** | fail-safe. Our timeout < the agent's timeout, so the hook responds before the agent gives up and ignores it. |
| Close the popup / click outside | no-decision → **Deny** on timeout | — |
| **Daemon absent** (nobody saw it) | **Defer** → agent uses its native prompt | never blocks nor leaves you unprotected; you just lose the blob in that moment. Claude=`ask`, Codex=native, Copilot=its own flow. |

> **Important distinction:** *timeout* (you saw it and didn't act) = **Deny**;
> *daemon absent* (nobody saw it) = **Defer**.

### Concurrency

Several agents/terminals can ask at the same time → **one-at-a-time queue** with
a counter on the blob ("+2"). A stack of cards is left for later.

### Risk (heuristic — default, pending refinement)

- `high`: `rm`/`del`, `sudo`, `curl … | sh`, network access, writes outside the cwd.
- `med` / `low`: everything else; pure reads = `low`.

## 6. Platform specifics (Windows first)

- **IPC:** named pipe `\\.\pipe\chris` with an ACL — only the current user. Important:
  whoever talks to the daemon can authorize code execution.
- **Blob window:** `transparent + decorations:false + alwaysOnTop + skipTaskbar`
  and **click-through** (`set_ignore_cursor_events`); turn off the window shadow.
- **Hook → executable:** configs point to `companion-hook.exe` with an absolute
  path (escaping `\` in the JSON).
- **Autostart:** `Startup` / `Run` registry key (via `companion install`).
- Cross-platform by construction: the same code produces binaries per OS; only
  the little that is OS-specific is isolated behind traits.

## 7. Blob UX

- **Side popup:** the blob reacts (`idle → alert`) **and** opens a popup by
  itself on the side, showing the command, cwd, risk and `Allow`/`Deny` buttons
  (immediate — the agent is blocked, waiting).
- **Blob states:** `idle`, `alert`, `approved`, `denied` (+ count badge).
- **Sprite:** placeholder animated **blob** (lightweight canvas/CSS). Final character TBD.
- **Sound:** default **off** in the MVP (optional later).

## 8. Milestones (end-to-end MVP, Copilot)

- **M0** — workspace + `core` types (`no_std`) + `postcard` protocol.
- **M0.5** — *smoke test*: the `core` compiles for the ESP32 target (`xtensa`/`riscv32`,
  via `espup`). Validates the `no_std` discipline early, before it rots.
- **M1** — Tauri shell: tray + transparent/always-on-top window with the `idle` blob.
- **M2** — named pipe server in the daemon + `companion-hook` sending a request and
  printing the decision.
- **M3** — approval popup + blob reacts (`idle→alert→approved/denied`) + decision sent back.
- **M4** — Copilot adapter (parse `preToolUse`, return allow/deny in the right format)
  + `companion install` for Copilot.
- **M5** — end-to-end test with the real Copilot CLI (run by the user).

## 9. Phase 2 (post-MVP)

- **Claude Code** and **Codex** adapters.
- **"Approve & remember"** / allowlist (e.g., "always allow this command in this session").
- **Pull Request notifications**: polling via `octocrab`; "approve" = submit an
  approving review. Cadence/scope TBD.
- Non-blocking events (agent finished) → blob reactions.
- **Physical ESP32 buddy**: write `firmware/` (display + buttons + a `Transport`),
  reusing the entire `core`. Link (Wi-Fi/MQTT, BLE or USB serial) TBD.

## 10. Planned crates

- **PTY** — discarded.
- **Desktop IPC:** `interprocess` (Windows named pipe / unix socket).
- **Wire serialization:** `postcard` (+ `serde` no_std). `serde_json` only in the adapters.
- **GitHub/PRs (phase 2):** `octocrab`.
- **ESP32 (phase 2):** `esp-hal` / `embedded-hal`, `espup`.

## 11. Open decisions

| # | Item | Assumed default |
|---|------|------------------|
| 1 | ~~Project name~~ | ✅ **CHRIS** (Coding-agent Hook Review Interactive Sidekick) |
| 2 | Allowlist / "approve & remember" | Phase 2 |
| 3 | Risk heuristic | Simple (see §5) |
| 4 | ESP32 link | TBD (does not block the MVP) |
| 5 | Final character/sprite | Blob placeholder |
| 6 | MVP test platform | Windows |

## Decisions already closed

- Integration mechanism: **hooks** (not PTY, not MCP).
- Integration: **background daemon** + short hook per call.
- MVP: **minimal end-to-end** with **Copilot CLI**.
- Cross-platform; **test on Windows first**.
- Timeout → **Deny**; daemon absent → **Defer**.
- UX: **blob + side popup**; concurrency in a **queue**.
- ESP32 as a **concrete target soon** → `no_std` `core`, `transport`/`presentation`
  as first-class citizens, M0.5 validation.
