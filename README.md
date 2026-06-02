# CHRIS

**C**oding-agent **H**ook **R**eview **I**nteractive **S**idekick.

Um companion de desktop (Rust + Tauri): quando o seu agente de codificação
(Copilot CLI) vai rodar um comando, um **blob** na tela reage e você
**aprova ou nega** num popup — sem voltar pro terminal.

> Quer entender o desenho/arquitetura? Veja [`DESIGN.md`](DESIGN.md).

---

## 1. Ver o visual em 30 segundos (sem instalar nada)

Abra estes arquivos direto no seu navegador (duplo-clique):

- `companiond/ui/index.html` → o **blob**. Clique nele pra ver os estados
  (idle → alerta → aprovado → negado).
- `companiond/ui/popup.html` → o **popup** de aprovação (exemplo `rm -rf`).

Isso é só a aparência. Pra funcionar de verdade, siga abaixo.

---

## 2. Rodar de verdade

### 2.1 Pré-requisitos (uma vez)

- **Rust**: instale em https://rustup.rs
- **Windows**: instale o "Desktop development with C++" (Microsoft C++ Build
  Tools). O WebView2 já vem no Windows 10/11.
- **Linux**: precisa das libs do WebKitGTK (ex. no Ubuntu:
  `sudo apt install libwebkit2gtk-4.1-dev build-essential`).
- **macOS**: Xcode Command Line Tools (`xcode-select --install`).

### 2.2 Passo a passo

Rode tudo **na pasta raiz do projeto**, em dois terminais.

**Terminal 1 — liga o companion (deixe aberto):**

```bash
cargo run -p companiond
```

➡️ Deve aparecer o **blob** na tela e um ícone do CHRIS na bandeja do sistema.
(A primeira compilação demora alguns minutos.)

**Terminal 2 — conecta o Copilot ao CHRIS:**

```bash
# 1) compila a CLI `chris`
cargo build -p chris-cli

# 2) entre na pasta do projeto onde você usa o Copilot e instale o hook
cd /caminho/do/seu/projeto
# Windows:
C:\caminho\do\chris\target\debug\chris.exe install --agent copilot
# Linux/macOS:
/caminho/do/chris/target/debug/chris install --agent copilot
```

➡️ Isso cria um arquivo `.github/hooks/chris.json` no seu projeto. Não precisa
colocar o `chris` no PATH: o instalador grava o caminho completo do binário.

### 2.3 Usar

Use o **Copilot CLI** normalmente nesse projeto. Quando ele for executar um
comando:

1. o blob fica **laranja (alerta)**;
2. abre o **popup** com o comando e o nível de risco;
3. clique **Permitir** ou **Negar** (ou `Esc` = negar).

Regras automáticas:
- Se você **não responder a tempo** → **nega** (seguro).
- Se o **companion estiver desligado** → o Copilot usa o prompt normal dele
  (você não fica travado).

---

## 3. Conferir que está tudo certo (opcional)

Rodar os testes da lógica (não precisa de tela):

```bash
cargo test
```

➡️ Deve mostrar **8 testes ok** (core, transporte, adapters e o teste
ponta-a-ponta do hook).

---

## 4. Mapa do projeto

| Pasta | O que é |
|-------|---------|
| `companiond` | O app que você roda: blob, bandeja e popup. |
| `crates/hook` | A CLI `chris` (`hook` + `install`). |
| `crates/adapters` | Tradução do formato do Copilot ⇄ formato interno. |
| `crates/transport-ipc` | A comunicação entre o `chris` e o `companiond`. |
| `crates/core` | O "cérebro" (regras), portável até pra ESP32. |

## Status

✅ MVP completo: blob + bandeja, popup de aprovação, hook do Copilot e
instalador. Próximo (fase 2): Claude/Codex, "aprovar e lembrar", notificações
de Pull Request e o buddy físico em ESP32.
