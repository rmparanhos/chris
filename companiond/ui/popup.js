// Popup de aprovação. Recebe o pedido do daemon (evento "approval"),
// preenche a tela e devolve a decisão pelo comando `decide`.

const card = document.getElementById("card");
const els = {
  risk: document.getElementById("risk"),
  agent: document.getElementById("agent"),
  tool: document.getElementById("tool"),
  summary: document.getElementById("summary"),
  cwd: document.getElementById("cwd"),
  allow: document.getElementById("allow"),
  deny: document.getElementById("deny"),
};

let currentId = null;

function render(req) {
  currentId = req.id;
  card.dataset.risk = req.risk || "low";
  els.risk.textContent = (req.risk || "low").toUpperCase();
  els.agent.textContent = req.agent || "";
  els.tool.textContent = req.tool || "";
  els.summary.textContent = req.summary || "";
  els.cwd.textContent = req.cwd || "";
}

// Integração com o Tauri (existe só quando roda dentro do app).
const tauri = window.__TAURI__;

function send(allow) {
  if (currentId === null) return;
  if (tauri) {
    tauri.core.invoke("decide", { id: currentId, allow });
  }
  currentId = null;
}

els.allow.addEventListener("click", () => send(true));
els.deny.addEventListener("click", () => send(false));
// Esc = negar (seguro)
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") send(false);
});

if (tauri) {
  tauri.event.listen("approval", (e) => render(e.payload));
} else {
  // prévia no navegador (sem Tauri): mostra um exemplo
  render({
    id: 1,
    agent: "Copilot",
    tool: "shell",
    summary: "rm -rf build/",
    cwd: "/home/dev/projeto",
    risk: "high",
  });
}
