use crate::appearance;
use eframe::egui;
use keyscribe::{
    cycle::{Cycle, MouseButton, Step},
    motion::Motion,
    shortcut::{NamedKey, Shortcut, Trigger},
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Text,
    Mouse,
    Cycle,
}

#[derive(Default)]
pub struct AutomationUi {
    pub mode: Mode,
    pub cycle: Cycle,
    pub motion: Motion,
    pub capture_target: Option<usize>,
    pub capture_deadline: Option<Instant>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Press,
    Click,
    Move,
    Wait,
}

impl AutomationUi {
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        match self.mode {
            Mode::Text => false,
            Mode::Mouse => self.show_mouse(ui),
            Mode::Cycle => self.show_cycle(ui),
        }
    }

    fn show_mouse(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let notice = concat!(
            "Точки, изгибы, длительность перемещения и паузы выбираются заново. ",
            "Заранее записанной траектории нет. Выход курсора из области останавливает режим."
        );
        appearance::card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            appearance::section(
                ui,
                "Область движения",
                "Центр — положение курсора в конце отсчёта",
            );
            ui.label("Движения без кликов, только на основном экране.");
            ui.horizontal_wrapped(|ui| {
                ui.label("Радиус по X:");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.motion.radius_x).range(5..=2000))
                    .changed();
                ui.label("Радиус по Y:");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.motion.radius_y).range(5..=2000))
                    .changed();
            });
            changed |= ui
                .checkbox(
                    &mut self.motion.until_stopped,
                    "Работать до ручной остановки",
                )
                .changed();
            ui.add_enabled_ui(!self.motion.until_stopped, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Длительность, с:");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.motion.seconds).range(1..=86400))
                        .changed();
                });
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Пауза между движениями, мс: от");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.motion.min_pause_ms).range(0..=60000))
                    .changed();
                ui.label("до");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.motion.max_pause_ms).range(0..=60000))
                    .changed();
            });
            ui.add_space(8.0);
            ui.small(notice);
            ui.add_space(12.0);
            show_motion_area(ui, self.motion.radius_x, self.motion.radius_y);
        });
        changed
    }

    fn show_cycle(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let notice = concat!(
            "Сценарий выполняет шаги в заданном порядке. ",
            "Координаты относятся к основному экрану. ",
            "Клавиши букв соответствуют физическим клавишам US; ",
            "раскладку для сочетаний выбери внутри Citrix."
        );
        appearance::card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            appearance::section(ui, "Параметры цикла", "Шаги выполняются сверху вниз");
            ui.horizontal_wrapped(|ui| {
                ui.label("Повторов:");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.cycle.repetitions).range(1..=10000))
                    .changed();
                ui.label("Между повторами, мс:");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.cycle.between_ms).range(0..=3600000))
                    .changed();
            });
            ui.small(notice);
        });
        ui.add_space(12.0);
        let mut edit = None;
        let count = self.cycle.steps.len();
        for (index, step) in self.cycle.steps.iter_mut().enumerate() {
            ui.push_id(index, |ui| {
                appearance::card(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Шаг {}", index + 1))
                                .strong()
                                .color(appearance::palette(ui).accent),
                        );
                        let mut kind = match step {
                            Step::Press(_) => Kind::Press,
                            Step::Click { .. } => Kind::Click,
                            Step::Move { .. } => Kind::Move,
                            Step::Wait(_) => Kind::Wait,
                        };
                        let original = kind;
                        egui::ComboBox::from_id_salt("kind")
                            .selected_text(step.label())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut kind, Kind::Press, "Клавиши");
                                ui.selectable_value(&mut kind, Kind::Click, "Клик");
                                ui.selectable_value(&mut kind, Kind::Move, "Движение мыши");
                                ui.selectable_value(&mut kind, Kind::Wait, "Пауза");
                            });
                        if kind != original {
                            *step = match kind {
                                Kind::Press => Step::Press(Shortcut {
                                    control: false,
                                    alt: false,
                                    shift: false,
                                    meta: false,
                                    trigger: Some(Trigger::Named(NamedKey::Tab)),
                                }),
                                Kind::Click => Step::Click {
                                    x: 100,
                                    y: 100,
                                    button: MouseButton::Left,
                                    double: false,
                                },
                                Kind::Move => Step::Move { x: 100, y: 100 },
                                Kind::Wait => Step::Wait(500),
                            };
                            changed = true;
                        }
                        if ui
                            .add_enabled(index > 0, egui::Button::new("Выше"))
                            .clicked()
                        {
                            edit = Some((index, 0));
                        }
                        if ui
                            .add_enabled(index + 1 < count, egui::Button::new("Ниже"))
                            .clicked()
                        {
                            edit = Some((index, 1));
                        }
                        if ui
                            .add_enabled(count < 200, egui::Button::new("Копия"))
                            .clicked()
                        {
                            edit = Some((index, 2));
                        }
                        if ui.button("Удалить").clicked() {
                            edit = Some((index, 3));
                        }
                    });
                    match step {
                        Step::Press(shortcut) => {
                            let before = *shortcut;
                            ui.horizontal_wrapped(|ui| {
                                ui.checkbox(&mut shortcut.control, "Ctrl");
                                ui.checkbox(&mut shortcut.alt, "Alt");
                                ui.checkbox(&mut shortcut.shift, "Shift");
                                ui.checkbox(&mut shortcut.meta, "Win");
                                egui::ComboBox::from_id_salt("key")
                                    .selected_text(shortcut.trigger.map_or_else(
                                        || "Выбери клавишу".into(),
                                        |key| key.to_string(),
                                    ))
                                    .show_ui(ui, |ui| {
                                        for key in NamedKey::ALL {
                                            ui.selectable_value(
                                                &mut shortcut.trigger,
                                                Some(Trigger::Named(key)),
                                                format!("{key:?}"),
                                            );
                                        }
                                        ui.selectable_value(
                                            &mut shortcut.trigger,
                                            Some(Trigger::Space),
                                            "Space",
                                        );
                                        for letter in 'a'..='z' {
                                            ui.selectable_value(
                                                &mut shortcut.trigger,
                                                Some(Trigger::Letter(letter)),
                                                letter.to_ascii_uppercase().to_string(),
                                            );
                                        }
                                        for number in 0..=9 {
                                            ui.selectable_value(
                                                &mut shortcut.trigger,
                                                Some(Trigger::Digit(number)),
                                                number.to_string(),
                                            );
                                        }
                                        for number in 1..=12 {
                                            ui.selectable_value(
                                                &mut shortcut.trigger,
                                                Some(Trigger::Function(number)),
                                                format!("F{number}"),
                                            );
                                        }
                                    });
                            });
                            changed |= before != *shortcut;
                        }
                        Step::Wait(ms) => {
                            ui.horizontal(|ui| {
                                ui.label("Пауза, мс:");
                                changed |= ui
                                    .add(egui::DragValue::new(ms).range(10..=3600000))
                                    .changed();
                            });
                        }
                        Step::Click { x, y, .. } | Step::Move { x, y } => {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("X:");
                                changed |=
                                    ui.add(egui::DragValue::new(x).range(0..=30000)).changed();
                                ui.label("Y:");
                                changed |=
                                    ui.add(egui::DragValue::new(y).range(0..=30000)).changed();
                                if ui
                                    .button(format!("Взять координаты: {}", crate::CAPTURE_LABEL))
                                    .clicked()
                                {
                                    self.capture_target = Some(index);
                                    self.capture_deadline = None;
                                }
                                if ui.button("Захват через 3 с").clicked() {
                                    self.capture_target = Some(index);
                                    self.capture_deadline =
                                        Some(Instant::now() + Duration::from_secs(3));
                                }
                            });
                            if let Step::Click { button, double, .. } = step {
                                ui.horizontal_wrapped(|ui| {
                                    for (value, label) in [
                                        (MouseButton::Left, "Левая"),
                                        (MouseButton::Right, "Правая"),
                                        (MouseButton::Middle, "Средняя"),
                                    ] {
                                        changed |=
                                            ui.selectable_value(button, value, label).changed();
                                    }
                                    changed |= ui.checkbox(double, "Двойной клик").changed();
                                });
                            }
                        }
                    }
                });
            });
            ui.add_space(8.0);
        }
        if let Some((index, action)) = edit {
            match action {
                0 => self.cycle.steps.swap(index, index - 1),
                1 => self.cycle.steps.swap(index, index + 1),
                2 => {
                    let copy = self.cycle.steps[index].clone();
                    self.cycle.steps.insert(index + 1, copy);
                }
                _ => {
                    self.cycle.steps.remove(index);
                }
            }
            self.capture_target = None;
            self.capture_deadline = None;
            changed = true;
        }
        if ui
            .add_enabled(
                self.cycle.steps.len() < 200,
                egui::Button::new("Добавить шаг"),
            )
            .clicked()
        {
            self.cycle.steps.push(Step::Wait(500));
            self.capture_target = None;
            self.capture_deadline = None;
            changed = true;
        }
        if let Some(index) = self.capture_target {
            ui.label(format!("Для шага {}: наведи курсор на нужную точку в целевом окне и нажми {}. Выполнение сценария при этом не начинается.", index + 1, crate::CAPTURE_LABEL));
        }
        if let Some(deadline) = self.capture_deadline {
            ui.label(format!(
                "Снимок координат через {:.1} с — наведи курсор на нужную точку.",
                deadline
                    .saturating_duration_since(Instant::now())
                    .as_secs_f32()
            ));
        }
        changed
    }

    pub fn capture(&mut self, x: i32, y: i32) -> Result<(), String> {
        self.capture_deadline = None;
        let index = self
            .capture_target
            .take()
            .ok_or("Сначала нажми кнопку захвата координат у нужного шага.")?;
        match self.cycle.steps.get_mut(index) {
            Some(
                Step::Click {
                    x: target_x,
                    y: target_y,
                    ..
                }
                | Step::Move {
                    x: target_x,
                    y: target_y,
                },
            ) => {
                *target_x = x;
                *target_y = y;
                Ok(())
            }
            _ => Err("Шаг для координат уже изменён.".into()),
        }
    }
}

fn show_motion_area(ui: &mut egui::Ui, radius_x: i32, radius_y: i32) {
    let p = appearance::palette(ui);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 150.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, 10, p.background);
    let scale = ((rect.width() - 70.0) / (radius_x * 2) as f32).min(90.0 / (radius_y * 2) as f32);
    let size = egui::vec2((radius_x * 2) as f32 * scale, (radius_y * 2) as f32 * scale);
    let center = rect.center() - egui::vec2(0.0, 10.0);
    let area = egui::Rect::from_center_size(center, size);
    painter.rect_filled(area, 6, p.accent_soft);
    painter.rect_stroke(
        area,
        6,
        egui::Stroke::new(1.0, p.accent),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [center - egui::vec2(6.0, 0.0), center + egui::vec2(6.0, 0.0)],
        egui::Stroke::new(1.5, p.accent),
    );
    painter.line_segment(
        [center - egui::vec2(0.0, 6.0), center + egui::vec2(0.0, 6.0)],
        egui::Stroke::new(1.5, p.accent),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 18.0),
        egui::Align2::CENTER_CENTER,
        format!("Область {} × {} px · схема", radius_x * 2, radius_y * 2),
        egui::FontId::proportional(12.0),
        p.muted,
    );
}
