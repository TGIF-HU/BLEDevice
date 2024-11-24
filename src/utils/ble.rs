use super::queue::FixedQueue;
use crate::config::DEVICE_ID;
use chrono::{DateTime, Utc};
use esp32_nimble::BLEAdvertisedDevice;
use serde::Serialize;
use serde_json::json;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize)]
pub struct BLEInfo {
    address: String,
    rssi: i32,
    manufacture_id: Option<Vec<u8>>,
    name: String,
    time: DateTime<Utc>,
}

impl BLEInfo {
    pub fn new(
        address: String,
        rssi: i32,
        manufacture_id: Option<Vec<u8>>,
        name: String,
        time: DateTime<Utc>,
    ) -> Self {
        Self {
            address,
            rssi,
            manufacture_id,
            name,
            time,
        }
    }

    #[allow(dead_code)]
    pub fn get_json(&self) -> String {
        json!(
            {
                "device_id": DEVICE_ID,
                "address": self.address,
                "rssi": self.rssi,
                "manufacture_id": self.manufacture_id,
                "name": self.name,
                "time": self.time
            }
        )
        .to_string()
    }
}

#[derive(Debug)]
pub struct BLEInfoQueue {
    ble: FixedQueue<BLEInfo>,
}

impl BLEInfoQueue {
    pub fn new(max_len: usize) -> Self {
        BLEInfoQueue {
            ble: FixedQueue::new(max_len),
        }
    }

    pub fn push(&mut self, item: BLEInfo) {
        self.ble.push(item);
    }

    pub fn get_json(&self) -> String {
        json!(
            {
                "device_id": DEVICE_ID,
                "ble": self.ble.get_queue()
            }
        )
        .to_string()
    }
}

pub fn get_bleinfo(param: &BLEAdvertisedDevice) -> BLEInfo {
    let address = param.addr().to_string();
    let rssi = param.rssi();
    let manufacture_id = param.get_manufacture_data().map(|data| data.to_vec());
    let name = param.name().to_owned().to_string();
    let time: DateTime<Utc> = SystemTime::now().into();

    BLEInfo::new(address, rssi, manufacture_id, name, time)
}
