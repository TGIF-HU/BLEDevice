# BLE Rust

## インストール

1. https://github.com/esp-rs/esp-idf-template を参考に必要なものをインストールする
2. Linuxの場合、`dialout`グループに`/dev/ttyUSB0`の権限をもたせる必要がある。
    - 現在のユーザーを`dialout`グループに所属させる
```
sudo usermod -a -G dialout $USER
newgrp dialout #反映
id #確認
```

# WIFIの設定

`src/config.rs`を作成し、WifiのSSIDとパスワードを書き込む

```src/config.rs
pub const SSID: &str = "ssid";
pub const PASSWORD: &str = "password";

pub const DEVICE_ID: usize = 1;

// このURLは、BLEを受信しサーバに送信するためのURL
pub const DEVICE_URL: &str = "http://192.168.2.103:5050/api/device";
// このURLは、測定用のURL
pub const MEASURE_URL: &str = "http://192.168.2.103:5050/api/measure";

```

# 実行方法

マイコンをjsonサーバーとして用いたい場合、

```bash
cargo run --bin ble_server
```

マイコンからあるサーバーにBLEデータを送信したい場合、

```bash
cargo run --bin ble_client
```

データを測定したい場合、

```bash
cargo run --bin ble_calibration
```