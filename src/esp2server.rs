mod ble;
mod config;
mod queue;

use anyhow::Result;
use ble::{scan_and_update_ble_info, BleInfoJson};
use config::{DEVICE_ID, WIFI_CONFIG};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfig};
use esp_idf_hal::{delay::FreeRtos, io::Write, peripherals::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::{
        server::{Configuration as HTTPConfig, EspHttpServer},
        Method,
    },
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    wifi::{BlockingWifi, EspWifi},
};
use heapless::String as heapString;
// use http::time2json;
use log::*;
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);

    // Wi-Fiの初期化
    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    // Wi-Fiの設定
    let ssid: heapString<32> = heapString::try_from(WIFI_CONFIG.ssid).expect("SSID Error");
    let password: heapString<64> =
        heapString::try_from(WIFI_CONFIG.password).expect("Password Error");

    wifi.set_configuration(&WifiConfig::Client(ClientConfiguration {
        ssid: ssid,
        password: password,
        auth_method: AuthMethod::None,
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration()?;
        info!("Waiting for station {:?}", config);
    }
    info!("Connected to Wi-Fi");

    // NTPの初期化 (時刻同期)
    let ntp = EspSntp::new_default()?;
    info!("Synchronizing with NTP Server");

    // HTTP Server用のBLEデバイス情報を格納する共有メモリ
    let ble_info = Arc::new(Mutex::new(BleInfoJson::new(DEVICE_ID, 50)));
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
