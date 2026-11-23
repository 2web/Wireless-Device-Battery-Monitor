# battery-protocol (открытая замена ядра)

Upstream собирает GUI против закрытого крейта `mouse-battery-core` (`../mouse-battery-core`),
которого нет в открытом доступе. Этот крейт воспроизводит его публичный API настолько,
насколько его использует GUI, и подключён в корневом `Cargo.toml` как
`battery-protocol = {path="battery-core"}`.

## Что реально работает

| Устройство | USB ID | Протокол |
| --- | --- | --- |
| ThunderRobot ML701 Pro (и другие мыши на Sinowealth с тем же ресивером, например Redragon M910-KS) | `258A:002F` ресивер, `258A:002E` кабель | feature report `0x05`, команда `0x90` |

```text
SET_FEATURE [05 90 00 00 00 00 00 00]
GET_FEATURE [05 90 link percent ...]
  link 0x11 = 2.4G: percent 0 → мышь спит (не 0 %)
  link 0x10 = кабель: заряжается; percent 0 → уровень ещё неизвестен, 100 → заряжена
```

Отправляется только read-only команда `0x90`. Тот же канал у Sinowealth принимает запись
конфигурации и команду `0x75` (бутлоадер), поэтому код не принимает произвольный байт команды.
Протокол взят из [dominikake/redragon-m910ks](https://github.com/dominikake/redragon-m910ks)
и проверен на ML701 Pro.

Остальные производители из `Source` (Logitech, Razer, ROG, AULA и др.) и Bluetooth —
заглушки: они нужны только для компиляции GUI, обнаружение их не находит.

## Проверка на железе

```powershell
cargo run --manifest-path battery-core\Cargo.toml --example read
```

## Сборка приложения

Release-сборка gpui ищет `fxc.exe` только в Windows SDK `10.0.26100.0`. Для другой версии SDK
укажите путь явно:

```powershell
$env:GPUI_FXC_PATH = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\fxc.exe"
cargo build --release
```
