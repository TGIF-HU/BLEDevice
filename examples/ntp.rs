use anyhow;
use chrono::{DateTime, Utc};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    wifi::{BlockingWifi, EspWifi},
};
use heapless::String;
use std::time::SystemTime;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    // SSID, Passwordの設定をするように
    let ssid: String<32> = String::try_from("ssid").unwrap();
    let password: String<64> = String::try_from("password").unwrap();

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid,
        bssid: None,
        auth_method: AuthMethod::None,
        password: password,
        channel: None,
        ..Default::default()
    }))?;

    // Start Wifi
    wifi.start()?;

    // Connect Wifi
    wifi.connect()?;

    // Wait until the network interface is up
    wifi.wait_netif_up()?;

    // Print Out Wifi Connection Configuration
    while !wifi.is_connected().unwrap() {
        // Get and print connection configuration
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }

    println!("Wifi Connected");

    // Create Handle and Configure SNTP
    let ntp = EspSntp::new_default().unwrap();

    // Synchronize NTP
    println!("Synchronizing with NTP Server");
    while ntp.get_sync_status() != SyncStatus::Completed {}
    println!("Time Sync Completed");

    loop {
        // Obtain System Time
        let st_now = SystemTime::now();
        // Convert to UTC Time
        let dt_now_utc: DateTime<Utc> = st_now.clone().into();
        // Format Time String
        let formatted = format!("{}", dt_now_utc.format("%d/%m/%Y %H:%M:%S"));
        // Print Time
        println!("{}", formatted);
        // Delay
        FreeRtos::delay_ms(1000);
    }
}
