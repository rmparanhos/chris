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
const photo = document.getElementById("sprite-photo");

const STATES = ["idle", "alert", "approved", "denied", "pr"];

// Characters drawn from PNG files (one image per state), keyed by sprite name
// -> folder under ui/. The blob/cat are inline SVG and not listed here.
const PHOTO_SPRITES = { dog: "sprites/dog" };

/** Points the <img> at the PNG for the current sprite + state. No-op for the
 *  SVG characters (blob/cat). */
function updatePhoto() {
  const dir = PHOTO_SPRITES[stage.dataset.sprite];
  if (!dir) return;
  const state = stage.dataset.state || "idle";
  photo.src = `${dir}/${state}.png`;
}

// If a PNG isn't there yet, fall back to the blob placeholder so we never show
// a broken image. (Remove the art, get the blob; drop the art in, get the dog.)
photo.addEventListener("error", () => {
  if (stage.dataset.sprite in PHOTO_SPRITES) {
    console.warn("CHRIS: sprite image missing, falling back to blob:", photo.src);
    setSprite("blob");
  }
});

/** Changes the companion's visual state. Driven by the backend at runtime.
 *  @param {string} state  idle | alert | approved | denied | pr
 *  @param {number} count  how many requests are queued (0 = hide the badge)
 */
function setBlobState(state, count = 0) {
  if (!STATES.includes(state)) return;
  stage.dataset.state = state;
  updatePhoto();
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
const SPRITES = ["blob", "cat", "dog"];

function setSprite(name) {
  if (!SPRITES.includes(name)) return;
  stage.dataset.sprite = name;
  document.querySelectorAll("#picker button").forEach((b) => {
    b.classList.toggle("active", b.dataset.sprite === name);
  });
  updatePhoto();
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

// ---------- PR notification counts (above the companion) ----------
const prcount = document.getElementById("prcount");
function setPrCounts(open, review) {
  document.getElementById("pr-open").textContent = open;
  document.getElementById("pr-review").textContent = review;
  // show the pill only when there is something worth showing
  prcount.hidden = open + review <= 0;
}

// Inside the app: the daemon drives the blob via the "blob-state" event.
const tauri = window.__TAURI__;
if (tauri) {
  tauri.event.listen("blob-state", (e) => {
    const { state, count } = e.payload || {};
    setBlobState(state, count || 0);
  });

  tauri.event.listen("pr-counts", (e) => {
    const { open, review } = e.payload || {};
    setPrCounts(open || 0, review || 0);
  });

  // Drag the whole companion window by grabbing the character anywhere (not
  // just the empty edges). The picker buttons are excluded so they stay
  // clickable. Uses the OS-level drag for smoothness.
  const appWin = tauri.window && tauri.window.getCurrentWindow && tauri.window.getCurrentWindow();
  stage.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return; // left button only
    if (e.target.closest("#picker")) return; // let the picker work
    if (appWin) appWin.startDragging().catch(() => {});
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
