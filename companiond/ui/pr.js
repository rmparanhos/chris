// Pull Request popup. Receives the PR from the daemon (the "pr" event), shows
// the details and offers Open / Approve / Dismiss.

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
  els.author.textContent = data.author ? `by ${data.author}` : "";
  els.status.textContent = "";
}

const tauri = window.__TAURI__;
const invoke = (cmd, args) => (tauri ? tauri.core.invoke(cmd, args) : Promise.resolve());

els.open.addEventListener("click", () => {
  if (pr) invoke("open_url", { url: pr.url });
});

els.approve.addEventListener("click", async () => {
  if (!pr) return;
  els.status.textContent = "Approving…";
  try {
    await invoke("approve_pr", { owner: pr.owner, repo: pr.repo, number: pr.number });
    els.status.textContent = "Approved ✓";
    setTimeout(() => invoke("hide_pr"), 800);
  } catch (e) {
    els.status.textContent = "Failed: " + e;
  }
});

els.dismiss.addEventListener("click", () => invoke("hide_pr"));

if (tauri) {
  tauri.event.listen("pr", (e) => render(e.payload));
} else {
  // browser preview
  render({
    owner: "acme",
    repo: "widget",
    number: 42,
    title: "Fix the parser and add tests",
    author: "alice",
    url: "https://github.com/acme/widget/pull/42",
  });
}
