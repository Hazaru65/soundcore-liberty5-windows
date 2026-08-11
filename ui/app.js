(() => {
  const tauri = window.__TAURI__;
  const invoke = tauri?.core?.invoke;
  const listen = tauri?.event?.listen;
  const $ = (id) => document.getElementById(id);
  const device = $("device");
  const connect = $("connect");
  const log = $("log");
  let connected = false;
  let hasPresets = false;
  let features = { anc: false, gameMode: false, eqPreset: false };
  let lastAncValue = null;
  let lastBatteryStatus = null;

  function timeLocale() {
    return I18n.getLang() === "tr" ? "tr-TR" : "en-US";
  }

  function fmtError(error) {
    if (typeof error === "string") return error;
    if (error && typeof error.code === "string") return I18n.t("errors." + error.code, { detail: error.detail || "" });
    try { return JSON.stringify(error); } catch (_) { return String(error); }
  }

  function writeLog(message) {
    const time = new Date().toLocaleTimeString(timeLocale());
    log.textContent += `[${time}] ${message}\n`;
    log.scrollTop = log.scrollHeight;
  }

  function setConnected(value) {
    connected = Boolean(value);
    $("connection").textContent = connected ? I18n.t("status.connected") : I18n.t("status.disconnected");
    $("connection").classList.toggle("online", connected);
    connect.textContent = connected ? I18n.t("disconnect") : I18n.t("connect");
    document.querySelectorAll("[data-anc], #game-mode, #eq").forEach((control) => {
      const supported = control.dataset.anc ? features.anc : control.id === "game-mode" ? features.gameMode : features.eqPreset && hasPresets;
      control.disabled = !connected || !supported;
    });
  }

  function showBattery(status) {
    lastBatteryStatus = status;
    const value = (part) => part == null ? "—" : `${part}%`;
    $("battery-left").textContent = value(status.left);
    $("battery-right").textContent = value(status.right);
    $("battery-case").textContent = value(status.case);
    $("battery-time").textContent = new Date().toLocaleTimeString(timeLocale());
  }

  async function refreshDevices() {
    if (!invoke) { writeLog(I18n.t("msg.noTauri")); return; }
    try {
      const devices = await invoke("list_devices");
      device.replaceChildren();
      for (const item of devices) {
        const option = document.createElement("option");
        option.value = item.deviceId;
        option.textContent = `${item.name} — ${item.address}`;
        device.append(option);
      }
      connect.disabled = devices.length === 0;
      writeLog(devices.length ? I18n.t(devices.length === 1 ? "msg.devicesFound.one" : "msg.devicesFound.many", { n: devices.length }) : I18n.t("msg.noDevices"));
    } catch (error) {
      connect.disabled = true;
      writeLog(I18n.t("msg.scanFailed", { error: fmtError(error) }));
    }
  }

  async function run(action, successMessage) {
    if (!invoke) return;
    try { await action(); writeLog(successMessage); }
    catch (error) { writeLog(I18n.t("msg.actionFailed", { error: fmtError(error) })); }
  }

  async function loadCapabilities() {
    if (!invoke) return;
    try { features = await invoke("get_capabilities"); }
    catch (error) { writeLog(I18n.t("msg.profileFailed", { error: fmtError(error) })); }
  }

  async function loadPresets() {
    if (!invoke) return;
    try {
      const presets = await invoke("get_eq_presets");
      hasPresets = presets.length > 0;
      const select = $("eq");
      select.replaceChildren();
      for (const [id, label] of presets) {
        const option = document.createElement("option"); option.value = id; option.textContent = label; select.append(option);
      }
      if (!presets.length) { const option = document.createElement("option"); option.textContent = I18n.t("eq.empty"); option.dataset.i18n = "eq.empty"; select.append(option); }
    } catch (error) { writeLog(I18n.t("msg.eqListFailed", { error: fmtError(error) })); }
  }

  function applyLanguage() {
    I18n.applyStatic();
    setConnected(connected);
    if (lastBatteryStatus) showBattery(lastBatteryStatus);
    $("anc-value").textContent = lastAncValue == null ? "—" : I18n.t("anc." + lastAncValue, {}, lastAncValue);
  }

  async function init() {
    if (listen) {
      await listen("battery", (event) => showBattery(event.payload));
      await listen("connection", (event) => setConnected(event.payload));
      await listen("anc", (event) => { lastAncValue = event.payload; $("anc-value").textContent = I18n.t("anc." + event.payload, {}, event.payload); });
      await listen("game-mode", (event) => { $("game-mode").checked = Boolean(event.payload); });
      await listen("device-info", (event) => {
        const info = event.payload;
        writeLog(I18n.t("msg.deviceInfo", { serial: info.serial ?? "—", firmware: info.firmware ?? "—", anc: I18n.t("anc." + (info.ancMode ?? "?"), {}, info.ancMode ?? "?") }));
      });
      await listen("device-error", (event) => writeLog(I18n.t("msg.deviceError", { msg: event.payload })));
    }
    document.querySelectorAll(".lang-btn").forEach((btn) => btn.addEventListener("click", () => {
      const next = btn.dataset.lang;
      if (next === I18n.getLang()) return;
      I18n.setLang(next);
      applyLanguage();
      if (invoke) invoke("set_language", { lang: next }).catch(() => {});
    }));
    $("refresh").addEventListener("click", refreshDevices);
    connect.addEventListener("click", () => {
      if (connected) return run(() => invoke("disconnect"), I18n.t("msg.disconnected"));
      if (!device.value) return;
      run(() => invoke("connect"), I18n.t("msg.connected"));
    });
    document.querySelectorAll("[data-anc]").forEach((button) => button.addEventListener("click", () => {
      const mode = button.dataset.anc;
      run(() => invoke("set_anc", { mode }), I18n.t("anc." + mode, {}, mode));
    }));
    $("game-mode").addEventListener("change", (event) => run(() => invoke("set_game_mode", { enabled: event.target.checked }), I18n.t(event.target.checked ? "msg.gameModeOn" : "msg.gameModeOff")));
    $("eq").addEventListener("change", (event) => run(() => invoke("set_eq_preset", { presetId: event.target.value }), I18n.t("msg.eqApplied", { preset: event.target.value })));
    $("clear-log").addEventListener("click", () => { log.textContent = ""; });
    await loadCapabilities();
    await loadPresets();
    setConnected(false);
    await refreshDevices();
    if (invoke) invoke("set_language", { lang: I18n.getLang() }).catch(() => {});
  }

  init().catch((error) => writeLog(I18n.t("msg.initFailed", { error: fmtError(error) })));
})();
