use chrono::{DateTime, Utc};
use esp32_nimble::{utilities::mutex::Mutex, BLEDevice};
use esp_idf_hal::task::block_on;
use log::*;
use serde::Serialize;
use std::{sync::Arc, time::SystemTime};

#[derive(Clone, Debug, Serialize)]
pub struct BleDeviceInfo {
    address: String,
    rssi: i32,
    time: DateTime<Utc>,
}

pub type BleArc = Arc<Mutex<Option<Vec<BleDeviceInfo>>>>;

// BLEデバイスのスキャンと更新を行う関数
pub fn scan_and_update_ble_devices(ble_devices: BleArc) {
    block_on(async {
        let ble_device = BLEDevice::take();
        let ble_scan = ble_device.get_scan();

        // `ble_devices`のクローンをクロージャに渡す
        let ble_devices_clone = Arc::clone(&ble_devices);

        ble_scan
            .active_scan(true)
            .interval(101)
            .window(100)
            .on_result(move |_scan, param| {
                let address = param.addr().to_string();
                let rssi = param.rssi();
                let time: DateTime<Utc> = SystemTime::now().into();

                // ログにデバイス情報を出力
                info!(
                    "Advertised Device: {:?}, RSSI: {}, Timestamp: {}",
                    address, rssi, time
                );

                // クロージャ内でデバイス情報を追加
                let mut ble_devices = ble_devices_clone.lock();
                if let Some(ref mut devices) = *ble_devices {
                    devices.push(BleDeviceInfo {
                        address,
                        rssi,
                        time,
                    });
                } else {
                    *ble_devices = Some(vec![BleDeviceInfo {
                        address,
                        rssi,
                        time,
                    }]);
                }
            });

        ble_scan.start(5001).await.unwrap();
        // info!("{:?}", ble_devices);
        info!("Scan end");
    });
}
