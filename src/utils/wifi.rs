use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfig};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use esp_idf_sys::EspError;
use heapless::String as heapString;
use log::*;

pub struct WifiSettings {
    pub ssid: &'static str,
    pub password: &'static str,
}

impl WifiSettings {
    pub const fn new(ssid: &'static str, password: &'static str) -> Self {
        Self { ssid, password }
    }
}

pub fn wifi_init(
    wifisettigs: WifiSettings,
    modem: Modem,
    sysloop: EspEventLoop<System>,
    nvs: EspDefaultNvsPartition,
) -> Result<BlockingWifi<EspWifi<'static>>, EspError> {
    // Wi-Fiの初期化
    let mut wifi: BlockingWifi<EspWifi<'_>> =
        BlockingWifi::wrap(EspWifi::new(modem, sysloop.clone(), Some(nvs))?, sysloop)?;

    // Wi-Fiの設定
    let ssid: heapString<32> = heapString::try_from(wifisettigs.ssid).expect("SSID Error");
    let password: heapString<64> =
        heapString::try_from(wifisettigs.password).expect("Password Error");

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

    Ok(wifi)
}
