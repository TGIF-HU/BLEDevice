mod config;
mod utils;

use config::{PASSWORD, SSID};
use esp32_nimble::{utilities::BleUuid, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, prelude::Peripherals};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use utils::{leddriver::LedDriver, wifi};

const SERVICE_UUID: BleUuid = BleUuid::Uuid16(0xABCD);
const WAITTIME: u32 = 20000; // 20000ms = 20s

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
        .name("BLE_Device")
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
        leddriver.running();

        // BLEの広告を開始
        advertising.start()?;
        FreeRtos::delay_ms(WAITTIME);
        advertising.stop()?;

        // 終わりの演出
        leddriver.ending();
        leddriver.waiting();
    }
}
