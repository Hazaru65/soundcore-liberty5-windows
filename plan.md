# Liberty 5 Windows Kontrol Uygulaması (Rust + Tauri)

## Güncelleme (2026-08-11): Kontrol kanalı RFCOMM olarak doğrulandı

Bu plan başlangıçta BLE GATT üzerinden yazılmıştı; gerçek cihazda yapılan keşif, kontrol kanalının **Bluetooth Classic RFCOMM** olduğunu kanıtladı (BLE GATT yalnızca jenerik servisler sunuyor: 0x1800/0x1801, Battery Service yok). Uygulama RFCOMM taşıyıcısına geçirildi. Ayrıntılar `docs/protocol-notes.md`'de; özet:

- Kontrol servisi: `0cf12d31-fac3-4553-bd80-d6832e7b3957`
- Çerçeve: `08ee|0000|00|<komut u16-LE>|<uzunluk u16-LE>|<payload>|<sağlama>`, cihaz yanıtı `09ff` ile başlar
- Doğrulanmış: cihaz bilgisi (0x0101, seri `395790BFD95CE5DC`, firmware `04.90`), ANC (0x8106, Off/Transparency/On — üçü de ACK aldı)
- Doğrulanmamış (kilitli): Game Mode (AeroFit'in 0x8701'i denendi, ACK yok — geçersiz), EQ, pil bildirimi (0x0301 henüz görülmedi)
- Uygulama mimarisi: `RfcommSession` (yeni modül) + `Liberty5Device` (RFCOMM); scanner `dump/monitor/write --force`; profil JSON'u RFCOMM komut şemasına taşındı

## Context

`C:\Users\Hazar\Documents\1vibec\anker` klasörü bu oturumda boş bulundu; mevcut kod, test veya yapılandırma yok. Kullanıcı Soundcore Liberty 5 kulaklığını telefon ve Windows PC ile kullanıyor, Windows için resmi Soundcore uygulaması olmadığından ANC, Game Mode ve EQ ayarlarını PC'den yönetemiyor. Amaç: Windows'ta çalışan, Bluetooth LE/GATT üzerinden Liberty 5'i bulup kontrol eden, **Rust + Tauri v2** ile yazılmış bir masaüstü uygulaması üretmek.

Teknoloji kararı: BLE katmanı `btleplug 0.12` (Windows'ta WinRT üzerinden çalışır), uygulama kabuğu `tauri 2.11` (WebView2; Win11'de hazır gelir), CLI keşif aracı aynı workspace içinde ayrı bir bin crate. Makinede Rust ve MSVC yok — plan kurulum adımlarıyla başlar. Soundcore'un resmi ürün sayfası Liberty 5 için Adaptive ANC 3.0, HearID 4.0 ve multipoint'i doğruluyor: https://www.soundcore.com/products/a3957-liberty-5-tws-earbuds. Windows BLE erişimi için referans API: https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.bluetoothledevice.

## Approach

### 1. Araç zincirini kur (doğrulanmış eksikler)

Tüm komutlar PowerShell'de, çalışma dizini `C:\Users\Hazar\Documents\1vibec\anker`. Plan modunda çalışma ağacına hiçbir dosya yazılmaz; uygulama başladıktan sonra ilk iş, bu canonical planın içeriğini köke `plan.md` olarak kopyalamaktır.

1. Rust (rustup + stable):

```powershell
winget install --id Rustlang.Rustup -e
# yeni terminal
rustup default stable
```

2. MSVC Build Tools (Rust MSVC hedefi link.exe ister; makinede yalnız Git'in link.exe'si var):

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

3. Tauri CLI (tauri.conf.json derlemeleri için; ilk derleme birkaç dakika sürer):

```powershell
cargo install tauri-cli --locked
```

4. Doğrula (hepsi çıktı vermeli):

```powershell
cargo --version
rustc --version
cargo tauri --version
where.exe link
```

`cargo tauri dev` WebView2 ister; Win11'de hazır gelir, yoksa `winget install --id Microsoft.EdgeWebView2Runtime -e` ile kur.

### 2. Workspace iskeletini kur

Kök `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/soundcore-lib5-core", "crates/scanner", "src-tauri"]
```

Oluşturulacak dizin/yapı (hepsi elle yazılır; `cargo new`, `tauri init` gibi interaktif sihirbazlar KULLANILMAZ — dosyalar aşağıda tanımlıdır):

```text
Cargo.toml
.gitignore                       (target/, .vs/, *.user)
crates/soundcore-lib5-core/      (BLE + profil katmanı, lib)
crates/scanner/                  (CLI keşif aracı, bin)
src-tauri/                       (Tauri uygulaması, bin)
ui/                              (statik HTML/JS/CSS arayüz)
docs/protocol-notes.md           (protokol bulguları, boş başlar)
```

`crates/soundcore-lib5-core/Cargo.toml`:

```toml
[package]
name = "soundcore-lib5-core"
version = "0.1.0"
edition = "2021"

[dependencies]
btleplug = "0.12"
tokio = { version = "1", features = ["rt", "macros", "time", "sync"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["serde"] }
hex = "0.4"
thiserror = "1"
```

`crates/scanner/Cargo.toml`:

```toml
[package]
name = "scanner"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "scanner"
path = "src/main.rs"

[dependencies]
soundcore-lib5-core = { path = "../soundcore-lib5-core" }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
clap = { version = "4", features = ["derive"] }
hex = "0.4"
```

`src-tauri/Cargo.toml`:

```toml
[package]
name = "soundcore-liberty5-app"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync"] }
soundcore-lib5-core = { path = "../crates/soundcore-lib5-core" }
```

İlk kabul: `cargo build --workspace` sıfır hata (henüz kaynak dosyalar boş ya da minimal olabilir, aşağıdaki adımlar tamamlanınca tam derlenir).

### 3. BLE katmanı (core crate)

`crates/soundcore-lib5-core/src/device_finder.rs`:

```rust
pub struct LibertyDeviceInfo {
    pub device_id: String,   // btleplug PeripheralId -> String
    pub name: String,
    pub address: String,     // PeripheralProperties.address -> String
    pub connected: bool,
}

pub async fn find_liberty_devices() -> Result<Vec<LibertyDeviceInfo>, BleError>;
```

Uygulama: `btleplug::platform::Central::new().await?` → `central.peripherals().await?` içinde `p.properties().await` → `local_name` case-insensitive `"liberty"` içerenleri topla. Eşleşme çıkmazsa `central.start_scan().await?` + 5 sn bekle + `stop_scan().await?` + aynı filtreyle tekrar topla (yakında ama eşlenmemiş cihazlar için). `PeripheralId`'yi `String`'e `to_string()` ile çevir; geri dönüşte `p.from_id(id.into())` yerine `central.peripheral(id)` kullan. `BleError` = `thiserror` ile `btleplug::Error` + `NotFound` + `NotSupported` varyantları.

`crates/soundcore-lib5-core/src/gatt_session.rs`:

```rust
pub struct CharacteristicSnapshot {
    pub service_uuid: uuid::Uuid,
    pub characteristic_uuid: uuid::Uuid,
    pub properties: Vec<&'static str>,   // "read" | "write" | "writeWithoutResponse" | "notify" | "indicate"
    pub value_hex: Option<String>,
    pub error: Option<String>,
}

pub struct GattSession { peripheral: Peripheral }

impl GattSession {
    pub async fn open(peripheral: Peripheral) -> Result<Self, BleError>;
    pub async fn enumerate(&self) -> Result<Vec<CharacteristicSnapshot>, BleError>;
    pub async fn read(&self, service_uuid: Uuid, characteristic_uuid: Uuid) -> Result<Vec<u8>, BleError>;
    pub async fn write(&self, service_uuid: Uuid, characteristic_uuid: Uuid, payload: &[u8], with_response: bool) -> Result<(), BleError>;
    pub async fn subscribe(&self, service_uuid: Uuid, characteristic_uuid: Uuid, on_value: impl Fn(Vec<u8>) + Send + 'static) -> Result<(), BleError>;
}
```

Uygulama: `open` içinde `peripheral.connect().await?` + `peripheral.discover_services().await?`; bağlantı hatası `AccessDenied` türünde ise hata mesajına "Telefondaki Soundcore uygulamasını kapatın; Windows'ta Ayarlar > Bluetooth ve cihazlar > cihazı kaldırıp yeniden eşleştirin (PIN 0000)" ekle. `enumerate` her karakteristik için `properties`'i `CharacteristicProperties` bit bayraklarından eşle; `READ` varsa `read(&c)` dene, hata olursa `error` alanına yaz ve devam et (tek karakteristik hatası taramayı durdurmasın). `write`: `with_response` → `WriteType::WithResponse`, değilse `WriteType::WithoutResponse`. `subscribe`: `properties` Notify/Indicate içermiyorsa `NotSupported` döndür; `peripheral.subscribe(&c).await?` + `peripheral.on_notification(move |n| on_value(n.value.to_vec()))`; aboneliği bırakma işlemi için `unsubscribe` gerekirse `disconnect` yeterlidir.

### 4. Komut profili ve yüksek seviye cihaz servisi

`crates/soundcore-lib5-core/src/command_profile.rs` — JSON şeması (System.Text.Json değil, serde):

```rust
#[derive(Deserialize)]
pub struct CommandProfile {
    pub device_name_pattern: String,
    #[serde(default)]
    pub eq_presets: HashMap<String, String>,
    #[serde(default)]
    pub characteristics: Vec<ProfileCharacteristic>,
}

#[derive(Deserialize)]
pub struct ProfileCharacteristic {
    pub service_uuid: uuid::Uuid,
    pub uuid: uuid::Uuid,
    pub kind: String,            // "anc" | "gameMode" | "eqPreset" | "battery" | "unknown"
    pub write_type: String,      // "withResponse" | "withoutResponse" | "none"
    #[serde(default)]
    pub commands: HashMap<String, String>,   // hex string -> bytes
    #[serde(default)]
    pub notify: bool,
}
```

`CommandProfile::embedded()`: `include_str!("../profiles/liberty5.json")` → `serde_json::from_str`; parse hatası veya geçersiz uuid/hex `BleError::Profile` döndürür; `get_characteristic(&self, kind: &str) -> Option<&ProfileCharacteristic>`; `decode_command(&self, kind: &str, command: &str) -> Result<Vec<u8>, BleError>` (eksik command → `NotFound`).

`crates/soundcore-lib5-core/profiles/liberty5.json` — yalnızca cihaz üzerinde doğrulanmış değerlerle doldurulur (Adım 6). Yer tutucu şablon:

```json
{
  "deviceNamePattern": "soundcore Liberty 5",
  "eqPresets": {},
  "characteristics": [
    {
      "serviceUuid": "0000180f-0000-1000-8000-00805f9b34fb",
      "uuid": "00002a19-0000-1000-8000-00805f9b34fb",
      "kind": "battery",
      "writeType": "none",
      "notify": true
    }
  ]
}
```

Alan adları JSON'da camelCase (`serviceUuid`, `writeType`), Rust tarafında `#[serde(rename_all = "camelCase")]` kullan.

`crates/soundcore-lib5-core/src/liberty5_device.rs`:

```rust
pub enum AncMode { Off, Transparency, On }

pub struct BatteryStatus { pub left: Option<u8>, pub right: Option<u8>, pub case: Option<u8> }

pub struct Liberty5Device { peripheral: Peripheral, session: GattSession, profile: Arc<CommandProfile> }

impl Liberty5Device {
    pub async fn open(peripheral: Peripheral, profile: Arc<CommandProfile>) -> Result<Self, BleError>;
    pub async fn read_battery(&self) -> Result<BatteryStatus, BleError>;
    pub async fn set_anc(&self, mode: AncMode) -> Result<(), BleError>;
    pub async fn set_game_mode(&self, enabled: bool) -> Result<(), BleError>;
    pub async fn set_eq_preset(&self, preset_id: &str) -> Result<(), BleError>;
    pub fn eq_presets(&self) -> &HashMap<String, String>;
    pub async fn disconnect(&self) -> Result<(), BleError>;
}
```

Davranış: profilde ilgili `kind` yoksa veya `commands`'ta anahtar yoksa `BleError::NotSupported` döndür, asla tahmini/uygulama içi sabit byte yazma. `write_type = "none"` olan karakteristiğe yazma denemesi `NotSupported`. Battery: standart `0x2a19` ise tek byte = yüzde; `left = right = byte`, `case = None`. Vendor formatı için yalnız `docs/protocol-notes.md`'de parse kuralı doğrulanmışsa parse et, yoksa `None` döndür.

### 5. CLI keşif aracı (scanner bin)

`crates/scanner/src/main.rs`, clap derive ile 4 alt komut:

```text
scanner list
scanner dump <device-id-veya-ismin-alt-dizesi>
scanner monitor <device-id-veya-ismin-alt-dizesi>
scanner write <device-id-veya-ismin-alt-dizesi> <service-uuid> <characteristic-uuid> <hex> --force
```

- `list`: `find_liberty_devices` sonuçlarını yazdır: ad, device_id, adres, bağlantı durumu. Boşsa "Liberty cihazı bulunamadı — eşleştirme yapıldığından emin olun" yaz, exit 1.
- `dump`: cihazı çöz (birden fazla eşleşme varsa seçenekleri yazdır, exit 2 — otomatik seçim yapma), `GattSession::open` + `enumerate`, servis/karakteristik ağacını UUID + özellikler + hex/ASCII değerle yazdır.
- `monitor`: `enumerate` sonrası Notify/Indicate içeren her karakteristiğe `subscribe`; her pakette karakteristik UUID + hex yazdır; Ctrl+C ile temiz çık (tokio `select!` + `ctrl_c`).
- `write`: hex'i `hex::decode` ile çöz; `--force` YOKSA hiçbir şey yazma ve "Bu komut kulaklığa zarar verebilir; --force gerekir" deyip exit 3. `--force` ile `with_response = true` kullan.
- `Console.OutputEncoding` eşdeğeri: Rust'ta `std::io::stdout` UTF-8 çıktısı varsayılan; ayrıca ayar gerekmez, ancak Windows terminali Türkçe karakter için `chcp 65001` gerekebilir — komut çıktıları ASCII/hex ağırlıklı tutulur.

### 6. Liberty 5 GATT/protokol profilini çıkar

CLI keşfinden sonra çalışır ve onun çıktısına bağımlıdır.

`tools/jadx/` altına jadx'ın resmi release arşivini indirip aç. Soundcore Android APK'sını kullanıcının sağladığı veya resmi/uygun kaynaktan edinilen `tools/soundcore.apk` olarak koy. Analiz:

```powershell
java -jar tools/jadx/jadx-cli.jar -d tools/soundcore-decompiled tools/soundcore.apk
```

`tools/soundcore-decompiled` içinde ara: `ffe0`, `ffe1`, `ffe2`, `ffe3`, `0000ffe`, `UUID.fromString`, `writeCharacteristic`, `WriteValue`, `WRITE_TYPE`, `ANC`, `game`, `equalizer`, `eq`. Bulunan UUID, write type, command byte dizisi ve durum okuma biçimini `docs/protocol-notes.md`'ye yaz. Bulgular cihaz üzerinde doğrulanmadan profil JSON'una işlenmez.

Android telefon mevcutsa ikinci doğrulama kanalı: Geliştirici seçenekleri → Bluetooth HCI snoop log aç; Soundcore uygulamasında sırayla ANC Off → Transparency → On, Game Mode On → Off, en az iki EQ preset değiştir; logu `adb pull /data/misc/bluetooth/logs/btsnoop_hci.log` (olmadıysa `/sdcard/btsnoop_hci.log`) ile al; Wireshark'ta `btatt` filtresiyle ATT Write Request/Command paketlerini UI aksiyonlarıyla eşleştir.

APK ve HCI çelişirse cihazda gerçekleşen HCI yazma paketi esas alınır. Android yoksa APK yolu birincil kalır; obfuscation nedeniyle komut bulunamazsa uygulama yalnız keşif modunda teslim edilir — uydurma byte dizisi yazılmaz.

Doğrulanan her komut için `crates/soundcore-lib5-core/profiles/liberty5.json`'a ilgili karakteristik kaydını ve hex command'ları ekle. Doğrulama yöntemi: `scanner write <cihaz> <svc-uuid> <char-uuid> <ancOnHex> --force` → ANC işitsel olarak devreye girer; EQ yazımı → ses değişimi işitsel; karakteristik `notify` içeriyorsa `scanner monitor` ile readback.

### 7. Tauri uygulaması

Elle yazılacak dosyalar:

`src-tauri/tauri.conf.json` (v2 şema):

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Soundcore Liberty 5",
  "version": "0.1.0",
  "identifier": "com.vibec.soundcore-liberty5",
  "build": {
    "frontendDist": "../ui"
  },
  "app": {
    "windows": [
      { "title": "Soundcore Liberty 5", "width": 480, "height": 640, "resizable": false }
    ],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": ["icons/icon.ico"]
  }
}
```

`devUrl` bilinçli olarak yok — `tauri dev` frontendDist'ten servis eder. `bundle.targets` yalnız `nsis` (WiX indirme adımını atlar).

`src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

Uygulamanın kendi `#[tauri::command]`'leri capability gerektirmez; `core:default` frontend'in `listen` yapmasına yeter.

İkonlar: PowerShell ile 512×512 düz PNG üret, sonra `cargo tauri icon` ile türet:

```powershell
Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap(512, 512)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(255, 18, 52, 86))
$font = New-Object System.Drawing.Font('Segoe UI', 220, [System.Drawing.FontStyle]::Bold)
$g.DrawString('L5', $font, [System.Drawing.Brushes]::White, 60, 100)
$bmp.Save('app-icon.png', [System.Drawing.Imaging.ImageFormat]::Png)
# sonra
cargo tauri icon app-icon.png
```

`cargo tauri icon`, `src-tauri/icons/` altında gerekli tüm boyutları (icon.ico dahil) üretir; kaynak PNG'yi `src-tauri/app-icon.png` altına taşı ve `ui/` altına koyma.

`src-tauri/src/lib.rs` — komutlar ve durum:

```rust
pub struct AppState {
    pub device: tokio::sync::Mutex<Option<Liberty5Device>>,
    pub profile: Arc<CommandProfile>,
    pub battery_task: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[tauri::command] async fn list_devices() -> Result<Vec<LibertyDeviceInfo>, String>;
#[tauri::command] async fn connect(state: tauri::State<'_, AppState>, device_id: String, app: tauri::AppHandle) -> Result<(), String>;
#[tauri::command] async fn disconnect(state: tauri::State<'_, AppState>) -> Result<(), String>;
#[tauri::command] async fn set_anc(state: tauri::State<'_, AppState>, mode: String) -> Result<(), String>;
#[tauri::command] async fn set_game_mode(state: tauri::State<'_, AppState>, enabled: bool) -> Result<(), String>;
#[tauri::command] async fn set_eq_preset(state: tauri::State<'_, AppState>, preset_id: String) -> Result<(), String>;
#[tauri::command] async fn read_battery(state: tauri::State<'_, AppState>) -> Result<BatteryStatus, String>;
#[tauri::command] async fn get_eq_presets(state: tauri::State<'_, AppState>) -> Result<Vec<(String, String)>, String>;
```

- `connect`: `find_liberty_devices` içinde `device_id` eşleşen `Peripheral`'ı bul (yoksa `NotFound` mesajı), `Liberty5Device::open`, state'e yaz, `app.emit("connection", true)`; ardından `tokio::spawn` ile 60 sn'de bir `read_battery` → `app.emit("battery", value)` döngüsünü başlat (battery_task'a kaydet; yeni connect eski task'ı `abort()` eder).
- `set_anc` → `mode: "Off"|"Transparency"|"On"` dışında değer → hata. `set_game_mode`, `set_eq_preset` doğrudan `Liberty5Device`'a delege eder; hata mesajı Türkçe olarak kullanıcıya döner ("Telefondaki Soundcore uygulamasını kapatın..." dahil).
- `setup` hook'u: `TrayIconBuilder::new().icon(app.default_window_icon().unwrap().clone())` + menü (Bağlan/Kes, ANC Döngüsü, Game Mode Aç/Kapat, Çıkış) + `on_menu_event`; ANC döngüsü `Off → Transparency → On → Off` sırasını state'te `AncMode` olarak takip eder. `on_exit`'te `device.disconnect()` çağır.
- `run()` fonksiyonu `tauri::Builder::default().manage(AppState{..}).invoke_handler(tauri::generate_handler![...]).setup(...).run(...)` şeklinde; `src-tauri/src/main.rs` yalnız `soundcore_liberty5_app_lib::run()` çağırır. `src-tauri/build.rs`: `fn main() { tauri_build::build() }`.

`ui/index.html` + `ui/app.js` + `ui/styles.css` (statik, derleme adımı yok, framework yok): cihaz dropdown + Bağlan/Kes; sol/sağ/kutu pil + bağlantı durumu; ANC üçlü buton (Off/Transparency/On); Game Mode toggle; EQ ComboBox (`get_eq_presets` ile doldurulur, `set_eq_preset` ile uygulanır); log paneli. JS: `window.__TAURI__.core.invoke("list_devices")`, `invoke("set_anc", { mode: "On" })`, `invoke("set_game_mode", { enabled: true })`, `window.__TAURI__.event.listen("battery", (e) => ...)`, `listen("connection", ...)`. Tüm invoke hatalarını log paneline Türkçe yaz. Profil JSON'unda `eqPresets` boşsa EQ ComboBox devre dışı kalır.

### 8. Yayınla ve plan.md'yi köke yaz

Uygulama çalıştıktan sonra kök `plan.md`, canonical planın birebir kopyası olur (kullanıcı isteği). Yayın:

```powershell
cd src-tauri
cargo tauri build
```

Çıktı: `src-tauri/target/release/bundle/nsis/Soundcore Liberty 5_0.1.0_x64-setup.exe`. Kurulumcu, WebView2 (Win11 hazır) ve sistem tepsisinde çalışan uygulamayı kurar.

## Critical files & anchors

- `crates/soundcore-lib5-core/profiles/liberty5.json` — doğrulanmış UUID, write type ve command byte'larının tek kaynağı; UI görünürlüğünü de bu sürer.
- `crates/soundcore-lib5-core/src/gatt_session.rs` — tüm BLE bağlantı, keşif, okuma, yazma, notification akışı; btleplug'a tek temas noktası (geri dönüş burada izole edilir).
- `crates/scanner/src/main.rs` — gerçek Liberty 5 servis/karakteristik çıktısını üreten keşif ve `--force` güvenlik kapılı write aracı.
- `src-tauri/src/lib.rs` — Tauri komutları, AppState, pil polling task'ı ve tepsi menüsü.
- `src-tauri/tauri.conf.json` — frontendDist, NSIS bundle, pencere ve ikon yapılandırması.

## Verification

1. `cargo build --workspace` → sıfır hata.
2. `cargo test --workspace` → core'da profil parse testleri (embedded JSON parse edilir, camelCase alan adları doğrulanır, eksik command `NotFound` döndürür, geçersiz hex `Profile` hatası döndürür) yeşil.
3. Liberty 5 Windows'a eşli, telefon Soundcore uygulaması kapalı:

```powershell
cargo run -p scanner -- list
cargo run -p scanner -- dump "Liberty 5"
```

`list` cihazı gösterir; `dump` servis ağacını, özellikleri ve okunabilir değerleri gösterir; standart Battery Service (0x180f/0x2a19) bulunması zorunlu değil, vendor servisleri görünmelidir.

4. `cargo run -p scanner -- monitor "Liberty 5"` çalışırken telefondan ANC/Game Mode/EQ değiştir; notification paketleri UUID + hex olarak görünür ve `docs/protocol-notes.md` bulgularıyla eşleşir.
5. Profil doldurulduktan sonra gerçek davranış testi:
   - `scanner write <cihaz> <svc> <char> <ancOnHex> --force` → ANC fiziksel olarak etkinleşir; `transparency` hex'i → çevre sesi duyulur; `off` → etki kalkar.
   - `set_game_mode(true)` → video/oyun ses gecikmesi azalır veya doğrulanmış readback değişir.
   - `set_eq_preset("bassBoost")` ve `"vocal"` → duyulabilir fark.
   - Pil değerleri telefon uygulamasıyla ±%5; vendor formatı doğrulanmamışsa `None` gösterilir.
6. `cd src-tauri; cargo tauri dev` ile uygulamayı aç; cihaz seç, bağlan, ANC üç modunu, Game Mode'u ve en az iki EQ preset'ini UI'dan değiştir; her işlem log paneline yazılır, hata durumunda sahte başarı gösterilmez. Pencere kapatılınca tepside kalır, Çıkış ile BLE bağlantısı kapanır.
7. `cargo tauri build` sonrası NSIS kurulumcusunu çalıştır, kur, exe'yi başlat ve Adım 6'daki kontrolleri tekrar yap.

## Assumptions & contingencies

- Makinede Rust/MSVC yok (doğrulandı: `cargo`, `rustc`, `vswhere` bulunamadı; yalnız Git'in `link.exe`'si var) — Adım 1 kurulumu zorunlu. Build Tools override'ı başarısız olursa Visual Studio Installer'ı elle açıp "C++ ile masaüstü geliştirme" iş yükünü kur, sonra Adım 1.4'ü tekrarla.
- Sürümler crates.io sparse index'ten doğrulandı: `btleplug 0.12.0`, `tauri 2.11.5`, `tauri-build 2.6.3`. Cargo `^` semver ile en yakın uyumluya çözer.
- Liberty 5'in özel GATT UUID ve byte komutları bu oturumda doğrulanmadı; başka Soundcore modellerinin UUID/byte dizileri Liberty 5'e kopyalanmaz. APK/HCI/device dump doğrulaması yoksa ANC/Game Mode/EQ write özelliği etkinleştirilmez, uygulama keşif + pil modunda kalır.
- btleplug Windows backend'i eşli cihazı `peripherals()` içinde göstermezse 5 sn'lik tarama fallback'i devreye girer; bağlantı "not paired" hatası verirse kullanıcıya yeniden eşleştirme talimatı gösterilir. btleplug vendor servislerde başarısız olursa (karakteristik yazma/abonelik hataları), `gatt_session.rs` içindeki uygulama `windows` crate (0.61+, `Windows.Devices.Bluetooth` WinRT) ile yeniden yazılır — API yüzeyi (`read`/`write`/`subscribe` imzaları) aynı kalır, diğer katmanlar değişmez.
- Android telefon dinamik capture opsiyoneldir; yoksa APK statik analizi + Windows GATT dump kullanılır.
- Kulaklık ses multipoint desteklese de kontrol BLE bağlantısı tek cihazla sınırlı olabilir; PC'den yazarken telefon Soundcore uygulaması kapalı tutulur, aksi halde hata loglanır.
- WebView2 Win11 LTSC 2024'te hazır varsayılır; `tauri dev` hata verirse `winget install --id Microsoft.EdgeWebView2Runtime -e`.
- Firmware update, factory reset ve doğrulanmamış raw command uygulamaya eklenmez.
