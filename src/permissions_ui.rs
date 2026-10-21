use crate::{permissions, App};
use eframe::egui::{self, RichText};

impl App {
    pub(crate) fn show_permissions_dialog(&mut self, ctx: &egui::Context) {
        if !self.permissions.dialog_open || !cfg!(target_os = "macos") {
            return;
        }
        let p = self.theme.palette();
        egui::Modal::new(egui::Id::new("macos-input-permissions"))
            .frame(
                egui::Frame::popup(&ctx.style())
                    .fill(p.surface)
                    .inner_margin(24)
                    .corner_radius(16),
            )
            .show(ctx, |ui| {
                ui.set_width(510.0_f32.min(ctx.content_rect().width() - 80.0));
                ui.heading("Разреши управление клавиатурой и мышью");
                ui.label("Keyscribe нужен доступ macOS «Универсальный доступ» (Accessibility), чтобы отправлять клавиши и движения мыши в выбранное окно.");
                ui.add_space(8.0);
                for (label, allowed) in [
                    ("Универсальный доступ", self.permissions.access.accessibility),
                    ("Подключение клавиатуры и мыши", self.permissions.access.backend_ready),
                ] {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(label);
                        ui.label(
                            RichText::new(if allowed { "Готово" } else { "Не готово" })
                                .strong()
                                .color(if allowed { p.accent } else { p.muted }),
                        );
                    });
                }
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .max_height((ctx.content_rect().height() - 340.0).max(180.0))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if ui.add_enabled(!self.permissions.access.accessibility, egui::Button::new("Запросить доступ")).clicked() {
                                self.permissions.request();
                            }
                            if ui.button("Открыть настройки").clicked() {
                                if let Err(error) = permissions::open_settings() {
                                    self.permissions.notice = Some(error);
                                }
                            }
                            if ui.button("Проверить ещё раз").clicked() {
                                self.permissions.refresh();
                                self.permissions.notice = Some(if self.permissions.access.allowed() {
                                    "Доступ подтверждён macOS. Нажми «Продолжить».".into()
                                } else if !self.permissions.access.accessibility {
                                    "macOS пока не разрешает ввод этой копии приложения. Включи доступ в настройках; при необходимости перезапусти Keyscribe.".into()
                                } else {
                                    "Accessibility разрешён, но подключить движок ввода не удалось. Перезапусти Keyscribe и проверь ещё раз.".into()
                                });
                            }
                        });
                        if let Some(notice) = &self.permissions.notice {
                            ui.label(RichText::new(notice).color(p.muted));
                        }
                        ui.add_space(8.0);
                        egui::CollapsingHeader::new("Уже включено, но после пересборки не работает?")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label("1. В настройках Accessibility выключи и снова включи Keyscribe.");
                                ui.label("2. Если доступа всё ещё нет, удали старую запись Keyscribe и добавь текущую копию приложения кнопкой «+». Затем перезапусти Keyscribe и проверь ещё раз.");
                                ui.small("Локальная ad hoc подпись может измениться при пересборке. Старая запись в списке не подтверждает доступ новой сборке.");
                            });
                        if let Some(path) = permissions::application_path() {
                            ui.add_space(6.0);
                            ui.label(RichText::new("Текущая копия приложения").strong());
                            ui.label(RichText::new(path.display().to_string()).monospace().small());
                            if ui.button("Показать в Finder").clicked() {
                                if let Err(error) = permissions::reveal_application() {
                                    self.permissions.notice = Some(error);
                                }
                            }
                            if path.extension().is_none_or(|extension| extension != "app") {
                                ui.small("Запущен отдельный бинарник. Для предсказуемого выбора в настройках используй Keyscribe.app из make build.");
                            }
                        }
                    });
                ui.add_space(10.0);
                ui.separator();
                ui.label(RichText::new("До подтверждения доступа запуск всех режимов заблокирован. Статус проверяется автоматически; выполнение само не начинается.").small().color(p.muted));
                if ui.add_enabled(self.permissions.access.allowed(), egui::Button::new("Продолжить").min_size(egui::vec2(150.0, 36.0))).clicked()
                    && self.permissions.confirm()
                {
                    self.ready = false;
                    self.status = "Доступ macOS подтверждён. Проверь настройки перед запуском.".into();
                }
            });
    }
}
