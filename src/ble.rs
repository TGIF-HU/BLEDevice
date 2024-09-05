use crate::{
    config::{DEVICE_ID, URL},
    queue::FixedQueue,
};

use chrono::{DateTime, Utc};
use embedded_svc::http::client::Client;
use esp32_nimble::{BLEAdvertisedDevice, BLEDevice};
use esp_idf_hal::task::block_on;
use esp_idf_svc::http::client::Configuration as HTTPConfig;
use esp_idf_svc::http::client::EspHttpConnection;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

#[derive(Clone, Debug, Serialize)]
struct BleInfo {
    address: String,
    rssi: i32,
    manufacture_id: Option<Vec<u8>>,
    name: String,
    time: DateTime<Utc>,
}

impl BleInfo {
    pub fn new(
        address: String,
        rssi: i32,
        manufacture_id: Option<Vec<u8>>,
        name: String,
        time: DateTime<Utc>,
    ) -> Self {
        BleInfo {
            address,
            rssi,
            manufacture_id,
            name,
            time,
        }
    }

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
pub struct BleInfoQueue {
    ble: FixedQueue<BleInfo>,
}

impl BleInfoQueue {
    pub fn new(max_len: usize) -> Self {
        BleInfoQueue {
            ble: FixedQueue::new(max_len),
        }
    }

    fn push(&mut self, item: BleInfo) {
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

fn get_bleinfo(param: &BLEAdvertisedDevice) -> BleInfo {
    let address = param.addr().to_string();
    let rssi = param.rssi();
    let manufacture_id = param.get_manufacture_data().map(|data| data.to_vec());
    let name = param.name().to_owned().to_string();
    let time: DateTime<Utc> = SystemTime::now().into();

    BleInfo::new(address, rssi, manufacture_id, name, time)
}

/// BLEデバイスのスキャンとデータの更新を行う関数
pub fn scan_and_update_ble_info(ble_info: Arc<Mutex<BleInfoQueue>>) {
    block_on(async {
        let ble_device = BLEDevice::take();
        let ble_scan = ble_device.get_scan();

        ble_scan
            .active_scan(true)
            .interval(100) // 測定間隔
            .window(50) // 測定時間
            .on_result(move |_scan, param| {
                let mut ble_info_lock = ble_info.lock().unwrap();

                // クロージャ内でデバイス情報を追加
                ble_info_lock.push(get_bleinfo(param));
                // info!("BLE Device Info: {:?}", ble_info_lock);
            });

        ble_scan.start(10000).await.unwrap();
        info!("Scan end");
    });
}

#[derive(Serialize)]
struct Json {
    content: String,
}

/// BLEデバイスのスキャンとデータをサーバーに送信する関数
pub fn scan_and_post_ble_info() {
    block_on(async {
        let ble_device = BLEDevice::take();
        let ble_scan = ble_device.get_scan();

        ble_scan
            .active_scan(true)
            .interval(100) // 測定間隔
            .window(50) // 測定時間
            .on_result(move |_scan, param| {
                let ble_info = get_bleinfo(param);

                // HTTPクライアントの初期化
                let httpconnection = EspHttpConnection::new(&HTTPConfig {
                    use_global_ca_store: true,
                    crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
                    ..Default::default()
                })
                .unwrap();

                let mut httpclient = Client::wrap(httpconnection);

                let header = [("Content-Type", "application/json")];
                let mut request = httpclient.post(URL, &header).unwrap();

                // let response_body = ble_info.get_json();
                let json_data = Json {
                    content: ble_info.get_json(),
                };
                let response_body = serde_json::to_string(&json_data).unwrap();
                request.write(response_body.as_bytes()).unwrap();
                request.submit().unwrap();
            });

        ble_scan.start(10000).await.unwrap();
        info!("Scan end");
    });
}
