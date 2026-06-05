// Approval popup. Receives the request from the daemon (the "approval" event),
// fills in the screen and sends the decision back via the `decide` command.

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

// Tauri integration (only present when running inside the app).
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
// Esc = deny (safe default)
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") send(false);
});

if (tauri) {
  tauri.event.listen("approval", (e) => render(e.payload));
} else {
  // browser preview (no Tauri): show an example
  render({
    id: 1,
    agent: "Copilot",
    tool: "shell",
    summary: "rm -rf build/",
    cwd: "/home/dev/project",
    risk: "high",
  });
}
