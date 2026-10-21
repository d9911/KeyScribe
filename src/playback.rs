use crate::keyboard::Stroke;
use crate::shortcut::{Modifier, Shortcut, ShortcutKey};

/// OS boundary: press/release physical keys, never paste text.
pub trait KeyboardSink {
    type Error;
    fn modifier(&mut self, modifier: Modifier, pressed: bool) -> Result<(), Self::Error>;
    fn key(&mut self, stroke: Stroke, pressed: bool) -> Result<(), Self::Error>;
}

/// Always attempt to release keys, including after cancellation or an OS error.
/// `wait_hold` returns false when cancelled during the key hold.
pub fn emit<S: KeyboardSink>(
    sink: &mut S,
    stroke: Stroke,
    wait_hold: impl FnOnce() -> bool,
) -> Result<bool, S::Error> {
    if stroke.shift {
        if let Err(error) = sink.modifier(Modifier::Shift, true) {
            let _ = sink.modifier(Modifier::Shift, false);
            return Err(error);
        }
    }
    let pressed = sink.key(stroke, true);
    let completed = pressed.is_ok() && wait_hold();
    let released = sink.key(stroke, false);
    let shift_released = if stroke.shift {
        sink.modifier(Modifier::Shift, false)
    } else {
        Ok(())
    };
    pressed?;
    released?;
    shift_released?;
    Ok(completed)
}

/// Release all attempted keys in reverse order, also on cancellation or failure.
pub fn emit_shortcut<S: KeyboardSink>(
    sink: &mut S,
    shortcut: Shortcut,
    mut wait_hold: impl FnMut() -> bool,
) -> Result<bool, S::Error> {
    let mut attempted = Vec::new();
    let mut result = Ok(true);
    for key in shortcut.keys() {
        attempted.push(key);
        if let Err(error) = send(sink, key, true) {
            result = Err(error);
            break;
        }
        if !wait_hold() {
            result = Ok(false);
            break;
        }
    }
    for key in attempted.into_iter().rev() {
        if let Err(error) = send(sink, key, false) {
            if result.is_ok() {
                result = Err(error);
            }
        }
    }
    result
}

fn send<S: KeyboardSink>(sink: &mut S, key: ShortcutKey, pressed: bool) -> Result<(), S::Error> {
    match key {
        ShortcutKey::Modifier(modifier) => sink.modifier(modifier, pressed),
        ShortcutKey::Key(stroke) => sink.key(stroke, pressed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Sink {
        events: Vec<&'static str>,
        fail_down: bool,
    }
    impl KeyboardSink for Sink {
        type Error = &'static str;
        fn modifier(&mut self, modifier: Modifier, pressed: bool) -> Result<(), Self::Error> {
            self.events.push(match (modifier, pressed) {
                (Modifier::Shift, true) => "shift down",
                (Modifier::Shift, false) => "shift up",
                (Modifier::Control, true) => "ctrl down",
                (Modifier::Control, false) => "ctrl up",
                (Modifier::Alt, true) => "alt down",
                (Modifier::Alt, false) => "alt up",
                (Modifier::Meta, true) => "win down",
                (Modifier::Meta, false) => "win up",
            });
            Ok(())
        }
        fn key(&mut self, _: Stroke, pressed: bool) -> Result<(), Self::Error> {
            self.events
                .push(if pressed { "key down" } else { "key up" });
            if pressed && self.fail_down {
                Err("OS error")
            } else {
                Ok(())
            }
        }
    }
    const SHIFTED: Stroke = Stroke {
        windows: 2,
        mac: 0x12,
        shift: true,
    };

    #[test]
    fn cancellation_releases_character_and_shift() {
        let mut sink = Sink::default();
        assert_eq!(emit(&mut sink, SHIFTED, || false), Ok(false));
        assert_eq!(
            sink.events,
            ["shift down", "key down", "key up", "shift up"]
        );
        let mut sink = Sink::default();
        assert_eq!(
            emit_shortcut(&mut sink, Shortcut::ALT_SHIFT, || false),
            Ok(false)
        );
        assert_eq!(sink.events, ["alt down", "alt up"]);
        let mut sink = Sink::default();
        assert_eq!(
            emit_shortcut(&mut sink, Shortcut::ALT_SHIFT, || true),
            Ok(true)
        );
        assert_eq!(
            sink.events,
            ["alt down", "shift down", "shift up", "alt up"]
        );
    }

    #[test]
    fn failed_press_still_attempts_both_releases() {
        let mut sink = Sink {
            fail_down: true,
            ..Sink::default()
        };
        assert_eq!(
            emit(&mut sink, SHIFTED, || panic!("must not wait")),
            Err("OS error")
        );
        assert_eq!(
            sink.events,
            ["shift down", "key down", "key up", "shift up"]
        );
        let mut sink = Sink {
            fail_down: true,
            ..Sink::default()
        };
        assert_eq!(
            emit_shortcut(&mut sink, Shortcut::WIN_SPACE, || true),
            Err("OS error")
        );
        assert_eq!(sink.events, ["win down", "key down", "key up", "win up"]);
        for (shortcut, modifier) in [
            (Shortcut::CTRL_OPTION_SPACE, "alt"),
            (Shortcut::CTRL_WIN_SPACE, "win"),
        ] {
            let mut sink = Sink {
                fail_down: true,
                ..Sink::default()
            };
            assert_eq!(emit_shortcut(&mut sink, shortcut, || true), Err("OS error"));
            assert_eq!(
                sink.events,
                [
                    "ctrl down",
                    if modifier == "alt" {
                        "alt down"
                    } else {
                        "win down"
                    },
                    "key down",
                    "key up",
                    if modifier == "alt" {
                        "alt up"
                    } else {
                        "win up"
                    },
                    "ctrl up",
                ]
            );
        }
    }

    #[test]
    fn successful_plain_key_does_not_touch_modifiers() {
        let mut sink = Sink::default();
        assert_eq!(
            emit(
                &mut sink,
                Stroke {
                    shift: false,
                    ..SHIFTED
                },
                || true
            ),
            Ok(true)
        );
        assert_eq!(sink.events, ["key down", "key up"]);
    }
}
