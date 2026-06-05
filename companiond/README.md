# companiond — o daemon visual do CHRIS (M1)

App Tauri que mostra o **blob** numa janela transparente, sem bordas e sempre
no topo, com um ícone na **bandeja do sistema** (mostrar/ocultar, sair).

> ⚠️ Esta parte usa a webview do sistema, então **só compila/roda numa máquina
> com ambiente gráfico** (Windows/macOS/Linux desktop). Não compila em
> servidores/containers sem display — por isso ela fica fora do `cargo build`
> padrão do workspace.

## Pré-requisitos (Windows)

1. **Rust** — https://rustup.rs
2. **Microsoft C++ Build Tools** (workload "Desktop development with C++").
3. **WebView2** — já vem no Windows 11 (e na maioria dos Windows 10).

## Rodar

Na raiz do repositório:

```bash
cargo run -p companiond
```

Isso compila e abre o blob. Clique nele para **pré-visualizar os estados**
(idle → alerta → aprovado → negado). Clique com o botão direito no ícone da
bandeja para mostrar/ocultar ou sair.

> Opcional: instalando o Tauri CLI (`cargo install tauri-cli`) você ganha
> hot-reload com `cargo tauri dev`.

## Prévia sem Rust

Como o blob (M1) é todo client-side, dá pra ver no navegador antes:
abra `companiond/ui/index.html` direto no Chrome/Edge e clique nele.

## Já faz (M3)

- Escuta o cano IPC e, a cada pedido, põe o blob em "alerta", abre o **popup**
  de aprovação, espera o clique (ou timeout = Deny) e responde ao hook.

## O que ainda NÃO faz

- Click-through na janela (deixar passar cliques quando o blob for decorativo).
- Fila visível com contador quando vários agentes pedem ao mesmo tempo
  (hoje os pedidos são atendidos um a um, em ordem).
- Adapters de Claude/Codex e notificações de PR (fase 2).
