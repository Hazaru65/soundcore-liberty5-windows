(() => {
  const STORAGE_KEY = "ui.lang";

  const translations = {
    tr: {
      "status.connected": "Bağlı",
      "status.disconnected": "Bağlı değil",
      "connect": "Bağlan",
      "disconnect": "Bağlantıyı Kes",
      "device.label": "Cihaz",
      "device.selectAriaLabel": "Liberty 5 cihazı",
      "refresh": "Yenile",
      "battery.title": "Pil",
      "battery.left": "Sol",
      "battery.right": "Sağ",
      "battery.case": "Kutu",
      "controls.ariaLabel": "Ses kontrolleri",
      "anc.Off": "Kapalı",
      "anc.Transparency": "Şeffaflık",
      "anc.On": "Açık",
      "eq.label": "EQ ön ayarı",
      "eq.empty": "Doğrulanmış preset yok",
      "log.title": "Günlük",
      "log.clear": "Temizle",
      "lang.switchLabel": "Dil seçimi",
      "msg.noTauri": "Tauri API bulunamadı; uygulamayı cargo tauri dev ile açın.",
      "msg.devicesFound.one": "1 cihaz bulundu.",
      "msg.devicesFound.many": "{n} cihaz bulundu.",
      "msg.noDevices": "Liberty cihazı bulunamadı; kulaklıklar Windows'a eşleştirilmemiş olabilir. Eşleştirildiyse ikisini de kutudan çıkarıp 5-10 sn bekleyin, telefon uygulaması kapalıyken Yenile'e basın.",
      "msg.scanFailed": "Cihaz taraması başarısız: {error}",
      "msg.actionFailed": "İşlem başarısız: {error}",
      "msg.profileFailed": "Özellik profili alınamadı: {error}",
      "msg.eqListFailed": "EQ listesi alınamadı: {error}",
      "msg.deviceInfo": "Cihaz bilgisi: seri={serial} firmware={firmware} ANC={anc}",
      "msg.deviceError": "Cihaz: {msg}",
      "msg.disconnected": "Bağlantı kesildi.",
      "msg.connected": "Cihaza bağlanıldı.",
      "msg.gameModeOn": "Game Mode: açık",
      "msg.gameModeOff": "Game Mode: kapalı",
      "msg.eqApplied": "EQ: {preset}",
      "msg.initFailed": "Başlatma başarısız: {error}",
      "errors.not_connected": "Önce bir Liberty 5 cihazına bağlanın.",
      "errors.ble": "Bluetooth işlemi başarısız: {detail}",
      "errors.not_found": "Liberty 5 cihazı bulunamadı",
      "errors.not_supported": "Bu özellik Liberty 5 profilinde doğrulanmadı",
      "errors.profile": "Profil hatası: {detail}",
      "errors.invalid_anc": "Geçersiz ANC modu: {detail}",
      "errors.connection": "Bluetooth bağlantısı reddedildi. Telefondaki Soundcore uygulamasını kapatın; Windows'ta cihazı kaldırıp yeniden eşleştirin. Ayrıntı: {detail}",
      "errors.windows": "Windows Bluetooth API hatası: {detail}"
    },
    en: {
      "status.connected": "Connected",
      "status.disconnected": "Not connected",
      "connect": "Connect",
      "disconnect": "Disconnect",
      "device.label": "Device",
      "device.selectAriaLabel": "Liberty 5 device",
      "refresh": "Refresh",
      "battery.title": "Battery",
      "battery.left": "Left",
      "battery.right": "Right",
      "battery.case": "Case",
      "controls.ariaLabel": "Audio controls",
      "anc.Off": "Off",
      "anc.Transparency": "Transparency",
      "anc.On": "On",
      "eq.label": "EQ preset",
      "eq.empty": "No verified presets",
      "log.title": "Log",
      "log.clear": "Clear",
      "lang.switchLabel": "Language",
      "msg.noTauri": "Tauri API not found; open the app with cargo tauri dev.",
      "msg.devicesFound.one": "1 device found.",
      "msg.devicesFound.many": "{n} devices found.",
      "msg.noDevices": "No Liberty device found; the earbuds may not be paired with Windows. If paired, take both out of the case and wait 5–10 seconds, then press Refresh with the phone app closed.",
      "msg.scanFailed": "Device scan failed: {error}",
      "msg.actionFailed": "Operation failed: {error}",
      "msg.profileFailed": "Could not load feature profile: {error}",
      "msg.eqListFailed": "Could not load EQ list: {error}",
      "msg.deviceInfo": "Device info: serial={serial} firmware={firmware} ANC={anc}",
      "msg.deviceError": "Device: {msg}",
      "msg.disconnected": "Disconnected.",
      "msg.connected": "Connected to device.",
      "msg.gameModeOn": "Game Mode: on",
      "msg.gameModeOff": "Game Mode: off",
      "msg.eqApplied": "EQ: {preset}",
      "msg.initFailed": "Startup failed: {error}",
      "errors.not_connected": "Connect to a Liberty 5 device first.",
      "errors.ble": "Bluetooth operation failed: {detail}",
      "errors.not_found": "Liberty 5 device not found",
      "errors.not_supported": "This feature is not verified in the Liberty 5 profile",
      "errors.profile": "Profile error: {detail}",
      "errors.invalid_anc": "Invalid ANC mode: {detail}",
      "errors.connection": "Bluetooth connection refused. Close the Soundcore app on your phone; remove and re-pair the device in Windows. Details: {detail}",
      "errors.windows": "Windows Bluetooth API error: {detail}"
    }
  };

  function loadLang() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored === "tr" || stored === "en") return stored;
    } catch (_) { /* localStorage erişilemezse varsayılana düş */ }
    return (navigator.language || "en").toLowerCase().startsWith("tr") ? "tr" : "en";
  }

  let lang = loadLang();

  function t(key, params = {}, fallback) {
    const template = translations[lang] && translations[lang][key];
    if (template == null) return fallback !== undefined ? fallback : key;
    return template.replace(/\{(\w+)\}/g, (match, name) => (params[name] != null ? params[name] : match));
  }

  function applyStatic() {
    document.querySelectorAll("[data-i18n]").forEach((el) => { el.textContent = t(el.dataset.i18n); });
    document.querySelectorAll("[data-i18n-aria]").forEach((el) => { el.setAttribute("aria-label", t(el.dataset.i18nAria)); });
    document.documentElement.lang = lang;
    document.querySelectorAll(".lang-btn").forEach((btn) => { btn.setAttribute("aria-pressed", String(btn.dataset.lang === lang)); });
  }

  function setLang(next) {
    lang = next === "en" ? "en" : "tr";
    try { localStorage.setItem(STORAGE_KEY, lang); } catch (_) { /* oturum içinde geçerli kalır */ }
    applyStatic();
    window.dispatchEvent(new CustomEvent("i18n:change", { detail: { lang } }));
  }

  function getLang() { return lang; }

  window.I18n = { t, getLang, setLang, applyStatic };
  applyStatic();
})();
