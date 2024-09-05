mod ble;
mod config;
mod queue;

use anyhow::Result;
use ble::scan_and_post_ble_info;
use config::WIFI_CONFIG;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfig};
use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    wifi::{BlockingWifi, EspWifi},
};
use heapless::String as heapString;
use log::*;

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
