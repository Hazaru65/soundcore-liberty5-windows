(() => {
  const STORAGE_KEY = "ui.lang";
  const translations = {
    tr: {
      "status.connected": "Bağlı",
      "status.disconnected": "Bağlı değil",
      "refresh": "Yenile",
      "battery.left": "Sol",
      "battery.right": "Sağ",
      "battery.case": "Kutu",
      "controls.ariaLabel": "Ses kontrolleri",
      "anc.Off": "Kapalı",
      "anc.Transparency": "Şeffaflık",
      "anc.On": "Açık",
      "eq.label": "EQ ön ayarı",
      "eq.empty": "Doğrulanmış preset yok",
      "lang.switchLabel": "Dil seçimi",
      "landing.title": "Cihazını Bağla",
      "landing.subtitle": "Bağlanmak istediğin Liberty 5 cihazını seç.",
      "landing.devicesLabel": "KULLANILABİLİR CİHAZLAR",
      "landing.connect": "Bağlan",
      "landing.connecting": "Bağlanıyor…",
      "landing.trouble": "Bağlantı sorunu mu yaşıyorsunuz?",
      "landing.helpHint": "Kulaklıklar Windows'a eşleştirilmediyse görünmez. İkisini de kutudan çıkarıp 5–10 sn bekleyin, telefon uygulaması kapalıyken Yenile'ye basın.",
      "landing.empty": "Cihaz bulunamadı. Yenile'ye basın.",
      "landing.deviceSubtitle": "Bağlanmaya hazır",
      "control.backAria": "Geri",
      "control.gameMode": "Oyun Modu",
      "control.deviceMeta": "Seri {serial} · Yazılım {firmware}",
      "msg.noTauri": "Tauri API bulunamadı; uygulamayı cargo tauri dev ile açın.",
      "msg.devicesFound.one": "1 cihaz bulundu.",
      "msg.devicesFound.many": "{n} cihaz bulundu.",
      "msg.noDevices": "Cihaz bulunamadı. Yenile'ye basın.",
      "msg.scanning": "Taranıyor…",
      "msg.scanFailed": "Cihaz taraması başarısız: {error}",
      "msg.actionFailed": "İşlem başarısız: {error}",
      "msg.profileFailed": "Özellik profili alınamadı: {error}",
      "msg.eqListFailed": "EQ listesi alınamadı: {error}",
      "msg.initFailed": "Başlatma başarısız: {error}",
      "msg.deviceError": "Cihaz: {msg}",
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
      "refresh": "Refresh",
      "battery.left": "Left",
      "battery.right": "Right",
      "battery.case": "Case",
      "controls.ariaLabel": "Audio controls",
      "anc.Off": "Off",
      "anc.Transparency": "Transparency",
      "anc.On": "On",
      "eq.label": "EQ preset",
      "eq.empty": "No verified presets",
      "lang.switchLabel": "Language",
      "landing.title": "Connect Your Device",
      "landing.subtitle": "Select the Liberty 5 device you want to connect.",
      "landing.devicesLabel": "AVAILABLE DEVICES",
      "landing.connect": "Connect",
      "landing.connecting": "Connecting…",
      "landing.trouble": "Trouble connecting?",
      "landing.helpHint": "If your earbuds don't appear, they may not be paired with Windows. Take both out of the case, wait 5–10 seconds, then press Refresh with the phone app closed.",
      "landing.empty": "No devices found. Press Refresh.",
      "landing.deviceSubtitle": "Ready to pair",
      "control.backAria": "Back",
      "control.gameMode": "Game Mode",
      "control.deviceMeta": "Serial {serial} · Firmware {firmware}",
      "msg.noTauri": "Tauri API not found; open the app with cargo tauri dev.",
      "msg.devicesFound.one": "1 device found.",
      "msg.devicesFound.many": "{n} devices found.",
      "msg.noDevices": "No devices found. Press Refresh.",
      "msg.scanning": "Scanning…",
      "msg.scanFailed": "Device scan failed: {error}",
      "msg.actionFailed": "Operation failed: {error}",
      "msg.profileFailed": "Could not load feature profile: {error}",
      "msg.eqListFailed": "Could not load EQ list: {error}",
      "msg.initFailed": "Startup failed: {error}",
      "msg.deviceError": "Device: {msg}",
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
    } catch (_) { /* localStorage erişilemezse varsayılanı kullan */ }
    return (navigator.language || "en").toLowerCase().startsWith("tr") ? "tr" : "en";
  }

  let lang = loadLang();

  function t(key, params = {}, fallback) {
    const template = translations[lang]?.[key];
    if (template == null) return fallback !== undefined ? fallback : key;
    return template.replace(/\{(\w+)\}/g, (match, name) => (params[name] != null ? params[name] : match));
  }

  function applyStatic() {
    document.querySelectorAll("[data-i18n]").forEach((element) => {
      element.textContent = t(element.dataset.i18n);
    });
    document.querySelectorAll("[data-i18n-aria]").forEach((element) => {
      element.setAttribute("aria-label", t(element.dataset.i18nAria));
    });
    document.documentElement.lang = lang;
    document.querySelectorAll(".lang-btn").forEach((button) => {
      button.setAttribute("aria-pressed", String(button.dataset.lang === lang));
    });
    document.querySelectorAll(".lang-switch").forEach((el) => {
      el.dataset.lang = lang;
    });
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
