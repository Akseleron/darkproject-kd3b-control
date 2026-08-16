const invoke = window.__TAURI__?.core?.invoke;

const state = {
  layout: [],
  effect: "wave",
  primary: "#4080ff",
  secondary: "#ff3000",
  brightness: 100,
  speed: 1,
  direction: "forward",
  startedAt: performance.now(),
  previewInFlight: false,
  previewEnabled: true,
  lastFrame: [],
  deviceReady: false,
  hardware: {
    armed: false,
    running: false,
    framesWritten: 0,
    detail: "Аппаратный вывод ещё не инициализирован.",
    lastError: null,
  },
};

const keyboard = document.querySelector("#keyboard");
const effectGrid = document.querySelector("#effect-grid");
const frameStatus = document.querySelector("#frame-status");
const primary = document.querySelector("#primary-color");
const secondary = document.querySelector("#secondary-color");
const brightness = document.querySelector("#brightness");
const speed = document.querySelector("#speed");

async function boot() {
  bindNavigation();
  bindControls();
  bindHardwareControls();
  if (!invoke) {
    setDeviceFailure("Tauri IPC недоступен. Запусти desktop-приложение, а не index.html напрямую.");
    frameStatus.textContent = "Tauri IPC недоступен";
    setHardwareError("Tauri IPC недоступен");
    return;
  }

  try {
    const [layout, effects] = await Promise.all([
      invoke("get_layout"),
      invoke("get_effect_catalog"),
    ]);
    state.layout = layout;
    renderKeyboard(layout);
    renderEffects(effects);
    await Promise.all([refreshDevice(), refreshHardwareStatus()]);
    state.startedAt = performance.now();
    previewLoop();
    window.setInterval(refreshHardwareStatus, 500);
  } catch (error) {
    frameStatus.textContent = `Ошибка инициализации: ${String(error)}`;
  }
}

function bindNavigation() {
  document.querySelectorAll(".nav-item[data-section]").forEach((button) => {
    button.addEventListener("click", () => {
      document.querySelectorAll(".nav-item[data-section]").forEach((item) => item.classList.remove("active"));
      button.classList.add("active");
      const section = button.dataset.section;
      document.querySelectorAll(".section").forEach((item) => item.classList.remove("active"));
      document.querySelector(`#section-${section}`)?.classList.add("active");
      document.querySelector("#page-title").textContent = section === "device" ? "Устройство" : "Подсветка";
    });
  });

  document.querySelector("#refresh-device").addEventListener("click", async () => {
    await refreshDevice();
    await refreshHardwareStatus();
  });
}

function bindControls() {
  primary.addEventListener("input", () => {
    state.primary = primary.value;
    document.querySelector("#primary-value").textContent = primary.value;
  });
  secondary.addEventListener("input", () => {
    state.secondary = secondary.value;
    document.querySelector("#secondary-value").textContent = secondary.value;
  });
  brightness.addEventListener("input", () => {
    state.brightness = Number(brightness.value);
    document.querySelector("#brightness-value").textContent = `${state.brightness}%`;
  });
  speed.addEventListener("input", () => {
    state.speed = Number(speed.value) / 100;
    document.querySelector("#speed-value").textContent = `${state.speed.toFixed(2)}×`;
  });
  document.querySelectorAll("[data-direction]").forEach((button) => {
    button.addEventListener("click", () => {
      document.querySelectorAll("[data-direction]").forEach((item) => item.classList.remove("active"));
      button.classList.add("active");
      state.direction = button.dataset.direction;
    });
  });
}

function bindHardwareControls() {
  document.querySelector("#arm-hardware").addEventListener("click", armHardwareOutput);
  document.querySelector("#disarm-hardware").addEventListener("click", disarmHardwareOutput);
  document.querySelector("#start-hardware").addEventListener("click", startHardwareEffect);
  document.querySelector("#stop-hardware").addEventListener("click", stopHardwareEffect);
  document.querySelector("#apply-frame").addEventListener("click", applyCurrentFrame);
  document.querySelector("#blackout-hardware").addEventListener("click", blackoutHardware);
}

async function refreshDevice() {
  const dot = document.querySelector("#device-dot");
  const title = document.querySelector("#device-title");
  const detail = document.querySelector("#device-detail");
  dot.className = "status-dot pending";
  title.textContent = "Проверка устройства…";
  detail.textContent = "Читаем только HID metadata.";

  try {
    const status = await invoke("get_device_status");
    const ready = status.configurationState === "ready";
    state.deviceReady = ready;
    dot.className = `status-dot ${ready ? "ready" : "blocked"}`;
    title.textContent = ready ? "KD3B подключена" : status.present ? "KD3B требует внимания" : "KD3B не найдена";
    detail.textContent = ready && status.selected
      ? `Interface ${status.selected.interfaceNumber} · ${status.selected.path}`
      : status.detail;
    renderDeviceSummary(status);
    renderHardwareStatus(state.hardware);
  } catch (error) {
    state.deviceReady = false;
    setDeviceFailure(String(error));
    renderHardwareStatus(state.hardware);
  }
}

function setDeviceFailure(message) {
  const dot = document.querySelector("#device-dot");
  dot.className = "status-dot blocked";
  document.querySelector("#device-title").textContent = "Ошибка проверки";
  document.querySelector("#device-detail").textContent = message;
  document.querySelector("#device-summary").innerHTML = `<div class="boundary-note"><strong>Ошибка</strong><p>${escapeHtml(message)}</p></div>`;
}

function renderDeviceSummary(status) {
  const selected = status.selected;
  const rows = [
    ["Состояние", status.configurationState],
    ["HID интерфейсов", String(status.matchingInterfaces)],
    ["Interface", selected ? String(selected.interfaceNumber) : "—"],
    ["Path", selected?.path ?? "—"],
    ["VID:PID", selected ? `${hex4(selected.vendorId)}:${hex4(selected.productId)}` : "195d:2061"],
    ["Release", selected ? `0x${hex4(selected.releaseNumber)}` : "—"],
    ["Bus", selected?.bus ?? "—"],
  ];
  document.querySelector("#device-summary").innerHTML = rows
    .map(([label, value]) => `<div class="summary-row"><span>${escapeHtml(label)}</span><span>${escapeHtml(value)}</span></div>`)
    .join("");
}

function renderKeyboard(layout) {
  keyboard.replaceChildren();
  const fragment = document.createDocumentFragment();
  for (const key of layout) {
    const element = document.createElement("div");
    element.className = "key";
    element.dataset.index = String(key.index);
    element.style.gridColumn = String(key.column + 1);
    element.style.gridRow = String(key.row + 1);
    element.title = key.name;
    const label = document.createElement("span");
    label.textContent = keyLabel(key.name);
    element.append(label);
    fragment.append(element);
  }
  keyboard.append(fragment);
}

function renderEffects(effects) {
  effectGrid.replaceChildren();
  for (const effect of effects) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = `effect-button${effect.id === state.effect ? " active" : ""}`;
    button.dataset.effect = effect.id;
    button.textContent = effect.label;
    button.addEventListener("click", () => {
      state.effect = effect.id;
      effectGrid.querySelectorAll(".effect-button").forEach((item) => item.classList.remove("active"));
      button.classList.add("active");
      state.startedAt = performance.now();
    });
    effectGrid.append(button);
  }
}

function previewLoop() {
  window.setInterval(updatePreview, 50);
}

async function updatePreview() {
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
    frameStatus.textContent = `${state.effect} · ${state.brightness}% · ${state.speed.toFixed(2)}×`;
    renderHardwareStatus(state.hardware);
  } catch (error) {
    state.previewEnabled = false;
    frameStatus.textContent = `Preview остановлен: ${String(error)}`;
  } finally {
    state.previewInFlight = false;
  }
}

function effectRequest() {
  return {
    kind: state.effect,
    primary: state.primary,
    secondary: state.secondary,
    speed: state.speed,
    brightnessPercent: state.brightness,
    direction: state.direction,
  };
}

async function armHardwareOutput() {
  const confirmation = window.prompt(
    "Это разрешит volatile Direct RGB записи на interface 2 до закрытия приложения.\n\nВведите точно: ENABLE VOLATILE RGB",
    "",
  );
  if (confirmation === null) return;

  try {
    const status = await invoke("arm_hardware_output", { confirmation });
    renderHardwareStatus(status);
  } catch (error) {
    setHardwareError(String(error));
  }
}

async function disarmHardwareOutput() {
  try {
    const status = await invoke("disarm_hardware_output");
    renderHardwareStatus(status);
  } catch (error) {
    setHardwareError(String(error));
  }
}

async function startHardwareEffect() {
  try {
    const status = await invoke("start_effect_output", { request: effectRequest() });
    renderHardwareStatus(status);
  } catch (error) {
    setHardwareError(String(error));
  }
}

async function stopHardwareEffect() {
  try {
    const status = await invoke("stop_effect_output");
    renderHardwareStatus(status);
  } catch (error) {
    setHardwareError(String(error));
  }
}

async function applyCurrentFrame() {
  if (state.lastFrame.length !== state.layout.length) return;
  try {
    const status = await invoke("apply_static_frame", { colors: state.lastFrame });
    renderHardwareStatus(status);
  } catch (error) {
    setHardwareError(String(error));
  }
}

async function blackoutHardware() {
  if (!state.layout.length) return;
  try {
    const colors = Array(state.layout.length).fill("#000000");
    const status = await invoke("apply_static_frame", { colors });
    renderHardwareStatus(status);
  } catch (error) {
    setHardwareError(String(error));
  }
}

async function refreshHardwareStatus() {
  if (!invoke) return;
  try {
    const status = await invoke("get_hardware_output_status");
    renderHardwareStatus(status);
  } catch (error) {
    setHardwareError(String(error));
  }
}

function renderHardwareStatus(status) {
  state.hardware = status;
  const title = document.querySelector("#hardware-title");
  const stateLabel = document.querySelector("#hardware-state");
  const detail = document.querySelector("#hardware-detail");
  const panel = document.querySelector("#hardware-panel");

  const hasError = Boolean(status.lastError);
  if (hasError) {
    title.textContent = "Ошибка аппаратного вывода";
    stateLabel.textContent = "ERROR";
    panel.dataset.state = "error";
  } else if (status.running) {
    title.textContent = "Эффект работает на клавиатуре";
    stateLabel.textContent = "STREAMING";
    panel.dataset.state = "running";
  } else if (status.armed) {
    title.textContent = "Аппаратный вывод разблокирован";
    stateLabel.textContent = "ARMED";
    panel.dataset.state = "armed";
  } else {
    title.textContent = "Аппаратный вывод заблокирован";
    stateLabel.textContent = "LOCKED";
    panel.dataset.state = "locked";
  }

  detail.textContent = status.lastError || status.detail || "Ожидание команды.";
  document.querySelector("#hardware-frames").textContent = String(status.framesWritten ?? 0);

  const canWrite = Boolean(status.armed && state.deviceReady);
  document.querySelector("#arm-hardware").disabled = Boolean(status.armed);
  document.querySelector("#disarm-hardware").disabled = !status.armed;
  document.querySelector("#start-hardware").disabled = !canWrite;
  document.querySelector("#stop-hardware").disabled = !status.running;
  document.querySelector("#apply-frame").disabled = !canWrite || state.lastFrame.length !== state.layout.length;
  document.querySelector("#blackout-hardware").disabled = !canWrite;
}

function setHardwareError(message) {
  renderHardwareStatus({
    ...state.hardware,
    running: false,
    detail: message,
    lastError: message,
  });
}

function applyFrame(colors) {
  colors.forEach((color, index) => {
    const key = keyboard.querySelector(`[data-index="${index}"]`);
    if (!key) return;
    key.style.backgroundColor = color;
    key.style.borderColor = tint(color, 0.25);
    key.style.boxShadow = `inset 0 -2px 0 rgba(0,0,0,.28), 0 0 11px ${withAlpha(color, 0.22)}`;
  });
}

function keyLabel(name) {
  const labels = {
    Escape: "Esc", Backspace: "Bksp", LeftBracket: "[", RightBracket: "]",
    Backslash: "\\", Semicolon: ";", Apostrophe: "'", Comma: ",", Period: ".",
    Slash: "/", Grave: "`", CapsLock: "Caps", LeftShift: "LShift", RightShift: "RShift",
    LeftCtrl: "LCtrl", RightCtrl: "RCtrl", LeftAlt: "LAlt", RightAlt: "RAlt",
    LeftMeta: "Meta", Space: "Space", PrintScreen: "PrtSc", ScrollLock: "ScrLk",
    PageUp: "PgUp", PageDown: "PgDn", Insert: "Ins", Delete: "Del",
    ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→",
  };
  if (labels[name]) return labels[name];
  if (name.startsWith("Digit")) return name.slice(5);
  return name;
}

function hex4(value) { return Number(value).toString(16).padStart(4, "0"); }
function escapeHtml(value) {
  return String(value).replace(/[&<>'"]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" }[char]));
}
function withAlpha(hex, alpha) {
  const value = hex.replace("#", "");
  const r = parseInt(value.slice(0, 2), 16);
  const g = parseInt(value.slice(2, 4), 16);
  const b = parseInt(value.slice(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}
function tint(hex, amount) {
  const value = hex.replace("#", "");
  const channels = [0, 2, 4].map((offset) => Math.min(255, Math.round(parseInt(value.slice(offset, offset + 2), 16) + 255 * amount)));
  return `rgb(${channels.join(",")})`;
}

boot();
