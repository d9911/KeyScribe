use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    #[default]
    Us,
    Russian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stroke {
    pub windows: u16,
    pub mac: u16,
    pub shift: bool,
}

impl Stroke {
    const fn new(windows: u16, mac: u16, shift: bool) -> Self {
        Self {
            windows,
            mac,
            shift,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Key(Stroke),
    SwitchLayout(Layout),
}

/// A switch assumes exactly two remote layouts and a known starting layout.
pub fn prepare_plan(
    text: &str,
    initial: Layout,
    auto_switch: bool,
) -> Result<Vec<Action>, UnsupportedCharacter> {
    build_plan(text, initial, auto_switch)
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnsupportedCharacter {
    pub character: char,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for UnsupportedCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Символ U+{:04X} не поддерживается выбранным режимом ввода (строка {}, столбец {}). Ничего не отправлено.", self.character as u32, self.line, self.column)
    }
}

impl std::error::Error for UnsupportedCharacter {}

// PC set-1 scancodes and macOS ANSI virtual keycodes for the same physical keys.
// Sources: Microsoft Keyboard Scan Code Specification; Apple Events.h (kVK_ANSI_*).
const KEYS: &[(char, char, u16, u16)] = &[
    ('a', 'A', 0x1e, 0x00),
    ('b', 'B', 0x30, 0x0b),
    ('c', 'C', 0x2e, 0x08),
    ('d', 'D', 0x20, 0x02),
    ('e', 'E', 0x12, 0x0e),
    ('f', 'F', 0x21, 0x03),
    ('g', 'G', 0x22, 0x05),
    ('h', 'H', 0x23, 0x04),
    ('i', 'I', 0x17, 0x22),
    ('j', 'J', 0x24, 0x26),
    ('k', 'K', 0x25, 0x28),
    ('l', 'L', 0x26, 0x25),
    ('m', 'M', 0x32, 0x2e),
    ('n', 'N', 0x31, 0x2d),
    ('o', 'O', 0x18, 0x1f),
    ('p', 'P', 0x19, 0x23),
    ('q', 'Q', 0x10, 0x0c),
    ('r', 'R', 0x13, 0x0f),
    ('s', 'S', 0x1f, 0x01),
    ('t', 'T', 0x14, 0x11),
    ('u', 'U', 0x16, 0x20),
    ('v', 'V', 0x2f, 0x09),
    ('w', 'W', 0x11, 0x0d),
    ('x', 'X', 0x2d, 0x07),
    ('y', 'Y', 0x15, 0x10),
    ('z', 'Z', 0x2c, 0x06),
    ('1', '!', 0x02, 0x12),
    ('2', '@', 0x03, 0x13),
    ('3', '#', 0x04, 0x14),
    ('4', '$', 0x05, 0x15),
    ('5', '%', 0x06, 0x17),
    ('6', '^', 0x07, 0x16),
    ('7', '&', 0x08, 0x1a),
    ('8', '*', 0x09, 0x1c),
    ('9', '(', 0x0a, 0x19),
    ('0', ')', 0x0b, 0x1d),
    ('-', '_', 0x0c, 0x1b),
    ('=', '+', 0x0d, 0x18),
    ('[', '{', 0x1a, 0x21),
    (']', '}', 0x1b, 0x1e),
    (';', ':', 0x27, 0x29),
    ('\'', '"', 0x28, 0x27),
    ('`', '~', 0x29, 0x32),
    ('\\', '|', 0x2b, 0x2a),
    (',', '<', 0x33, 0x2b),
    ('.', '>', 0x34, 0x2f),
    ('/', '?', 0x35, 0x2c),
];

fn us(character: char) -> Option<Stroke> {
    KEYS.iter().find_map(|&(lower, upper, windows, mac)| {
        if character == lower || character == upper {
            Some(Stroke::new(windows, mac, character == upper))
        } else {
            None
        }
    })
}

fn russian(character: char) -> Option<Stroke> {
    // Standard Windows Russian ЙЦУКЕН layout, not the macOS Russian layout.
    const RU: &str = "йцукенгшщзхъфывапролджэячсмитьбюё";
    const US: &str = "qwertyuiop[]asdfghjkl;'zxcvbnm,.`";
    let lower = character.to_lowercase().next()?;
    if let Some(index) = RU.chars().position(|c| c == lower) {
        let key = US.chars().nth(index)?;
        let mut stroke = us(key)?;
        stroke.shift = character != lower;
        return Some(stroke);
    }
    let equivalent = match character {
        '0'..='9' | '-' | '_' | '=' | '+' | '\\' | '!' | '%' | '*' | '(' | ')' => character,
        '"' => '@',
        '№' => '#',
        ';' => '$',
        ':' => '^',
        '?' => '&',
        '/' => '|',
        '.' => '/',
        ',' => '?',
        _ => return None,
    };
    us(equivalent)
}

/// Validate the entire input before any OS events are emitted.
/// CRLF and lone CR each become one Enter. Tabs remain real Tab presses.
pub fn prepare(text: &str, layout: Layout) -> Result<Vec<Stroke>, UnsupportedCharacter> {
    build_plan(text, layout, false).map(|actions| {
        actions
            .into_iter()
            .filter_map(|action| match action {
                Action::Key(stroke) => Some(stroke),
                Action::SwitchLayout(_) => None,
            })
            .collect()
    })
}

fn build_plan(
    text: &str,
    mut layout: Layout,
    auto_switch: bool,
) -> Result<Vec<Action>, UnsupportedCharacter> {
    let mut result = Vec::new();
    let mut chars = text.chars().peekable();
    let (mut line, mut column) = (1, 1);
    while let Some(character) = chars.next() {
        let stroke = match character {
            '\r' | '\n' => {
                if character == '\r' && chars.peek() == Some(&'\n') {
                    chars.next();
                }
                Stroke::new(0x1c, 0x24, false)
            }
            '\t' => Stroke::new(0x0f, 0x30, false),
            ' ' => Stroke::new(0x39, 0x31, false),
            _ => {
                let lookup = |layout| match layout {
                    Layout::Us => us(character),
                    Layout::Russian => russian(character),
                };
                let mut key = lookup(layout);
                if key.is_none() && auto_switch {
                    let other = match layout {
                        Layout::Us => Layout::Russian,
                        Layout::Russian => Layout::Us,
                    };
                    if let Some(other_key) = lookup(other) {
                        layout = other;
                        result.push(Action::SwitchLayout(other));
                        key = Some(other_key);
                    }
                }
                key.ok_or(UnsupportedCharacter {
                    character,
                    line,
                    column,
                })?
            }
        };
        result.push(Action::Key(stroke));
        if matches!(character, '\r' | '\n') {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_all_printable_ascii_and_actual_control_keys() {
        let text: String = (' '..='~').collect();
        assert_eq!(prepare(&text, Layout::Us).expect("ASCII").len(), 95);
        assert_eq!(
            prepare("a\r\nb\rc\n\t", Layout::Us).expect("lines"),
            vec![
                Stroke::new(0x1e, 0, false),
                Stroke::new(0x1c, 0x24, false),
                Stroke::new(0x30, 0x0b, false),
                Stroke::new(0x1c, 0x24, false),
                Stroke::new(0x2e, 8, false),
                Stroke::new(0x1c, 0x24, false),
                Stroke::new(0x0f, 0x30, false),
            ]
        );
        let mixed = include_str!("../samples/mixed.txt");
        let plan = prepare_plan(mixed, Layout::Us, true).expect("mixed user sample");
        assert_eq!(
            plan.iter()
                .filter(|action| matches!(action, Action::Key(_)))
                .count(),
            mixed.chars().count()
        );
    }

    #[test]
    fn uppercase_and_symbols_use_shift_on_the_same_physical_key() {
        assert_eq!(
            prepare("aA1!", Layout::Us).expect("ASCII"),
            vec![
                Stroke::new(0x1e, 0, false),
                Stroke::new(0x1e, 0, true),
                Stroke::new(2, 0x12, false),
                Stroke::new(2, 0x12, true),
            ]
        );
        // Independent expected physical sequence: US A, RU Я, US #, RU ё,
        // one Enter for CRLF, US z, then RU №Ж and common backslash/Tab.
        assert_eq!(
            prepare_plan("AЯ#ё\r\nz№Ж\\\t", Layout::Us, true).expect("mixed"),
            vec![
                Action::Key(Stroke::new(0x1e, 0, true)),
                Action::SwitchLayout(Layout::Russian),
                Action::Key(Stroke::new(0x2c, 6, true)),
                Action::SwitchLayout(Layout::Us),
                Action::Key(Stroke::new(4, 0x14, true)),
                Action::SwitchLayout(Layout::Russian),
                Action::Key(Stroke::new(0x29, 0x32, false)),
                Action::Key(Stroke::new(0x1c, 0x24, false)),
                Action::SwitchLayout(Layout::Us),
                Action::Key(Stroke::new(0x2c, 6, false)),
                Action::SwitchLayout(Layout::Russian),
                Action::Key(Stroke::new(4, 0x14, true)),
                Action::Key(Stroke::new(0x27, 0x29, true)),
                Action::Key(Stroke::new(0x2b, 0x2a, false)),
                Action::Key(Stroke::new(0x0f, 0x30, false)),
            ]
        );
    }

    #[test]
    fn russian_maps_letters_and_windows_punctuation() {
        assert_eq!(
            prepare("фФёЁ№.,", Layout::Russian).expect("Russian"),
            vec![
                Stroke::new(0x1e, 0, false),
                Stroke::new(0x1e, 0, true),
                Stroke::new(0x29, 0x32, false),
                Stroke::new(0x29, 0x32, true),
                Stroke::new(4, 0x14, true),
                Stroke::new(0x35, 0x2c, false),
                Stroke::new(0x35, 0x2c, true),
            ]
        );
        let alphabet = "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя";
        assert_eq!(
            prepare(alphabet, Layout::Russian).expect("alphabet").len(),
            66
        );
    }

    #[test]
    fn rejects_wrong_layout_emoji_and_hidden_controls_before_typing() {
        let error = prepare("ok\r\n🙂", Layout::Us).expect_err("unsupported");
        assert_eq!((error.line, error.column, error.character), (2, 1, '🙂'));
        for text in ["a\0", "a\u{1b}", "Русский", "\u{00a0}"] {
            assert!(prepare(text, Layout::Us).is_err());
        }
        assert!(prepare("Latin", Layout::Russian).is_err());
        assert!(prepare_plan("Latin", Layout::Russian, false).is_err());
        let error = prepare_plan("AЯ\r\n№🙂", Layout::Us, true).expect_err("unsupported Unicode");
        assert_eq!((error.line, error.column, error.character), (2, 2, '🙂'));
    }
}
