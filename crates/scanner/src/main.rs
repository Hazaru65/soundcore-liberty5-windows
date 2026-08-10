use std::io::{self, Write};
use std::time::Duration;

use clap::{Parser, Subcommand};
use soundcore_lib5_core::{find_liberty_devices, find_liberty_peripherals, CommandProfile, Liberty5Device};

#[derive(Debug, Parser)]
#[command(name = "scanner", about = "Soundcore Liberty 5 keşif ve kontrol aracı")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Bluetooth üzerinden Liberty cihazlarını listeler.
    List,
    /// RFCOMM kontrol kanalına bağlanır; cihaz bilgisi ve durum çerçevelerini basar.
    Dump { device: String },
    /// RFCOMM kontrol kanalını dinler; gelen çerçeveleri basar.
    Monitor {
        device: String,
        #[arg(default_value_t = 10)]
        seconds: u64,
    },
    /// Doğrulanmış profildeki bir komutu gönderir. `--force` zorunludur.
    Write {
        device: String,
        command_hex: String,
        payload_hex: String,
        #[arg(long)]
        force: bool,
    },
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_u16_hex(value: &str) -> Result<u16, String> {
    let bytes = hex::decode(value).map_err(|error| format!("geçersiz hex: {error}"))?;
    if bytes.len() != 2 { return Err(format!("komut kodu 2 bayt olmalı, {len} verildi", len = bytes.len())); }
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

async fn select_device(query: &str) -> Result<soundcore_lib5_core::DiscoveredDevice, i32> {
    let query = query.to_ascii_lowercase();
    let matches: Vec<_> = match find_liberty_peripherals().await {
        Ok(devices) => devices.into_iter().filter(|device| device.info.device_id.to_ascii_lowercase() == query || device.info.name.to_ascii_lowercase().contains(&query) || device.info.address.to_ascii_lowercase() == query).collect(),
        Err(error) => { eprintln!("Tarama başarısız: {error}"); return Err(1); }
    };
    match matches.len() {
        1 => Ok(matches.into_iter().next().expect("bir eşleşme")),
        0 => { eprintln!("'{query}' için Liberty cihazı bulunamadı"); Err(1) }
        _ => { eprintln!("'{query}' birden çok cihazla eşleşti, daha net bir ad verin"); Err(1) }
    }
}

#[tokio::main]
async fn main() {
    let exit_code = match run(Cli::parse()).await {
        Ok(()) => 0,
        Err(code) => code,
    };
    std::process::exit(exit_code);
}

async fn run(cli: Cli) -> Result<(), i32> {
    let profile = CommandProfile::embedded().map_err(|error| { eprintln!("Profil hatası: {error}"); 1 })?;
    match cli.command {
        Command::List => {
            match find_liberty_devices().await {
                Ok(devices) => {
                    if devices.is_empty() {
                        println!("Liberty cihazı bulunamadı; kulaklıklar eşleştirilmişse ikisini de kutudan çıkarıp bekleyin");
                        return Err(1);
                    }
                    for device in devices {
                        println!("ad: {}\nid: {}\nadres: {}\nbağlı: {}\n", device.name, device.device_id, device.address, device.connected);
                    }
                    Ok(())
                }
                Err(error) => { eprintln!("Tarama başarısız: {error}"); Err(1) }
            }
        }
        Command::Dump { device } => {
            select_device(&device).await?;
            println!("RFCOMM kontrol kanalına bağlanılıyor ({}), cihaz bilgisi isteniyor...", profile.control_service_uuid);
            let mut session = Liberty5Device::open(std::sync::Arc::new(profile)).await.map_err(|error| { eprintln!("Bağlantı başarısız: {error}"); 1 })?;
            match session.read_device_info().await {
                Ok(info) => {
                    println!("seri: {}", info.serial.unwrap_or_else(|| "-".to_string()));
                    println!("firmware: {}", info.firmware.unwrap_or_else(|| "-".to_string()));
                    println!("anc: {}", info.anc_mode.unwrap_or_else(|| "bilinmiyor".to_string()));
                    Ok(())
                }
                Err(error) => { eprintln!("Cihaz bilgisi okunamadı: {error}"); Err(1) }
            }
        }
        Command::Monitor { device, seconds } => {
            select_device(&device).await?;
            println!("RFCOMM dinleniyor ({seconds} saniye)...");
            let mut session = Liberty5Device::open(std::sync::Arc::new(profile)).await.map_err(|error| { eprintln!("Bağlantı başarısız: {error}"); 1 })?;
            match session.monitor(Duration::from_secs(seconds)).await {
                Ok(frames) => {
                    if frames.is_empty() { println!("çerçeve alınamadı"); }
                    for frame in frames {
                        println!("komut=0x{:04x} payload={}", frame.command, hex(&frame.payload));
                    }
                    Ok(())
                }
                Err(error) => { eprintln!("Dinleme başarısız: {error}"); Err(1) }
            }
        }
        Command::Write { device, command_hex, payload_hex, force } => {
            if !force {
                eprintln!("Hata: --force zorunludur. Doğrulanmamış yazımlar cihaza zarar verebilir.");
                return Err(1);
            }
            select_device(&device).await?;
            let command = parse_u16_hex(&command_hex).map_err(|error| { eprintln!("{error}"); 1 })?;
            let payload = hex::decode(&payload_hex).map_err(|error| { eprintln!("payload geçersiz hex: {error}"); 1 })?;
            print!("komut=0x{command:04x} payload={} gönderilecek. Devam? (e/H) ", hex(&payload));
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            if !input.trim().eq_ignore_ascii_case("e") { println!("iptal edildi"); return Ok(()); }
            let mut session = Liberty5Device::open(std::sync::Arc::new(profile)).await.map_err(|error| { eprintln!("Bağlantı başarısız: {error}"); 1 })?;
            match session.send_raw(command, &payload).await {
                Ok(frames) => {
                    if frames.is_empty() { println!("yanıt alınamadı"); }
                    for frame in frames {
                        println!("komut=0x{:04x} payload={}", frame.command, hex(&frame.payload));
                    }
                    Ok(())
                }
                Err(error) => { eprintln!("Yazım başarısız: {error}"); Err(1) }
            }
        }
    }
}

#[allow(dead_code)]
fn unused(_: &soundcore_lib5_core::LibertyDeviceInfo) {}
