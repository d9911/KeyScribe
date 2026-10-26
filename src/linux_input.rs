//! X11 physical keycodes resolved by XKB key names, never guessed from Windows scancodes.
use std::collections::HashMap;
use x11rb::protocol::xkb::{ConnectionExt as _, NameDetail, ID};

pub(crate) fn ensure_session() -> Result<(), String> {
    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    if session.eq_ignore_ascii_case("wayland")
        || (std::env::var_os("WAYLAND_DISPLAY").is_some() && !session.eq_ignore_ascii_case("x11"))
    {
        return Err("Wayland: автоматизация отключена, поскольку глобальная аварийная остановка не поддерживается. Выйди из сеанса и выбери Ubuntu on Xorg/X11. XWayland не обеспечивает ввод во все окна Wayland.".into());
    }
    if std::env::var("DISPLAY").unwrap_or_default().is_empty() {
        return Err("Для ввода нужен графический сеанс X11 с DISPLAY.".into());
    }
    Ok(())
}

pub(crate) struct KeyboardMap(HashMap<u16, u16>);

impl KeyboardMap {
    pub(crate) fn load() -> Result<Self, String> {
        ensure_session()?;
        let (connection, _) =
            x11rb::connect(None).map_err(|error| format!("Нет подключения X11: {error}"))?;
        let extension = connection
            .xkb_use_extension(1, 0)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        if !extension.supported {
            return Err("Сервер X11 не поддерживает XKB 1.0.".into());
        }
        let reply = connection
            .xkb_get_names(u16::from(ID::USE_CORE_KBD), NameDetail::KEY_NAMES)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        let names = reply
            .value_list
            .key_names
            .ok_or("XKB не вернул физические имена клавиш.")?;
        let names: HashMap<String, u16> = names
            .into_iter()
            .zip(u16::from(reply.first_key)..)
            .map(|(entry, code)| {
                (
                    String::from_utf8_lossy(&entry.name)
                        .trim_end_matches('\0')
                        .to_owned(),
                    code,
                )
            })
            .collect();
        let mut codes = HashMap::new();
        for windows in (0x01..=0x58).chain([
            0xe053, 0xe04b, 0xe04d, 0xe048, 0xe050, 0xe047, 0xe04f, 0xe049, 0xe051,
        ]) {
            if let Some(name) = physical_name(windows) {
                let code = names.get(&name).copied().ok_or_else(|| {
                    format!("XKB: отсутствует физическая клавиша {name}. Ничего не отправлено.")
                })?;
                codes.insert(windows, code);
            }
        }
        Ok(Self(codes))
    }

    pub(crate) fn code(&self, windows: u16) -> Result<u16, enigo::InputError> {
        self.0.get(&windows).copied().ok_or_else(|| {
            enigo::InputError::Mapping("Нет физического XKB-кода этой клавиши.".into())
        })
    }
}

// XKB physical names describe positions independent of symbols/layout.
// Source: xkeyboard-config/keycodes/evdev; numeric inputs are the existing PC set-1 identifiers.
fn physical_name(code: u16) -> Option<String> {
    let name = match code {
        0x02..=0x0d => return Some(format!("AE{:02}", code - 1)),
        0x10..=0x1b => return Some(format!("AD{:02}", code - 0x0f)),
        0x1e..=0x28 => return Some(format!("AC{:02}", code - 0x1d)),
        0x2c..=0x35 => return Some(format!("AB{:02}", code - 0x2b)),
        0x3b..=0x44 => return Some(format!("FK{:02}", code - 0x3a)),
        0x01 => "ESC",
        0x0e => "BKSP",
        0x0f => "TAB",
        0x1c => "RTRN",
        0x29 => "TLDE",
        0x2b => "BKSL",
        0x39 => "SPCE",
        0x57 => "FK11",
        0x58 => "FK12",
        0xe053 => "DELE",
        0xe04b => "LEFT",
        0xe04d => "RGHT",
        0xe048 => "UP",
        0xe050 => "DOWN",
        0xe047 => "HOME",
        0xe04f => "END",
        0xe049 => "PGUP",
        0xe051 => "PGDN",
        _ => return None,
    };
    Some(name.into())
}
