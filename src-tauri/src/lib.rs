use std::sync::Arc;
use std::time::Duration;

use soundcore_lib5_core::{
    find_liberty_devices, AncMode, BatteryStatus, BleError, CommandProfile, Liberty5Device,
};
use serde::Serialize;
use tauri::{
    menu::MenuBuilder,
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tokio::sync::Mutex;

/// Pil yoklama aralığı (cihaz %10 kademeli raporlar).
const BATTERY_POLL_SECONDS: u64 = 30;

pub struct AppState {
    pub device: Arc<Mutex<Option<Liberty5Device>>>,
    pub profile: Arc<CommandProfile>,
    pub anc_mode: Arc<Mutex<AncMode>>,
    pub battery_poll: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl AppState {
    fn new(profile: CommandProfile) -> Self {
        Self {
            device: Arc::new(Mutex::new(None)),
            profile: Arc::new(profile),
            anc_mode: Arc::new(Mutex::new(AncMode::Off)),
            battery_poll: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    detail: String,
}

impl ApiError {
    fn new(code: &'static str, detail: String) -> Self {
        Self { code, detail }
    }
    fn not_connected() -> Self {
        Self::new("not_connected", String::new())
    }
}

fn api_error(error: BleError) -> ApiError {
    ApiError::new(error.code(), error.detail())
}

#[tauri::command]
async fn list_devices() -> Result<Vec<soundcore_lib5_core::LibertyDeviceInfo>, ApiError> {
    find_liberty_devices().await.map_err(api_error)
}

#[tauri::command]
async fn connect(state: State<'_, AppState>, app: AppHandle, device_address: String) -> Result<(), ApiError> {
    if let Some(handle) = state.battery_poll.lock().await.take() {
        handle.abort();
    }
    if let Some(mut old) = state.device.lock().await.take() {
        let _ = old.disconnect().await;
    }
    let mut device = Liberty5Device::open(Arc::clone(&state.profile), &device_address).await.map_err(api_error)?;
    let info = device.read_device_info().await.map_err(api_error)?;
    let anc_mode = info.anc_mode.clone();
    *state.anc_mode.lock().await = match anc_mode.as_deref() {
        Some("Transparency") => AncMode::Transparency,
        Some("Off") => AncMode::Off,
        _ => AncMode::On,
    };
    *state.device.lock().await = Some(device);
    let _ = app.emit("connection", true);
    let _ = app.emit("device-info", info);
    let _ = app.emit("anc", anc_mode.unwrap_or_else(|| "On".to_string()));

    // İlk pil okuması + bağlıyken 30 sn'de bir yoklama.
    let status = {
        let mut guard = state.device.lock().await;
        match guard.as_mut() {
            Some(device) => Some(device.read_battery().await.map_err(api_error)?),
            None => None,
        }
    };
    if let Some(status) = status {
        let _ = app.emit("battery", &status);
    }
    let device_arc = Arc::clone(&state.device);
    let app_handle = app.clone();
    let handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(BATTERY_POLL_SECONDS)).await;
            let status = {
                let mut guard = device_arc.lock().await;
                match guard.as_mut() {
                    Some(device) => device.read_battery().await.ok(),
                    None => break,
                }
            };
            if let Some(status) = status {
                let _ = app_handle.emit("battery", &status);
            }
        }
    });
    *state.battery_poll.lock().await = Some(handle);
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, AppState>, app: AppHandle) -> Result<(), ApiError> {
    if let Some(handle) = state.battery_poll.lock().await.take() {
        handle.abort();
    }
    if let Some(mut device) = state.device.lock().await.take() {
        device.disconnect().await.map_err(api_error)?;
    }
    let _ = app.emit("connection", false);
    Ok(())
}

#[tauri::command]
async fn set_anc(state: State<'_, AppState>, mode: String, app: AppHandle) -> Result<(), ApiError> {
    let mode = AncMode::parse(&mode).map_err(api_error)?;
    let mut guard = state.device.lock().await;
    let device = guard.as_mut().ok_or_else(ApiError::not_connected)?;
    device.set_anc(mode).await.map_err(api_error)?;
    *state.anc_mode.lock().await = mode;
    let _ = app.emit("anc", mode.as_str());
    Ok(())
}

#[tauri::command]
async fn set_game_mode(state: State<'_, AppState>, enabled: bool, app: AppHandle) -> Result<(), ApiError> {
    let mut guard = state.device.lock().await;
    let device = guard.as_mut().ok_or_else(ApiError::not_connected)?;
    device.set_game_mode(enabled).await.map_err(api_error)?;
    let _ = app.emit("game-mode", enabled);
    Ok(())
}

#[tauri::command]
async fn set_eq_preset(state: State<'_, AppState>, preset_id: String) -> Result<(), ApiError> {
    let mut guard = state.device.lock().await;
    let device = guard.as_mut().ok_or_else(ApiError::not_connected)?;
    device.set_eq_preset(&preset_id).await.map_err(api_error)
}

#[tauri::command]
async fn read_battery(state: State<'_, AppState>, app: AppHandle) -> Result<BatteryStatus, ApiError> {
    let mut guard = state.device.lock().await;
    let device = guard.as_mut().ok_or_else(ApiError::not_connected)?;
    let status = device.read_battery().await.map_err(api_error)?;
    let _ = app.emit("battery", &status);
    Ok(status)
}

#[tauri::command]
async fn get_eq_presets(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    Ok(state.profile.eq_presets.iter().map(|(id, label)| (id.clone(), label.clone())).collect())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FeatureAvailability {
    anc: bool,
    game_mode: bool,
    eq_preset: bool,
}

#[tauri::command]
async fn get_capabilities(state: State<'_, AppState>) -> Result<FeatureAvailability, String> {
    let has = |kind: &str| state.profile.command(kind).map(|command| !command.payloads.is_empty()).unwrap_or(false);
    Ok(FeatureAvailability {
        anc: has("anc"),
        game_mode: has("gameMode"),
        eq_preset: !state.profile.eq_presets.is_empty() && has("eqPreset"),
    })
}

#[tauri::command]
fn set_language(app: AppHandle, lang: String) -> Result<(), String> {
    let menu = tray_menu(&app, &lang).map_err(|e| e.to_string())?;
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn tray_menu(app: &tauri::AppHandle, lang: &str) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let (show, disconnect, anc_cycle, quit) = if lang == "tr" {
        ("Pencereyi Göster", "Bağlantıyı Kes", "ANC Döngüsü", "Çıkış")
    } else {
        ("Show Window", "Disconnect", "Cycle ANC", "Quit")
    };
    MenuBuilder::new(app)
        .text("show", show)
        .text("disconnect", disconnect)
        .text("anc-cycle", anc_cycle)
        .text("quit", quit)
        .build()
}

fn build_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = tray_menu(app.handle(), "tr")?;
    let tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref().to_string();
            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                match id.as_str() {
                    "show" => {
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.show();
                        }
                    }
                    "disconnect" => {
                        let state = handle.state::<AppState>();
                        if let Some(handle) = state.battery_poll.lock().await.take() {
                            handle.abort();
                        }
                        if let Some(mut device) = state.device.lock().await.take() {
                            let _ = device.disconnect().await;
                        }
                        let _ = handle.emit("connection", false);
                    }
                    "anc-cycle" => {
                        let state = handle.state::<AppState>();
                        let mut guard = state.device.lock().await;
                        let next = match *state.anc_mode.lock().await {
                            AncMode::Off => AncMode::Transparency,
                            AncMode::Transparency => AncMode::On,
                            AncMode::On => AncMode::Off,
                        };
                        *state.anc_mode.lock().await = next;
                        if let Some(device) = guard.as_mut() {
                            let _ = device.set_anc(next).await;
                            let _ = handle.emit("anc", next.as_str());
                        }
                    }
                    "quit" => handle.exit(0),
                    _ => {}
                }
            });
        })
        .build(app)?;
    app.manage(AppState::new(CommandProfile::embedded().expect("gömülü profil geçerli")));
    let _ = tray;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_devices,
            connect,
            disconnect,
            set_anc,
            set_game_mode,
            set_eq_preset,
            read_battery,
            get_eq_presets,
            get_capabilities,
            set_language,
        ])
        .on_window_event(|window, event| {
            // Uygulama sabit 9:16 telefon penceresi: fullscreen'i geri al,
            // boyutu her zaman 405x720 mantıksal piksele sabitle.
            if let tauri::WindowEvent::Resized(_) = event {
                if window.is_fullscreen().unwrap_or(false) {
                    let _ = window.set_fullscreen(false);
                }
                let _ = window.set_size(tauri::LogicalSize::new(405.0, 720.0));
            }
        })
        .setup(build_tray)
        .run(tauri::generate_context!())
        .expect("Tauri uygulaması başlatılamadı");
}
