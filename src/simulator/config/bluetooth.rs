use serde::{Deserialize, Serialize};

/// Configuration for a Bluetooth Low Energy (BLE) peripheral
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BluetoothConfig {
    /// The local name of the device advertised
    pub local_name: String,

    /// List of GATT services
    #[serde(default)]
    pub services: Vec<GattService>,

    /// Advertising interval in milliseconds (optional)
    pub advertising_interval: Option<u64>,
}

/// A GATT Service definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GattService {
    /// Service UUID (16-bit or 128-bit)
    pub uuid: String,

    /// Characteristics within this service
    #[serde(default)]
    pub characteristics: Vec<GattCharacteristic>,

    /// Whether this is a primary service (default: true)
    #[serde(default = "default_true")]
    pub primary: bool,
}

/// A GATT Characteristic definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GattCharacteristic {
    /// Characteristic UUID
    pub uuid: String,

    /// User-friendly name for logging/UI
    pub name: Option<String>,

    /// Properties (read, write, notify, indicate)
    #[serde(default)]
    pub properties: Vec<CharacteristicProperty>,

    /// Initial value (hex string or text)
    pub value: Option<String>,

    /// Descriptors
    #[serde(default)]
    pub descriptors: Vec<GattDescriptor>,
}

/// GATT Characteristic Properties
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CharacteristicProperty {
    Read,
    Write,
    WriteWithoutResponse,
    Notify,
    Indicate,
    Broadcast,
}

/// A GATT Descriptor
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GattDescriptor {
    pub uuid: String,
    pub value: String,
}

fn default_true() -> bool {
    true
}
