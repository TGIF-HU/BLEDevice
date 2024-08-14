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