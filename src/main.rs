mod ble;
mod config;

use anyhow::Result;
use ble::{scan_and_update_ble_devices, BleDeviceInfo};
use config::WIFI_CONFIG;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfig};
use esp32_nimble::utilities::mutex::Mutex;
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
use serde_json::json;
use std::sync::Arc;

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

    // NTPの初期化
    let ntp = EspSntp::new_default()?;
    info!("Synchronizing with NTP Server");
    while ntp.get_sync_status() != SyncStatus::Completed {}
    info!("Time Sync Completed");

    // BLEデバイス情報を格納するための共有メモリを初期化
    let ble_info_http = Arc::new(Mutex::new(None::<Vec<BleDeviceInfo>>));
    // BLEスキャンを開始する別スレッド
    let ble_info_scan = ble_info_http.clone();

    // HTTP Serverの初期化
    let mut httpserver = EspHttpServer::new(&HTTPConfig::default())?;

    httpserver.fn_handler("/", Method::Get, move |request| {
        let ble_info_http_lock = ble_info_http.lock();
        // BLEデバイス情報があれば、それをJSON形式で返す
        if let Some(info) = &*ble_info_http_lock {
            let response_body = json!(info);
            request
                .into_ok_response()?
                .write_all(response_body.to_string().as_bytes())
        } else {
            request.into_ok_response()?.write_all("{}".as_bytes())
        }
    })?;

    std::thread::spawn(move || loop {
        // BLEデバイスのスキャンと更新を行う関数を呼び出す
        scan_and_update_ble_devices(Arc::clone(&ble_info_scan));
    });

    // HTTP Serverでのリクエストを待ち受ける
    loop {
        FreeRtos::delay_ms(1000);
    }
}
