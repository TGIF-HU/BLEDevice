mod config;
mod utils;

use anyhow::Result;
use config::{DEVICE_URL, PASSWORD, SSID};
use embedded_svc::http::client::Client;
use esp32_nimble::BLEDevice;
use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals, task::block_on};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::{Configuration, EspHttpConnection},
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
};
use log::*;
use utils::{ble, wifi};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);

    // Peripheralの初期化
    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Wi-Fiの初期化
    let wifi_settings = wifi::WifiSettings::new(SSID, PASSWORD);
    let _wifi = wifi::wifi_init(wifi_settings, peripherals.modem, sysloop, nvs)?;

    // NTPの初期化 (時刻同期)
    let ntp = EspSntp::new_default()?;
    info!("Synchronizing with NTP Server");

    // BLEデバイスのスキャンと更新を行う
    let ble_device = BLEDevice::take();
    let ble_scan = ble_device.get_scan();

    std::thread::spawn(move || loop {
        block_on(async {
            ble_scan
                .active_scan(true)
                .interval(100) // 測定間隔
                .window(50) // 測定時間
                .on_result(move |_scan, param| {
                    let ble_info = ble::get_bleinfo(param);

                    // HTTPクライアントの初期化
                    let httpconnection = match EspHttpConnection::new(&Configuration {
                        use_global_ca_store: false, // httpsの場合はtrue
                        // crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach), // httpsの場合は必須
                        ..Default::default()
                    }) {
                        Ok(conn) => conn,
                        Err(e) => {
                            warn!("HTTPクライアントの初期化に失敗しました: {:?}", e);
                            return;
                        }
                    };

                    let mut httpclient = Client::wrap(httpconnection);

                    let header = [("Content-Type", "application/json")];
                    let mut request = match httpclient.post(DEVICE_URL, &header) {
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
    });

    // Serverのリクエストの受信 ∧ 時間の較正
    loop {
        // NTPの時刻同期
        while ntp.get_sync_status() != SyncStatus::Completed {}
        info!("Time Sync Completed");

        FreeRtos::delay_ms(u32::MAX / 100); // 2^32 ms / 100 ~= 49.7 days / 100 ~= 0.5 day
    }
}
