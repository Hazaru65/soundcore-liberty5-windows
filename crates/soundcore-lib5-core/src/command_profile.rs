use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BleError;

/// Liberty 5 kontrol profili. Komutlar Bluetooth Classic RFCOMM üzerinden
/// Soundcore v1 çerçevesiyle (`ee08`/`ff09`, u16-LE komut, u16-LE uzunluk,
/// tek bayt sağlama) gönderilir. Yalnızca gerçek cihazda doğrulanmış baytlar
/// burada yer alır.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandProfile {
    pub device_name_pattern: String,
    pub control_service_uuid: Uuid,
    #[serde(default)]
    pub eq_presets: HashMap<String, String>,
    #[serde(default)]
    pub commands: HashMap<String, ProfileCommand>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCommand {
    /// Soundcore komut kodu (u16).
    pub command: u16,
    /// Seçenek adı -> payload hex. Boş string boş payload anlamına gelir.
    #[serde(default)]
    pub payloads: HashMap<String, String>,
}

impl CommandProfile {
    pub fn embedded() -> Result<Self, BleError> {
        let json = include_str!("../profiles/liberty5.json");
        let profile: CommandProfile = serde_json::from_str(json).map_err(|error| BleError::Profile(format!("profil JSON ayrıştırılamadı: {error}")))?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn validate(&self) -> Result<(), BleError> {
        if self.control_service_uuid.is_nil() {
            return Err(BleError::Profile("controlServiceUuid eksik".to_string()));
        }
        for (kind, command) in &self.commands {
            if command.command == 0 {
                return Err(BleError::Profile(format!("{kind} komut kodu 0 olamaz")));
            }
            for (option, payload) in &command.payloads {
                if !payload.is_empty() && hex::decode(payload).is_err() {
                    return Err(BleError::Profile(format!("{kind}/{option} geçersiz hex: {payload}")));
                }
            }
        }
        Ok(())
    }

    pub fn command(&self, kind: &str) -> Option<&ProfileCommand> { self.commands.get(kind) }

    pub fn payload(&self, kind: &str, option: &str) -> Result<Vec<u8>, BleError> {
        let command = self.commands.get(kind).ok_or_else(|| BleError::Profile(format!("{kind} komutu profilde yok")))?;
        let payload = command.payloads.get(option).ok_or_else(|| BleError::Profile(format!("{kind}/{option} profilde yok")))?;
        if payload.is_empty() { return Ok(Vec::new()); }
        hex::decode(payload).map_err(|error| BleError::Profile(format!("{kind}/{option} hex çözülemedi: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_profile_parses_verified_rfcomm_commands() {
        let profile = CommandProfile::embedded().expect("gömülü profil geçerli olmalı");
        assert_eq!(profile.device_name_pattern, "soundcore Liberty 5");
        assert_eq!(
            profile.control_service_uuid.to_string(),
            "0cf12d31-fac3-4553-bd80-d6832e7b3957"
        );
        let anc = profile.command("anc").expect("anc komutu olmalı");
        assert_eq!(anc.command, 0x8106);
        assert_eq!(anc.payloads.get("On").map(String::as_str), Some("001000010001"));
        assert_eq!(anc.payloads.get("Transparency").map(String::as_str), Some("011000010001"));
        assert_eq!(anc.payloads.get("Off").map(String::as_str), Some("021000010001"));
    }

    #[test]
    fn missing_commands_are_not_supported() {
        let profile = CommandProfile::embedded().expect("gömülü profil geçerli olmalı");
        // Henüz doğrulanmamış özellikler profilde bulunmaz.
        assert!(profile.command("eqPreset").is_none());
    }

    #[test]
    fn verified_game_mode_and_battery_payloads() {
        let profile = CommandProfile::embedded().expect("gömülü profil geçerli olmalı");
        let game = profile.command("gameMode").expect("gameMode komutu olmalı");
        assert_eq!(game.command, 0x8510);
        assert_eq!(game.payloads.get("On").map(String::as_str), Some("01"));
        assert_eq!(game.payloads.get("Off").map(String::as_str), Some("00"));
        let battery = profile.command("battery").expect("battery komutu olmalı");
        assert_eq!(battery.command, 0x0301);
        assert_eq!(battery.payloads.get("request").map(String::as_str), Some(""));
    }

    #[test]
    fn invalid_hex_is_a_profile_error() {
        let mut profile = CommandProfile::embedded().expect("gömülü profil geçerli olmalı");
        profile.commands.insert(
            "anc".to_string(),
            ProfileCommand { command: 0x8106, payloads: HashMap::from([("On".to_string(), "zz".to_string())]) },
        );
        assert!(matches!(profile.payload("anc", "On"), Err(BleError::Profile(_))));
    }
}
