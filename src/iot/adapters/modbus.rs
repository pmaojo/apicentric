use crate::iot::config::AdapterConfig;
use crate::iot::model::VariableValue;
use crate::iot::traits::ProtocolAdapter;
use async_trait::async_trait;
use log::info; // removed warn
               // Removed tokio-modbus imports to avoid unused warnings since we are using raw TCP for MVP
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct ModbusStore {
    holding_registers: HashMap<u16, u16>,
}

impl ModbusStore {
    fn new() -> Self {
        Self {
            holding_registers: HashMap::new(),
        }
    }
}

pub struct ModbusAdapter {
    store: Arc<Mutex<ModbusStore>>,
}

impl ModbusAdapter {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(ModbusStore::new())),
        }
    }
}

#[async_trait]
impl ProtocolAdapter for ModbusAdapter {
    async fn init(&mut self, config: &AdapterConfig) -> anyhow::Result<()> {
        let port = config
            .params
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(5020) as u16;
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let _store = self.store.clone();

        tokio::spawn(async move {
            match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    info!("Modbus TCP Server listening on {}", addr);
                    loop {
                        if let Ok((mut stream, peer)) = listener.accept().await {
                            info!("Modbus connection from {}", peer);
                            tokio::spawn(async move {
                                let mut buf = [0u8; 1024];
                                loop {
                                    match stream.read(&mut buf).await {
                                        Ok(0) => break, // Connection closed
                                        Ok(_n) => {
                                            // Echo back a dummy exception response for any request
                                            // to verify connectivity without full protocol logic.
                                            // Modbus Error: [TransactionID][ProtocolID][Length][UnitID][ErrorFuncCode][ExceptionCode]
                                            // Minimal implementation to keep connection alive or respond.
                                            let response = [
                                                0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x81,
                                                0x01,
                                            ];
                                            if let Err(_) = stream.write_all(&response).await {
                                                break;
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to bind Modbus TCP listener: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn publish(&self, key: &str, value: &VariableValue) -> anyhow::Result<()> {
        // Map key to register address. For MVP assume key is "addr_100"
        if let Some(addr_str) = key.strip_prefix("addr_") {
            if let Ok(addr) = addr_str.parse::<u16>() {
                let val_u16 = match value {
                    VariableValue::Integer(i) => *i as u16,
                    VariableValue::Float(f) => *f as u16,
                    _ => 0,
                };
                let mut store = self.store.lock().unwrap();
                store.holding_registers.insert(addr, val_u16);
            }
        }
        Ok(())
    }
}
