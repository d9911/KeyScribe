use crate::{
    appearance::{self, Theme},
    automation_ui::Mode,
    typing_setup::Environment,
    App,
};
use eframe::egui::{self, RichText, Stroke};
use keyscribe::{
    keyboard::Layout,
    shortcut::{Shortcut, Trigger},
};
use std::sync::atomic::Ordering;

impl App {
    pub(crate) fn show_workspace(&mut self, ctx: &egui::Context, running: bool) {
        let p = self.theme.palette();
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .inner_margin(egui::Margin::symmetric(24, 16)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(&self.brand_icon)
                            .fit_to_exact_size(egui::vec2(46.0, 46.0)),
                    );
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Keyscribe").size(22.0).strong());
                        ui.label(
                            RichText::new("Клавиши. Мышь. Меньше рутины.")
                                .size(13.0)
                                .color(p.muted),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if cfg!(target_os = "macos")
                            && ui
                                .add_enabled(
                                    !running,
                                    egui::Button::new(if self.permissions.access.allowed() {
                                        "Доступ macOS ✓"
                                    } else {
                                        "Доступ macOS…"
                                    }),
                                )
                                .clicked()
                        {
                            self.permissions.refresh();
                            self.permissions.dialog_open = true;
                            self.ready = false;
                        }
                        let previous = self.theme;
                        ui.selectable_value(&mut self.theme, Theme::Dark, "Тёмная");
                        ui.selectable_value(&mut self.theme, Theme::Light, "Светлая");
                        ui.label(RichText::new("Тема").small().color(p.muted));
                        if previous != self.theme {
                            self.theme.apply(ctx);
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("run-controls")
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .inner_margin(egui::Margin::symmetric(24, 14)),
            )
            .show(ctx, |ui| self.show_run_controls(ui, running));

        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(192.0)
            .frame(egui::Frame::new().fill(p.surface).inner_margin(16))
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new("РАБОЧЕЕ ПРОСТРАНСТВО")
                        .size(10.0)
                        .color(p.muted),
                );
                ui.add_space(10.0);
                ui.add_enabled_ui(!running, |ui| {
                    for (mode, title, caption) in [
                        (Mode::Text, "Набор текста", "English + Русский"),
                        (Mode::Mouse, "Движение мыши", "Случайные траектории"),
                        (Mode::Cycle, "Цикл действий", "Клавиши и клики"),
                    ] {
                        let active = self.automation.mode == mode;
                        let response = ui.add_sized(
                            [160.0, 42.0],
                            egui::Button::new(RichText::new(title).strong())
                                .fill(if active { p.accent_soft } else { p.surface })
                                .stroke(Stroke::new(1.0, if active { p.accent } else { p.surface }))
                                .corner_radius(10),
                        );
                        ui.label(RichText::new(caption).size(11.0).color(p.muted));
                        ui.add_space(12.0);
                        if response.clicked() && !active {
                            self.automation.mode = mode;
                            self.ready = false;
                            self.automation.capture_target = None;
                            self.automation.capture_deadline = None;
                            self.sent = 0;
                            self.total = 0;
                            self.status = match mode {
                                Mode::Text => "Готово к вводу текста",
                                Mode::Mouse => "Готово к движению мыши",
                                Mode::Cycle => "Готово к выполнению сценария",
                            }
                            .into();
                        }
                    }
                });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(
                        RichText::new("Возврат в Keyscribe\nостанавливает выполнение.")
                            .size(12.0)
                            .color(p.muted),
                    );
                    ui.separator();
                    ui.label(RichText::new("Ctrl + Alt + F12").monospace().size(12.0));
                    ui.label(RichText::new("Быстрая остановка").size(12.0).strong());
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(p.background).inner_margin(0))
            .show(ctx, |ui| {
                ui.spacing_mut().scroll = egui::style::ScrollStyle {
                    bar_width: 8.0,
                    bar_inner_margin: 12.0,
                    bar_outer_margin: 12.0,
                    handle_min_length: 36.0,
                    foreground_color: true,
                    ..egui::style::ScrollStyle::solid()
                };
                egui::ScrollArea::vertical()
                    .id_salt(("workspace", self.automation.mode as u8))
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(egui::Margin {
                                left: 24,
                                right: 12,
                                top: 24,
                                bottom: 32,
                            })
                            .show(ui, |ui| {
                                self.show_mode_content(ui, running);
                            });
                    });
            });
    }

    fn show_mode_content(&mut self, ui: &mut egui::Ui, running: bool) {
        let p = appearance::palette(ui);
        let (title, caption) = match self.automation.mode {
            Mode::Text => (
                "Точный текст, привычные клавиши",
                "Вставь текст, настрой раскладки и выбери целевое окно после запуска.",
            ),
            Mode::Mouse => (
                "Движение в заданных границах",
                "Разные точки, изгибы и паузы — внутри выбранной области.",
            ),
            Mode::Cycle => (
                "Один сценарий. Нужное число повторов.",
                "Собери последовательность клавиш, кликов и пауз для повторяющейся задачи.",
            ),
        };
        ui.heading(title);
        ui.label(RichText::new(caption).color(p.muted));
        ui.add_space(16.0);
        ui.add_enabled_ui(!running, |ui| {
            if self.automation.mode == Mode::Text {
                if ui.available_width() >= 790.0 {
                    let editor_width = ui.available_width() - 350.0;
                    ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(editor_width);
                            self.show_editor(ui);
                        });
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.set_width(332.0);
                            self.show_text_settings(ui);
                        });
                    });
                } else {
                    self.show_editor(ui);
                    ui.add_space(10.0);
                    self.show_text_settings(ui);
                }
            } else {
                if self.automation.mode == Mode::Cycle {
                    appearance::card(ui).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.show_input_context(ui);
                        ui.small("Клавиши цикла используют текущую раскладку целевого окна. Автопереключение текста к циклу не применяется.");
                    });
                    ui.add_space(12.0);
                }
                if self.automation.show(ui) {
                    self.ready = false;
                }
                ui.add_space(12.0);
                appearance::card(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    self.show_timing(ui);
                });
                if let Some(error) = &self.capture_error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            }
        });
        ui.add_space(12.0);
    }

    fn show_editor(&mut self, ui: &mut egui::Ui) {
        let p = appearance::palette(ui);
        let hint = concat!(
            "Вставь текст сюда…\n\n",
            "English, русский, цифры и символы.\n",
            "Переносы строк и пробелы сохранятся."
        );
        let notice = concat!(
            "Enter и Tab отправляются как клавиши: ",
            "они могут выполнить команду или сменить поле. ",
            "Автоотступы и автозакрытие скобок в редакторе меняют результат."
        );
        appearance::card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Текст для набора").size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Очистить").clicked() {
                        self.text.clear();
                        self.ready = false;
                    }
                });
            });
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .id_salt("source")
                .max_height(340.0)
                .show(ui, |ui| {
                    if ui
                        .add(
                            egui::TextEdit::multiline(&mut self.text)
                                .font(egui::TextStyle::Monospace)
                                .margin(16)
                                .hint_text(hint)
                                .desired_width(f32::INFINITY)
                                .desired_rows(17)
                                .lock_focus(true),
                        )
                        .changed()
                    {
                        self.ready = false;
                    }
                });
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Символов: {}", self.text.chars().count()))
                        .small()
                        .color(p.muted),
                );
                ui.label(
                    RichText::new("Текст хранится только в памяти")
                        .small()
                        .color(p.muted),
                );
            });
            ui.add_space(6.0);
            ui.label(RichText::new(notice).size(12.0).color(p.muted));
        });
    }

    fn show_text_settings(&mut self, ui: &mut egui::Ui) {
        appearance::card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.show_layout_settings(ui);
        });
        ui.add_space(10.0);
        appearance::card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.show_timing(ui);
        });
    }

    fn show_layout_settings(&mut self, ui: &mut egui::Ui) {
        self.show_input_context(ui);
        ui.add_space(6.0);
        if ui
            .checkbox(&mut self.auto_switch, "Переключать US / RU по тексту")
            .changed()
        {
            self.ready = false;
        }
        if self.auto_switch {
            self.show_switch_settings(ui);
        }
        egui::CollapsingHeader::new("Настройки Citrix на Mac").show(ui, |ui| {
            ui.small("Режим Scancode, синхронизация Don't sync. Сочетание должно менять язык внутри сессии.");
        });
    }

    fn show_input_context(&mut self, ui: &mut egui::Ui) {
        appearance::section(ui, "Окружение и язык", "Выбери перед запуском");
        ui.label("Где работает Keyscribe");
        let previous = self.typing_setup.environment;
        let mut environment = previous;
        egui::ComboBox::from_id_salt("input-environment")
            .selected_text(environment.map_or("Выбери окружение…", Environment::label))
            .show_ui(ui, |ui| {
                for value in [Environment::MacOs, Environment::Windows] {
                    ui.selectable_value(&mut environment, Some(value), value.label());
                }
            });
        if environment != previous {
            if let Some(environment) = environment {
                self.typing_setup.select_environment(environment);
            }
            self.ready = false;
        }
        ui.add_space(6.0);
        ui.label("Текущий язык в целевом окне / Citrix");
        ui.horizontal_wrapped(|ui| {
            for (layout, label) in [
                (Layout::Us, "English US"),
                (Layout::Russian, "Русская ЙЦУКЕН"),
            ] {
                if ui
                    .selectable_value(&mut self.typing_setup.initial_layout, Some(layout), label)
                    .changed()
                {
                    self.ready = false;
                }
            }
        });
        if self.typing_setup.initial_layout.is_none() {
            ui.small("Язык не выбран. Проверь его в целевом окне и укажи здесь.");
        }
        ui.small("Раскладки целевого окна: English US и Windows ЙЦУКЕН. Профиль задаёт сочетание, но не меняет настройки системы.");
    }

    fn show_switch_settings(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label("Сочетание внутри сессии");
        let previous = self.typing_setup.shortcut;
        egui::ComboBox::from_id_salt("switch-preset")
            .selected_text(self.typing_setup.shortcut.layout_label())
            .show_ui(ui, |ui| {
                for (shortcut, label) in Shortcut::LAYOUT_PRESETS {
                    ui.selectable_value(&mut self.typing_setup.shortcut, shortcut, label);
                }
            });
        ui.small("Проверь сочетание в целевом окне: оно должно менять удалённый язык, а не язык локального компьютера.");
        egui::CollapsingHeader::new("Своё сочетание").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.typing_setup.shortcut.control, "Ctrl");
                ui.checkbox(&mut self.typing_setup.shortcut.alt, "Alt / Option");
                ui.checkbox(&mut self.typing_setup.shortcut.shift, "Shift");
                ui.checkbox(&mut self.typing_setup.shortcut.meta, "Win / Command");
            });
            egui::ComboBox::from_id_salt("switch-trigger")
                .selected_text(
                    self.typing_setup
                        .shortcut
                        .trigger
                        .map_or_else(|| "Без доп. клавиши".into(), |key| key.to_string()),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.typing_setup.shortcut.trigger,
                        None,
                        "Без доп. клавиши",
                    );
                    ui.selectable_value(
                        &mut self.typing_setup.shortcut.trigger,
                        Some(Trigger::Space),
                        "Space",
                    );
                    for number in 0..=9 {
                        ui.selectable_value(
                            &mut self.typing_setup.shortcut.trigger,
                            Some(Trigger::Digit(number)),
                            number.to_string(),
                        );
                    }
                    for number in 1..=12 {
                        ui.selectable_value(
                            &mut self.typing_setup.shortcut.trigger,
                            Some(Trigger::Function(number)),
                            format!("F{number}"),
                        );
                    }
                });
        });
        if self.typing_setup.shortcut != previous {
            self.ready = false;
        }
        number_field(
            ui,
            "Пауза смены языка",
            &mut self.switch_delay_ms,
            100..=3000,
            " мс",
        );
    }

    fn show_timing(&mut self, ui: &mut egui::Ui) {
        appearance::section(
            ui,
            "Темп выполнения",
            "Увеличь паузы для медленного соединения",
        );
        number_field(ui, "До начала", &mut self.countdown, 3..=60, " с");
        if self.automation.mode != Mode::Mouse {
            number_field(
                ui,
                "Между действиями",
                &mut self.interval_ms,
                20..=2000,
                " мс",
            );
            number_field(ui, "Удержание клавиши", &mut self.hold_ms, 10..=200, " мс");
        }
    }

    fn show_run_controls(&mut self, ui: &mut egui::Ui, running: bool) {
        let p = appearance::palette(ui);
        let context_error = match self.automation.mode {
            Mode::Text => self.typing_setup.validate_text(self.auto_switch).err(),
            Mode::Cycle => self.typing_setup.validate_context().err(),
            Mode::Mouse => None,
        };
        if !running {
            if let Some(error) = context_error {
                self.ready = false;
                ui.label(RichText::new(error).small().color(p.muted));
            }
        }
        ui.add_enabled_ui(!running && context_error.is_none() && self.permissions.can_start(), |ui| {
            ui.checkbox(&mut self.ready, match self.automation.mode {
                Mode::Text if self.auto_switch => "В Citrix только US и RU; начальная раскладка верна, переключение проверено, Caps Lock выключен",
                Mode::Text => "Раскладка в Citrix выбрана верно, Caps Lock выключен",
                Mode::Mouse => "Область вокруг курсора будет внутри нужного окна",
                Mode::Cycle => "Окружение, текущий язык, шаги, число повторов и координаты проверены",
            });
        });
        if let Some(error) = &self.hotkey_error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !running
                        && self.permissions.can_start()
                        && context_error.is_none()
                        && self.ready
                        && (self.automation.mode != Mode::Text || !self.text.is_empty())
                        && self.hotkeys.is_some(),
                    egui::Button::new(
                        RichText::new(match self.automation.mode {
                            Mode::Text => "Начать набор",
                            Mode::Mouse => "Запустить мышь",
                            Mode::Cycle => "Запустить цикл",
                        })
                        .strong()
                        .color(p.on_accent),
                    )
                    .fill(p.accent)
                    .stroke(Stroke::NONE)
                    .corner_radius(24)
                    .min_size(egui::vec2(170.0, 42.0)),
                )
                .clicked()
            {
                self.start();
            }
            if ui
                .add_enabled(
                    running,
                    egui::Button::new("Остановить")
                        .corner_radius(24)
                        .min_size(egui::vec2(124.0, 42.0)),
                )
                .clicked()
            {
                self.cancel.store(true, Ordering::SeqCst);
            }
            ui.vertical(|ui| {
                ui.label(RichText::new(&self.status).strong());
                ui.label(
                    RichText::new("Стоп из любого окна: Ctrl + Alt + F12")
                        .size(12.0)
                        .color(p.muted),
                );
            });
        });
        if self.total > 0 {
            ui.add(
                egui::ProgressBar::new(self.sent as f32 / self.total as f32)
                    .fill(p.accent)
                    .desired_height(18.0)
                    .text(format!(
                        "{}: {} / {}",
                        self.progress_caption, self.sent, self.total
                    )),
            );
        }
    }
}

fn number_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u64,
    range: std::ops::RangeInclusive<u64>,
    unit: &str,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::DragValue::new(value).range(range).suffix(unit));
        });
    });
}
