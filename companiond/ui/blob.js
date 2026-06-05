// Companion controller. The visuals run entirely on the browser side, so you
// can open this file directly in a browser and watch the blob work. Inside the
// app the daemon (Rust) drives the state as approvals arrive.
//
// Dragging is handled natively by `data-tauri-drag-region` on #stage (set in
// index.html), so there is no manual drag code here — the OS moves the window
// smoothly and the daemon repositions the notifications to follow it.

const stage = document.getElementById("stage");
const badge = document.getElementById("badge");
const blob = document.getElementById("blob");

const STATES = ["idle", "alert", "approved", "denied", "pr"];

/** Changes the companion's visual state. Driven by the backend at runtime.
 *  @param {string} state  idle | alert | approved | denied | pr
 *  @param {number} count  how many requests are queued (0 = hide the badge)
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

// expose the function globally
window.setBlobState = setBlobState;

// ---------- character (sprite) picker ----------
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

// restore the last choice and wire up the buttons
setSprite(localStorage.getItem("chris.sprite") || "blob");
document.querySelectorAll("#picker button").forEach((b) => {
  b.addEventListener("click", (e) => {
    e.stopPropagation(); // don't trigger the blob's state cycle
    setSprite(b.dataset.sprite);
  });
});

// Inside the app: the daemon drives the blob via the "blob-state" event.
const tauri = window.__TAURI__;
if (tauri) {
  tauri.event.listen("blob-state", (e) => {
    const { state, count } = e.payload || {};
    setBlobState(state, count || 0);
  });
} else {
  // --- browser demo mode (no Tauri) ---
  // Clicking the blob cycles through the states in order.
  let i = 0;
  blob.addEventListener("click", () => {
    i = (i + 1) % STATES.length;
    const fakeCount = STATES[i] === "alert" ? 2 : 0;
    setBlobState(STATES[i], fakeCount);
  });
}
