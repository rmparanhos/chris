# CHRIS

**C**oding-agent **H**ook **R**eview **I**nteractive **S**idekick.

Um companion de desktop (Rust + Tauri) que **reage e intermedia aprovações** de
agentes de codificação. Quando o agente (Copilot CLI, e depois Claude/Codex)
pede para rodar uma ferramenta, um **blob** na sua tela reage e você aprova ou
nega ali mesmo — via **hooks**, sem ficar no meio do terminal.

> Desenho completo em [`DESIGN.md`](DESIGN.md).

## Como funciona

```
agente dispara preToolUse
   │ (payload no stdin, e espera)
   ▼
chris hook ──IPC (named pipe)──▶ companiond (Tauri)
   ▲                               blob reage + popup de aprovação
   └──────── decisão ──────────────┘
```

- **Timeout** (você viu e não respondeu) → **Deny** (seguro).
- **Daemon desligado** (ninguém viu) → **Defer**: o agente usa o prompt nativo
  dele. Você nunca fica travado nem desprotegido.

## Estrutura

| Pasta | O que é |
|-------|---------|
| `crates/core` | O "cérebro" `no_std` (tipos, protocolo, risco). Compila no PC e no ESP32. |
| `crates/transport-ipc` | O cano IPC (named pipe / unix socket). |
| `crates/adapters` | Tradução payload do agente ⇄ core (MVP: Copilot). |
| `crates/hook` | A CLI `chris` (`hook` e `install`). |
| `companiond` | O daemon Tauri: blob, bandeja e popup. |

## Build e testes

A lógica (tudo menos a GUI) compila em qualquer lugar:

```bash
cargo test            # core + transport + adapters + hook (8 testes)
```

O app gráfico precisa de uma máquina com webview (Windows/macOS/Linux desktop):

```bash
cargo run -p companiond
```

## Ligar no Copilot CLI

1. Compile a CLI: `cargo build -p chris-cli` (gera o binário `chris`).
2. No seu repositório, rode `chris install --agent copilot`
   (escreve `.github/hooks/chris.json`).
3. Deixe o `companiond` rodando.
4. Use o Copilot CLI normalmente — quando ele pedir para rodar algo, o blob
   reage e você decide pelo popup.

## Status

✅ M0 cérebro · ✅ M0.5 compila p/ ESP32 · ✅ M1 blob+tray · ✅ M2 IPC ·
✅ M3 popup+orquestração · ✅ M4 adapter Copilot + install

Fase 2: adapters Claude/Codex, "approve & remember", notificações de PR,
buddy físico ESP32.
