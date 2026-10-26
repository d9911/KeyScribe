# Keyscribe

<img src="assets/icons/keyscribe.svg" alt="Keyscribe icon" width="112">

[English](README.md) · [Русский](README.ru.md) · [Español](README.es.md)

![Rust 2021](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)
![egui 0.33](https://img.shields.io/badge/UI-egui_0.33-4472C4)
![macOS Apple Silicon](https://img.shields.io/badge/macOS-Apple_Silicon-333333?logo=apple&logoColor=white)
![Windows x64](https://img.shields.io/badge/Windows-x64-0078D4)

Text, mouse and action cycles · English and Russian together · macOS and Windows

<details>
<summary>Contents</summary>

[Features](#features) · [Technology stack](#technology-stack) · [Quick start](#quick-start) · [Keyboard layouts and characters](#keyboard-layouts-and-characters) · [Random mouse movement](#random-mouse-movement) · [Repeating action cycles](#repeating-action-cycles) · [Citrix on Mac](#citrix-on-mac) · [Make commands](#make-commands) · [Build for macOS](#build-for-macos) · [Build for Windows](#build-for-windows) · [Development and verification](#development-and-verification) · [Author](#author) · [License](#license)

</details>

![Keyscribe on macOS: mixed text and configurable layout switching](assets/screenshots/keyscribe-light.jpg)

**Paste here. Type into the window you choose.** Keyscribe turns text into
physical key presses with configurable delays. This screenshot shows the current
macOS application with demonstration text; no remote typing was started for it.
The application UI is currently Russian; documentation is available in three languages.

Choose **Светлая** (Light) or **Тёмная** (Dark) in the top bar. The text editor and settings sit side by side in a wide window and stack in a narrower one. The theme selection lasts until the application closes.

<details>
<summary>Dark theme</summary>

![Keyscribe in the dark green theme](assets/screenshots/keyscribe-dark.jpg)

</details>

<details>
<summary>Random mouse movement</summary>

![Mouse movement settings](assets/screenshots/keyscribe-mouse.jpg)

</details>

<details>
<summary>Repeating action cycle editor</summary>

![Action cycle settings](assets/screenshots/keyscribe-cycle.jpg)

</details>

## Features

| Feature               | Behavior                                                                    |
| --------------------- | --------------------------------------------------------------------------- |
| Mixed text            | English US and Russian ЙЦУКЕН, both cases, digits, printable ASCII and `№`  |
| Layout switching      | Alt+Shift, Ctrl+Shift, Win+Space or a custom combination                    |
| Lines and indentation | Physical Enter and Tab presses; spaces are preserved                        |
| Timing                | Separate countdown, key hold, inter-key and layout-switch delays            |
| Validation            | The complete text is checked before the first key is sent                   |
| Stop                  | Ctrl+Alt+F12, the Stop button or returning to Keyscribe                     |
| Local operation       | Text stays in memory; no server or installation inside Citrix is needed     |
| Random mouse          | Random targets, curves, travel times and pauses inside a bounded area       |
| Action cycles         | Ordered keys, combinations, clicks, movements and waits with a repeat count |
| Appearance | Light cream and dark green themes; navigation and run controls stay visible |

## Technology stack

| Component            | Implementation                                           |
| -------------------- | -------------------------------------------------------- |
| Language             | Rust 2021; minimum compiler declared in Cargo.toml: 1.88 |
| Desktop UI           | egui / eframe 0.33, OpenGL via glow                      |
| Keyboard events      | Enigo 0.6, physical keycodes                             |
| Global stop shortcut | global-hotkey 0.9                                        |
| Playback             | Worker thread, cancellable waits and key release cleanup |
| Build                | Cargo, pinned lockfile and platform build scripts        |

The application runs locally on macOS Apple Silicon (M1 and newer) or Windows x64.
There is no JavaScript runtime, backend service or application network connection.
Text and settings are held in process memory and are not saved between launches.
Keyscribe does not log or transmit the source text. Clipboard access is used only
when pasting into its local editor; it does not paste into the target application.

## Quick start

If a required choice is missing, clicking a start button scrolls to it and shows
a rounded red outline with an explanation. The outline pulses three times, then
stays visible until the field is completed. Guidance moves to the next missing
choice. Continue in the access dialog also points to the next setup step.
Completing the fields never starts input automatically.

1. Open the application built using the platform instructions below.
2. Paste text into the Keyscribe editor using Cmd+V or Ctrl+V.
3. Select **where Keyscribe runs**: macOS / Citrix from Mac or Windows / Citrix
   from Windows. The Mac profile suggests **Control+Option+Space**, Windows
   **Win+Space**; **Ctrl+Win+Space**, Alt+Shift, Ctrl+Shift and custom shortcuts
   remain available. Verify the shortcut inside your remote session.
   Then select the **current layout inside the remote Windows session**: English US
   or standard Russian ЙЦУКЕН. For mixed text, leave automatic switching enabled
   and configure the combination that switches languages inside Citrix.
4. Select the same layout in Citrix and turn Caps Lock off. Disable editor
   auto-indent, automatic bracket pairing and smart quotes when exact text is required.
5. Confirm the checkbox, click **Начать набор** (Start typing), then click the
   target field before the countdown ends. Do not hold modifiers or change target
   windows while typing.
6. Stop with **Ctrl+Alt+F12**, the Stop button or by returning to Keyscribe.
   On Mac use Control+Option+F12, adding Fn if F12 controls media functions.
   Check the already typed fragment before restarting; there is no automatic resume.

Defaults: a 5-second countdown, 25 ms key hold, 100 ms between keys and 400 ms
after switching layout. All four delays can be adjusted in the UI.

## Keyboard layouts and characters

Automatic switching requires **exactly two remote layouts**: English US and
standard Russian ЙЦУКЕН. Select the correct initial layout before each run.
The configured shortcut must toggle between these layouts, rather than always
selecting one fixed language. Choose Control+Option+Space, Win+Space,
Ctrl+Win+Space, Alt+Shift, Ctrl+Shift or combine
Ctrl/Alt/Shift/Win with Space, a digit or F1–F12. For one layout only, disable
automatic switching.

Characters available in the current layout are typed without switching. After a
run or cancellation, the remote layout is left in its last state; check it again
before the next run. Increase the layout-switch delay if the remote session needs
more time to process a change.

Environment and language start unselected; text and cycles cannot start without
both. After execution, cancellation or a session startup error, select the current
language again. Changing environment clears the language and applies that profile's
shortcut preset. Profiles do not change system settings or add support for the
macOS Russian layout: the target Russian layout remains Windows ЙЦУКЕН.

Every line break becomes a real Enter, including a trailing line break. CRLF and
lone CR each become one Enter. No extra Enter is appended. Tab is a real Tab:
the target may change focus or trigger completion. Use spaces for indentation if
the target editor handles Tab differently.

The complete input is validated before typing. Unsupported characters, including
emoji, non-breaking spaces and letters outside these layouts, produce an error
with the line and column; they are never silently replaced. Arbitrary Unicode is
not supported. The progress counter counts sent text keys, excluding layout
shortcuts; it does not confirm receipt by the remote application.

Try the [mixed example](samples/mixed.txt), [US example](samples/us.txt) or
[Russian example](samples/ru.txt) in an empty remote text editor before a long run.
Check case, punctuation, whitespace, Enter, layout switching and emergency stop.

## Random mouse movement

Open **Движение мыши** (Random mouse). Set X/Y radii (5–2000 coordinate units),
the pause range and a duration, or choose to run until manually stopped. Defaults
are radii of 120/80, pauses of 200–1800 ms and a 60-second timer.

The region is anchored around the cursor at the end of the countdown: place it
inside the desired target area before then. This version supports the primary
display and clips the region to its bounds. This mode never clicks. Moving the
cursor outside the region stops playback when checked before the next movement;
returning focus to Keyscribe also stops it.

OS randomness supplies fresh targets, two curve control points, a travel time
of 250–1400 ms and a pause. There is no prerecorded path; random selection does
not guarantee that every movement is unique.

## Repeating action cycles

Open **Цикл действий** (Action cycle) and add steps:

- **Keys:** Enter, Tab, Escape, Backspace, Delete, arrows, Home/End, PageUp/PageDown,
  Space, A–Z, digits and F1–F12, optionally with Ctrl/Alt/Shift/Win.
- **Click:** primary-display X/Y, left/right/middle button, single or double click.
- **Mouse move:** move to specific coordinates.
- **Wait:** 10 ms to one hour.

Reorder, duplicate or delete steps. A cycle supports up to 200 steps and 1–10000
repetitions, with a separate delay between repetitions. To capture coordinates,
arm the chosen step's **Ctrl+Alt+F11** button, point at the target in the external
window and press that shortcut. Alternatively, click **Захват через 3 с** (Capture
in 3 seconds) and place the cursor before the timer ends. Manual X/Y entry is
also available. Physical F-keys may require Fn on Mac.

Verify one repetition before increasing the count. All steps and coordinates are
validated before the first action. Ctrl+Alt+F11/F12 are reserved. Letters in a
cycle are physical US keys interpreted using the target's current layout; the
text mode's automatic language switching does not apply to them.

All modes share the countdown, **Ctrl+Alt+F12**, Stop button and stop-on-return
behavior; only one mode runs at a time. No action is retried automatically after
failure or cancellation. Progress counts sent actions, not successful task
closures. There is no screen recognition or waiting for application results:
include the necessary wait steps. Settings and the cycle currently last only
until the application closes.

## Citrix on Mac

Set **Keyboard input mode → Scancode**. For this mode, Citrix recommends
**Don't sync** for keyboard layout synchronization. Select the initial layout
inside the session and reconnect after changing these preferences. Automatic
input mode may select Unicode and handle layouts differently.

For Win+Space, enable sending the Windows logo key using **Command (right)** in
Citrix: Keyscribe sends Win as right Command on Mac. Verify that your configured
combination changes the remote layout rather than the local computer's layout.
Browser clients and other Citrix versions may expose different preferences.

Sources: [Citrix keyboard settings](https://docs.citrix.com/us/en/citrix-workspace-app-for-mac/devices/keyboard.html),
[Windows language and layout settings](https://support.microsoft.com/en-us/windows/hardware/input-devices/manage-the-language-and-keyboard-input-layout-settings-in-windows).

**End-to-end accuracy in a specific Citrix session is unverified.** Keyscribe
cannot read the remote screen or detect dropped events, an incorrect layout,
focus moving to another external window or editor transformations. Emergency stop
also needs checking in your fullscreen Citrix session. Increase the inter-key and
hold delays if characters are lost.

## Make commands

Run commands from the `keyscribe/` directory. GNU Make and the platform build
prerequisites below must be available in `PATH`; on Windows, install GNU Make
separately and use the MSVC build environment. The Makefile calls Windows PowerShell
for packaging on Windows and the shell script on macOS.

| Command | Action |
| --- | --- |
| `make` or `make dev` | Build and run the development version |
| `make build` | Create the release application and ZIP for the current OS |
| `make run` | Open the existing release without rebuilding |
| `make start` | Build the release, then open it |
| `make check` | Check formatting, run Clippy and tests |
| `make test` | Run tests |
| `make lint` | Run Clippy, treating warnings as errors |
| `make fmt` | Format Rust sources |
| `make fmt-check` | Check formatting without changing files |
| `make help` | Show available commands |

`make run` reports a missing build instead of compiling it. macOS launches
`dist/Keyscribe.app`; Windows launches `dist/windows-x64/Keyscribe.exe`.
To build and open the packaged application with one command, use `make start`.
Close another Keyscribe instance before launching: its global shortcuts may already
be registered. Starting the application does not start typing or mouse actions.

## Build for macOS

From the source project directory, with Rust/rustup and Xcode Command Line Tools:

```sh
sh scripts/build-macos.sh
open dist/Keyscribe.app
```

Outputs: `dist/Keyscribe.app` and `dist/Keyscribe-macos-arm64.zip`.

Missing macOS access opens a blocking dialog. It checks Accessibility and opens
an Enigo connection without posting events; no separate event-posting permission
is required by the app. It offers **Запросить доступ** (Request), **Открыть настройки**
(Open settings), **Проверить ещё раз** (Recheck) and **Показать в Finder** (Reveal
the current app). Enable Keyscribe in **System Settings → Privacy & Security →
Accessibility**. Status refreshes automatically, on return to the app and before
starting. Continue rechecks access; if denied, the dialog stays open and highlights
the permission block. It never starts a job automatically. Reopen the dialog using **Доступ macOS** in the header.
Revoking access cancels the active job. macOS may suppress repeated system
prompts; use Open settings in that case.

Close Keyscribe before rebuilding: the build script refuses to replace a running
app. If access fails after moving or rebuilding despite an enabled switch, select
**Сбросить старое разрешение…** in the dialog and confirm. Keyscribe registers its
current path with LaunchServices, then runs only
`/usr/bin/tccutil reset Accessibility local.keyscribe.desktop`. Other applications'
permissions are unchanged. It requests access again and opens settings. Enable the
current copy and relaunch; if missing, add its `.app` using `+`. The dialog shows
the path and a Finder button. Alternatively, remove the old entry and add the
current app manually. Reset requires explicit confirmation and never grants access
automatically. With
`cargo run`, permission may belong to the terminal or binary; for regular use,
launch the `.app` from a stable location. The default ad hoc signing requirement
is tied to the code hash and may change after rebuilding. With an existing stable
certificate, use `KEYSCRIBE_SIGNING_IDENTITY="certificate name" make build`.
Changing the certificate may also require fresh consent. Apple notarization is
not performed. Sources: [Apple Accessibility prompt](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions),
[Apple code identity](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements).

## Build for Windows

Build on Windows with Rust stable and Visual Studio Build Tools including
**Desktop development with C++** and the Windows SDK:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1
```

The EXE icon is embedded by `build.rs` as an [ICON resource](https://learn.microsoft.com/en-us/windows/win32/menurc/icon-resource).
Build from Developer PowerShell for Visual Studio with Windows SDK `rc.exe` in
`PATH`. Checking Windows code from macOS skips resource compilation.

Outputs: `dist/windows-x64/Keyscribe.exe` and `dist/Keyscribe-windows-x64.zip`.
Run it on the local computer alongside Citrix. Installation inside the remote
desktop is not required. Windows restrictions on input to elevated processes
are documented in [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput).

## Development and verification

From the source checkout with Rust installed:

```sh
cargo run --locked
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

| Module                 | Responsibility                                                     |
| ---------------------- | ------------------------------------------------------------------ |
| `src/keyboard.rs`      | Validate text and plan physical key presses and layout changes     |
| `src/shortcut.rs`      | Configure the layout-switch combination                            |
| `src/playback.rs`      | Emit keys and release modifiers on stop or failure                 |
| `src/desktop.rs`       | Run playback on a worker thread                                    |
| `src/main.rs`          | Desktop UI and global stop shortcut                                |
| `src/cycle.rs`         | Ordered steps, finite repetitions and cancellation without retries |
| `src/motion.rs`        | Bounded randomized mouse trajectories using OS randomness          |
| `src/workspace_ui.rs` | Workspace layout and fixed run controls |
| `src/appearance.rs` | Light and dark palettes, typography and component styles |
| `src/automation_ui.rs` | Cycle editor, coordinate capture and mouse settings                |

Dependencies are pinned in `Cargo.lock`. Physical key semantics are documented
in [Enigo Keyboard::raw](https://docs.rs/enigo/0.6.1/enigo/trait.Keyboard.html#tymethod.raw).
A macOS arm64 release has been built locally. Windows x64 compilation has been
checked and a build script is provided; a Windows executable has not been built
and run on Windows. These checks do not establish compatibility with a particular
Citrix session.

## Icon source

[keyscribe.svg](assets/icons/keyscribe.svg) is the vector source, with no embedded
bitmap or system fonts. It retains the K and cursor design without the raster
texture. `keyscribe-128.png`, `keyscribe.icns` and `keyscribe.ico` are generated
for the UI and platform packaging. Regenerate them with
`node scripts/generate-icons.mjs /path/to/node_modules/sharp` using Node.js and a
separately installed `sharp` (verified with 0.35.5). Normal builds and the app do
not need Node.js. `.DS_Store`, Finder `Icon` and `Icon` ending in CR are ignored
at every Git directory level and excluded from packages. Keep local folder icons.

## Author

Denis Gutsuliak · [d9911.org](https://d9911.org).

## License

Read the complete terms in [LICENSE](LICENSE).

[![License](https://img.shields.io/badge/license-see_LICENSE-blue)](LICENSE)


## Native installers and Ubuntu X11

Updated packaging: `make setup`, `make doctor`, `make build`, `make local`, `make run`; Windows also has `./package.ps1`. Ubuntu X11 input and DEB packaging are added. See [INSTALLERS.md](docs/INSTALLERS.md) for current prerequisites, platform-specific emergency shortcuts and verification limits; this updates the earlier macOS/Windows-only packaging description.
