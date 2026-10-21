use crate::keyboard::Stroke;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    Control,
    Alt,
    Shift,
    Meta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Space,
    Digit(u8),
    Function(u8),
    Letter(char),
    Named(NamedKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedKey {
    Enter,
    Tab,
    Escape,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
}

impl NamedKey {
    pub const ALL: [Self; 13] = [
        Self::Enter,
        Self::Tab,
        Self::Escape,
        Self::Backspace,
        Self::Delete,
        Self::Left,
        Self::Right,
        Self::Up,
        Self::Down,
        Self::Home,
        Self::End,
        Self::PageUp,
        Self::PageDown,
    ];
}

impl Trigger {
    pub fn stroke(self) -> Option<Stroke> {
        let (windows, mac) = match self {
            Self::Space => (0x39, 0x31),
            Self::Digit(number @ 0..=9) => (
                if number == 0 {
                    0x0b
                } else {
                    u16::from(number) + 1
                },
                [0x1d, 0x12, 0x13, 0x14, 0x15, 0x17, 0x16, 0x1a, 0x1c, 0x19][usize::from(number)],
            ),
            Self::Function(number @ 1..=12) => (
                match number {
                    11 => 0x57,
                    12 => 0x58,
                    _ => 0x3a + u16::from(number),
                },
                [
                    0x7a, 0x78, 0x63, 0x76, 0x60, 0x61, 0x62, 0x64, 0x65, 0x6d, 0x67, 0x6f,
                ][usize::from(number - 1)],
            ),
            Self::Letter(letter) if letter.is_ascii_alphabetic() => {
                return crate::keyboard::prepare(
                    &letter.to_ascii_lowercase().to_string(),
                    crate::keyboard::Layout::Us,
                )
                .ok()?
                .into_iter()
                .next()
            }
            Self::Named(key) => match key {
                NamedKey::Enter => (0x1c, 0x24),
                NamedKey::Tab => (0x0f, 0x30),
                NamedKey::Escape => (0x01, 0x35),
                NamedKey::Backspace => (0x0e, 0x33),
                NamedKey::Delete => (0xe053, 0x75),
                NamedKey::Left => (0xe04b, 0x7b),
                NamedKey::Right => (0xe04d, 0x7c),
                NamedKey::Up => (0xe048, 0x7e),
                NamedKey::Down => (0xe050, 0x7d),
                NamedKey::Home => (0xe047, 0x73),
                NamedKey::End => (0xe04f, 0x77),
                NamedKey::PageUp => (0xe049, 0x74),
                NamedKey::PageDown => (0xe051, 0x79),
            },
            _ => return None,
        };
        Some(Stroke {
            windows,
            mac,
            shift: false,
        })
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Space => f.write_str("Space"),
            Self::Digit(n) => write!(f, "{n}"),
            Self::Function(n) => write!(f, "F{n}"),
            Self::Letter(letter) => write!(f, "{}", letter.to_ascii_uppercase()),
            Self::Named(key) => write!(f, "{key:?}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutKey {
    Modifier(Modifier),
    Key(Stroke),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub trigger: Option<Trigger>,
}

impl Default for Shortcut {
    fn default() -> Self {
        Self::ALT_SHIFT
    }
}

impl Shortcut {
    pub const ALT_SHIFT: Self = Self {
        control: false,
        alt: true,
        shift: true,
        meta: false,
        trigger: None,
    };
    pub const CTRL_SHIFT: Self = Self {
        control: true,
        alt: false,
        ..Self::ALT_SHIFT
    };
    pub const WIN_SPACE: Self = Self {
        control: false,
        alt: false,
        shift: false,
        meta: true,
        trigger: Some(Trigger::Space),
    };
    pub const CTRL_WIN_SPACE: Self = Self {
        control: true,
        ..Self::WIN_SPACE
    };
    pub const CTRL_OPTION_SPACE: Self = Self {
        control: true,
        alt: true,
        shift: false,
        meta: false,
        trigger: Some(Trigger::Space),
    };

    pub const LAYOUT_PRESETS: [(Self, &'static str); 5] = [
        (Self::CTRL_OPTION_SPACE, "Control+Option+Space (Mac)"),
        (Self::WIN_SPACE, "Win+Space"),
        (Self::CTRL_WIN_SPACE, "Ctrl+Win+Space"),
        (Self::ALT_SHIFT, "Alt+Shift"),
        (Self::CTRL_SHIFT, "Ctrl+Shift"),
    ];

    pub fn layout_label(self) -> &'static str {
        Self::LAYOUT_PRESETS
            .into_iter()
            .find_map(|(shortcut, label)| (self == shortcut).then_some(label))
            .unwrap_or("Своё сочетание")
    }

    pub fn keys(self) -> Vec<ShortcutKey> {
        let mut keys = Vec::new();
        for (enabled, modifier) in [
            (self.control, Modifier::Control),
            (self.alt, Modifier::Alt),
            (self.shift, Modifier::Shift),
            (self.meta, Modifier::Meta),
        ] {
            if enabled {
                keys.push(ShortcutKey::Modifier(modifier));
            }
        }
        if let Some(stroke) = self.trigger.and_then(Trigger::stroke) {
            keys.push(ShortcutKey::Key(stroke));
        }
        keys
    }

    pub fn is_valid(self) -> bool {
        // A language shortcut must have a modifier and at least two keys.
        (self.control || self.alt || self.shift || self.meta)
            && self.trigger.is_none_or(|key| key.stroke().is_some())
            && self.keys().len() >= 2
            && !self.is_reserved()
    }

    pub fn is_reserved(self) -> bool {
        self.control && self.alt && matches!(self.trigger, Some(Trigger::Function(11 | 12)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_and_custom_shortcuts_preserve_modifier_order() {
        assert_eq!(
            Shortcut::ALT_SHIFT.keys(),
            [
                ShortcutKey::Modifier(Modifier::Alt),
                ShortcutKey::Modifier(Modifier::Shift)
            ]
        );
        let custom = Shortcut {
            trigger: Some(Trigger::Digit(2)),
            ..Shortcut::CTRL_SHIFT
        };
        assert_eq!(
            custom.keys(),
            [
                ShortcutKey::Modifier(Modifier::Control),
                ShortcutKey::Modifier(Modifier::Shift),
                ShortcutKey::Key(Stroke {
                    windows: 3,
                    mac: 0x13,
                    shift: false
                })
            ]
        );
        for shortcut in [
            Shortcut::ALT_SHIFT,
            Shortcut::CTRL_SHIFT,
            Shortcut::WIN_SPACE,
            Shortcut::CTRL_WIN_SPACE,
            Shortcut::CTRL_OPTION_SPACE,
            custom,
        ] {
            assert!(shortcut.is_valid());
        }
    }

    #[test]
    fn invalid_shortcuts_cannot_start_input() {
        assert!(!Shortcut {
            shift: false,
            ..Shortcut::CTRL_SHIFT
        }
        .is_valid());
        assert!(!Shortcut {
            trigger: Some(Trigger::Function(13)),
            ..Shortcut::ALT_SHIFT
        }
        .is_valid());
        assert!(!Shortcut {
            trigger: Some(Trigger::Digit(10)),
            ..Shortcut::ALT_SHIFT
        }
        .is_valid());
        for function in [11, 12] {
            for (shift, meta) in [(false, false), (true, false), (false, true), (true, true)] {
                assert!(!Shortcut {
                    control: true,
                    alt: true,
                    shift,
                    meta,
                    trigger: Some(Trigger::Function(function)),
                }
                .is_valid());
            }
        }
    }
}
