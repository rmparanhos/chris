// Controle do blob. Em M1 é tudo no lado do navegador (sem chamar APIs do
// Tauri ainda) — assim você pode abrir este arquivo direto no navegador e ver
// o blob funcionando. A partir do M3 o daemon (Rust) é que vai dirigir o
// estado, conforme as aprovações chegarem.

const stage = document.getElementById("stage");
const badge = document.getElementById("badge");
const blob = document.getElementById("blob");

const STATES = ["idle", "alert", "approved", "denied", "pr"];

/** Muda o estado visual do blob. Será chamada pelo backend no futuro.
 *  @param {string} state  idle | alert | approved | denied
 *  @param {number} count  quantos pedidos esperam na fila (0 = esconde badge)
 */
function setBlobState(state, count = 0) {
  if (!STATES.includes(state)) return;
  stage.dataset.state = state;
  if (count > 0) {
    badge.textContent = "+" + count;
    badge.hidden = false;
  } else {
    badge.hidden = true;
  }
}

// expõe a função globalmente
window.setBlobState = setBlobState;

// ---------- seletor de personagem (sprite) ----------
const SPRITES = ["blob", "cat"];

function setSprite(name) {
  if (!SPRITES.includes(name)) return;
  stage.dataset.sprite = name;
  document.querySelectorAll("#picker button").forEach((b) => {
    b.classList.toggle("active", b.dataset.sprite === name);
  });
  try {
    localStorage.setItem("chris.sprite", name);
  } catch (_) {}
}
window.setSprite = setSprite;

// restaura a última escolha e liga os botões
setSprite(localStorage.getItem("chris.sprite") || "blob");
document.querySelectorAll("#picker button").forEach((b) => {
  b.addEventListener("click", (e) => {
    e.stopPropagation(); // não dispara o ciclo de estados do blob
    setSprite(b.dataset.sprite);
  });
});

// Dentro do app: o daemon dirige o blob via evento "blob-state".
const tauri = window.__TAURI__;
if (tauri) {
  tauri.event.listen("blob-state", (e) => {
    const { state, count } = e.payload || {};
    setBlobState(state, count || 0);
  });
} else {
  // --- modo demonstração no navegador (sem Tauri) ---
  // Clicar no blob percorre os estados em sequência.
  let i = 0;
  blob.addEventListener("click", () => {
    i = (i + 1) % STATES.length;
    const fakeCount = STATES[i] === "alert" ? 2 : 0;
    setBlobState(STATES[i], fakeCount);
  });
}

// ---------- arrasto do blob ----------
let isDragging = false;

blob.addEventListener("mousedown", (e) => {
  isDragging = true;
  blob.style.cursor = "grabbing";
  e.preventDefault();
});

document.addEventListener("mousemove", (e) => {
  if (!isDragging) return;
  
  if (tauri) {
    // Usa movementX/Y que são o delta do mouse (mais robusto)
    tauri.core.invoke("move_window_by", {
      dx: e.movementX,
      dy: e.movementY,
    }).catch(err => {
      console.error("Move error:", err);
    });
  }
});

document.addEventListener("mouseup", () => {
  if (!isDragging) return;
  isDragging = false;
  blob.style.cursor = "move";
});

