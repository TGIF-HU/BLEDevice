mod config;
mod utils;

use config::{MEASURE_URL, PASSWORD, SSID};
use embedded_svc::http::client::Client;
use esp32_nimble::{utilities::BleUuid, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, prelude::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::{Configuration, EspHttpConnection},
    nvs::EspDefaultNvsPartition,
};
use esp_idf_sys::exit;
use log::*;
use utils::{leddriver::LedDriver, wifi};

const SERVICE_UUID: BleUuid = BleUuid::Uuid16(0xABCD);
const WAITTIME: u32 = 2000; // 10000ms = 2s
const DEVICE_NAME: &str = "CalibrationDevice";

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Peripheralの初期化
    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Wi-Fiの初期化
    let wifi_settings = wifi::WifiSettings::new(SSID, PASSWORD);
    let _wifi = wifi::wifi_init(wifi_settings, peripherals.modem, sysloop, nvs)?;

    // BLEの初期化
    let ble_device = BLEDevice::take();
    let server = ble_device.get_server();

    let service = server.create_service(SERVICE_UUID);
    let characteristic = service
        .lock()
        .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::READ);
    characteristic.lock().set_value("Hello, world!".as_bytes());

    let mut advertising_data = BLEAdvertisementData::new();
    advertising_data
        .name(DEVICE_NAME)
        .manufacturer_data(&[0x01, 0x02, 0x03]);

    let mut advertising = ble_device.get_advertising().lock();
    advertising.set_data(&mut advertising_data)?;

    // ボタンの初期化
    let led = PinDriver::output(peripherals.pins.gpio27)?;
    let button = PinDriver::input(peripherals.pins.gpio2)?;
    let mut leddriver = LedDriver::new(led, button);
    leddriver.init();

    loop {
        FreeRtos::delay_ms(10);

        if !leddriver.is_button_pushed() {
            continue;
        }
        // ボタンが押されたら、測定を開始
        leddriver.running();

        // HTTPリクエストの送信
        let httpconnection = match EspHttpConnection::new(&Configuration {
            use_global_ca_store: false, // httpsの場合はtrue
            // crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach), // httpsの場合は必須
            ..Default::default()
        }) {
            Ok(conn) => conn,
            Err(e) => {
                warn!("HTTPクライアントの初期化に失敗しました: {:?}", e);
                unsafe { exit(1) } // ToDo: これ以外の書き方はない？
            }
        };

        let mut httpclient = Client::wrap(httpconnection);
        let header = [("Content-Type", "application/json")];

        let mut request = match httpclient.post(MEASURE_URL, &header) {
            Ok(request) => request,
            Err(e) => {
                warn!("サーバーが見つかりませんでした");
                warn!("Error: {:?}", e);
                FreeRtos::delay_ms(3000);
                unsafe {
                    exit(1);
                }
            }
        };
        let payload = serde_json::json!({
            "device_id": "12345",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        info!("Requesting to server");
        request.write(payload.to_string().as_bytes())?;
        request.submit()?;
        info!("Request sent");
        FreeRtos::delay_ms(3000);

        // BLEの広告を開始
        info!("Advertising start");
        advertising.start()?;
        FreeRtos::delay_ms(WAITTIME);
        advertising.stop()?;
        info!("Advertising stopped");

        // 終わりの演出
        leddriver.ending();
        leddriver.waiting();
    }
}
