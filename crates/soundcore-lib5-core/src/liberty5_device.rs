use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;

use crate::command_profile::CommandProfile;
use crate::error::BleError;
use crate::rfcomm_session::{Frame, RfcommSession};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AncMode { Off, Transparency, On }

impl AncMode {
    pub fn as_str(self) -> &'static str {
        match self { Self::Off => "Off", Self::Transparency => "Transparency", Self::On => "On" }
    }

    pub fn parse(value: &str) -> Result<Self, BleError> {
        match value {
            "Off" => Ok(Self::Off),
            "Transparency" => Ok(Self::Transparency),
            "On" => Ok(Self::On),
            other => Err(BleError::InvalidAncMode(other.to_string())),
        }
    }

    /// 0x0106 bildirimindeki mod baytı (Gadgetbridge eşlemesi, gerçek cihazda
    /// doğrulandı: 0x00 = ANC, 0x01 = Transparency, 0x02 = Off).
    fn from_mode_byte(byte: u8) -> Option<Self> {
        match byte { 0x00 => Some(Self::On), 0x01 => Some(Self::Transparency), 0x02 => Some(Self::Off), _ => None }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryStatus {
    pub left: Option<u8>,
    pub right: Option<u8>,
    pub case: Option<u8>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub serial: Option<String>,
    pub firmware: Option<String>,
    pub anc_mode: Option<String>,
}

const CMD_GET_DEVICE_INFO: u16 = 0x0101;
const CMD_NOTIFY_AUDIO_MODE: u16 = 0x0106;
/// Cihaz her isteğe yanıt verdikten sonra RFCOMM kanalını kapatır; sonraki
/// yazım 0x800710DD ile düşer. Komut hatası durumunda oturum yeniden açılır.
const RETRY_DELAY: Duration = Duration::from_millis(1500);

pub struct Liberty5Device {
    session: Option<RfcommSession>,
    profile: Arc<CommandProfile>,
    device_address: String,
}

impl Liberty5Device {
    /// Liberty 5 kontrol servisine RFCOMM üzerinden bağlanır.
    pub async fn open(profile: Arc<CommandProfile>, device_address: &str) -> Result<Self, BleError> {
        let session = RfcommSession::open(&profile.control_service_uuid, device_address).await?;
        Ok(Self { session: Some(session), profile, device_address: device_address.to_string() })
    }

    /// Komut gönderir; taşıyıcı hatasında (0x800710DD gibi) oturumu yeniden
    /// açıp tek kez dener. Aynı oturumdaki ikinci komutun cihaz tarafından
    /// kapatılan kanala yazılmasını engeller.
    async fn command(&mut self, command: u16, payload: &[u8]) -> Result<Vec<Frame>, BleError> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            if self.session.is_none() {
                self.session = Some(RfcommSession::open(&self.profile.control_service_uuid, &self.device_address).await?);
            }
            match self.session.as_mut().expect("oturum açık").command(command, payload).await {
                Ok(frames) => return Ok(frames),
                Err(_error) if attempts < 2 => {
                    eprintln!("[retry] komut=0x{command:04x} hatasi: {_error}; oturum yeniden aciliyor");
                    self.session = None;
                    tokio::time::sleep(RETRY_DELAY).await;
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// 0x0101 cihaz bilgisi: seri numarası, firmware ve mevcut ANC modu.
    pub async fn read_device_info(&mut self) -> Result<DeviceInfo, BleError> {
        let payload = self.profile.payload("deviceInfo", "request")?;
        let frames = self.command(CMD_GET_DEVICE_INFO, &payload).await?;
        let mut info = DeviceInfo::default();
        for frame in &frames {
            if frame.command == CMD_GET_DEVICE_INFO && frame.payload.len() >= 32 {
                info.serial = Some(String::from_utf8_lossy(&frame.payload[16..32]).into_owned());
                let fw1 = String::from_utf8_lossy(&frame.payload[6..11]).into_owned();
                let fw2 = String::from_utf8_lossy(&frame.payload[11..16]).into_owned();
                info.firmware = Some(if fw1 == fw2 { fw1 } else { format!("{fw1}/{fw2}") });
            }
            if let Some(mode) = Self::audio_mode(&frame) {
                info.anc_mode = Some(mode.as_str().to_string());
            }
        }
        Ok(info)
    }

    /// 0x0106 bildiriminden geçerli ANC modu.
    fn audio_mode(frame: &Frame) -> Option<AncMode> {
        if frame.command == CMD_NOTIFY_AUDIO_MODE {
            return frame.payload.first().and_then(|byte| AncMode::from_mode_byte(*byte));
        }
        None
    }

    /// ANC modunu ayarlar; cihazdan onay bekler. Gerçek cihazda doğrulandı.
    pub async fn set_anc(&mut self, mode: AncMode) -> Result<(), BleError> {
        let payload = self.profile.payload("anc", mode.as_str())?;
        let command_code = self.profile.command("anc").ok_or(BleError::NotSupported)?.command;
        let frames = self.command(command_code, &payload).await?;
        if frames.iter().any(|frame| frame.command == command_code && frame.payload.is_empty()) {
            Ok(())
        } else {
            Err(BleError::Connection("ANC komutuna onay alınamadı".to_string()))
        }
    }

    /// Doğrulanmış komut bulunmadığı için kilitli.
    pub async fn set_eq_preset(&mut self, _preset_id: &str) -> Result<(), BleError> { Err(BleError::NotSupported) }

    /// Game Mode: 0x8510 [0x01]=açık, [0x00]=kapalı. Telefon uygulamasının
    /// HCI yakalamasından alındı; gerçek cihazda işitsel olarak doğrulandı
    /// (açıkken ses boğuklaşıyor, kapalıyken netleşiyor).
    pub async fn set_game_mode(&mut self, enabled: bool) -> Result<(), BleError> {
        let payload = self.profile.payload("gameMode", if enabled { "On" } else { "Off" })?;
        let command_code = self.profile.command("gameMode").ok_or(BleError::NotSupported)?.command;
        let frames = self.command(command_code, &payload).await?;
        if frames.iter().any(|frame| frame.command == command_code && frame.payload.is_empty()) {
            Ok(())
        } else {
            Err(BleError::Connection("Game Mode komutuna onay alınamadı".to_string()))
        }
    }

    /// Pil: 0x0301 istek -> `[sol][sağ][kutu]`, yüzde = (değer+1)*10.
    /// Gerçek cihazda doğrulandı: `09 06 06` -> %100/%70/%70 (uygulamayla birebir).
    pub async fn read_battery(&mut self) -> Result<BatteryStatus, BleError> {
        let payload = self.profile.payload("battery", "request")?;
        let command_code = self.profile.command("battery").ok_or(BleError::NotSupported)?.command;
        let frames = self.command(command_code, &payload).await?;
        let status = frames
            .iter()
            .find(|frame| frame.command == command_code && frame.payload.len() >= 3)
            .map(|frame| decode_battery(&frame.payload))
            .unwrap_or_default();
        Ok(status)
    }

    /// Belirtilen süre boyunca gelen tüm çerçeveleri toplar.
    pub async fn monitor(&mut self, duration: Duration) -> Result<Vec<Frame>, BleError> {
        let session = self.ensure_session().await?;
        session.read_until(duration, |_| false).await
    }

    /// Ham komut gönderir (yalnızca protokol keşfi için).
    pub async fn send_raw(&mut self, command: u16, payload: &[u8]) -> Result<Vec<Frame>, BleError> {
        self.command(command, payload).await
    }

    async fn ensure_session(&mut self) -> Result<&mut RfcommSession, BleError> {
        if self.session.is_none() {
            self.session = Some(RfcommSession::open(&self.profile.control_service_uuid, &self.device_address).await?);
        }
        Ok(self.session.as_mut().expect("oturum açık"))
    }

    pub async fn disconnect(&mut self) -> Result<(), BleError> {
        if let Some(mut session) = self.session.take() {
            session.detach().await?;
        }
        Ok(())
    }
}

/// 0x0301 pil yükü: `[sol][sağ][kutu]`, yüzde = (değer+1)*10.
/// Gerçek cihazda doğrulandı: 09 06 06 -> %100/%70/%70.
fn decode_battery(payload: &[u8]) -> BatteryStatus {
    let pct = |value: u8| u8::try_from((value as u16 + 1) * 10).ok();
    BatteryStatus {
        left: pct(payload[0]),
        right: pct(payload[1]),
        case: pct(payload[2]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_decode_matches_verified_device_values() {
        let status = decode_battery(&[0x09, 0x06, 0x06]);
        assert_eq!(status.left, Some(100));
        assert_eq!(status.right, Some(70));
        assert_eq!(status.case, Some(70));
    }

    #[test]
    fn battery_decode_handles_low_values() {
        let status = decode_battery(&[0x00, 0x01, 0x05]);
        assert_eq!(status.left, Some(10));
        assert_eq!(status.right, Some(20));
        assert_eq!(status.case, Some(60));
    }
}
