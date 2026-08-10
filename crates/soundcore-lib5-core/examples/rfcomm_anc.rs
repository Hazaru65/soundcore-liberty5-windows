// Gerçek cihazda protokol doğrulama aracı (geliştirme amaçlı).
// Modlar: anc | trans | off | status | sequence | game-on | game-off |
//         eq01 | eq02 | eq03 | eq00 | preset | battery | statdiff
//
// game-*: telefon uygulamasından yakalanan 0x8510 komutu (Game Mode adayı)
// eqXX:  telefon uygulamasından yakalanan 0x8703 EQ band yükleri (preset 01/02/03/00)
// preset: 0x8110 preset seçim adayı yazımları
// battery: 0x0301 istek + 10 sn bildirim dinleme
// statdiff: 0x9403 durum blobu, toggle, tekrar blob (bit farkı tespiti)
use std::sync::Arc;
use std::time::Duration;

use soundcore_lib5_core::{AncMode, CommandProfile, Liberty5Device};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "status".to_string());
    let profile = Arc::new(CommandProfile::embedded()?);

    match mode.as_str() {
        "sequence" => {
            println!(">> sequence: baglan -> device-info -> ANC-off -> ANC-on");
            let mut device = Liberty5Device::open(profile).await?;
            let info = device.read_device_info().await?;
            println!("seri={:?} firmware={:?} anc={:?}", info.serial, info.firmware, info.anc_mode);
            device.set_anc(AncMode::Off).await?;
            println!("ANC-off OK");
            device.set_anc(AncMode::On).await?;
            println!("ANC-on OK");
            device.set_anc(AncMode::Transparency).await?;
            println!("ANC-transparency OK");
            device.disconnect().await?;
            println!("sequence basarili");
        }
        "anc" => set_mode(profile, AncMode::On).await?,
        "trans" => set_mode(profile, AncMode::Transparency).await?,
        "off" => set_mode(profile, AncMode::Off).await?,
        "game-on" | "game-off" => {
            let mut device = Liberty5Device::open(profile).await?;
            let payload = if mode == "game-on" { vec![0x01] } else { vec![0x00] };
            let frames = device.send_raw(0x8510, &payload).await?;
            report(&mode, &frames);
            device.disconnect().await?;
        }
        "eq01" | "eq02" | "eq03" | "eq00" => {
            let id = match mode.as_str() { "eq01" => 1, "eq02" => 2, "eq03" => 3, _ => 0 };
            let payload = eq_payload(id);
            let mut device = Liberty5Device::open(profile).await?;
            let frames = device.send_raw(0x8703, &payload).await?;
            report(&mode, &frames);
            device.disconnect().await?;
        }
        "preset" => {
            let mut device = Liberty5Device::open(profile).await?;
            for (label, payload) in [
                ("preset-on-1", vec![0x01, 0x01, 0x01]),
                ("preset-on-2", vec![0x01, 0x01, 0x02]),
                ("preset-off", vec![0x00, 0x01, 0x00]),
                ("preset-on-0", vec![0x01, 0x01, 0x00]),
            ] {
                let frames = device.send_raw(0x8110, &payload).await?;
                report(label, &frames);
            }
            device.disconnect().await?;
        }
        "battery" => {
            let mut device = Liberty5Device::open(profile).await?;
            let frames = device.send_raw(0x0301, &[]).await?;
            report("0x0301 istek", &frames);
            println!(">> 10 sn bildirim dinleniyor...");
            let notifs = device.monitor(Duration::from_secs(10)).await?;
            for f in &notifs {
                println!("notif cmd=0x{:04x} len={} payload={}", f.command, f.payload.len(), hex(f));
            }
            device.disconnect().await?;
        }
        "features" => {
            // Uygulamanin kullandigi kutuphane yollari: read_battery + set_game_mode
            let mut device = Liberty5Device::open(profile).await?;
            let battery = device.read_battery().await?;
            println!("pil: sol={:?} sag={:?} kutu={:?}", battery.left, battery.right, battery.case);
            device.set_game_mode(true).await?;
            println!("game-mode ON OK (ses boguklasmali)");
            std::thread::sleep(std::time::Duration::from_secs(3));
            device.set_game_mode(false).await?;
            println!("game-mode OFF OK (ses netlesmeli)");
            let battery2 = device.read_battery().await?;
            println!("pil (tekrar): sol={:?} sag={:?} kutu={:?}", battery2.left, battery2.right, battery2.case);
            device.disconnect().await?;
            println!("features basarili");
        }
        "statdiff" => {            let mut device = Liberty5Device::open(profile).await?;
            let before = device.send_raw(0x9403, &[]).await?;
            println!("0x9403 once:");
            for f in &before { println!("  cmd=0x{:04x} payload={}", f.command, hex(f)); }
            let game = device.send_raw(0x8510, &[0x01]).await?;
            report("0x8510 [01]", &game);
            let after = device.send_raw(0x9403, &[]).await?;
            println!("0x9403 sonra:");
            for f in &after { println!("  cmd=0x{:04x} payload={}", f.command, hex(f)); }
            // fark karsilastirmasi
            let a = before.iter().find(|f| f.command == 0x9403).map(|f| f.payload.clone());
            let b = after.iter().find(|f| f.command == 0x9403).map(|f| f.payload.clone());
            if let (Some(a), Some(b)) = (a, b) {
                if a.len() == b.len() {
                    let diffs: Vec<String> = a.iter().zip(&b).enumerate()
                        .filter(|(_, (x, y))| x != y)
                        .map(|(i, (x, y))| format!("{i}:{x:02x}->{y:02x}"))
                        .collect();
                    println!("farkli baytlar: {}", if diffs.is_empty() { "YOK".into() } else { diffs.join(" ") });
                }
            }
            device.disconnect().await?;
        }
        _ => {
            println!(">> status: device-info");
            let mut device = Liberty5Device::open(profile).await?;
            let info = device.read_device_info().await?;
            println!("seri={:?} firmware={:?} anc={:?}", info.serial, info.firmware, info.anc_mode);
            device.disconnect().await?;
        }
    }
    Ok(())
}

fn report(label: &str, frames: &[soundcore_lib5_core::Frame]) {    let ack = frames.iter().any(|f| f.payload.is_empty());
    println!("{label}: cerceve={} ack={}", frames.len(), if ack { "EVET" } else { "HAYIR" });
    for f in frames {
        if !f.payload.is_empty() {
            println!("  cmd=0x{:04x} payload={}", f.command, hex(f));
        }
    }
}

fn hex(f: &soundcore_lib5_core::Frame) -> String {
    f.payload.iter().map(|b| format!("{b:02x}")).collect()
}

/// Telefon uygulamasindan yakalanan 0x8703 EQ band yukleri (birebir kopya).
fn eq_payload(id: u8) -> Vec<u8> {
    let preset = match id {
        1 => "01000000a0828c8ca0a0a08c7800a0828c8ca0a0a08c7800ffff00ffffffffffffffffff00ffffffffffffffffff000000000000ffffffffffffffffff00ffffffffffffffffff00a00082008c008c00a000a000a0008c0078000000a00082008c008c00a000a000a0008c00780000000000",
        2 => "02000000a0968278787878787800a0968278787878787800ffff00ffffffffffffffffff00ffffffffffffffffff000000000000ffffffffffffffffff00ffffffffffffffffff00a000960082007800780078007800780078000000a0009600820078007800780078007800780000000000",
        3 => "03000000505a6e78787878787800505a6e78787878787800ffff00ffffffffffffffffff00ffffffffffffffffff000000000000ffffffffffffffffff00ffffffffffffffffff0050005a006e00780078007800780078007800000050005a006e0078007800780078007800780000000000",
        _ => "000000007878787878787878780078787878787878787800ffff00ffffffffffffffffff00ffffffffffffffffff000000000000ffffffffffffffffff00ffffffffffffffffff00780078007800780078007800780078007800000078007800780078007800780078007800780000000000",
    };
    let bytes: Vec<u8> = (0..preset.len() / 2)
        .map(|i| u8::from_str_radix(&preset[i * 2..i * 2 + 2], 16).unwrap())
        .collect();
    assert_eq!(bytes.len(), 114, "EQ payload 114 bayt olmali");
    bytes
}

async fn set_mode(profile: Arc<CommandProfile>, mode: AncMode) -> Result<(), Box<dyn std::error::Error>> {
    let mut device = Liberty5Device::open(profile).await?;
    device.set_anc(mode).await?;
    println!("ANC {} OK", mode.as_str());
    device.disconnect().await?;
    Ok(())
}
