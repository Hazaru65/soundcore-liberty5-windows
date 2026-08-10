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

  function writeLog(message) {
    const time = new Date().toLocaleTimeString("tr-TR");
    log.textContent += `[${time}] ${message}\n`;
    log.scrollTop = log.scrollHeight;
  }

  function setConnected(value) {
    connected = Boolean(value);
    $("connection").textContent = connected ? "Bağlı" : "Bağlı değil";
    $("connection").classList.toggle("online", connected);
    connect.textContent = connected ? "Bağlantıyı Kes" : "Bağlan";
    document.querySelectorAll("[data-anc], #game-mode, #eq").forEach((control) => {
      const supported = control.dataset.anc ? features.anc : control.id === "game-mode" ? features.gameMode : features.eqPreset && hasPresets;
      control.disabled = !connected || !supported;
    });
  }

  function showBattery(status) {
    const value = (part) => part == null ? "—" : `${part}%`;
    $("battery-left").textContent = value(status.left);
    $("battery-right").textContent = value(status.right);
    $("battery-case").textContent = value(status.case);
    $("battery-time").textContent = new Date().toLocaleTimeString("tr-TR");
  }

  async function refreshDevices() {
    if (!invoke) { writeLog("Tauri API bulunamadı; uygulamayı cargo tauri dev ile açın."); return; }
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
      writeLog(devices.length ? `${devices.length} cihaz bulundu.` : "Liberty cihazı bulunamadı; kulaklıklar Windows'a eşleştirilmemiş olabilir. Eşleştirildiyse ikisini de kutudan çıkarıp 5-10 sn bekleyin, telefon uygulaması kapalıyken Yenile'e basın.");
    } catch (error) {
      connect.disabled = true;
      writeLog(`Cihaz taraması başarısız: ${error}`);
    }
  }

  async function run(action, successMessage) {
    if (!invoke) return;
    try { await action(); writeLog(successMessage); }
    catch (error) { writeLog(`İşlem başarısız: ${error}`); }
  }

  async function loadCapabilities() {
    if (!invoke) return;
    try { features = await invoke("get_capabilities"); }
    catch (error) { writeLog(`Özellik profili alınamadı: ${error}`); }
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
      if (!presets.length) { const option = document.createElement("option"); option.textContent = "Doğrulanmış preset yok"; select.append(option); }
    } catch (error) { writeLog(`EQ listesi alınamadı: ${error}`); }
  }

  async function init() {
    if (listen) {
      await listen("battery", (event) => showBattery(event.payload));
      await listen("connection", (event) => setConnected(event.payload));
      await listen("anc", (event) => { $("anc-value").textContent = event.payload; });
      await listen("game-mode", (event) => { $("game-mode").checked = Boolean(event.payload); });
      await listen("device-info", (event) => {
        const info = event.payload;
        writeLog(`Cihaz bilgisi: seri=${info.serial ?? "—"} firmware=${info.firmware ?? "—"} ANC=${info.ancMode ?? "?"}`);
      });
      await listen("device-error", (event) => writeLog(`Cihaz: ${event.payload}`));
    }
    $("refresh").addEventListener("click", refreshDevices);
    connect.addEventListener("click", () => {
      if (connected) return run(() => invoke("disconnect"), "Bağlantı kesildi.");
      if (!device.value) return;
      run(() => invoke("connect"), "Cihaza bağlanıldı.");
    });
    document.querySelectorAll("[data-anc]").forEach((button) => button.addEventListener("click", () => {
      const mode = button.dataset.anc;
      run(() => invoke("set_anc", { mode }), `ANC: ${mode}`);
    }));
    $("game-mode").addEventListener("change", (event) => run(() => invoke("set_game_mode", { enabled: event.target.checked }), `Game Mode: ${event.target.checked ? "açık" : "kapalı"}`));
    $("eq").addEventListener("change", (event) => run(() => invoke("set_eq_preset", { presetId: event.target.value }), `EQ: ${event.target.value}`));
    $("clear-log").addEventListener("click", () => { log.textContent = ""; });
    await loadCapabilities();
    await loadPresets();
    setConnected(false);
    await refreshDevices();
  }

  init().catch((error) => writeLog(`Başlatma başarısız: ${error}`));
})();
