use thiserror::Error;

#[derive(Debug, Error)]
pub enum BleError {
    #[error("Bluetooth işlemi başarısız: {0}")]
    Bluetooth(#[from] btleplug::Error),
    #[error("Liberty 5 cihazı bulunamadı")]
    NotFound,
    #[error("Bu özellik Liberty 5 profilinde doğrulanmadı")]
    NotSupported,
    #[error("Profil hatası: {0}")]
    Profile(String),
    #[error("Geçersiz ANC modu: {0}")]
    InvalidAncMode(String),
    #[error("Bluetooth bağlantısı reddedildi. Telefondaki Soundcore uygulamasını kapatın; Windows'ta cihazı kaldırıp yeniden eşleştirin. Ayrıntı: {0}")]
    Connection(String),
    #[error("Windows Bluetooth API hatası: {0}")]
    Windows(#[from] windows::core::Error),
}
