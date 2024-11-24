mod config;
mod utils;

use anyhow::Result;
use ble::{get_bleinfo, BLEInfoQueue};
use config::{PASSWORD, SSID};
use esp32_nimble::BLEDevice;
use esp_idf_hal::{delay::FreeRtos, io::Write, peripherals::Peripherals, task::block_on};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::{
        server::{Configuration as HTTPConfig, EspHttpServer},
        Method,
    },
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
};
use log::*;
use std::sync::{Arc, Mutex};
use utils::{ble, wifi};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);

    // Peripheralsの初期化
    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Wi-Fiの初期化
    let wifi_settings = wifi::WifiSettings::new(SSID, PASSWORD);
    let _wifi = wifi::wifi_init(wifi_settings, peripherals.modem, sysloop, nvs)?;

    // NTPの初期化 (時刻同期)
    let ntp = EspSntp::new_default()?;
    info!("Synchronizing with NTP Server");

    // HTTP Server用のBLEデバイス情報を格納する共有メモリ
    let ble_info = Arc::new(Mutex::new(BLEInfoQueue::new(50)));
    let ble_info_http = ble_info.clone();
    // BLEスキャン用の共有メモリ
    let ble_info_scan = ble_info.clone();

    // HTTP Serverの初期化
    let mut httpserver = EspHttpServer::new(&HTTPConfig::default())?;

    httpserver.fn_handler("/", Method::Get, move |request| {
        let ble_info_http_lock = ble_info_http.lock().unwrap();

        // BLEデバイス情報があれば、それをJSON形式で返す
        let response_body = ble_info_http_lock.get_json();
        request
            .into_ok_response()?
            .write_all(response_body.as_bytes())
    })?;

    std::thread::spawn(move || loop {
        // BLEデバイスのスキャンと更新を行う
        scan_and_update_ble_info(ble_info_scan.clone());
    });

    // Serverのリクエストの受信 ∧ 時間の較正
    loop {
        // NTPの時刻同期
        while ntp.get_sync_status() != SyncStatus::Completed {}
        info!("Time Sync Completed");

        FreeRtos::delay_ms(u32::MAX / 100); // 2^32 ms / 100 ~= 49.7 days / 100 ~= 0.5 day
    }
}

fn scan_and_update_ble_info(ble_info: Arc<Mutex<BLEInfoQueue>>) {
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
