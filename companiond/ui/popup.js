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
  defer: document.getElementById("defer"),
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
  els.summary.scrollTop = 0; // always show the start of the context
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

// Step aside: let the agent's own prompt handle it in the terminal.
function sendDefer() {
  if (currentId === null) return;
  if (tauri) {
    tauri.core.invoke("defer", { id: currentId });
  }
  currentId = null;
}

els.allow.addEventListener("click", () => send(true));
els.deny.addEventListener("click", () => send(false));
els.defer.addEventListener("click", () => sendDefer());
// keyboard: Enter = allow, Esc = deny
window.addEventListener("keydown", (e) => {
  if (e.key === "Enter") send(true);
  else if (e.key === "Escape") send(false);
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
