# CHRIS

**C**oding-agent **H**ook **R**eview **I**nteractive **S**idekick.

Um companion de desktop (Rust + Tauri): quando o seu agente de codificação
(**Copilot CLI** ou **Claude Code**) vai rodar um comando, um **blob** na tela
reage e você **aprova ou nega** num popup — sem voltar pro terminal.

> Quer entender o desenho/arquitetura? Veja [`DESIGN.md`](DESIGN.md).

---

## 1. Ver o visual em 30 segundos (sem instalar nada)

Abra estes arquivos no navegador (duplo-clique):

- `companiond/ui/index.html` → o **blob**. Clique nele pra ver os estados.
- `companiond/ui/popup.html` → o **popup** de aprovação (exemplo `rm -rf`).

Isso é só a aparência. Pra funcionar de verdade, siga abaixo.

---

## 2. Instalar e rodar — Windows (automático)

Não precisa saber nada de Rust: os scripts instalam tudo (Rust, ferramentas de
compilação e o WebView2) e compilam o projeto.

**Passo 0 — baixar o código:** nesta página do GitHub, clique no botão verde
**`Code` → `Download ZIP`**, e extraia a pasta (botão direito → Extrair tudo).

**Passo 1 — instalar:** entre na pasta extraída e dê **duplo-clique em
`setup.bat`**. Aceite os pedidos de permissão do Windows. (A primeira vez
demora — ele baixa e compila bastante coisa. Pode ir tomar um café.)

> Se ao final ele pedir para "fechar a janela e rodar de novo", é só fechar e
> dar duplo-clique no `setup.bat` mais uma vez.

**Passo 2 — iniciar o companion:** duplo-clique em **`run.bat`**.
➡️ O **blob** aparece na tela e um ícone do CHRIS vai pra bandeja. Deixe essa
janela aberta enquanto estiver usando.

**Passo 3 — ligar no seu projeto:** duplo-clique em **`connect.bat`**, cole o
caminho da pasta do projeto e tecle Enter. Isso liga **tanto o Copilot CLI
quanto o Claude Code** (cada um no arquivo de config dele; nada é sobrescrito).

Pronto. Agora use o **Copilot CLI** ou o **Claude Code** nesse projeto: quando
ele for rodar um comando, o blob fica **laranja** e abre o **popup** — clique
**Permitir** ou **Negar** (ou `Esc` = negar).

> Quer ligar só um deles? Rode na pasta do projeto:
> `chris install --agent claude`  (ou `--agent copilot`).

---

## 3. Instalar e rodar — Linux/macOS

No terminal, dentro da pasta do projeto:

```bash
./setup.sh                      # instala tudo e compila
./run.sh                        # inicia o companion (deixe aberto)
./connect.sh /caminho/do/projeto   # liga o CHRIS ao Copilot nesse projeto
```

---

## Regras automáticas (bom saber)

- Se você **não responder a tempo** → o CHRIS **nega** (seguro).
- Se o **companion estiver desligado** → o Copilot usa o prompt normal dele;
  você nunca fica travado.

## Notificações de Pull Request

O companion também avisa quando um **PR pede a sua revisão**: o blob fica
**azul** e abre um popup com o título do PR e os botões **Abrir** (no navegador)
e **Aprovar** (manda o review de aprovação direto).

Para ligar, o CHRIS precisa de um token do GitHub. Use **uma** opção:

- Tenha o **GitHub CLI** logado (`gh auth login`) — o CHRIS pega o token
  sozinho; **ou**
- Defina a variável de ambiente `GITHUB_TOKEN` com um token (escopo `repo`)
  antes de iniciar o `run.bat`/`run.sh`.

Sem token, as notificações de PR ficam simplesmente desligadas (o resto funciona
normal).

---

## Conferir que está tudo certo (opcional, para devs)

```bash
cargo test     # 8 testes da lógica (não precisa de tela)
```

## Mapa do projeto

| Pasta / arquivo | O que é |
|-----------------|---------|
| `setup.* / run.* / connect.*` | Os scripts de instalar, iniciar e conectar. |
| `companiond` | O app: blob, bandeja e popup. |
| `crates/hook` | A CLI `chris` (`hook` + `install`). |
| `crates/adapters` | Tradução do formato do Copilot ⇄ formato interno. |
| `crates/transport-ipc` | Comunicação entre o `chris` e o `companiond`. |
| `crates/core` | O "cérebro" (regras), portável até pra ESP32. |

## Status

✅ Blob + bandeja, popup de aprovação, hooks do **Copilot CLI e do Claude
Code**, instalador automático e **notificações de Pull Request** (abrir/aprovar).
Próximo (fase 2): adapter do Codex, "aprovar e lembrar" e o buddy físico em ESP32.
