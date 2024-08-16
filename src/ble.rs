use crate::queue::FixedQueue;
use chrono::{DateTime, Utc};
use esp32_nimble::BLEDevice;
use esp_idf_hal::task::block_on;
use log::*;
use serde::Serialize;
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

#[derive(Clone, Debug, Serialize)]
struct BleDeviceInfo {
    address: String,
    rssi: i32,
    time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct BleInfoJson {
    id: usize,
    bleinfo: FixedQueue<BleDeviceInfo>,
}

impl BleInfoJson {
    pub fn new(id: usize, max_len: usize) -> Self {
        BleInfoJson {
            id,
            bleinfo: FixedQueue::new(max_len),
        }
    }

    fn push(&mut self, item: BleDeviceInfo) {
        self.bleinfo.push(item);
    }

    pub fn get_json(&self) -> String {
        json!(
            {
                "id": self.id,
                "bleinfo": self.bleinfo.get_queue()
            }
        )
        .to_string()
    }
}

/// BLEデバイスのスキャンとデータの更新を行う関数
pub fn scan_and_update_ble_info(ble_info: Arc<Mutex<BleInfoJson>>) {
    block_on(async {
        let ble_device = BLEDevice::take();
        let ble_scan = ble_device.get_scan();

        ble_scan
            .active_scan(true)
            .interval(100) // 測定間隔
            .window(50) // 測定時間
            .on_result(move |_scan, param| {
                let address = param.addr().to_string();
                let rssi = param.rssi();
                let time: DateTime<Utc> = SystemTime::now().into();

                // ログにデバイス情報を出力
                /* info!(
                    "Advertised Device: {:?}, RSSI: {}, Time: {}",
                    address, rssi, time
                ); */
                let mut ble_info_lock = ble_info.lock().unwrap();

                // クロージャ内でデバイス情報を追加
                ble_info_lock.push(BleDeviceInfo {
                    address,
                    rssi,
                    time,
                });
                // info!("BLE Device Info: {:?}", ble_info_lock);
            });

        ble_scan.start(10000).await.unwrap();

        info!("Scan end");
    });
}
