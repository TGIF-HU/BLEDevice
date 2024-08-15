mod ble;
mod config;
mod http;

use anyhow::Result;
use chrono::{DateTime, Utc};
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
use std::{collections::HashMap, sync::Arc, time::SystemTime};

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

    // HTTP Serverの初期化
    let mut httpserver = EspHttpServer::new(&HTTPConfig::default())?;

    // BLEデバイス情報を格納するための共有メモリを初期化
    // let ble_devices = Arc::new(Mutex::new(HashMap::new()));

    httpserver.fn_handler("/", Method::Get, |request| {
        request
            .into_ok_response()?
            .write_all(index_html().as_bytes())
    })?;

    loop {
        FreeRtos::delay_ms(1000);
    }
}

fn index_html() -> String {
    let st_now = SystemTime::now();
    let dt_now_utc: DateTime<Utc> = st_now.clone().into();

    json!({
        "time": dt_now_utc,
    })
    .to_string()
}
