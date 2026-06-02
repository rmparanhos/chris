// Controle do blob. Em M1 é tudo no lado do navegador (sem chamar APIs do
// Tauri ainda) — assim você pode abrir este arquivo direto no navegador e ver
// o blob funcionando. A partir do M3 o daemon (Rust) é que vai dirigir o
// estado, conforme as aprovações chegarem.

const stage = document.getElementById("stage");
const badge = document.getElementById("badge");
const blob = document.getElementById("blob");

const STATES = ["idle", "alert", "approved", "denied"];

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

// expõe a função globalmente (o Tauri poderá chamá-la via JS depois)
window.setBlobState = setBlobState;

// --- modo demonstração (só pra você ver os estados) ---
// Clicar no blob percorre os estados em sequência.
let i = 0;
blob.addEventListener("click", () => {
  i = (i + 1) % STATES.length;
  const fakeCount = STATES[i] === "alert" ? 2 : 0;
  setBlobState(STATES[i], fakeCount);
});
