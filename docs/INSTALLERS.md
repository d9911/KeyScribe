# Keyscribe: сборка, установка и проверка

## Команды

Для сборки нужны Rust не ниже версии из Cargo.toml, Cargo и Python 3.11+.
Установленному приложению Rust, Cargo, Python, Make и терминал не нужны.
Версия пакетов берётся из Cargo.toml; идентификатор `local.keyscribe.desktop` сохранён.

| Команда       | Действие                                                               |
| ------------- | ---------------------------------------------------------------------- |
| `make help`   | Команды и каталог результатов                                          |
| `make doctor` | Диагностика без установки пакетов, изменения разрешений или служб      |
| `make setup`  | Проектные DMG-инструменты на Mac, нативный Rust target, rustfmt/Clippy |
| `make dev`    | Прежний запуск через Cargo                                             |
| `make build`  | Release, нативный установщик и совместимый ZIP на Mac/Windows          |
| `make local`  | Собрать текущий код и открыть установщик для ручной установки          |
| `make run`    | Запустить существующую release-копию без пересборки                    |
| `make start`  | Собрать и открыть release-копию                                        |
| `make check`  | Существующие fmt, Clippy и тесты                                       |

PowerShell без GNU Make: `./package.ps1 setup`, `./package.ps1 doctor`,
`./package.ps1 local`, `./package.ps1 run`. Можно передать `-PythonExe` с путём к Python.
Старые `scripts/build-macos.sh` и `scripts/build-windows.ps1` делегируют сборку одному
локальному `scripts/project.py` и проверяют нативную ОС.

Сборщик берёт путь EXE непосредственно из JSON-результата Cargo, поэтому учитывает
`CARGO_TARGET_DIR`. Проверена сборка из каталога с пробелами. При ошибке компиляции
или упаковки предыдущие готовые пакеты не заменяются; активные приложения не завершаются.
В исходники не включаются build/dist и зависимости инструментов.

## macOS

```sh
make setup
make doctor
make local
```

Результаты текущей arm64-сборки:

- `dist/Keyscribe.app`;
- `dist/Keyscribe-0.1.0-macos-arm64.dmg`;
- `dist/Keyscribe-macos-arm64.zip`.

Перетащите приложение в Applications в окне DMG и запускайте установленную копию
оттуда. Перед обновлением остановите ввод, сохраните нужный текст и закройте приложение.
Иконка, сохранённый Finder layout и ссылка Applications включены в DMG без Finder Automation.
Проверяются подписи исходной .app и копии внутри образа, а также checksum DMG.

По умолчанию используется локальная ad-hoc подпись. Для имеющегося Developer ID
сохранена `KEYSCRIBE_SIGNING_IDENTITY`. Notarization и публикация не выполняются.
Сборка arm64 не является universal; Intel требует отдельной нативной сборки и проверки.
Разрешения Accessibility не сбрасываются сборщиком. После смены подписи/пути может
потребоваться пользовательское подтверждение для новой копии в системных настройках.

## Windows x64

Собирать в Developer PowerShell для Visual Studio с C++ Build Tools и Windows SDK,
Rust MSVC target, Python 3.11+ и Inno Setup 6.3+. Нужен `dumpbin.exe` либо `objdump.exe`
для фактической проверки DLL. `ISCC` может задавать путь до Inno Setup.

```powershell
.\package.ps1 setup
.\package.ps1 doctor
.\package.ps1 local
```

Результаты: `dist/windows-x64/Keyscribe.exe`, `dist/Keyscribe-windows-x64.zip`
и `dist/Keyscribe-0.1.0-windows-x86_64-setup.exe`. DLL проверяются рекурсивно;
отсутствие необходимой библиотеки останавливает создание установщика.

Установка для текущего пользователя, ярлык меню и необязательный ярлык рабочего стола;
удаление через штатный список приложений. Mutex блокирует обновление/удаление работающей
копии. Автозапуск приложения и службы не создаются.
Нативный установщик Windows в этой итерации не собран и не проверен.

## Ubuntu 24.04, X11

Инструменты разработки, использованные в проверочной Ubuntu:

```sh
sudo apt-get update
sudo apt-get install build-essential pkg-config python3 python3-venv dpkg-dev binutils xdg-utils libx11-dev libxi-dev libxtst-dev libxrandr-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libxkbcommon-x11-dev libgl1-mesa-dev libegl1-mesa-dev libwayland-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libssl-dev
make setup
make doctor
make build
sudo apt install ./dist/keyscribe_0.1.0_arm64.deb
```

Последняя команда относится к проверенной arm64-сборке. На amd64 используйте
пакет с фактической архитектурой из его имени; amd64 в этой итерации не проверен.
Приложение появляется в меню; `/usr/bin/keyscribe` запускает `/opt/keyscribe/keyscribe`.
Runtime-зависимости из ELF вычисляются `dpkg-shlibdeps`; необходимые библиотеки GUI
указаны дополнительно для динамически загружаемых X11/OpenGL-компонентов.
Нужен графический X11-сеанс с OpenGL-драйвером. GUI проверен с Mesa/Xvfb.
Обновление и удаление отказываются менять работающую установленную копию, не завершая её.

### Ввод и остановка

Linux получает физические коды из имён XKB текущего сервера. Это не сложение
Windows scan code с константой; карта полностью проверяется до отправки событий.
Сохранены выбор начальной раскладки, проверка текста и явное подтверждение запуска.

- Linux: остановка `Ctrl+Shift+F12`, резервная `Ctrl+Shift+Pause`;
- Linux: захват координат `Ctrl+Shift+F11`;
- macOS/Windows: прежние `Ctrl+Alt+F12` и `Ctrl+Alt+F11`.

Причина различия проверена на X11: `Ctrl+Alt+F12` не дошёл до обработчика при XKB VT-action
в раскладке F12; вариант без Alt сработал. Резервная Pause проверена с удержанным Alt.
Конфликтующие сочетания зарезервированы и не принимаются как действия/переключатели раскладки.

**Wayland:** автоматизация явно блокируется до отправки событий. Текущий global-hotkey
поддерживает Linux/X11; XWayland не обеспечивает аварийную остановку во всех native Wayland
окнах. Для автоматизации выбирайте X11/Xorg при входе в систему. Полный Wayland backend
не реализован; проверено блокирование режима, а не реальный Wayland-сеанс.

## Свежая проверка 26 октября 2026

- macOS arm64: 32 существующих теста, fmt и Clippy; `.app`, DMG и ZIP собраны.
- Копия из read-only DMG вне репозитория: подпись, ресурсы, версия и реальное окно
  проверены. Запуск ввода на Mac не выполнялся; окно требует разрешение Accessibility.
- Ubuntu 24.04 arm64 в отдельном контейнере: fmt, Clippy и 32 существующих теста.
- Реальный `desktop::Session` в отдельном Xvfb отправил `aA1!` с правильным Shift;
  остановлен глобальным сочетанием, все клавиши освобождены. Проверены основной и
  резервный варианты, включая Alt, и раннее отклонение Wayland.
- `.deb` установлен, GUI открыт обычным пользователем с PATH без Cargo/Rust;
  подтверждена регистрация всех восьми Linux stop-grab комбинаций.
- Обновление/удаление работающей копии блокируется; после закрытия обновление и удаление
  прошли, существующий пользовательский файл сохранён.

Это проверки текущего Mac и виртуального X11 в Ubuntu, не подтверждение конкретной
Citrix-сессии, иных дистрибутивов, Windows-установщика или Intel Mac.
Текст, настройки и сценарии по-прежнему хранятся в памяти до закрытия окна; новая
система постоянного хранения в рамках упаковки не добавлялась.

Источники API и требований:

- [global-hotkey platforms](https://github.com/tauri-apps/global-hotkey).
- [XKB physical key names](https://sources.debian.org/src/xkeyboard-config/2.42-1/keycodes/evdev/).
- [eframe dependencies](https://github.com/emilk/egui/tree/main/crates/eframe).
- [Inno Setup AppMutex](https://jrsoftware.org/ishelp/topic_setup_appmutex.htm).

Итоговая упаковка включает эту публичную инструкцию. Дополнительно проверен откат при искусственной ошибке продвижения файлов: прежние .app, установщики и manifest восстановлены; output-symlink отклоняется до замены. Повторная проверка активного экземпляра выполняется непосредственно перед продвижением.

## Исходное состояние перед изменениями

| ОС / архитектура     | Прежняя команда                           | Инструменты                               | Прежний результат | Пробел                                                  |
| -------------------- | ----------------------------------------- | ----------------------------------------- | ----------------- | ------------------------------------------------------- |
| macOS arm64 / Intel  | `make build`                              | Rust, Xcode CLT                           | .app и ZIP        | Нет DMG и единого doctor/setup                          |
| Windows x64 MSVC     | `make build`, `scripts/build-windows.ps1` | Rust MSVC, C++ Build Tools / SDK          | EXE и ZIP         | Нет установщика, удаления и проверки runtime DLL        |
| Ubuntu arm64 / amd64 | `make build` отказывал                    | Rust и ещё не реализованный Linux backend | Нет               | compile_error, физические клавиши и аварийная остановка |

Источник таблицы — исходные Makefile, scripts/build-windows.ps1 и src/main.rs из HEAD; наличие прежнего сценария не означает проверку всех архитектур.
