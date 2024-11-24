mod config;
mod utils;

use anyhow::Result;
use config::{PASSWORD, SSID, URL};
use embedded_svc::http::client::Client;
use esp32_nimble::BLEDevice;
use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals, task::block_on};
use esp_idf_svc::http::client::Configuration as HTTPConfig;
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
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
    std::thread::spawn(move || loop {
        scan_and_post_ble_info();
    });

    // Serverのリクエストの受信 ∧ 時間の較正
    loop {
        // NTPの時刻同期
        while ntp.get_sync_status() != SyncStatus::Completed {}
        info!("Time Sync Completed");

        FreeRtos::delay_ms(u32::MAX / 100); // 2^32 ms / 100 ~= 49.7 days / 100 ~= 0.5 day
    }
}

fn scan_and_post_ble_info() {
    // ToDo: エラー処理
    block_on(async {
        let ble_device = BLEDevice::take();
        let ble_scan = ble_device.get_scan();

        ble_scan
            .active_scan(true)
            .interval(100) // 測定間隔
            .window(50) // 測定時間
            .on_result(move |_scan, param| {
                let ble_info = ble::get_bleinfo(param);

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
