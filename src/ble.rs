use crate::{
    config::{DEVICE_ID, URL},
    queue::FixedQueue,
};

use chrono::{DateTime, Utc};
use embedded_svc::http::client::Client;
use esp32_nimble::{BLEAdvertisedDevice, BLEDevice};
use esp_idf_hal::{delay::FreeRtos, task::block_on};
use esp_idf_svc::http::client::Configuration as HTTPConfig;
use esp_idf_svc::http::client::EspHttpConnection;
use log::*;
use serde::Serialize;
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
#[allow(dead_code)]
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

/// BLEデバイスのスキャンとデータをサーバーに送信する関数
#[allow(dead_code)]
pub fn scan_and_post_ble_info() {
    // ToDo: エラー処理
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
                    use_global_ca_store: false, // httpsの場合はtrue
                    // crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach), // httpsの場合は必須
                    ..Default::default()
                })
                .unwrap();

                let mut httpclient = Client::wrap(httpconnection);

                let header = [("Content-Type", "application/json")];
                let mut request = match httpclient.post(URL, &header) {
                    Ok(request) => request,
                    Err(e) => {
                        warn!("サーバーが見つかりませんでした");
                        warn!("Error: {:?}", e);
                        FreeRtos::delay_ms(3000);
                        return;
                    }
                };

                let response_body = ble_info.get_json();
                request.write(response_body.as_bytes()).unwrap();
                request.submit().unwrap();
            });

        ble_scan.start(10000).await.unwrap();
        info!("Scan end");
    });
}
