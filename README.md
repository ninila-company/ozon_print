# Ozon Print

TUI-приложение для печати штрихкодов на PDF. Работает в терминале на Linux и Windows.

## Возможности

- Просмотр PDF-файлов со штрихкодами в директории
- Выбор файлов для печати (Space)
- Указание количества копий для каждого штрихкода (e)
- Поиск по имени файла (i)
- Настройка путей и команд через встроенный редактор конфига (c)
- Авто-сохранение конфига: `config_for_ozon_print.json` рядом с .exe

## Управление

| Клавиша | Действие |
|---------|----------|
| ↑↓ / j/k | Навигация |
| Space | Выбрать/снять |
| e | Редактировать количество |
| i | Поиск |
| a | Выбрать все |
| n | Снять все |
| c | Настройки |
| Enter | Печать |
| q | Выход |

## Сборка

```bash
# Linux
cargo build --release

# Windows (кросс-компиляция)
cross build --target x86_64-pc-windows-gnu --release
```

## Конфигурация

Автоматически создаётся `config_for_ozon_print.json` при первом запуске.

Настраиваемые поля:
- `barcode_directory` — директория со штрихкодами
- `printer_name` — имя принтера
- `temp_directory` — временная папка
- `preview_command` — команда для открытия PDF
- `print_command` — команда печати

## Зависимости

- [clap](https://github.com/clap-rs/clap) — CLI аргументы
- [ratatui](https://github.com/ratatui-org/ratatui) — TUI интерфейс
- [crossterm](https://github.com/crossterm-rs/crossterm) — терминал
- [lopdf](https://github.com/J-F-Liu/lopdf) — работа с PDF
