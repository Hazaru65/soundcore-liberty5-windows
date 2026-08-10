use std::time::Duration;

use tokio::time::timeout;
use uuid::Uuid;
use windows::core::GUID;
use windows::Devices::Bluetooth::Rfcomm::{RfcommDeviceService, RfcommServiceId};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Networking::Sockets::StreamSocket;
use windows::Storage::Streams::{DataReader, DataWriter, InputStreamOptions};

use crate::error::BleError;

const START_OF_PACKET_HOST: u8 = 0xee;
const START_OF_PACKET_DEVICE: u8 = 0xff;
const DIRECTION_HOST: u8 = 0x00;
const DIRECTION_DEVICE: u8 = 0x01;
/// Çerçeve başlığı: marker(2) + 0x0000(2) + yön(1) + komut(2) + uzunluk(2).
const HEADER_LENGTH: usize = 9;

/// Cihazdan gelen tek bir Soundcore v1 çerçevesi.
#[derive(Clone, Debug)]
pub struct Frame {
    pub command: u16,
    pub payload: Vec<u8>,
}

fn to_guid(uuid: &Uuid) -> GUID {
    let (a, b, c, d) = uuid.as_fields();
    GUID::from_values(a, b, c, *d)
}

/// Windows RFCOMM (Bluetooth Classic) taşıyıcısı.
pub struct RfcommSession {
    reader: DataReader,
    writer: DataWriter,
    _socket: StreamSocket,
}

impl RfcommSession {
    /// Liberty 5 kontrol servisine (profildeki `controlServiceUuid`) bağlanır.
    pub async fn open(service_uuid: &Uuid) -> Result<Self, BleError> {
        let service_id = RfcommServiceId::FromUuid(to_guid(service_uuid))?;
        let selector = RfcommDeviceService::GetDeviceSelector(&service_id)?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.await?;
        let Some(info) = devices.into_iter().next() else {
            return Err(BleError::NotFound);
        };
        let service = RfcommDeviceService::FromIdAsync(&info.Id()?)?.await?;
        let socket = StreamSocket::new()?;
        socket
            .ConnectAsync(&service.ConnectionHostName()?, &service.ConnectionServiceName()?)
            .map_err(|error| BleError::Connection(error.to_string()))?
            .await
            .map_err(|error| BleError::Connection(error.to_string()))?;
        let reader = DataReader::CreateDataReader(&socket.InputStream()?)?;
        reader.SetInputStreamOptions(InputStreamOptions::Partial)?;
        let writer = DataWriter::CreateDataWriter(&socket.OutputStream()?)?;
        Ok(Self { reader, writer, _socket: socket })
    }

    fn host_frame(command: u16, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LENGTH + payload.len() + 1);
        // Kablodaki sıra: 0x08 0xee (u16-LE 0xee08).
        bytes.push(0x08);
        bytes.push(START_OF_PACKET_HOST);
        bytes.extend_from_slice(&[0x00, 0x00, DIRECTION_HOST]);
        bytes.extend_from_slice(&command.to_le_bytes());
        bytes.extend_from_slice(&((HEADER_LENGTH + payload.len() + 1) as u16).to_le_bytes());
        bytes.extend_from_slice(payload);
        let checksum = bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        bytes.push(checksum);
        bytes
    }

    /// Tek bir Soundcore v1 çerçevesi gönderir ve yanıt çerçevelerini toplar.
    /// Cihaz her komuta aynı komut kodlu (boş payload) bir onay çerçevesiyle
    /// döner; bu onay beklendikten sonra döner.
    pub async fn command(&mut self, command: u16, payload: &[u8]) -> Result<Vec<Frame>, BleError> {
        let packet = Self::host_frame(command, payload);
        self.writer.WriteBytes(&packet)?;
        self.writer.StoreAsync()?.await?;
        self.read_until(Duration::from_secs(3), |frame| frame.command == command).await
    }

    /// Belirtilen süre boyunca çerçeve okur. `until` eşleşirse erken döner.
    pub async fn read_until<F>(&mut self, duration: Duration, until: F) -> Result<Vec<Frame>, BleError>
    where
        F: Fn(&Frame) -> bool,
    {
        let mut frames = Vec::new();
        let mut buffer: Vec<u8> = Vec::new();
        let deadline = std::time::Instant::now() + duration;
        while std::time::Instant::now() < deadline {
            let remaining = deadline - std::time::Instant::now();
            match timeout(remaining, self.reader.LoadAsync(1024)?).await {
                Ok(Ok(n)) if n > 0 => {
                    let mut chunk = vec![0u8; n as usize];
                    self.reader.ReadBytes(&mut chunk)?;
                    buffer.extend_from_slice(&chunk);
                    while let Some(frame) = Self::take_frame(&mut buffer)? {
                        frames.push(frame.clone());
                        if until(&frame) {
                            return Ok(frames);
                        }
                    }
                }
                Ok(Ok(_)) => {}
                Ok(Err(error)) => return Err(BleError::Windows(error)),
                Err(_) => break,
            }
        }
        Ok(frames)
    }

    /// Yazıcı akışını ayırır ve soketi kapatır.
    pub async fn detach(&mut self) -> Result<(), BleError> {
        let _ = self.writer.DetachStream()?;
        Ok(())
    }

    /// Tampondan eksiksiz çerçeveleri çıkarır. Bozuk sağlama çerçeveyi düşürür.
    fn take_frame(buffer: &mut Vec<u8>) -> Result<Option<Frame>, BleError> {
        loop {
            if buffer.len() < HEADER_LENGTH {
                return Ok(None);
            }
            if buffer[0] != 0x09 || buffer[1] != START_OF_PACKET_DEVICE {
                // Hizalama kaybı: bayt at.
                buffer.remove(0);
                continue;
            }
            if buffer[3] != 0x00 || buffer[4] != DIRECTION_DEVICE {
                buffer.remove(0);
                continue;
            }
            let command = u16::from_le_bytes([buffer[5], buffer[6]]);
            let length = u16::from_le_bytes([buffer[7], buffer[8]]) as usize;
            if length < HEADER_LENGTH + 1 {
                buffer.remove(0);
                continue;
            }
            if buffer.len() < length {
                return Ok(None);
            }
            let frame_bytes: Vec<u8> = buffer.drain(..length).collect();
            let checksum = frame_bytes.iter().take(length - 1).fold(0u8, |sum, byte| sum.wrapping_add(*byte));
            if checksum != frame_bytes[length - 1] {
                continue;
            }
            let payload = frame_bytes[HEADER_LENGTH..length - 1].to_vec();
            return Ok(Some(Frame { command, payload }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_frame_matches_verified_wire_bytes() {
        // Gerçek cihazda yakalanan ANC-on paketi ile birebir.
        let frame = RfcommSession::host_frame(0x8106, &[0x00, 0x10, 0x00, 0x01, 0x00, 0x01]);
        assert_eq!(frame, vec![0x08, 0xee, 0x00, 0x00, 0x00, 0x06, 0x81, 0x10, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00, 0x01, 0x9f]);

        let info = RfcommSession::host_frame(0x0101, &[]);
        assert_eq!(info, vec![0x08, 0xee, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0a, 0x00, 0x02]);
    }

    #[test]
    fn device_frame_parsing_matches_verified_capture() {
        // Gerçek cihaz yanıtı: ACK (0x8106, boş payload).
        let mut buffer = vec![0x09, 0xff, 0x00, 0x00, 0x01, 0x06, 0x81, 0x0a, 0x00, 0x9a];
        let frame = RfcommSession::take_frame(&mut buffer).expect("çerçeve çözülmeli").expect("bir çerçeve olmalı");
        assert_eq!(frame.command, 0x8106);
        assert!(frame.payload.is_empty());
        assert!(buffer.is_empty());

        // 164 baytlık device-info yanıtı (gerçek yakalama).
        let mut full = vec![0x09, 0xff, 0x00, 0x00, 0x01, 0x01, 0x01, 0xa4, 0x00];
        full.extend_from_slice(&[0x01, 0x00, 0xff, 0x08, 0x00, 0x00]);
        full.extend_from_slice(b"04.9004.90395790BFD95CE5DC0");
        while full.len() < 163 {
            full.push(0x00);
        }
        let checksum = full.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        full.push(checksum);
        assert_eq!(full.len(), 164);
        let frame = RfcommSession::take_frame(&mut full).expect("çerçeve çözülmeli").expect("bir çerçeve olmalı");
        assert_eq!(frame.command, 0x0101);
        assert_eq!(frame.payload.len(), 154);
        assert_eq!(&frame.payload[6..11], b"04.90");
        assert_eq!(&frame.payload[11..16], b"04.90");
        assert_eq!(&frame.payload[16..32], b"395790BFD95CE5DC");
    }
}
