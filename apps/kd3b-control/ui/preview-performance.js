(() => {
  const TARGET_RENDER_HZ = 60;
  const RENDER_INTERVAL_MS = 1000 / TARGET_RENDER_HZ;
  const STATUS_INTERVAL_MS = 250;
  const FPS_SAMPLE_MS = 750;

  const canvas = document.createElement("canvas");
  canvas.className = "keyboard-color-canvas";
  canvas.setAttribute("aria-hidden", "true");

  const context = canvas.getContext("2d", { alpha: true });
  let keyRects = [];
  let geometryWidth = 0;
  let geometryHeight = 0;
  let geometryClientWidth = 0;
  let geometryClientHeight = 0;
  let lastStatusAt = 0;
  let displayFrames = 0;
  let displayWindowStartedAt = performance.now();
  let renderFrames = 0;
  let renderWindowStartedAt = performance.now();
  let measuredDisplayFps = 0;
  let measuredRenderFps = 0;

  function ensureCanvasAttached() {
    if (canvas.parentElement !== keyboard) {
      keyboard.prepend(canvas);
    }
    keyboard.classList.add("canvas-preview");
  }

  function roundedRectPath(ctx, x, y, width, height, radius) {
    const r = Math.max(0, Math.min(radius, width / 2, height / 2));
    ctx.beginPath();
    ctx.moveTo(x + r, y);
    ctx.lineTo(x + width - r, y);
    ctx.quadraticCurveTo(x + width, y, x + width, y + r);
    ctx.lineTo(x + width, y + height - r);
    ctx.quadraticCurveTo(x + width, y + height, x + width - r, y + height);
    ctx.lineTo(x + r, y + height);
    ctx.quadraticCurveTo(x, y + height, x, y + height - r);
    ctx.lineTo(x, y + r);
    ctx.quadraticCurveTo(x, y, x + r, y);
    ctx.closePath();
  }

  function syncCanvasGeometry() {
    if (!context) return;
    ensureCanvasAttached();

    const bounds = keyboard.getBoundingClientRect();
    if (bounds.width <= 0 || bounds.height <= 0) return;

    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const pixelWidth = Math.max(1, Math.round(bounds.width * dpr));
    const pixelHeight = Math.max(1, Math.round(bounds.height * dpr));

    if (canvas.width !== pixelWidth || canvas.height !== pixelHeight) {
      canvas.width = pixelWidth;
      canvas.height = pixelHeight;
    }

    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    geometryWidth = bounds.width;
    geometryHeight = bounds.height;
    geometryClientWidth = keyboard.clientWidth;
    geometryClientHeight = keyboard.clientHeight;
    keyRects = state.keyElements.map((element) => {
      if (!element) return null;
      const rect = element.getBoundingClientRect();
      return {
        x: rect.left - bounds.left,
        y: rect.top - bounds.top,
        width: rect.width,
        height: rect.height,
      };
    });

    if (state.lastFrame.length === state.layout.length) {
      paintFrame(state.lastFrame, false, true);
    }
  }

  function ensureGeometry(colors) {
    if (
      canvas.parentElement !== keyboard ||
      keyRects.length !== colors.length ||
      geometryClientWidth !== keyboard.clientWidth ||
      geometryClientHeight !== keyboard.clientHeight
    ) {
      syncCanvasGeometry();
    }
  }

  function paintFrame(colors, countFrame = true, geometryAlreadySynced = false) {
    if (!context || !colors.length) return;
    if (!geometryAlreadySynced) ensureGeometry(colors);
    if (keyRects.length !== colors.length || canvas.parentElement !== keyboard) return;

    context.clearRect(0, 0, geometryWidth, geometryHeight);
    for (let index = 0; index < colors.length; index += 1) {
      const rect = keyRects[index];
      if (!rect) continue;
      context.fillStyle = colors[index];
      roundedRectPath(context, rect.x, rect.y, rect.width, rect.height, 6);
      context.fill();
    }

    if (countFrame) {
      renderFrames += 1;
      const now = performance.now();
      const elapsed = now - renderWindowStartedAt;
      if (elapsed >= FPS_SAMPLE_MS) {
        measuredRenderFps = (renderFrames * 1000) / elapsed;
        renderFrames = 0;
        renderWindowStartedAt = now;
        updateFpsLabel();
      }
    }
  }

  function updateFpsLabel() {
    const label = document.querySelector("#fps-label");
    if (!label) return;
    const display = Math.round(measuredDisplayFps);
    const render = Math.round(measuredRenderFps);
    label.textContent = display > 0
      ? `${display} FPS display · ${render || TARGET_RENDER_HZ} Hz render`
      : `${TARGET_RENDER_HZ} Hz render`;
  }

  function sampleDisplayFps(timestamp) {
    displayFrames += 1;
    const elapsed = timestamp - displayWindowStartedAt;
    if (elapsed >= FPS_SAMPLE_MS) {
      measuredDisplayFps = (displayFrames * 1000) / elapsed;
      displayFrames = 0;
      displayWindowStartedAt = timestamp;
      updateFpsLabel();
    }
    window.requestAnimationFrame(sampleDisplayFps);
  }

  // The old DOM renderer changed 87 element styles for every frame. On WebKitGTK with the
  // NVIDIA DMABUF renderer disabled that creates avoidable style/paint work during scrolling.
  // Keep the semantic key elements for labels, hit testing and selection, but paint all colors
  // in one canvas layer instead.
  applyFrame = function applyFrameCanvas(colors) {
    paintFrame(colors);
  };

  // requestAnimationFrame is kept only as a passive display-FPS probe. Preview generation uses
  // its own timer so a WebKit rendering update or asynchronous scroll does not become the clock
  // for the effect engine.
  previewLoop = function previewLoopDisabled() {};

  updatePreview = async function updatePreviewOptimized() {
    if (!state.previewEnabled || state.previewInFlight || !state.layout.length) return;
    state.previewInFlight = true;
    try {
      const elapsedSeconds = (performance.now() - state.startedAt) / 1000;
      const frame = await invoke("preview_effect", {
        request: {
          ...effectRequest(),
          elapsedSeconds,
        },
      });
      state.lastFrame = frame.colors;
      applyFrame(frame.colors);

      const now = performance.now();
      if (now - lastStatusAt >= STATUS_INTERVAL_MS) {
        frameStatus.textContent = `${state.effect} · ${state.brightness}% · ${state.speed.toFixed(2)}×`;
        lastStatusAt = now;
      }
    } catch (error) {
      state.previewEnabled = false;
      frameStatus.textContent = `Preview остановлен: ${String(error)}`;
    } finally {
      state.previewInFlight = false;
    }
  };

  window.setInterval(() => {
    if (document.hidden || state.activeSection !== "lighting") return;
    if (state.lightingMode === "direct") {
      if (state.directDirty) renderDirectFrame();
      return;
    }
    void updatePreview();
  }, RENDER_INTERVAL_MS);

  const resizeObserver = typeof ResizeObserver === "function"
    ? new ResizeObserver(syncCanvasGeometry)
    : null;
  resizeObserver?.observe(keyboard);
  window.addEventListener("resize", syncCanvasGeometry, { passive: true });
  window.requestAnimationFrame(sampleDisplayFps);
})();
