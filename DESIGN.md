# Familiar — Design

> **Codinome provisório:** "Familiar" (um *familiar* é a criatura-companheira
> mágica, agnóstica a qual bruxo/agente serve — encaixa com o blob + multi-agente).
> Nome a definir.

## 1. Conceito

Um **companion desktop** (Rust + Tauri, cross-platform) que **reage e intermedia
aprovações** de agentes de codificação. Quando um agente (Copilot CLI, Claude
Code, Codex, …) precisa de aprovação do usuário para executar uma ferramenta, o
companion — um **blob animado** — reage e o usuário aprova/nega por ele, sem
precisar voltar ao terminal.

O companion é **agnóstico ao agente**: a integração se dá por **hooks**
(mecanismo estável e estruturado), não por interceptação de terminal (PTY) nem
por servidor MCP.

**Visão de futuro (alvo concreto):** o "corpo" do companion pode migrar para um
**buddy físico em ESP32** — uma telinha com o sprite + botões físicos de
Approve/Deny na mesa. Por isso o **cérebro** é desenhado portável (`no_std`)
desde o início.

## 2. Por que hooks (e não PTY ou MCP)

| Mecanismo | Veredito | Motivo |
|-----------|----------|--------|
| **Hooks** | ✅ escolhido | Payload estruturado, decisão de volta limpa, síncrono. Convergente: os 3 agentes têm `PreToolUse`. |
| PTY wrapper | ❌ | Frágil (parsear TUI/ANSI), intrusivo (fica no meio do terminal). |
| MCP permission tool | ❌ | Depende de o agente delegar permissão via MCP; menos universal que hooks. |

### Suporte por agente (verificado)

| Agente | Evento de aprovação | Como devolve a decisão | Onde configura |
|--------|--------------------|------------------------|----------------|
| **Copilot CLI** (MVP) | `preToolUse` (payload snake_case: `tool_name`, `tool_input`) | retornar `deny` bloqueia; síncrono — a sessão espera o hook | `.github/hooks/NAME.json` |
| **Claude Code** | `PreToolUse` | JSON em stdout com `permissionDecision: allow/deny/ask` (ou exit 2) | `.claude/settings.json` |
| **Codex CLI** | `PreToolUse` / `PermissionRequest` | resposta do hook + `/hooks` para confiar | config em `~/.codex` |

Os três também expõem eventos **não-bloqueantes** ("agente terminou / precisa de
atenção": `agentStop`, `Stop`/`Notification`, `notify`) — usados para o blob
reagir fora de uma aprovação. (Fase 2.)

## 3. Arquitetura

Separação rígida entre **cérebro** (lógica pura, portável) e **corpo** (I/O,
apresentação).

```
        PC (sempre presente)                          Corpo (escolha)
┌─────────────────────────────────┐
│ hook + adapters + CORE + bridge │ ──transport──▶  ┌─ Tauri blob (tela do PC)   [MVP]
│  (intercepta o agente, decide)  │  ◀──decisão──   └─ ESP32 buddy (tela+botões)  [futuro]
└─────────────────────────────────┘
```

O ESP32 **não intercepta** o agente (o agente roda no PC) — ele é puramente um
**corpo remoto**: apresentação + input de aprovação. Sempre existe uma peça no PC.

### Fluxo de uma aprovação

```
agente (copilot) dispara preToolUse
        │  (passa payload JSON no stdin, e ESPERA)
        ▼
┌──────────────────┐   named pipe (ACL)   ┌────────────────────┐
│ companion-hook   │ ──ApprovalRequest──▶ │   companiond        │
│ (binário curto)  │ ◀──Decision──────────│ (Tauri: blob,       │
│ normaliza→core   │                      │  popup, tray, estado)│
└────────┬─────────┘                      └────────────────────┘
         │ escreve a decisão no formato do agente (stdout / exit code)
         ▼
   agente segue ou bloqueia
```

### Componentes

- **`companiond`** — daemon = app Tauri. Vive na tray, dono do blob, do popup e
  do estado. Autostart (registro `Run` no Windows). É também o *host* do
  transport e o *bridge* para o corpo.
- **`companion-hook`** — binário curto chamado no `PreToolUse`. Lê o payload,
  normaliza via adapter, fala com o daemon por named pipe, devolve a decisão no
  formato do agente. (`companion-hook.exe` no Windows; sem extensão em Unix.)
- **`companion install`** — escreve a config de hook de cada agente apontando
  para o `companion-hook`, e configura o timeout do agente alto o suficiente.

## 4. Camadas / workspace

```
├── Cargo.toml                 # workspace
├── crates/
│   ├── core/        # #![no_std] + alloc. Tipos, protocolo (postcard), risco,
│   │                #   state machine, traits Transport e Presentation.
│   │                #   ZERO I/O, ZERO async. Compila no PC e no ESP32.
│   ├── adapters/    # std. Parseia payload dos agentes (serde_json) → core. Lado PC.
│   ├── transport-ipc/ # std. named pipe (desktop).
│   └── hook/        # std bin. companion-hook + subcomando install.
├── companiond/      # std. Tauri = presentation desktop + bridge + host do transport.
└── firmware/        # #![no_std] bin. ESP32 (esp-hal). Target xtensa/riscv, build separado.
```

**Regra de ouro:** toda lógica pura mora no `core` `no_std`; todo I/O (Tauri,
Tokio, named pipe, serde_json, Wi-Fi/BLE) fica nas bordas. Desktop e ESP32
compartilham o mesmo cérebro; só trocam de corpo.

### Traits centrais (esboço)

```rust
// core (no_std + alloc)
pub enum Decision { Allow, Deny, Defer }

pub trait Transport {            // named pipe no PC; Wi-Fi/BLE/serial no ESP32
    fn send(&mut self, msg: &Msg) -> Result<(), Error>;
    fn recv(&mut self) -> Result<Option<Msg>, Error>;
}

pub trait Presentation {         // webview no PC; display + GPIO no ESP32
    fn react(&mut self, state: BlobState);
    fn show(&mut self, req: &ApprovalRequest);
    fn poll_input(&mut self) -> Option<Decision>;
}
```

### Protocolo (wire)

- **`Msg` serializado em `postcard`** (binário, no_std, compacto) — leve para o
  MCU. **JSON só existe no `adapters`**, no lado PC (parse do payload do agente).
- **Byte de versão** no início do `Msg`: firmware (ESP32) e daemon atualizam em
  ritmos diferentes — o skew de versão precisa ser detectável.

Esboço de mensagens:
```
hook → daemon:  ApprovalRequest { id, agent, tool, summary, detail, cwd, risk }
daemon → hook:  Decision { id, decision: Allow|Deny|Defer, reason }
```

### Disciplina `no_std` no `core`

- `no_std + alloc` (ambos os alvos têm allocator — evita a dor do `heapless` puro).
- Sem `anyhow`/`thiserror` (std) → erros são enums simples.
- Sem `std::time` → timestamp entra como parâmetro.
- `uuid` em modo no_std, ou `ReqId(u32)` simples.
- Sem Tokio/async no core.

## 5. Política de decisão

| Situação | Decisão | Racional |
|----------|---------|----------|
| Usuário clica **Allow** / **Deny** | conforme o clique | — |
| **Timeout** (usuário viu e não respondeu) | **Deny** | fail-safe. Nosso timeout < timeout do agente, para o hook responder antes de o agente desistir e ignorar. |
| Fechar popup / clicar fora | sem-decisão → **Deny** no timeout | — |
| **Daemon ausente** (ninguém viu) | **Defer** → agente usa o prompt nativo | nunca trava nem desprotege; só perde o blob naquele momento. Claude=`ask`, Codex=nativo, Copilot=fluxo próprio. |

> **Distinção importante:** *timeout* (você viu e não agiu) = **Deny**;
> *daemon ausente* (ninguém viu) = **Defer**.

### Concorrência

Vários agentes/terminais podem pedir ao mesmo tempo → **fila um-por-vez** com
contador no blob ("+2"). Pilha de cards fica para depois.

### Risco (heurística — default, pendente de refino)

- `high`: `rm`/`del`, `sudo`, `curl … | sh`, acesso de rede, escrita fora do cwd.
- `med` / `low`: demais; leitura pura = `low`.

## 6. Especificidades de plataforma (Windows primeiro)

- **IPC:** named pipe `\\.\pipe\familiar` com ACL — só o usuário corrente. Importante:
  quem fala com o daemon pode autorizar execução de código.
- **Janela do blob:** `transparent + decorations:false + alwaysOnTop + skipTaskbar`
  e **click-through** (`set_ignore_cursor_events`); desligar a sombra da janela.
- **Hook → executável:** configs apontam para `companion-hook.exe` com caminho
  absoluto (escapando `\` no JSON).
- **Autostart:** `Startup` / chave `Run` do registro (via `companion install`).
- Cross-platform por construção: o mesmo código gera binários por SO; só o pouco
  que é específico de SO fica isolado atrás de traits.

## 7. UX do blob

- **Popup ao lado:** o blob reage (`idle → alerta`) **e** abre sozinho um popup
  ao lado mostrando comando, cwd, risco e botões `Allow`/`Deny` (imediato — o
  agente está bloqueado esperando).
- **Estados do blob:** `idle`, `alerta`, `aprovado`, `negado` (+ badge de contagem).
- **Sprite:** placeholder **blob** animado (canvas/CSS leve). Personagem final a definir.
- **Som:** default **off** no MVP (opcional depois).

## 8. Marcos (MVP fim-a-fim, Copilot)

- **M0** — workspace + tipos do `core` (`no_std`) + protocolo `postcard`.
- **M0.5** — *smoke test*: o `core` compila para o target ESP32 (`xtensa`/`riscv32`,
  via `espup`). Valida a disciplina `no_std` cedo, antes de apodrecer.
- **M1** — casca do Tauri: tray + janela transparent/always-on-top com o blob `idle`.
- **M2** — servidor de named pipe no daemon + `companion-hook` mandando request e
  imprimindo decisão.
- **M3** — popup de aprovação + blob reage (`idle→alerta→aprovado/negado`) + decisão de volta.
- **M4** — adapter do Copilot (parse `preToolUse`, devolve allow/deny no formato certo)
  + `companion install` para o Copilot.
- **M5** — teste fim-a-fim com o Copilot CLI real (rodado pelo usuário).

## 9. Fase 2 (pós-MVP)

- Adapters **Claude Code** e **Codex**.
- **"Approve & remember"** / allowlist (ex.: "sempre permitir este comando nesta sessão").
- **Notificações de Pull Request**: polling via `octocrab`; "approve" = submeter
  review aprovando. Cadência/escopo a definir.
- Eventos não-bloqueantes (agente terminou) → reações do blob.
- **Buddy físico ESP32**: escrever `firmware/` (display + botões + um `Transport`),
  reaproveitando o `core` inteiro. Link (Wi-Fi/MQTT, BLE ou USB serial) a decidir.

## 10. Crates previstas

- **PTY** — descartado.
- **IPC desktop:** `interprocess` (named pipe Windows / unix socket).
- **Serialização wire:** `postcard` (+ `serde` no_std). `serde_json` só nos adapters.
- **GitHub/PRs (fase 2):** `octocrab`.
- **ESP32 (fase 2):** `esp-hal` / `embedded-hal`, `espup`.

## 11. Decisões em aberto

| # | Item | Default assumido |
|---|------|------------------|
| 1 | Nome do projeto | "Familiar" (provisório) |
| 2 | Allowlist / "approve & remember" | Fase 2 |
| 3 | Heurística de risco | Simples (ver §5) |
| 4 | Link do ESP32 | A decidir (não bloqueia o MVP) |
| 5 | Personagem/sprite final | Blob placeholder |
| 6 | Plataforma de teste do MVP | Windows |

## Decisões já fechadas

- Mecanismo de integração: **hooks** (não PTY, não MCP).
- Integração: **daemon em background** + hook curto por chamada.
- MVP: **fim-a-fim mínimo** com **Copilot CLI**.
- Cross-platform; **teste no Windows primeiro**.
- Timeout → **Deny**; daemon ausente → **Defer**.
- UX: **blob + popup ao lado**; concorrência em **fila**.
- ESP32 como **alvo concreto em breve** → `core` `no_std`, `transport`/`presentation`
  como cidadãos de primeira classe, validação M0.5.
