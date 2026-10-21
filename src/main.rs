#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
compile_error!("Keyscribe currently supports macOS and Windows only");

mod appearance;
mod automation_ui;
mod desktop;
mod guidance;
mod permissions;
mod permissions_ui;
mod typing_setup;
mod workspace_ui;

use automation_ui::{AutomationUi, Mode};
use desktop::{Job, Session, Status, Timing};
use eframe::egui;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use keyscribe::keyboard::{prepare_plan, Action, Layout};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use typing_setup::TypingSetup;

fn main() -> eframe::Result {
    eframe::run_native(
        "Keyscribe",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_icon(appearance::app_icon())
                .with_inner_size([1160.0, 860.0])
                .with_min_inner_size([800.0, 680.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    brand_icon: egui::TextureHandle,
    theme: appearance::Theme,
    permissions: permissions::Permissions,
    automation: AutomationUi,
    text: String,
    typing_setup: TypingSetup,
    auto_switch: bool,
    countdown: u64,
    interval_ms: u64,
    hold_ms: u64,
    switch_delay_ms: u64,
    ready: bool,
    guidance: guidance::Guidance,
    cancel: Arc<AtomicBool>,
    focused: Arc<AtomicBool>,
    session: Option<Session>,
    status: String,
    sent: usize,
    total: usize,
    progress_caption: &'static str,
    capture_signal: Arc<AtomicBool>,
    capture_hotkey: HotKey,
    capture_error: Option<String>,
    stop_hotkeys: [HotKey; 4],
    hotkeys: Option<GlobalHotKeyManager>,
    hotkey_error: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = if cc.egui_ctx.style().visuals.dark_mode {
            appearance::Theme::Dark
        } else {
            appearance::Theme::Light
        };
        theme.apply(&cc.egui_ctx);
        let icon = appearance::app_icon();
        let brand_icon = cc.egui_ctx.load_texture(
            "keyscribe-brand",
            egui::ColorImage::from_rgba_unmultiplied(
                [icon.width as usize, icon.height as usize],
                &icon.rgba,
            ),
            egui::TextureOptions::LINEAR,
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::F12);
        let stop_hotkeys = [
            hotkey,
            HotKey::new(
                Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
                Code::F12,
            ),
            HotKey::new(
                Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META),
                Code::F12,
            ),
            HotKey::new(
                Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT | Modifiers::META),
                Code::F12,
            ),
        ];
        let stop_ids = stop_hotkeys.map(|hotkey| hotkey.id());
        let capture_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::F11);
        let capture_signal = Arc::new(AtomicBool::new(false));
        let callback_capture = Arc::clone(&capture_signal);
        let worker_cancel = Arc::clone(&cancel);
        let ctx = cc.egui_ctx.clone();
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if stop_ids.contains(&event.id) && event.state == HotKeyState::Pressed {
                worker_cancel.store(true, Ordering::SeqCst);
                ctx.request_repaint();
            } else if event.id == capture_hotkey.id() && event.state == HotKeyState::Pressed {
                callback_capture.store(true, Ordering::SeqCst);
                ctx.request_repaint();
            }
        }));
        let manager = GlobalHotKeyManager::new().and_then(|manager| {
            for (index, hotkey) in stop_hotkeys.iter().enumerate() {
                if let Err(error) = manager.register(*hotkey) {
                    for previous in &stop_hotkeys[..index] {
                        let _ = manager.unregister(*previous);
                    }
                    return Err(error);
                }
            }
            Ok(manager)
        });
        let (hotkeys, hotkey_error) = match manager {
            Ok(manager) => (Some(manager), None),
            Err(error) => (None, Some(format!("Не удалось включить Ctrl+Alt+F12: {error}. Закрой другой экземпляр Keyscribe или приложение, занявшее сочетание, и перезапусти Keyscribe."))),
        };
        let capture_error = hotkeys.as_ref().and_then(|manager| manager.register(capture_hotkey).err().map(|error| format!("Не удалось включить захват Ctrl+Alt+F11: {error}. Координаты можно ввести вручную.")));
        Self {
            brand_icon,
            theme,
            permissions: permissions::Permissions::default(),
            automation: AutomationUi::default(),
            text: String::new(),
            typing_setup: TypingSetup::default(),
            auto_switch: true,
            countdown: 5,
            interval_ms: 100,
            hold_ms: 25,
            switch_delay_ms: 400,
            ready: false,
            guidance: guidance::Guidance::default(),
            cancel,
            focused: Arc::new(AtomicBool::new(true)),
            session: None,
            status: "Готово к вводу текста".into(),
            sent: 0,
            total: 0,
            progress_caption: "Отправлено нажатий",
            capture_signal,
            capture_hotkey,
            capture_error,
            stop_hotkeys,
            hotkeys,
            hotkey_error,
        }
    }

    fn start(&mut self) {
        self.permissions.refresh();
        if !self.permissions.can_start() {
            self.permissions.dialog_open = true;
            self.ready = false;
            self.guidance.point_to(Some(guidance::Field::Permissions));
            return;
        }
        if let Some(field) = self.next_required_field() {
            self.guidance.point_to(Some(field));
            self.status = "Заполни подсвеченный блок, затем нажми запуск ещё раз.".into();
            return;
        }
        self.guidance.point_to(None);
        let job = match self.automation.mode {
            Mode::Text => {
                let layout = match self.typing_setup.validate_text(self.auto_switch) {
                    Ok(layout) => layout,
                    Err(error) => {
                        self.status = error.into();
                        return;
                    }
                };
                let actions = match prepare_plan(&self.text, layout, self.auto_switch) {
                    Ok(strokes) => strokes,
                    Err(error) => {
                        self.status = error.to_string();
                        return;
                    }
                };
                if actions.is_empty() {
                    return;
                }
                self.total = actions
                    .iter()
                    .filter(|action| matches!(action, Action::Key(_)))
                    .count();
                self.progress_caption = "Отправлено нажатий";
                Job::Text {
                    actions,
                    shortcut: self.typing_setup.shortcut,
                }
            }
            Mode::Cycle => {
                if let Err(error) = self.typing_setup.validate_context() {
                    self.status = error.into();
                    return;
                }
                if let Err(error) = self.automation.cycle.validate() {
                    self.status = error;
                    return;
                }
                self.total = self.automation.cycle.total_steps();
                self.progress_caption = "Выполнено шагов";
                Job::Cycle(self.automation.cycle.clone())
            }
            Mode::Mouse => {
                if let Err(error) = self.automation.motion.validate() {
                    self.status = error;
                    return;
                }
                self.total = if self.automation.motion.until_stopped {
                    0
                } else {
                    self.automation.motion.seconds as usize
                };
                self.progress_caption = "Прошло секунд";
                Job::Motion(self.automation.motion.clone())
            }
        };
        self.sent = 0;
        self.automation.capture_target = None;
        self.automation.capture_deadline = None;
        self.cancel.store(false, Ordering::SeqCst);
        match Session::start(
            job,
            Timing {
                countdown: self.countdown,
                interval_ms: self.interval_ms,
                hold_ms: self.hold_ms,
                switch_delay_ms: self.switch_delay_ms,
            },
            Arc::clone(&self.cancel),
            Arc::clone(&self.focused),
        ) {
            Ok(session) => {
                self.session = Some(session);
                self.status = "Проверка доступа к управлению…".into();
            }
            Err(error) => {
                self.status = format!("Не удалось запустить ввод: {error}");
                self.ready = false;
                if self.automation.mode != Mode::Mouse {
                    self.typing_setup.require_current_language();
                }
            }
        }
    }

    fn update_status(&mut self) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        // Join before draining the final messages, so completion cannot race the receiver.
        let finished = session.is_finished();
        let join_error = if finished {
            session.finish().err()
        } else {
            None
        };
        for status in session.receiver.try_iter() {
            match status {
                Status::Countdown(seconds) => {
                    self.status = format!("Начало через {seconds} с — выбери целевое окно")
                }
                Status::Switching(layout) => {
                    self.status = format!(
                        "Отправка переключения на {}…",
                        match layout {
                            Layout::Us => "English US",
                            Layout::Russian => "русскую ЙЦУКЕН",
                        }
                    )
                }
                Status::Typing(sent) => {
                    self.sent = sent;
                    self.status = "Отправка клавиш…".into();
                }
                Status::Completed(sent) => {
                    self.sent = sent;
                    self.status = "Клавиши отправлены. Сверь результат в Citrix.".into();
                }
                Status::Stopped(sent) => {
                    self.sent = sent;
                    self.status =
                        "Остановлено. Перед повтором проверь уже выполненные действия.".into();
                }
                Status::Failed(error) => self.status = error,
                Status::AccessDenied(error) => {
                    self.permissions.refresh();
                    self.permissions.dialog_open = true;
                    self.permissions.notice = Some(error.clone());
                    self.status = error;
                }
                Status::Activity {
                    completed,
                    total,
                    message,
                } => {
                    self.sent = completed;
                    self.total = total;
                    self.status = message;
                }
                Status::Finished { completed, message } => {
                    self.sent = completed;
                    self.status = message;
                }
            }
        }
        if let Some(error) = join_error {
            self.status = error;
        }
        if finished {
            self.session = None;
            self.ready = false;
            if self.automation.mode != Mode::Mouse {
                self.typing_setup.require_current_language();
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        let was_focused = self.focused.swap(focused, Ordering::SeqCst);
        if self.permissions.poll(focused && !was_focused) {
            self.ready = false;
            if !self.permissions.can_start() {
                self.cancel.store(true, Ordering::SeqCst);
                self.automation.capture_deadline = None;
                self.automation.capture_target = None;
                self.typing_setup.require_current_language();
            }
        }
        if cfg!(target_os = "macos") {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
        if ctx.input(|i| i.viewport().close_requested() || i.key_pressed(egui::Key::Escape)) {
            self.cancel.store(true, Ordering::SeqCst);
        }
        self.update_status();
        let timed_capture = self
            .automation
            .capture_deadline
            .is_some_and(|deadline| Instant::now() >= deadline);
        if (self.capture_signal.swap(false, Ordering::SeqCst) || timed_capture)
            && self.session.is_none()
            && self.permissions.can_start()
        {
            self.automation.capture_deadline = None;
            match desktop::cursor_position()
                .and_then(|(x, y)| self.automation.capture(x, y).map(|()| (x, y)))
            {
                Ok((x, y)) => {
                    self.ready = false;
                    self.status =
                        format!("Координаты записаны: X={x}, Y={y}. Проверь шаг перед запуском.");
                }
                Err(error) => self.status = error,
            }
        }
        let running = self.session.is_some();
        if running || self.automation.capture_deadline.is_some() {
            ctx.request_repaint_after(Duration::from_millis(25));
        }
        self.show_workspace(ctx, running);
        self.show_permissions_dialog(ctx);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        // Release synthetic keys before unregistering the emergency shortcut.
        self.session.take();
        if let Some(manager) = &self.hotkeys {
            let _ = manager.unregister(self.capture_hotkey);
            for hotkey in self.stop_hotkeys {
                let _ = manager.unregister(hotkey);
            }
        }
    }
}
