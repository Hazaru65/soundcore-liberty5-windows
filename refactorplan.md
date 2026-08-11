# Refactor Planı

Durum: 2026-08-11. Analiz kaynağı: `improve-codebase-architecture` skill keşfi (ArchScout) + elle inceleme. Amaç: ileride bakılacak derinleşme fırsatları. Hiçbiri acil değil; şimdi uygulanmadı (nedenleri sonda).

## Değerlendirme Özeti

| # | Aday | Önem | Aciliyet |
|---|------|------|----------|
| 1 | İkiz komutlar `set_anc`/`set_game_mode` | Orta | Yok |
| 2 | Oturum kapatma mantığı 3 yerde kopyalı | Orta | Yok |
| 3 | Taşıyıcı seam'i yok (oturum test edilemez) | Yüksek | Kod büyürse |
| 4 | Ölü EQ stub (`set_eq_preset`) | Düşük | Yok |
| 5 | Pil yoklama yaşam döngüsü dağınık | Düşük-orta | Yok |

## Adaylar

### 1. İkiz komutlar: `set_anc` / `set_game_mode`

- **Dosyalar:** `crates/soundcore-lib5-core/src/liberty5_device.rs`
- **Problem:** İki fn neredeyse birebir: payload → `profile.command()` → gönder → aynı komut kodlu + boş payload ACK ara → yoksa `BleError::Connection("<komut> onay alınamadı")`. Tek fark komut adı ve Türkçe hata metni.
- **Çözüm:** `send_and_await_ack(kind: &str, option: &str)` yardımcısı; `set_anc`/`set_game_mode` ona delege eder. Hata metni de merkezileşir (`"{kind} komutuna onay alınamadı"`).
- **Fayda:** Duplikasyon gider; yeni komut eklemek (örn. EQ) tek yerden.
- **Not:** Davranış aynı kalır — risk düşük, ama acil değil.

### 2. Oturum kapatma 3 yerde kopyalı

- **Dosyalar:** `src-tauri/src/lib.rs`
- **Problem:** Aynı sıra (battery_poll abort → device disconnect → `connection:false` emit) üç yerde: `disconnect` komutu, `connect` başındaki eski cihazı koparma, tray "disconnect" handler'ı.
- **Çözüm:** `async fn close_session(state: &State<AppState>, app: &AppHandle)` tek fn; üç çağrı yerine. (connect'te emit istenmezse parametre ile kontrol.)
- **Fayda:** Tray ile komut davranışı asla sapmaz; locality.
- **Not:** Şu an üçü de aynı çalışıyor — kozmetik risk, acil değil.

### 3. Taşıyıcı seam'i yok — oturum mantığı test edilemez (en önemlisi)

- **Dosyalar:** `crates/soundcore-lib5-core/src/rfcomm_session.rs`, `liberty5_device.rs`
- **Problem:** `Liberty5Device` doğrudan `RfcommSession`'a (Windows API) bağlı. Kritik mantık — retry döngüsü (0x800710DD → oturum yeniden aç, 1.5 sn gecikme, 2 deneme), ACK kontrolü, device-info offset ayrıştırma (seri [16..32], firmware [6..16]) — hiçbiri test edilemiyor. Mevcut testler yalnızca saf `decode_battery` (ve `rfcomm_session`'da `host_frame`/`take_frame`).
- **Çözüm:** `Transport` trait'i (ör. `async fn command(&mut self, command: u16, payload: &[u8]) -> Result<Vec<Frame>, BleError>`) arkasına `RfcommSession`; testlerde sahte taşıyıcı (önce hata, sonra başarı → retry doğrular; eksik ACK → hata doğrular). `Liberty5Device::open` trait nesnesi alır.
- **Fayda:** Gerçek hataların yaşadığı yer test altına girer; regresyon koruması. `protocol-notes.md`'deki "flaky 0x800710DD" davranışı otomatik test edilir.
- **Not:** "Two adapters = real seam" ilkesi — şu an tek adapter (gerçek cihaz), seam henüz kendini kanıtlamadı. Kod büyürse (yeni cihaz/taşıyıcı) veya oturum mantığı değişirse öncelik kazanır. O zamana kadar yatırımı hak etmiyor.

### 4. Ölü EQ stub

- **Dosyalar:** `crates/soundcore-lib5-core/src/liberty5_device.rs`, `src-tauri/src/lib.rs` (ve frontend bağlantısı)
- **Problem:** `set_eq_preset()` her zaman `Err(BleError::NotSupported)` (kilitli stub — `docs/protocol-notes.md`: EQ bilinçli sonraya bırakıldı, isitsel doğrulama yok). Yine de Tauri'de `set_eq_preset` komutu register, frontend'te `#eq` select + invoke zinciri duruyor.
- **Çözüm:** İki seçenek: (a) stub'ı belgele (`// EQ: isitsel dogrulama bekliyor, protocol-notes.md`), (b) komut zincirini tamamen sök — EQ geldiğinde geri ekle.
- **Fayda:** Ölü kod çağrı yüzeyini kirletir; AI-navigability düşer.
- **Not:** `get_eq_presets` boş dizi döndüğünden frontend `#eq` zaten disabled — kullanıcıya zarar yok. En ucuz hamle: belgeleme satırı.

### 5. Pil yoklama yaşam döngüsü dağınık

- **Dosyalar:** `src-tauri/src/lib.rs`
- **Problem:** Spawn `connect` içinde; abort `disconnect`, `connect` başı, tray "disconnect" — üç yerde. `AppState::battery_poll`'e dokunma dağınık.
- **Çözüm:** `AppState::start_battery_poll(...)` / `stop_battery_poll()` metotları; spawn/abort + emit tek noktadan.
- **Fayda:** Yaşam döngüsü kuralı tek yerde; aday 2 ile birlikte yapılabilir (aynı bölge).
- **Not:** Çalışıyor; acil değil.

## Neden şimdi yapılmadı

- Tek kullanıcılı, deneysel hobby proje; doğrulama gerçek cihazda manuel (`protocol-notes.md` "4/4 gerçek cihaz koşusu başarılı").
- Aday 3 için henüz tek adapter var — "one adapter = hypothetical seam" ilkesi gereği soyutlama erken.
- Aday 1/2/5 duplikasyonları davranış olarak aynı çalışıyor; regresyon riski düşük, kazanç kozmetik.
- Aday 4 bilinçli erteleme (kullanıcı kararı).

## Yapılma sırası önerisi (ileride karar verilirse)

1. Aday 1 (küçük, tek dosya) → 2 + 5 birlikte (aynı bölge: `lib.rs` oturum/poll) → 4 (belgeleme) → 3 (en büyük yatırım, en son).
