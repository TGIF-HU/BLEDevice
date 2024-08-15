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
    ssid: "elecom-a19d07",
    password: "c2prrkjhc8nk",
};
```