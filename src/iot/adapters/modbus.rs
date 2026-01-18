use crate::iot::config::AdapterConfig;
use crate::iot::model::VariableValue;
use crate::iot::traits::ProtocolAdapter;
use async_trait::async_trait;
use log::{error, info};
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

impl Default for ModbusAdapter {
    fn default() -> Self {
        Self::new()
    }
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

        let store = self.store.clone();

        tokio::spawn(async move {
            match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    info!("Modbus TCP Server listening on {}", addr);
                    loop {
                        if let Ok((socket, peer)) = listener.accept().await {
                            info!("Modbus connection from {}", peer);
                            let store_clone = store.clone();
                            tokio::spawn(handle_connection(socket, store_clone));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to bind Modbus TCP listener: {}", e);
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
                    VariableValue::Boolean(b) => {
                        if *b {
                            1
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
                let mut store = self.store.lock().unwrap();
                store.holding_registers.insert(addr, val_u16);
            }
        }
        Ok(())
    }

    async fn subscribe(&mut self, _topic: &str) -> anyhow::Result<()> {
        // Not implemented for Modbus
        Ok(())
    }

    async fn poll(&mut self) -> Option<(String, VariableValue)> {
        // Not implemented for Modbus
        None
    }
}

async fn handle_connection(mut stream: tokio::net::TcpStream, store: Arc<Mutex<ModbusStore>>) {
    let mut buffer = Vec::with_capacity(1024);
    let mut temp_buf = [0u8; 1024];

    loop {
        // 1. Try to process existing buffer
        loop {
            if buffer.len() < 7 {
                break; // Need more data for header
            }

            // Parse MBAP Header
            // Transaction ID (0-1), Protocol ID (2-3), Length (4-5), Unit ID (6)
            let proto_id = u16::from_be_bytes([buffer[2], buffer[3]]);
            let length_field = u16::from_be_bytes([buffer[4], buffer[5]]) as usize;

            // Total frame size = 6 (header excluding unit_id which is counted in length) + length
            // MBAP Header is 7 bytes: Trans(2) + Proto(2) + Len(2) + Unit(1)
            // The 'Length' field counts bytes following the Length field itself (UnitID + PDU)
            // So total bytes required = 6 + length_field
            let total_required = 6 + length_field;

            if buffer.len() < total_required {
                break; // Need more data for full frame
            }

            // We have a full frame
            let frame = &buffer[0..total_required];
            let unit_id = frame[6];
            let func_code = frame[7];

            if proto_id != 0 {
                // Not Modbus TCP, drop frame or close?
                // We'll just drop this frame and continue
                buffer.drain(0..total_required);
                continue;
            }

            let mut response = Vec::with_capacity(32);
            // Copy Transaction ID and Protocol ID
            response.extend_from_slice(&frame[0..4]);
            // Placeholder for Length (2 bytes)
            response.push(0);
            response.push(0);
            // Unit ID
            response.push(unit_id);

            match func_code {
                0x03 => {
                    // Read Holding Registers
                    if frame.len() >= 12 {
                        let start_addr = u16::from_be_bytes([frame[8], frame[9]]);
                        let count = u16::from_be_bytes([frame[10], frame[11]]);

                        let mut byte_count = 0;
                        let mut data = Vec::new();

                        {
                            let store = store.lock().unwrap();
                            for i in 0..count {
                                let addr = start_addr.wrapping_add(i);
                                let val = store.holding_registers.get(&addr).copied().unwrap_or(0);
                                data.extend_from_slice(&val.to_be_bytes());
                                byte_count += 2;
                            }
                        }

                        response.push(0x03);
                        response.push(byte_count);
                        response.extend_from_slice(&data);
                    } else {
                        // Should not happen if length check passed, but just in case
                        response.push(0x03 | 0x80);
                        response.push(0x03);
                    }
                }
                0x06 => {
                    // Write Single Register
                    if frame.len() >= 12 {
                        let addr = u16::from_be_bytes([frame[8], frame[9]]);
                        let val = u16::from_be_bytes([frame[10], frame[11]]);

                        {
                            let mut store = store.lock().unwrap();
                            store.holding_registers.insert(addr, val);
                        }

                        // Echo request
                        response.push(0x06);
                        response.extend_from_slice(&frame[8..12]);
                    }
                }
                _ => {
                    // Exception: Illegal Function
                    response.push(func_code | 0x80);
                    response.push(0x01);
                }
            }

            // Fill Length
            let payload_len = (response.len() - 6) as u16;
            let len_bytes = payload_len.to_be_bytes();
            response[4] = len_bytes[0];
            response[5] = len_bytes[1];

            if (stream.write_all(&response).await).is_err() {
                return; // Connection error
            }

            // Remove processed frame
            buffer.drain(0..total_required);
        }

        // 2. Read more data
        match stream.read(&mut temp_buf).await {
            Ok(0) => return, // Closed
            Ok(n) => {
                buffer.extend_from_slice(&temp_buf[0..n]);
            }
            Err(_) => return,
        }
    }
}
