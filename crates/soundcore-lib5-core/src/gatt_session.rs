use btleplug::api::{CharPropFlags, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::error::BleError;

#[derive(Clone, Debug)]
pub struct CharacteristicSnapshot {
    pub service_uuid: Uuid,
    pub characteristic_uuid: Uuid,
    pub properties: Vec<&'static str>,
    pub value_hex: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct GattSession {
    pub(crate) peripheral: Peripheral,
}

impl GattSession {
    pub async fn open(peripheral: Peripheral) -> Result<Self, BleError> {
        if !peripheral.is_connected().await.unwrap_or(false) {
            peripheral.connect().await.map_err(|error| BleError::Connection(error.to_string()))?;
        }
        peripheral.discover_services().await?;
        Ok(Self { peripheral })
    }

    fn characteristic(&self, service_uuid: Uuid, characteristic_uuid: Uuid) -> Result<btleplug::api::Characteristic, BleError> {
        self.peripheral
            .characteristics()
            .into_iter()
            .find(|characteristic| characteristic.service_uuid == service_uuid && characteristic.uuid == characteristic_uuid)
            .ok_or(BleError::NotFound)
    }

    fn property_names(properties: CharPropFlags) -> Vec<&'static str> {
        let mut names = Vec::new();
        if properties.contains(CharPropFlags::READ) { names.push("read"); }
        if properties.contains(CharPropFlags::WRITE) { names.push("write"); }
        if properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) { names.push("writeWithoutResponse"); }
        if properties.contains(CharPropFlags::NOTIFY) { names.push("notify"); }
        if properties.contains(CharPropFlags::INDICATE) { names.push("indicate"); }
        names
    }

    pub async fn enumerate(&self) -> Result<Vec<CharacteristicSnapshot>, BleError> {
        let mut snapshots = Vec::new();
        for characteristic in self.peripheral.characteristics() {
            let properties = Self::property_names(characteristic.properties);
            let (value_hex, error) = if characteristic.properties.contains(CharPropFlags::READ) {
                match self.peripheral.read(&characteristic).await {
                    Ok(value) => (Some(hex::encode(value)), None),
                    Err(error) => (None, Some(error.to_string())),
                }
            } else {
                (None, None)
            };
            snapshots.push(CharacteristicSnapshot {
                service_uuid: characteristic.service_uuid,
                characteristic_uuid: characteristic.uuid,
                properties,
                value_hex,
                error,
            });
        }
        Ok(snapshots)
    }

    pub async fn read(&self, service_uuid: Uuid, characteristic_uuid: Uuid) -> Result<Vec<u8>, BleError> {
        let characteristic = self.characteristic(service_uuid, characteristic_uuid)?;
        Ok(self.peripheral.read(&characteristic).await?)
    }

    pub async fn write(&self, service_uuid: Uuid, characteristic_uuid: Uuid, payload: &[u8], with_response: bool) -> Result<(), BleError> {
        let characteristic = self.characteristic(service_uuid, characteristic_uuid)?;
        let write_type = if with_response { WriteType::WithResponse } else { WriteType::WithoutResponse };
        Ok(self.peripheral.write(&characteristic, payload, write_type).await?)
    }

    pub async fn subscribe<F>(&self, service_uuid: Uuid, characteristic_uuid: Uuid, on_value: F) -> Result<(), BleError>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let characteristic = self.characteristic(service_uuid, characteristic_uuid)?;
        if !characteristic.properties.intersects(CharPropFlags::NOTIFY | CharPropFlags::INDICATE) {
            return Err(BleError::NotSupported);
        }
        self.peripheral.subscribe(&characteristic).await?;
        let mut notifications = self.peripheral.notifications().await?;
        tokio::spawn(async move {
            while let Some(notification) = notifications.next().await {
                if notification.uuid == characteristic_uuid {
                    on_value(notification.value);
                }
            }
        });
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), BleError> {
        if self.peripheral.is_connected().await.unwrap_or(false) {
            self.peripheral.disconnect().await?;
        }
        Ok(())
    }
}
