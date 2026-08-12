(() => {
  const tauri = window.__TAURI__;
  const invoke = tauri?.core?.invoke;
  const listen = tauri?.event?.listen;
  const $ = (id) => document.getElementById(id);
  const state = {
    connected: false,
    devices: [],
    selectedDevice: null,
    battery: { left: null, right: null, case: null },
    ancMode: null,
    gameMode: false,
    features: { anc: false, gameMode: false, eqPreset: false },
    hasPresets: false,
  };

  const payloadOf = (event) => event && Object.prototype.hasOwnProperty.call(event, "payload") ? event.payload : event;
  const validAddress = (address) => typeof address === "string" && address.trim() !== "" && address.trim() !== "—";

  function fmtError(error) {
    if (error && typeof error === "object" && error.code) {
      const detail = error.detail || "";
      return I18n.t(`errors.${error.code}`, { detail }, detail || error.code);
    }
    if (error instanceof Error) return error.message;
    if (typeof error === "string") return error;
    try { return JSON.stringify(error); } catch (_) { return String(error); }
  }

  function setNotice(element, message, error = false) {
    if (!element) return;
    element.textContent = message || "";
    element.classList.toggle("error", Boolean(error));
  }

  function showView(view) {
    $("view-landing").hidden = view !== "landing";
    $("view-control").hidden = view !== "control";
  }

  function setConnectLabel(key) {
    const button = $("landing-connect");
    const label = button?.querySelector('[data-i18n="landing.connect"]');
    if (label) label.textContent = I18n.t(key);
    else if (button) button.textContent = I18n.t(key);
  }

  function updateConnectionUi() {
    const key = state.connected ? "status.connected" : "status.disconnected";
    const connection = $("connection");
    const landingConnection = $("landing-connection");
    if (connection) {
      connection.textContent = I18n.t(key);
      connection.classList.toggle("online", state.connected);
    }
    if (landingConnection) {
      landingConnection.textContent = I18n.t(key);
      landingConnection.classList.toggle("online", state.connected);
    }
  }

  function updateControlAvailability() {
    document.querySelectorAll("[data-anc]").forEach((button) => {
      button.disabled = !state.connected || !state.features.anc;
    });
    if ($("game-mode")) $("game-mode").disabled = !state.connected || !state.features.gameMode;
    if ($("eq")) $("eq").disabled = !state.connected || !state.features.eqPreset || !state.hasPresets;
  }

  function setConnected(value) {
    state.connected = Boolean(value);
    updateConnectionUi();
    showView(state.connected ? "control" : "landing");
    updateControlAvailability();
    if (!state.connected) {
      const button = $("landing-connect");
      if (button) {
        button.disabled = false;
        button.classList.remove("is-connecting");
      }
      setConnectLabel("landing.connect");
      renderDeviceList();
    }
  }

  function createDeviceRow(device) {
    const row = document.createElement("button");
    row.type = "button";
    row.className = "device-row w-full glass-panel rounded-xl flex items-center justify-between p-4 cursor-pointer hover:bg-white/50 transition-colors group";
    row.setAttribute("role", "option");
    row.setAttribute("aria-selected", "false");
    row.disabled = !validAddress(device?.address);

    const left = document.createElement("span");
    left.className = "flex items-center gap-4";
    const iconWrap = document.createElement("span");
    iconWrap.className = "w-12 h-12 rounded-full bg-white/40 border border-white/30 flex items-center justify-center shadow-sm";
    const icon = document.createElement("span");
    icon.className = "material-symbols-outlined text-primary";
    icon.setAttribute("aria-hidden", "true");
    icon.textContent = "headphones";
    iconWrap.append(icon);

    const copy = document.createElement("span");
    const name = document.createElement("span");
    name.className = "device-name font-body-md text-body-md font-semibold text-on-surface";
    name.textContent = device?.name || "Soundcore Liberty 5";
    const subtitle = document.createElement("span");
    subtitle.className = "device-subtitle font-label-sm text-label-sm text-on-surface-variant";
    subtitle.textContent = I18n.t("landing.deviceSubtitle");
    copy.append(name, subtitle);
    left.append(iconWrap, copy);
    row.append(left);

    const arrow = document.createElement("span");
    arrow.className = "material-symbols-outlined text-on-surface-variant group-hover:text-primary transition-colors";
    arrow.setAttribute("aria-hidden", "true");
    arrow.textContent = "expand_more";
    row.append(arrow);
    return row;
  }

  function renderDeviceList() {
    const list = $("device-list");
    const connectButton = $("landing-connect");
    if (!list || !connectButton) return;
    list.replaceChildren();
    let selectedWasRendered = false;

    state.devices.forEach((device) => {
      const row = createDeviceRow(device);
      const selected = validAddress(device?.address) && state.selectedDevice?.address === device?.address;
      if (selected) {
        selectedWasRendered = true;
        row.classList.add("active");
        row.setAttribute("aria-selected", "true");
      }
      if (!row.disabled) {
        row.addEventListener("click", () => {
          state.selectedDevice = device;
          list.querySelectorAll(".device-row").forEach((candidate) => {
            const active = candidate === row;
            candidate.classList.toggle("active", active);
            candidate.setAttribute("aria-selected", String(active));
          });
          connectButton.disabled = false;
        });
      }
      list.append(row);
    });

    if (state.selectedDevice && !selectedWasRendered) state.selectedDevice = null;
    connectButton.disabled = !state.selectedDevice || !validAddress(state.selectedDevice.address);
    if (state.devices.length === 0) {
      const empty = document.createElement("p");
      empty.className = "text-center text-xs text-on-surface-variant";
      empty.textContent = I18n.t("landing.empty");
      list.append(empty);
    }
  }

  async function refreshDevices() {
    if (!invoke) {
      setNotice($("landing-status"), I18n.t("msg.noTauri"), true);
      return;
    }
    setNotice($("landing-status"), I18n.t("msg.scanning"));
    try {
      const devices = await invoke("list_devices");
      state.devices = Array.isArray(devices) ? devices : [];
      renderDeviceList();
      const count = state.devices.length;
      setNotice($("landing-status"), count === 0
        ? I18n.t("msg.noDevices")
        : count === 1
          ? I18n.t("msg.devicesFound.one")
          : I18n.t("msg.devicesFound.many", { n: count }));
    } catch (error) {
      state.devices = [];
      renderDeviceList();
      setNotice($("landing-status"), I18n.t("msg.scanFailed", { error: fmtError(error) }), true);
    }
  }

  async function connectSelected() {
    const address = state.selectedDevice?.address;
    if (!invoke || !validAddress(address)) return;
    const button = $("landing-connect");
    button.disabled = true;
    button.classList.add("is-connecting");
    setConnectLabel("landing.connecting");
    setNotice($("landing-status"), "");
    try {
      await invoke("connect", { deviceAddress: address });
    } catch (error) {
      button.disabled = false;
      button.classList.remove("is-connecting");
      setConnectLabel("landing.connect");
      setNotice($("landing-status"), I18n.t("msg.actionFailed", { error: fmtError(error) }), true);
    }
  }

  function showBattery(event) {
    const value = payloadOf(event) || {};
    state.battery = value;
    if ($("battery-left")) $("battery-left").textContent = value.left == null ? "—" : `${value.left}%`;
    if ($("battery-right")) $("battery-right").textContent = value.right == null ? "—" : `${value.right}%`;
    if ($("battery-case")) $("battery-case").textContent = value.case == null ? "—" : `${value.case}%`;
  }

  function setAncActive(mode) {
    document.querySelectorAll("[data-anc]").forEach((button) => {
      const active = mode != null && button.dataset.anc === mode;
      button.classList.toggle("active", active);
      button.setAttribute("aria-pressed", String(active));
    });
    if ($("anc-value")) $("anc-value").textContent = mode == null ? "—" : I18n.t(`anc.${mode}`, {}, mode);
  }

  async function loadCapabilities() {
    if (!invoke) return;
    try {
      state.features = { ...state.features, ...(await invoke("get_capabilities") || {}) };
      updateControlAvailability();
    } catch (error) {
      setNotice(state.connected ? $("control-notice") : $("landing-status"), I18n.t("msg.profileFailed", { error: fmtError(error) }), true);
    }
  }

  async function loadPresets() {
    const select = $("eq");
    if (!select) return;
    select.replaceChildren();
    try {
      const presets = invoke ? await invoke("get_eq_presets") : [];
      const entries = Array.isArray(presets) ? presets : [];
      state.hasPresets = entries.length > 0;
      entries.forEach(([id, label]) => {
        const option = document.createElement("option");
        option.value = id;
        option.textContent = label;
        select.append(option);
      });
      if (!state.hasPresets) {
        const option = document.createElement("option");
        option.value = "";
        option.textContent = I18n.t("eq.empty");
        select.append(option);
      }
      updateControlAvailability();
    } catch (error) {
      state.hasPresets = false;
      const option = document.createElement("option");
      option.textContent = I18n.t("eq.empty");
      select.append(option);
      updateControlAvailability();
      setNotice(state.connected ? $("control-notice") : $("landing-status"), I18n.t("msg.eqListFailed", { error: fmtError(error) }), true);
    }
  }

  function applyLanguage() {
    I18n.applyStatic();
    updateConnectionUi();
    setAncActive(state.ancMode);
    renderDeviceList();
    setConnectLabel("landing.connect");
  }

  async function bindListeners() {
    if (!listen) return;
    await listen("battery", showBattery);
    await listen("connection", (event) => {
      const connected = Boolean(payloadOf(event));
      setConnected(connected);
      if (!connected) refreshDevices();
    });
    await listen("anc", (event) => {
      state.ancMode = payloadOf(event);
      setAncActive(state.ancMode);
    });
    await listen("game-mode", (event) => {
      state.gameMode = Boolean(payloadOf(event));
      if ($("game-mode")) $("game-mode").checked = state.gameMode;
    });
    await listen("device-info", (event) => {
      const info = payloadOf(event) || {};
      setNotice($("device-meta"), I18n.t("control.deviceMeta", { serial: info.serial || "—", firmware: info.firmware || "—" }));
    });
    await listen("device-error", (event) => {
      const value = payloadOf(event);
      const message = typeof value === "string" ? value : value?.msg || fmtError(value);
      setNotice($("control-notice"), I18n.t("msg.deviceError", { msg: message }), true);
    });
  }

  function bindUiEvents() {
    document.querySelectorAll(".lang-btn").forEach((button) => {
      button.addEventListener("click", async () => {
        I18n.setLang(button.dataset.lang);
        applyLanguage();
        if (invoke) {
          try { await invoke("set_language", { lang: I18n.getLang() }); } catch (error) { setNotice($("control-notice"), fmtError(error), true); }
        }
      });
    });
    $("landing-connect")?.addEventListener("click", connectSelected);
    $("landing-trouble")?.addEventListener("click", () => { $("landing-help").hidden = !$("landing-help").hidden; });
    $("back-button")?.addEventListener("click", async () => {
      if (!state.connected) { setConnected(false); return; }
      if (!invoke) { setConnected(false); return; }
      try {
        await invoke("disconnect");
        setConnected(false);
      } catch (error) { setNotice($("control-notice"), fmtError(error), true); }
    });
    document.querySelectorAll("[data-anc]").forEach((button) => {
      button.addEventListener("click", async () => {
        const mode = button.dataset.anc;
        const previous = state.ancMode;
        setAncActive(mode);
        if (!invoke) return;
        try { await invoke("set_anc", { mode }); } catch (error) {
          setAncActive(previous);
          setNotice($("control-notice"), I18n.t("msg.actionFailed", { error: fmtError(error) }), true);
        }
      });
    });
    $("game-mode")?.addEventListener("change", async (event) => {
      const previous = state.gameMode;
      state.gameMode = event.target.checked;
      if (!invoke) return;
      try { await invoke("set_game_mode", { enabled: state.gameMode }); } catch (error) {
        state.gameMode = previous;
        event.target.checked = previous;
        setNotice($("control-notice"), I18n.t("msg.actionFailed", { error: fmtError(error) }), true);
      }
    });
    $("eq")?.addEventListener("change", async (event) => {
      if (!invoke) return;
      try { await invoke("set_eq_preset", { presetId: event.target.value }); } catch (error) { setNotice($("control-notice"), I18n.t("msg.actionFailed", { error: fmtError(error) }), true); }
    });
    window.addEventListener("i18n:change", applyLanguage);
  }

  async function init() {
    bindUiEvents();
    await bindListeners();
    await Promise.all([loadCapabilities(), loadPresets()]);
    setConnected(false);
    await refreshDevices();
    if (invoke) await invoke("set_language", { lang: I18n.getLang() });
  }

  init().catch((error) => setNotice($("landing-status"), I18n.t("msg.initFailed", { error: fmtError(error) }), true));
})();
