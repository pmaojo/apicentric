use crate::simulator::config::BluetoothConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Manages Bluetooth device emulation
pub struct BluetoothManager {
    devices: Arc<RwLock<HashMap<String, BluetoothDevice>>>,
}

struct BluetoothDevice {
    config: BluetoothConfig,
    #[allow(dead_code)] // Will be used when we add interactive features
    is_advertising: bool,
}

impl BluetoothManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register and start a simulated Bluetooth device
    pub async fn start_device(&self, service_name: &str, config: BluetoothConfig) {
        let mut devices = self.devices.write().await;

        info!(
            target: "bluetooth",
            device = %config.local_name,
            service = %service_name,
            "Starting Bluetooth Peripheral Emulation (Mock Mode)"
        );

        for service in &config.services {
            info!(
                target: "bluetooth",
                uuid = %service.uuid,
                "  + Service registered"
            );
            for char in &service.characteristics {
                 info!(
                    target: "bluetooth",
                    uuid = %char.uuid,
                    props = ?char.properties,
                    "    - Characteristic"
                );
            }
        }

        devices.insert(service_name.to_string(), BluetoothDevice {
            config,
            is_advertising: true,
        });

        // In a real implementation, this would call btleplug/bluer to start advertising
        warn!(target: "bluetooth", "Note: This is a virtual device. Real radio usage requires hardware access and system dependencies.");
    }

    /// Stop a simulated device
    pub async fn stop_device(&self, service_name: &str) {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.remove(service_name) {
             info!(
                target: "bluetooth",
                device = %device.config.local_name,
                "Stopping Bluetooth Peripheral"
            );
        }
    }
}
