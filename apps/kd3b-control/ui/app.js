const invoke = window.__TAURI__?.core?.invoke;

const PREVIEW_INTERVAL_MS = 1000 / 60;
const HARDWARE_STATUS_INTERVAL_MS = 1000;
const PROFILE_STORAGE_KEY = "kd3b-control.profiles.v1";

const state = {
  layout: [],
  keyElements: [],
  lightingMode: "effect",
  effect: "wave",
  primary: "#4080ff",
  secondary: "#ff3000",
  brightness: 100,
  speed: 1,
  direction: "forward",
  directColor: "#4080ff",
  directColors: [],
  directInitialized: false,
  directDirty: false,
  selectedKeys: new Set(),
  profiles: [],
  startedAt: performance.now(),
  previewInFlight: false,
  previewEnabled: true,
  previewLastAt: 0,
  activeSection: "lighting",
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
const directColor = document.querySelector("#direct-color");

const VISUAL_LAYOUT = buildVisualLayout();

async function boot() {
  bindNavigation();
  bindControls();
  bindHardwareControls();
  bindDirectEditor();
  bindProfiles();

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
    state.directColors = Array(layout.length).fill("#000000");
    renderKeyboard(layout);
    renderEffects(effects);
    loadProfiles();
    await Promise.all([refreshDevice(), refreshHardwareStatus()]);
    state.startedAt = performance.now();
    window.requestAnimationFrame(previewLoop);
    window.setInterval(refreshHardwareStatus, HARDWARE_STATUS_INTERVAL_MS);
  } catch (error) {
    frameStatus.textContent = `Ошибка инициализации: ${String(error)}`;
  }
}

function bindNavigation() {
  const titles = {
    lighting: "Подсветка",
    profiles: "Профили",
    device: "Устройство",
  };

  document.querySelectorAll(".nav-item[data-section]").forEach((button) => {
    button.addEventListener("click", () => {
      document.querySelectorAll(".nav-item[data-section]").forEach((item) => item.classList.remove("active"));
      button.classList.add("active");
      const section = button.dataset.section;
      state.activeSection = section;
      document.querySelectorAll(".section").forEach((item) => item.classList.remove("active"));
      document.querySelector(`#section-${section}`)?.classList.add("active");
      document.querySelector("#page-title").textContent = titles[section] ?? "KD3B Control";
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
  document.querySelectorAll("[data-lighting-mode]").forEach((button) => {
    button.addEventListener("click", () => switchLightingMode(button.dataset.lightingMode));
  });
}

function bindDirectEditor() {
  keyboard.addEventListener("click", (event) => {
    if (state.lightingMode !== "direct") return;
    const key = event.target.closest(".key");
    if (!key) return;
    const index = Number(key.dataset.index);
    if (!Number.isInteger(index)) return;
    if (state.selectedKeys.has(index)) state.selectedKeys.delete(index);
    else state.selectedKeys.add(index);
    updateSelectionVisuals();
  });

  directColor.addEventListener("input", () => {
    state.directColor = directColor.value;
    document.querySelector("#direct-value").textContent = directColor.value;
  });

  document.querySelector("#fill-selection").addEventListener("click", () => {
    if (!state.selectedKeys.size) {
      document.querySelector("#selection-status").textContent = "Сначала выбери клавиши";
      return;
    }
    for (const index of state.selectedKeys) {
      if (index >= 0 && index < state.directColors.length) state.directColors[index] = state.directColor;
    }
    state.directDirty = true;
    renderDirectFrame();
  });

  document.querySelectorAll("[data-key-group]").forEach((button) => {
    button.addEventListener("click", () => selectKeyGroup(button.dataset.keyGroup));
  });
}

function bindProfiles() {
  document.querySelector("#save-profile").addEventListener("click", saveCurrentProfile);
  document.querySelector("#profile-name").addEventListener("keydown", (event) => {
    if (event.key === "Enter") saveCurrentProfile();
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
  state.keyElements = Array(layout.length);
  const fragment = document.createDocumentFragment();

  for (const key of layout) {
    const element = document.createElement("div");
    const geometry = VISUAL_LAYOUT.get(key.name);
    element.className = "key";
    element.dataset.index = String(key.index);
    element.dataset.name = key.name;

    if (geometry) {
      element.style.gridColumn = `${geometry.start} / span ${geometry.span}`;
      element.style.gridRow = String(geometry.row);
    } else {
      element.style.gridColumn = `${key.column * 4 + 1} / span 4`;
      element.style.gridRow = String(key.row + 1);
    }

    element.title = key.name;
    const label = document.createElement("span");
    label.textContent = keyLabel(key.name);
    element.append(label);
    state.keyElements[key.index] = element;
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

function switchLightingMode(mode) {
  if (mode !== "effect" && mode !== "direct") return;
  if (mode === "direct" && !state.directInitialized) {
    if (state.lastFrame.length === state.layout.length) state.directColors = [...state.lastFrame];
    state.directInitialized = true;
    state.directDirty = true;
  }

  state.lightingMode = mode;
  document.querySelectorAll("[data-lighting-mode]").forEach((button) => {
    button.classList.toggle("active", button.dataset.lightingMode === mode);
  });
  document.querySelector("#effect-editor").hidden = mode !== "effect";
  document.querySelector("#direct-editor").hidden = mode !== "direct";
  document.querySelector("#effect-parameters").hidden = mode !== "effect";
  keyboard.classList.toggle("direct-edit", mode === "direct");

  if (mode === "effect") {
    state.startedAt = performance.now();
    state.previewEnabled = true;
  } else {
    renderDirectFrame();
  }
  updateSelectionVisuals();
  renderHardwareStatus(state.hardware);
}

function previewLoop(timestamp) {
  window.requestAnimationFrame(previewLoop);

  if (document.hidden || state.activeSection !== "lighting") return;
  if (state.lightingMode === "direct") {
    if (state.directDirty) renderDirectFrame();
    return;
  }
  if (timestamp - state.previewLastAt < PREVIEW_INTERVAL_MS) return;

  state.previewLastAt = timestamp;
  void updatePreview();
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
  } catch (error) {
    state.previewEnabled = false;
    frameStatus.textContent = `Preview остановлен: ${String(error)}`;
  } finally {
    state.previewInFlight = false;
  }
}

function renderDirectFrame() {
  if (state.directColors.length !== state.layout.length) return;
  state.lastFrame = [...state.directColors];
  applyFrame(state.directColors);
  state.directDirty = false;
  frameStatus.textContent = `direct · ${state.selectedKeys.size} выбрано`;
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

function selectKeyGroup(group) {
  const groups = {
    wasd: ["W", "A", "S", "D"],
    arrows: ["Up", "Down", "Left", "Right"],
    function: Array.from({ length: 12 }, (_, index) => `F${index + 1}`),
    navigation: ["Insert", "Home", "PageUp", "Delete", "End", "PageDown"],
  };

  state.selectedKeys.clear();
  if (group === "all") {
    state.layout.forEach((key) => state.selectedKeys.add(key.index));
  } else if (group !== "none") {
    const names = new Set(groups[group] ?? []);
    state.layout.forEach((key) => {
      if (names.has(key.name)) state.selectedKeys.add(key.index);
    });
  }
  updateSelectionVisuals();
}

function updateSelectionVisuals() {
  state.keyElements.forEach((element, index) => {
    element?.classList.toggle("selected", state.lightingMode === "direct" && state.selectedKeys.has(index));
  });
  const status = document.querySelector("#selection-status");
  if (status) status.textContent = `${state.selectedKeys.size} выбрано`;
}

function loadProfiles() {
  try {
    const raw = window.localStorage.getItem(PROFILE_STORAGE_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    state.profiles = Array.isArray(parsed) ? parsed.filter(isValidProfile) : [];
  } catch {
    state.profiles = [];
  }
  renderProfiles();
}

function persistProfiles() {
  window.localStorage.setItem(PROFILE_STORAGE_KEY, JSON.stringify(state.profiles));
}

function saveCurrentProfile() {
  const input = document.querySelector("#profile-name");
  const name = input.value.trim();
  if (!name) {
    input.focus();
    input.placeholder = "Введи название профиля";
    return;
  }

  const profile = {
    version: 1,
    name,
    updatedAt: new Date().toISOString(),
    lightingMode: state.lightingMode,
    effect: state.effect,
    primary: state.primary,
    secondary: state.secondary,
    brightness: state.brightness,
    speed: state.speed,
    direction: state.direction,
    directColor: state.directColor,
    directColors: [...state.directColors],
  };

  const existing = state.profiles.findIndex((item) => item.name === name);
  if (existing >= 0) state.profiles[existing] = profile;
  else state.profiles.push(profile);
  state.profiles.sort((left, right) => left.name.localeCompare(right.name, "ru"));
  persistProfiles();
  renderProfiles();
  input.value = "";
}

function loadProfile(profile) {
  state.effect = profile.effect;
  state.primary = profile.primary;
  state.secondary = profile.secondary;
  state.brightness = profile.brightness;
  state.speed = profile.speed;
  state.direction = profile.direction;
  state.directColor = profile.directColor;
  if (profile.directColors.length === state.layout.length) {
    state.directColors = [...profile.directColors];
    state.directInitialized = true;
    state.directDirty = true;
  }
  syncControlsFromState();
  switchLightingMode(profile.lightingMode);
}

function deleteProfile(name) {
  state.profiles = state.profiles.filter((profile) => profile.name !== name);
  persistProfiles();
  renderProfiles();
}

function renderProfiles() {
  const list = document.querySelector("#profile-list");
  if (!list) return;
  list.replaceChildren();

  if (!state.profiles.length) {
    const empty = document.createElement("div");
    empty.className = "profile-empty";
    empty.textContent = "Host-профилей пока нет. Настрой подсветку и сохрани текущее состояние.";
    list.append(empty);
    return;
  }

  for (const profile of state.profiles) {
    const card = document.createElement("article");
    card.className = "profile-item";
    const info = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = profile.name;
    const meta = document.createElement("span");
    meta.textContent = profile.lightingMode === "direct"
      ? "Direct RGB · 87-key frame"
      : `${profile.effect} · ${profile.brightness}% · ${profile.speed.toFixed(2)}×`;
    info.append(title, meta);

    const actions = document.createElement("div");
    actions.className = "profile-actions";
    const load = document.createElement("button");
    load.type = "button";
    load.className = "profile-button primary";
    load.textContent = "Загрузить";
    load.addEventListener("click", () => loadProfile(profile));
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "profile-button";
    remove.textContent = "Удалить";
    remove.addEventListener("click", () => deleteProfile(profile.name));
    actions.append(load, remove);
    card.append(info, actions);
    list.append(card);
  }
}

function isValidProfile(profile) {
  return Boolean(
    profile &&
    profile.version === 1 &&
    typeof profile.name === "string" &&
    (profile.lightingMode === "effect" || profile.lightingMode === "direct") &&
    typeof profile.effect === "string" &&
    typeof profile.primary === "string" &&
    typeof profile.secondary === "string" &&
    Number.isFinite(profile.brightness) &&
    Number.isFinite(profile.speed) &&
    (profile.direction === "forward" || profile.direction === "reverse") &&
    typeof profile.directColor === "string" &&
    Array.isArray(profile.directColors)
  );
}

function syncControlsFromState() {
  primary.value = state.primary;
  secondary.value = state.secondary;
  brightness.value = String(state.brightness);
  speed.value = String(Math.round(state.speed * 100));
  directColor.value = state.directColor;
  document.querySelector("#primary-value").textContent = state.primary;
  document.querySelector("#secondary-value").textContent = state.secondary;
  document.querySelector("#brightness-value").textContent = `${state.brightness}%`;
  document.querySelector("#speed-value").textContent = `${state.speed.toFixed(2)}×`;
  document.querySelector("#direct-value").textContent = state.directColor;
  document.querySelectorAll("[data-direction]").forEach((button) => {
    button.classList.toggle("active", button.dataset.direction === state.direction);
  });
  effectGrid.querySelectorAll(".effect-button").forEach((button) => {
    button.classList.toggle("active", button.dataset.effect === state.effect);
  });
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
  if (state.lightingMode !== "effect") return;
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
  document.querySelector("#start-hardware").disabled = !canWrite || state.lightingMode !== "effect";
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
    const key = state.keyElements[index];
    if (!key || key.dataset.color === color) return;
    key.dataset.color = color;
    key.style.backgroundColor = color;
  });
}

function buildVisualLayout() {
  const map = new Map();
  const add = (row, x, width, names) => {
    names.forEach((name, index) => {
      const startX = Array.isArray(x) ? x[index] : x + index;
      const keyWidth = Array.isArray(width) ? width[index] : width;
      map.set(name, {
        row,
        start: Math.round(startX * 4) + 1,
        span: Math.round(keyWidth * 4),
      });
    });
  };

  add(1, [0, 2, 3, 4, 5, 6.5, 7.5, 8.5, 9.5, 11, 12, 13, 14, 15.5, 16.5, 17.5], 1,
    ["Esc", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "PrintScreen", "ScrollLock", "Pause"]);
  add(2, 0, 1, ["Backtick", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9", "Digit0", "Minus", "Equal"]);
  add(2, [13, 15.5, 16.5, 17.5], [2, 1, 1, 1], ["Backspace", "Insert", "Home", "PageUp"]);
  add(3, [0, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5, 10.5, 11.5, 12.5, 13.5, 15.5, 16.5, 17.5],
    [1.5, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1.5, 1, 1, 1],
    ["Tab", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "LeftBracket", "RightBracket", "Backslash", "Delete", "End", "PageDown"]);
  add(4, [0, 1.75, 2.75, 3.75, 4.75, 5.75, 6.75, 7.75, 8.75, 9.75, 10.75, 11.75, 12.75],
    [1.75, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2.25],
    ["CapsLock", "A", "S", "D", "F", "G", "H", "J", "K", "L", "Semicolon", "Quote", "Enter"]);
  add(5, [0, 2.25, 3.25, 4.25, 5.25, 6.25, 7.25, 8.25, 9.25, 10.25, 11.25, 16.5],
    [2.25, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2.75, 1],
    ["LeftShift", "Z", "X", "C", "V", "B", "N", "M", "Comma", "Period", "RightShift", "Up"]);
  add(5, 10.25, 1, ["Slash"]);
  add(6, [0, 1.25, 2.5, 3.75, 10, 11.25, 12.5, 13.75, 15.5, 16.5, 17.5],
    [1.25, 1.25, 1.25, 6.25, 1.25, 1.25, 1.25, 1.25, 1, 1, 1],
    ["LeftCtrl", "LeftMeta", "LeftAlt", "Space", "RightAlt", "Fn", "Menu", "RightCtrl", "Left", "Down", "Right"]);

  return map;
}

function keyLabel(name) {
  const labels = {
    Esc: "Esc", Backtick: "`", Backspace: "Bksp", LeftBracket: "[", RightBracket: "]",
    Backslash: "\\", Semicolon: ";", Quote: "'", Comma: ",", Period: ".",
    Slash: "/", CapsLock: "Caps", LeftShift: "LShift", RightShift: "RShift",
    LeftCtrl: "LCtrl", RightCtrl: "RCtrl", LeftAlt: "LAlt", RightAlt: "RAlt",
    LeftMeta: "Meta", Space: "Space", PrintScreen: "PrtSc", ScrollLock: "ScrLk",
    PageUp: "PgUp", PageDown: "PgDn", Insert: "Ins", Delete: "Del",
    Up: "↑", Down: "↓", Left: "←", Right: "→",
  };
  if (labels[name]) return labels[name];
  if (name.startsWith("Digit")) return name.slice(5);
  return name;
}

function hex4(value) { return Number(value).toString(16).padStart(4, "0"); }
function escapeHtml(value) {
  return String(value).replace(/[&<>'"]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" }[char]));
}

boot();
