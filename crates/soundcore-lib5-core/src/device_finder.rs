use std::collections::HashSet;

use btleplug::api::{Central, Manager as _, Peripheral as _, PeripheralProperties, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use serde::Serialize;
use tokio::time::{sleep, timeout, Duration};

use crate::error::BleError;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibertyDeviceInfo { pub device_id: String, pub name: String, pub address: String, pub connected: bool }

#[derive(Clone, Debug)]
pub struct DiscoveredDevice { pub info: LibertyDeviceInfo, pub peripheral: Peripheral }

async fn adapters() -> Result<Vec<Adapter>, BleError> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    if adapters.is_empty() { return Err(BleError::NotFound); }
    Ok(adapters)
}

#[cfg(windows)]
async fn winrt_cached_name(peripheral: &Peripheral) -> Option<String> {
    use windows::Devices::Bluetooth::BluetoothLEDevice;
    let address = u64::from(peripheral.address());
    let device = BluetoothLEDevice::FromBluetoothAddressAsync(address).ok()?.await.ok()?;
    let name = device.Name().ok()?.to_string();
    (!name.trim().is_empty()).then_some(name)
}

#[cfg(not(windows))]
async fn winrt_cached_name(_peripheral: &Peripheral) -> Option<String> { None }

async fn enrich_properties(peripheral: &Peripheral, mut properties: PeripheralProperties) -> PeripheralProperties {
    if properties.local_name.is_some() || properties.advertisement_name.is_some() { return properties; }
    if let Some(name) = winrt_cached_name(peripheral).await {
        properties.local_name = Some(name);
        return properties;
    }

    // WinRT can expose paired peripherals with no advertised name. btleplug populates the
    // cached GAP name after a short GATT connection, so use that read-only path as a fallback.
    let was_connected = peripheral.is_connected().await.unwrap_or(false);
    if !was_connected && timeout(Duration::from_secs(2), peripheral.connect()).await.is_ok() {
        if let Ok(Some(refreshed)) = peripheral.properties().await { properties = refreshed; }
        let _ = peripheral.disconnect().await;
    }
    properties
}

async fn collect_liberty(adapter: &Adapter) -> Result<Vec<DiscoveredDevice>, BleError> {
    let mut found = Vec::new();
    for peripheral in adapter.peripherals().await? {
        let Some(properties) = peripheral.properties().await? else { continue };
        let properties = enrich_properties(&peripheral, properties).await;
        let name = properties.local_name.or(properties.advertisement_name).unwrap_or_else(|| format!("Bluetooth {}", properties.address));
        if !name.to_ascii_lowercase().contains("liberty") { continue; }
        found.push(DiscoveredDevice { info: LibertyDeviceInfo { device_id: peripheral.id().to_string(), name, address: properties.address.to_string(), connected: peripheral.is_connected().await.unwrap_or(false) }, peripheral });
    }
    Ok(found)
}

fn merge_unique(target: &mut Vec<DiscoveredDevice>, source: Vec<DiscoveredDevice>) {
    let mut ids: HashSet<String> = target.iter().map(|device| device.info.device_id.clone()).collect();
    for device in source { if ids.insert(device.info.device_id.clone()) { target.push(device); } }
}

pub async fn find_liberty_peripherals() -> Result<Vec<DiscoveredDevice>, BleError> {
    let adapters = adapters().await?;
    let mut found = Vec::new();
    for adapter in &adapters { merge_unique(&mut found, collect_liberty(adapter).await?); }
    if found.is_empty() {
        for adapter in &adapters { adapter.start_scan(ScanFilter::default()).await?; }
        sleep(Duration::from_secs(5)).await;
        for adapter in &adapters { merge_unique(&mut found, collect_liberty(adapter).await?); }
        for adapter in &adapters { let _ = adapter.stop_scan().await; }
    }
    Ok(found)
}

pub async fn find_liberty_devices() -> Result<Vec<LibertyDeviceInfo>, BleError> {
    let ble = find_liberty_peripherals().await?;
    if !ble.is_empty() {
        return Ok(ble.into_iter().map(|device| device.info).collect());
    }
    // BLE reklamı yoksa (uyku/kutu modu) eşleştirilmiş cihaz listesine düş:
    // cihaz canlı olmasa bile Windows adını saklar.
    #[cfg(windows)]
    {
        let paired = paired_liberty_devices().await;
        if !paired.is_empty() {
            return Ok(paired);
        }
    }
    Ok(Vec::new())
}

/// Windows eşleştirme listesinden "liberty" adlı cihazları toplar; cihazın o an
/// reklam vermesi gerekmez. Klasik (RFCOMM kontrol) adres tercih edilir; aynı
/// isimli LE kaydı yinelenmez.
#[cfg(windows)]
pub async fn paired_liberty_devices() -> Vec<LibertyDeviceInfo> {
    use windows::core::HSTRING;
    use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothLEDevice};
    use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationCollection};

    async fn find_all(selector: &HSTRING) -> Option<DeviceInformationCollection> {
        DeviceInformation::FindAllAsyncAqsFilter(selector).ok()?.await.ok()
    }

    async fn classic_address(id: &HSTRING) -> Option<u64> {
        BluetoothDevice::FromIdAsync(id).ok()?.await.ok()?.BluetoothAddress().ok()
    }

    async fn le_address(id: &HSTRING) -> Option<u64> {
        BluetoothLEDevice::FromIdAsync(id).ok()?.await.ok()?.BluetoothAddress().ok()
    }

    fn bt_address_string(address: u64) -> String {
        format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            (address >> 40) & 0xff,
            (address >> 32) & 0xff,
            (address >> 24) & 0xff,
            (address >> 16) & 0xff,
            (address >> 8) & 0xff,
            address & 0xff,
        )
    }

    let mut result: Vec<LibertyDeviceInfo> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // 1) Klasik (BR/EDR) eşleştirilmiş cihazlar.
    if let Ok(selector) = BluetoothDevice::GetDeviceSelectorFromPairingState(true) {
        if let Some(collection) = find_all(&selector).await {
            let size = collection.Size().unwrap_or(0);
            for index in 0..size {
                let Ok(info) = collection.GetAt(index) else { continue };
                let name = info.Name().map(|n| n.to_string()).unwrap_or_default();
                if !name.to_ascii_lowercase().contains("liberty") { continue; }
                seen.insert(name.to_ascii_lowercase());
                let address = match info.Id().ok() {
                    Some(id) => classic_address(&id).await.map(bt_address_string).unwrap_or_else(|| "—".to_string()),
                    None => "—".to_string(),
                };
                result.push(LibertyDeviceInfo {
                    device_id: format!("classic:{address}"),
                    name,
                    address,
                    connected: false,
                });
            }
        }
    }

    // 2) LE eşleştirilmiş cihazlar; isim klasik listede zaten varsa atla.
    if let Ok(selector) = BluetoothLEDevice::GetDeviceSelectorFromPairingState(true) {
        if let Some(collection) = find_all(&selector).await {
            let size = collection.Size().unwrap_or(0);
            for index in 0..size {
                let Ok(info) = collection.GetAt(index) else { continue };
                let name = info.Name().map(|n| n.to_string()).unwrap_or_default();
                if !name.to_ascii_lowercase().contains("liberty") { continue; }
                if !seen.insert(name.to_ascii_lowercase()) { continue; }
                let address = match info.Id().ok() {
                    Some(id) => le_address(&id).await.map(bt_address_string).unwrap_or_else(|| "—".to_string()),
                    None => "—".to_string(),
                };
                result.push(LibertyDeviceInfo {
                    device_id: format!("le:{address}"),
                    name,
                    address,
                    connected: false,
                });
            }
        }
    }

    result
}

#[cfg(not(windows))]
pub async fn paired_liberty_devices() -> Vec<LibertyDeviceInfo> { Vec::new() }

pub async fn resolve_liberty_peripheral(query: &str) -> Result<DiscoveredDevice, BleError> {
    let query = query.to_ascii_lowercase();
    let matches: Vec<_> = find_liberty_peripherals().await?.into_iter().filter(|device| device.info.device_id.to_ascii_lowercase() == query || device.info.name.to_ascii_lowercase().contains(&query) || device.info.address.to_ascii_lowercase() == query).collect();
    match matches.len() { 1 => Ok(matches.into_iter().next().expect("one match")), _ => Err(BleError::NotFound) }
}
