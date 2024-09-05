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

`src/config.rs`や`examples/config.rs`ファイルを作成し、WifiのSSIDとパスワードを書き込む

```src/config.rs
pub struct WifiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

pub const WIFI_CONFIG: WifiConfig = WifiConfig {
    ssid: "ssid",
    password: "password",
};

pub const DEVICE_ID: usize = 1;

pub const URL: &str = "http://192.209.177.40:5000";
```

# 実行方法

マイコンをjsonサーバーとして用いたい場合、

```bash
cargo run --bin ble_api
```

マイコンからあるサーバーにBLEデータを送信したい場合、

```bash
cargo run --bin ble_post
```
