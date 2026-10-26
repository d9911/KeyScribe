use crate::{automation_ui::Mode, typing_setup::TypingSetup, App};
use eframe::egui::{self, RichText, Stroke, StrokeKind};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Field {
    Environment,
    Language,
    Shortcut,
    Text,
    Confirmation,
    Hotkey,
    Permissions,
}

impl Field {
    fn message(self) -> &'static str {
        match self {
            Self::Environment => "Выбери окружение: где запущен Keyscribe — macOS или Windows.",
            Self::Language => "Проверь язык внутри целевого окна и выбери English US или русскую ЙЦУКЕН.",
            Self::Shortcut => "Выбери сочетание переключения языка: минимум две клавиши с модификатором.",
            Self::Text => "Вставь текст, который нужно набрать в целевом окне.",
            Self::Confirmation => "Проверь настройки и отметь этот пункт. Затем нажми кнопку запуска ещё раз.",
            Self::Hotkey => if cfg!(target_os = "linux") { "Освободи Ctrl+Shift+F12 и Ctrl+Shift+Pause, затем перезапусти Keyscribe." } else { "Для запуска нужна быстрая остановка. Освободи Ctrl+Alt+F12 и перезапусти Keyscribe." },
            Self::Permissions => "Сначала разреши управление в настройках macOS. Если галочка уже включена, используй восстановление ниже.",
        }
    }
}

#[derive(Default)]
pub(crate) struct Guidance {
    target: Option<Field>,
    started: Option<Instant>,
    scroll_pending: bool,
}

impl Guidance {
    pub(crate) fn point_to(&mut self, target: Option<Field>) {
        self.target = target;
        self.started = target.map(|_| Instant::now());
        self.scroll_pending = target.is_some();
    }

    pub(crate) fn follow(&mut self, target: Option<Field>) {
        if self.target.is_some() && self.target != target {
            self.point_to(target);
        }
    }
}

// Three gentle pulses over 2.4 seconds; keep a steady outline afterwards.
fn pulse_strength(elapsed: Duration) -> f32 {
    let seconds = elapsed.as_secs_f32();
    if seconds >= 2.4 {
        return 0.0;
    }
    (1.0 - (seconds / 0.8 * std::f32::consts::TAU).cos()) / 2.0
}

pub(crate) fn next_missing(
    mode: Mode,
    setup: &TypingSetup,
    auto_switch: bool,
    text: &str,
    ready: bool,
    hotkeys_available: bool,
) -> Option<Field> {
    if mode != Mode::Mouse {
        if setup.environment.is_none() {
            return Some(Field::Environment);
        }
        if setup.initial_layout.is_none() {
            return Some(Field::Language);
        }
    }
    if mode == Mode::Text {
        if auto_switch && !setup.shortcut.is_valid() {
            return Some(Field::Shortcut);
        }
        if text.is_empty() {
            return Some(Field::Text);
        }
    }
    if !ready {
        return Some(Field::Confirmation);
    }
    if !hotkeys_available {
        return Some(Field::Hotkey);
    }
    None
}

impl App {
    pub(crate) fn next_required_field(&self) -> Option<Field> {
        next_missing(
            self.automation.mode,
            &self.typing_setup,
            self.auto_switch,
            &self.text,
            self.ready,
            self.hotkeys.is_some(),
        )
    }

    pub(crate) fn follow_guidance(&mut self) {
        let next = if self.permissions.dialog_open && cfg!(target_os = "macos") {
            Some(Field::Permissions)
        } else {
            self.next_required_field()
        };
        self.guidance.follow(next);
    }

    pub(crate) fn guided_field<R>(
        &mut self,
        ui: &mut egui::Ui,
        field: Field,
        show: impl FnOnce(&mut Self, &mut egui::Ui) -> R,
    ) -> R {
        let active = self.guidance.target == Some(field);
        let response = egui::Frame::new().inner_margin(6).show(ui, |ui| {
            let result = show(self, ui);
            if active {
                ui.label(
                    RichText::new(field.message())
                        .small()
                        .strong()
                        .color(ui.visuals().error_fg_color),
                );
            }
            result
        });
        if active {
            let elapsed = self
                .guidance
                .started
                .map_or(Duration::ZERO, |start| start.elapsed());
            let pulse = pulse_strength(elapsed);
            ui.painter().rect_stroke(
                response.response.rect,
                10,
                Stroke::new(
                    1.5 + pulse * 1.5,
                    ui.visuals()
                        .error_fg_color
                        .gamma_multiply(0.55 + pulse * 0.45),
                ),
                StrokeKind::Inside,
            );
            if elapsed < Duration::from_millis(2400) {
                ui.ctx().request_repaint_after(Duration::from_millis(16));
            }
            if self.guidance.scroll_pending {
                // Scroll to the top of tall blocks instead of hiding their heading.
                let mut top = response.response.rect;
                top.max.y = top.max.y.min(top.min.y + 80.0);
                ui.scroll_to_rect(top, Some(egui::Align::Min));
                self.guidance.scroll_pending = false;
            }
        }
        response.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing_setup::Environment;
    use keyscribe::keyboard::Layout;

    #[test]
    fn guides_a_text_run_through_required_choices_without_auto_confirmation() {
        let mut setup = TypingSetup::default();
        let missing = |setup: &TypingSetup, text, ready| {
            next_missing(Mode::Text, setup, true, text, ready, true)
        };
        assert_eq!(missing(&setup, "", false), Some(Field::Environment));
        setup.select_environment(Environment::MacOs);
        assert_eq!(missing(&setup, "", false), Some(Field::Language));
        setup.initial_layout = Some(Layout::Us);
        assert_eq!(missing(&setup, "", false), Some(Field::Text));
        assert_eq!(missing(&setup, "hello", false), Some(Field::Confirmation));
        assert_eq!(missing(&setup, "hello", true), None);
        setup.require_current_language();
        assert_eq!(missing(&setup, "hello", false), Some(Field::Language));
    }

    #[test]
    fn requirements_match_the_selected_mode_and_shortcut_policy() {
        let mut setup = TypingSetup::default();
        assert_eq!(
            next_missing(Mode::Mouse, &setup, true, "", false, true),
            Some(Field::Confirmation)
        );
        assert_eq!(
            next_missing(Mode::Mouse, &setup, true, "", true, false),
            Some(Field::Hotkey)
        );
        setup.select_environment(Environment::Windows);
        setup.initial_layout = Some(Layout::Russian);
        setup.shortcut = keyscribe::shortcut::Shortcut {
            control: false,
            alt: false,
            shift: false,
            meta: false,
            trigger: None,
        };
        assert_eq!(
            next_missing(Mode::Text, &setup, true, "hello", true, true),
            Some(Field::Shortcut)
        );
        assert_eq!(
            next_missing(Mode::Text, &setup, false, "hello", true, true),
            None
        );
        assert_eq!(
            next_missing(Mode::Cycle, &setup, true, "", true, true),
            None
        );
    }

    #[test]
    fn guidance_only_follows_after_an_attempt_and_can_be_restarted() {
        let mut guidance = Guidance::default();
        guidance.follow(Some(Field::Environment));
        assert_eq!(guidance.target, None);
        guidance.point_to(Some(Field::Environment));
        guidance.scroll_pending = false;
        guidance.follow(Some(Field::Language));
        assert_eq!(guidance.target, Some(Field::Language));
        assert!(guidance.scroll_pending);
        guidance.follow(None);
        assert_eq!(guidance.target, None);
        guidance.point_to(Some(Field::Confirmation));
        assert_eq!(guidance.target, Some(Field::Confirmation));
    }

    #[test]
    fn outline_pulses_exactly_three_times_then_stays_still() {
        for ms in [400, 1200, 2000] {
            assert!((pulse_strength(Duration::from_millis(ms)) - 1.0).abs() < 0.001);
        }
        for ms in [0, 800, 1600, 2400, 3200, 10000] {
            assert!(pulse_strength(Duration::from_millis(ms)).abs() < 0.001);
        }
    }
}
