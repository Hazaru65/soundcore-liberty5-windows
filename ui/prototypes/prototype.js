/* Three radically different standalone Liberty 5 UI examples. Switch with ?variant=a|b|c. */
(() => {
  const variants = [
    { key: "a", name: "Focus", description: "Tek bakışta durum ve tek ana aksiyon" },
    { key: "b", name: "Console", description: "Ayarları gruplayan kontrol merkezi" },
    { key: "c", name: "Now", description: "ANC'yi öne alan hızlı kullanım yüzeyi" }
  ];

  const state = {
    anc: "Transparency",
    gameMode: false
  };

  const stage = document.getElementById("prototype-stage");
  const label = document.getElementById("variant-label");
  const description = document.getElementById("variant-description");
  const previous = document.getElementById("previous-variant");
  const next = document.getElementById("next-variant");

  function currentKey() {
    const requested = new URLSearchParams(window.location.search).get("variant");
    return variants.some((variant) => variant.key === requested) ? requested : "a";
  }

  function variantIndex(key) {
    return variants.findIndex((variant) => variant.key === key);
  }

  function setVariant(key) {
    const url = new URL(window.location.href);
    url.searchParams.set("variant", key);
    window.history.replaceState({}, "", url);
    render();
  }

  function cycle(step) {
    const index = variantIndex(currentKey());
    const nextIndex = (index + step + variants.length) % variants.length;
    setVariant(variants[nextIndex].key);
  }

  function markButton(button, active) {
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-pressed", String(active));
  }

  function focusView() {
    return `
      <section class="variant-focus" aria-labelledby="focus-title">
        <div class="prototype-mark">PROTOTYPE · A</div>
        <header class="focus-header">
          <div>
            <p class="eyebrow">SOUNDCORE</p>
            <h1 id="focus-title">Sadece<br>dinlemeye devam et.</h1>
          </div>
          <span class="focus-status">Bağlı</span>
        </header>
        <div class="focus-device" aria-label="Kulaklık toplam pili yüzde 82">
          <div class="focus-device-art" aria-hidden="true"></div>
        </div>
        <section class="focus-card" aria-labelledby="focus-anc-title">
          <div class="focus-card-header">
            <div><span class="label">AKTİF MOD</span><h2 id="focus-anc-title">Gürültü kontrolü</h2></div>
            <span class="value">${state.anc === "Transparency" ? "Şeffaflık" : state.anc === "On" ? "ANC Açık" : "Kapalı"}</span>
          </div>
          <div class="focus-controls" role="group" aria-label="Gürültü kontrolü">
            ${["Off", "Transparency", "On"].map((mode) => `<button class="control-button ${state.anc === mode ? "is-active" : ""}" type="button" data-anc="${mode}" aria-pressed="${state.anc === mode}">${mode === "Off" ? "Kapalı" : mode === "Transparency" ? "Şeffaflık" : "ANC Açık"}</button>`).join("")}
          </div>
        </section>
        <div class="focus-footer"><span class="muted">Sol %84 · Sağ %80 · Kutu %62</span><button class="button-primary" type="button" data-action="connect">Bağlantıyı kes</button></div>
      </section>`;
  }

  function consoleView() {
    return `
      <section class="variant-console" aria-labelledby="console-title">
        <aside class="console-rail">
          <div class="console-brand">sound<span>core</span></div>
          <nav class="console-nav" aria-label="Ayar bölümleri">
            <button class="is-active" type="button">Genel bakış</button>
            <button type="button">Ses</button>
            <button type="button">Cihaz</button>
          </nav>
          <p class="console-rail-note">Liberty 5<br><strong>Bağlı ve hazır</strong></p>
        </aside>
        <div class="console-main">
          <header class="console-topbar">
            <div><p class="eyebrow">GENEL BAKIŞ</p><h1 id="console-title">Kontrol merkezi</h1><span class="console-device-name">Liberty 5 · Son eşleşme bugün, 09:42</span></div>
            <span class="connection-pill">● Bağlı</span>
          </header>
          <div class="console-grid">
            <section class="console-panel console-panel-wide" aria-labelledby="console-anc-title">
              <div class="panel-heading"><h2 id="console-anc-title">Gürültü kontrolü</h2><span class="panel-icon">◒</span></div>
              <div class="anc-display"><strong>${state.anc === "Transparency" ? "Şeffaflık" : state.anc === "On" ? "ANC" : "Kapalı"}</strong><span>aktif</span></div>
              <div class="console-controls" role="group" aria-label="Gürültü kontrolü">
                ${["Off", "Transparency", "On"].map((mode) => `<button class="console-control ${state.anc === mode ? "is-active" : ""}" type="button" data-anc="${mode}" aria-pressed="${state.anc === mode}"><span>${mode === "Off" ? "Kapalı" : mode === "Transparency" ? "Şeffaflık" : "ANC Açık"}</span><span>→</span></button>`).join("")}
              </div>
            </section>
            <section class="console-panel" aria-labelledby="battery-title">
              <div class="panel-heading"><h2 id="battery-title">Pil</h2><span class="panel-icon">◌</span></div>
              <div class="battery-list">
                <div class="battery-line"><span>Sol</span><div class="battery-track"><span style="width:84%"></span></div><strong>84%</strong></div>
                <div class="battery-line"><span>Sağ</span><div class="battery-track"><span style="width:80%"></span></div><strong>80%</strong></div>
                <div class="battery-line"><span>Kutu</span><div class="battery-track"><span style="width:62%"></span></div><strong>62%</strong></div>
              </div>
            </section>
            <section class="console-panel" aria-labelledby="quick-title">
              <div class="panel-heading"><h2 id="quick-title">Hızlı ayar</h2><span class="panel-icon">✦</span></div>
              <button class="console-control ${state.gameMode ? "is-active" : ""}" type="button" data-action="game" aria-pressed="${state.gameMode}"><span>Game Mode</span><span>${state.gameMode ? "Açık" : "Kapalı"}</span></button>
            </section>
          </div>
          <div class="console-footer"><button class="button-quiet" type="button" data-action="refresh">Cihazı yenile</button></div>
        </div>
      </section>`;
  }

  function nowView() {
    return `
      <section class="variant-now" aria-labelledby="now-title">
        <header class="now-header">
          <span class="now-brand">SOUNDCORE / LIBERTY 5</span>
          <div class="now-header-actions"><button type="button" aria-label="Ayarlar">⚙</button><button type="button" aria-label="Bildirimler">○</button></div>
        </header>
        <div class="now-main">
          <p class="eyebrow">ŞİMDİ DİNLENİYOR</p>
          <h1 id="now-title">Dışarıdaki sesi <em>azalt.</em></h1>
          <span class="now-connection">Liberty 5 bağlı · pil %82</span>
          <section class="now-command" aria-labelledby="now-command-title">
            <div class="now-command-heading"><strong id="now-command-title">Gürültü kontrolü</strong><span>${state.anc === "Transparency" ? "Şeffaflık" : state.anc === "On" ? "ANC Açık" : "Kapalı"}</span></div>
            <div class="now-anc-controls" role="group" aria-label="Gürültü kontrolü">
              ${["Off", "Transparency", "On"].map((mode) => `<button class="now-anc-button ${state.anc === mode ? "is-active" : ""}" type="button" data-anc="${mode}" aria-pressed="${state.anc === mode}">${mode === "Off" ? "Kapalı" : mode === "Transparency" ? "Şeffaflık" : "ANC Açık"}</button>`).join("")}
            </div>
          </section>
          <div class="now-stats"><div class="now-stat"><span class="label">SOL</span><strong>84%</strong></div><div class="now-stat"><span class="label">SAĞ</span><strong>80%</strong></div><div class="now-stat"><span class="label">KUTU</span><strong>62%</strong></div></div>
          <div class="now-activity"><p>Son işlem · Şeffaflık modu 2 dk önce etkinleştirildi</p></div>
        </div>
      </section>`;
  }

  function render() {
    const key = currentKey();
    const variant = variants[variantIndex(key)];
    stage.innerHTML = key === "a" ? focusView() : key === "b" ? consoleView() : nowView();
    label.textContent = `${variant.key.toUpperCase()} — ${variant.name}`;
    description.textContent = variant.description;
    document.title = `Liberty 5 — ${variant.name} prototype`;
  }

  stage.addEventListener("click", (event) => {
    const ancButton = event.target.closest("[data-anc]");
    if (ancButton) {
      state.anc = ancButton.dataset.anc;
      render();
      return;
    }
    const action = event.target.closest("[data-action]")?.dataset.action;
    if (action === "game") {
      state.gameMode = !state.gameMode;
      render();
    }
  });

  previous.addEventListener("click", () => cycle(-1));
  next.addEventListener("click", () => cycle(1));
  window.addEventListener("popstate", render);
  window.addEventListener("keydown", (event) => {
    const target = event.target;
    if (target.matches("input, select, textarea, [contenteditable=\"true\"]")) return;
    if (event.key === "ArrowLeft") { event.preventDefault(); cycle(-1); }
    if (event.key === "ArrowRight") { event.preventDefault(); cycle(1); }
  });

  render();
})();
