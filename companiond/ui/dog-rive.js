// Optional Rive-powered dog.
//
// If `sprites/dog/dog.riv` exists, the dog is rendered by Rive — mesh
// deformation gives real chest breathing and per-state reactions, which a flat
// PNG + CSS scale can't do. If the .riv is missing (or the runtime fails to
// load), this stays inert and blob.js keeps using the per-state PNGs, so there
// is no regression. Build the .riv with the recipe in sprites/dog/DOG_RIVE.md.
//
// Contract the .riv must follow (see the recipe):
//   - a State Machine (the first one in the file is used), and
//   - a Number input named `state`: 0=idle 1=alert 2=approved 3=denied 4=pr.
(function () {
  const STATES = ["idle", "alert", "approved", "denied", "pr"];
  const RIV_URL = "sprites/dog/dog.riv";

  const canvas = document.getElementById("dog-rive");
  const stage = document.getElementById("stage");

  let instance = null; // rive.Rive
  let stateInput = null; // the Number input named "state"
  let status = "idle"; // idle | loading | active | failed
  let pending = 0; // last requested state index (applied once loaded)

  function indexOf(state) {
    const i = STATES.indexOf(state);
    return i < 0 ? 0 : i;
  }

  function setState(idx) {
    pending = idx;
    if (stateInput) stateInput.value = idx;
  }

  function isActive() {
    return status === "active";
  }

  function fail() {
    status = "failed";
    stage.dataset.dogRive = "off";
    if (instance) {
      try {
        instance.cleanup();
      } catch (_) {}
      instance = null;
    }
  }

  // Lazily start Rive the first time the dog is selected. No-op if already
  // tried (loading/active/failed) or if the runtime/canvas isn't there.
  function ensure() {
    if (status !== "idle") return;
    if (!window.rive || !canvas) {
      fail();
      return;
    }
    status = "loading";
    // The app is offline, so load the wasm from our vendored copy instead of
    // the default CDN.
    try {
      window.rive.RuntimeLoader.setWasmUrl("vendor/rive.wasm");
    } catch (_) {}

    instance = new window.rive.Rive({
      src: RIV_URL,
      canvas: canvas,
      autoplay: true,
      layout: new window.rive.Layout({
        fit: window.rive.Fit.contain,
        alignment: window.rive.Alignment.center,
      }),
      onLoad: () => {
        try {
          const names = instance.stateMachineNames || [];
          if (!names.length) return fail();
          const sm = names[0];
          instance.play(sm);
          const inputs = instance.stateMachineInputs(sm) || [];
          stateInput = inputs.find((i) => i.name === "state") || null;
          instance.resizeDrawingSurfaceToCanvas();
          status = "active";
          stage.dataset.dogRive = "on"; // CSS swaps the PNG for the canvas
          setState(pending); // apply whatever state we're in right now
        } catch (_) {
          fail();
        }
      },
      onLoadError: fail,
    });
  }

  // keep the drawing surface crisp if the window is resized
  window.addEventListener("resize", () => {
    if (instance && status === "active") {
      try {
        instance.resizeDrawingSurfaceToCanvas();
      } catch (_) {}
    }
  });

  window.dogRive = { ensure, setState, isActive, indexOf };
})();
