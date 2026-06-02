// Popup de Pull Request. Recebe o PR do daemon (evento "pr"), mostra os
// detalhes e oferece Abrir / Aprovar / Dispensar.

const els = {
  repo: document.getElementById("repo"),
  title: document.getElementById("title"),
  author: document.getElementById("author"),
  status: document.getElementById("status"),
  open: document.getElementById("open"),
  approve: document.getElementById("approve"),
  dismiss: document.getElementById("dismiss"),
};

let pr = null;

function render(data) {
  pr = data;
  els.repo.textContent = `${data.owner}/${data.repo} #${data.number}`;
  els.title.textContent = data.title || "";
  els.author.textContent = data.author ? `por ${data.author}` : "";
  els.status.textContent = "";
}

const tauri = window.__TAURI__;
const invoke = (cmd, args) => (tauri ? tauri.core.invoke(cmd, args) : Promise.resolve());

els.open.addEventListener("click", () => {
  if (pr) invoke("open_url", { url: pr.url });
});

els.approve.addEventListener("click", async () => {
  if (!pr) return;
  els.status.textContent = "Aprovando…";
  try {
    await invoke("approve_pr", { owner: pr.owner, repo: pr.repo, number: pr.number });
    els.status.textContent = "Aprovado ✓";
    setTimeout(() => invoke("hide_pr"), 800);
  } catch (e) {
    els.status.textContent = "Falhou: " + e;
  }
});

els.dismiss.addEventListener("click", () => invoke("hide_pr"));

if (tauri) {
  tauri.event.listen("pr", (e) => render(e.payload));
} else {
  // prévia no navegador
  render({
    owner: "acme",
    repo: "widget",
    number: 42,
    title: "Corrige o parser e adiciona testes",
    author: "alice",
    url: "https://github.com/acme/widget/pull/42",
  });
}
