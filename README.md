# rlhelp

**Interactive command constructor based on `man` and `--help`**

`rlhelp` is a CLI utility written in **Rust** that turns command help texts (`--help` and `man`) into an interactive command builder directly in your terminal.

## Description

### The Problem
Reading long `man` pages or scrolling through `--help` output just to find a single flag is slow and annoying.

### The Solution
Run:

```bash
rlhelp <command>
```

You will see an interactive list of all available flags, select the ones you need with checkboxes, preview the resulting command, and execute it immediately.

## Features

- **Auto parsing** — extracts flags and descriptions from `--help`. If the output is incomplete or missing, automatically falls back to `man`.
- **Freeze protection** — smart timeout system prevents hanging on interactive programs (e.g. `vim`, `less`).
- **Keyboard navigation** — arrow keys or Vim-style bindings (`j` / `k`).
- **Language switch** — ability to force English help (`LC_ALL=C`) if system localization is broken or confusing.

## Installation

Requires **Rust (Cargo)**.

```bash
cargo install --path .
```

## Usage

Just provide the command you want to configure:

```bash
rlhelp git
rlhelp ls
rlhelp grep
rlhelp ffmpeg
```

## Key Bindings

| Key        | Action                        |
|------------|-------------------------------|
| ↑ / k      | Move up                       |
| ↓ / j      | Move down                     |
| Space      | Toggle flag selection         |
| Enter      | Execute command               |
| p          | Print command (dry run)       |
| l          | Toggle language (System / EN) |
| q / Esc    | Quit                          |

---

# rlhelp

**Интерактивный конструктор команд на основе `man` и `--help`**

`rlhelp` — это CLI-утилита на **Rust**, которая превращает справку любой команды (`man` и `--help`) в интерактивный конструктор команд прямо в терминале.

## Описание

### Проблема
Читать длинные `man`-страницы или листать вывод `--help`, чтобы найти один нужный флаг — медленно и неудобно.

### Решение
Запустите:

```bash
rlhelp <команда>
```

Вы увидите интерактивный список всех доступных флагов, сможете отметить нужные галочками, посмотреть итоговую команду и сразу её выполнить.

## Возможности

- **Авто-парсинг** — извлекает флаги и описания из `--help`. Если справка неполная или отсутствует, автоматически использует `man`.
- **Защита от зависания** — умная система таймаутов не даёт зависнуть на интерактивных программах (например, `vim` или `less`).
- **Навигация с клавиатуры** — стрелки или Vim-биндинги (`j` / `k`).
- **Переключение языка** — возможность принудительно включить английскую справку (`LC_ALL=C`), если системная локализация некорректна.

## Установка

#### Требуется установленный **Rust (Cargo)**.
Убедитесь, что у вас установлен rust версии не ниже 1.68 или новее:
```bash
rustc --version
#### При необходимости обновите:
```bash
sudo apt update
sudo apt upgrade rustc
```
#### Установите, если у вас ещё не установлены:
```bash
sudo apt install git
sudo apt install cargo
```
#### Клонируйте репозиторий с GitHub:
```bash
 git clone https://github.com/shockwave-supernova/man_help.git
 ```
#### Смените каталог на папку проекта и установите rhelp:
```bash
cargo install --path .
```
Во время установки вы увидите сообщение "Updating crates.io index", которое говорит о том, что Cargo обновляет индекс пакетов из реестра crates.io, индекса информации о доступных пакетах и их версиях, что позволяет Cargo находить и устанавливать необходимые зависимости для вашего проекта. Это может занять некоторое время, особенно при невысокой скорости интернет-соединения. Дождитесь окончания процесса.

## Использование

Просто укажите команду, для которой хотите собрать параметры:

```bash
rlhelp git
rlhelp ls
rlhelp grep
rlhelp ffmpeg
```

## Горячие клавиши

| Клавиша | Действие |
|--------|----------|
| ↑ / k  | Вверх   |
| ↓ / j  | Вниз    |
| Space  | Выбрать флаг |
| Enter  | Выполнить команду |
| p      | Показать команду (без запуска) |
| l      | Сменить язык (System / English) |
| q / Esc| Выход   |

## License

Distributed under the **MIT License**.
