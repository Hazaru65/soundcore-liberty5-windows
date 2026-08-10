// Windows eşleştirilmiş-cihaz listesinden Liberty araması (teşhis aracı).
// Cihaz uykuda/kutuda olsa bile eşleştirme kaydı varsa listelenir.
// Kullanım: cargo run -p soundcore-lib5-core --example paired_list
use soundcore_lib5_core::paired_liberty_devices;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let devices = paired_liberty_devices().await;
    if devices.is_empty() {
        println!("Eşleştirilmiş Liberty cihazı bulunamadı.");
        return;
    }
    for device in &devices {
        println!("ad: {} adres: {} bağlı: {}", device.name, device.address, device.connected);
    }
}
