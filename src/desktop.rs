use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use keyscribe::{
    cycle::{self, Cycle, MouseButton, PointerSink, Step},
    keyboard::{Action, Layout, Stroke},
    motion::{self, Bounds, Motion, Point, SystemRandom},
    playback::{emit, emit_shortcut, KeyboardSink},
    shortcut::{Modifier, Shortcut},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum Status {
    Countdown(u64),
    Switching(Layout),
    Typing(usize),
    Completed(usize),
    Stopped(usize),
    Failed(String),
    AccessDenied(String),
    Activity {
        completed: usize,
        total: usize,
        message: String,
    },
    Finished {
        completed: usize,
        message: String,
    },
}

pub enum Job {
    Text {
        actions: Vec<Action>,
        shortcut: Shortcut,
    },
    Cycle(Cycle),
    Motion(Motion),
}

pub fn cursor_position() -> Result<(i32, i32), String> {
    if !crate::permissions::current_access().allowed() {
        return Err(
            "macOS не разрешает доступ к вводу. Открой окно «Доступ macOS» в Keyscribe.".into(),
        );
    }
    let input = Enigo::new(&Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    })
    .map_err(|error| format!("Нет доступа к координатам курсора: {error}"))?;
    let (x, y) = input.location().map_err(|error| error.to_string())?;
    let (width, height) = input.main_display().map_err(|error| error.to_string())?;
    if x < 0 || y < 0 || x >= width || y >= height {
        return Err("Захват координат поддерживается только на основном экране.".into());
    }
    Ok((x, y))
}

pub struct Timing {
    pub countdown: u64,
    pub interval_ms: u64,
    pub hold_ms: u64,
    pub switch_delay_ms: u64,
}

pub struct Session {
    pub receiver: Receiver<Status>,
    handle: Option<JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
}

impl Session {
    pub fn start(
        job: Job,
        timing: Timing,
        cancel: Arc<AtomicBool>,
        focused: Arc<AtomicBool>,
    ) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let worker_cancel = Arc::clone(&cancel);
        let handle = thread::Builder::new().name("keyscribe-input".into()).spawn(move || {
            let run = || -> Result<(), String> {
                // Preflight again at the OS boundary. Only the access dialog may request permission.
                let denied = || { let _ = sender.send(Status::AccessDenied("macOS не разрешает управление клавиатурой и мышью этой копии Keyscribe. Проверь доступ в открывшемся окне.".into())); };
                if !crate::permissions::current_access().allowed() {
                    denied();
                    return Ok(());
                }
                let mut keyboard = match Enigo::new(&Settings {
                    open_prompt_to_get_permissions: false,
                    ..Settings::default()
                }) {
                    Ok(input) => NativeKeyboard(input),
                    Err(enigo::NewConError::NoPermission) => {
                        denied();
                        return Ok(());
                    }
                    Err(error) => return Err(format!("Не удалось подключить клавиатуру и мышь: {error}")),
                };
                for seconds in (1..=timing.countdown).rev() {
                    if sender.send(Status::Countdown(seconds)).is_err() { return Ok(()); }
                    if !wait(Duration::from_secs(1), &worker_cancel, None) {
                        let _ = sender.send(Status::Stopped(0));
                        return Ok(());
                    }
                }
                if focused.load(Ordering::SeqCst) {
                    return Err("Выбери окно Citrix до конца обратного отсчёта. Ничего не отправлено.".into());
                }
                let (actions, shortcut) = match job {
                    Job::Text { actions, shortcut } => (actions, shortcut),
                    Job::Cycle(config) => return run_cycle(config, &mut keyboard, &timing, &worker_cancel, &focused, &sender),
                    Job::Motion(config) => return run_motion(config, &mut keyboard, &worker_cancel, &focused, &sender),
                };
                let mut sent = 0;
                for action in actions {
                    if worker_cancel.load(Ordering::SeqCst) || focused.load(Ordering::SeqCst) {
                        let _ = sender.send(Status::Stopped(sent));
                        return Ok(());
                    }
                    let hold = || wait(Duration::from_millis(timing.hold_ms), &worker_cancel, Some(&focused));
                    let (completed, pause) = match action {
                        Action::Key(stroke) => {
                            let completed = emit(&mut keyboard, stroke, hold)
                                .map_err(|error| format!("Ошибка ввода после {sent} нажатий: {error}. Повтор автоматически не выполняется."))?;
                            sent += 1;
                            if sender.send(Status::Typing(sent)).is_err() { return Ok(()); }
                            (completed, timing.interval_ms)
                        }
                        Action::SwitchLayout(layout) => {
                            if sender.send(Status::Switching(layout)).is_err() { return Ok(()); }
                            let completed = emit_shortcut(&mut keyboard, shortcut, hold)
                                .map_err(|error| format!("Не удалось переключить раскладку после {sent} символов: {error}. Проверь раскладку в Citrix перед повтором."))?;
                            (completed, timing.switch_delay_ms)
                        }
                    };
                    if !completed || !wait(Duration::from_millis(pause), &worker_cancel, Some(&focused)) {
                        let _ = sender.send(Status::Stopped(sent));
                        return Ok(());
                    }
                }
                let _ = sender.send(Status::Completed(sent));
                Ok(())
            };
            if let Err(error) = run() { let _ = sender.send(Status::Failed(error)); }
        })?;
        Ok(Self {
            receiver,
            handle: Some(handle),
            cancel,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.handle
            .as_ref()
            .is_none_or(|handle| handle.is_finished())
    }

    pub fn finish(&mut self) -> Result<(), String> {
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| {
                "Поток ввода неожиданно завершился. Проверь уже набранный текст перед повтором."
                    .to_owned()
            })?;
        }
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        let _ = self.finish();
    }
}

fn wait(duration: Duration, cancel: &AtomicBool, focused: Option<&AtomicBool>) -> bool {
    let start = Instant::now();
    loop {
        if cancel.load(Ordering::SeqCst) || focused.is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            return false;
        }
        let remaining = duration.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            return true;
        }
        thread::sleep(remaining.min(Duration::from_millis(10)));
    }
}

struct NativeKeyboard(Enigo);

impl KeyboardSink for NativeKeyboard {
    type Error = enigo::InputError;

    fn modifier(&mut self, modifier: Modifier, pressed: bool) -> Result<(), Self::Error> {
        let key = match modifier {
            Modifier::Control => Key::Control,
            Modifier::Alt => Key::Alt,
            Modifier::Shift => Key::Shift,
            #[cfg(target_os = "macos")]
            Modifier::Meta => Key::Other(0x36), // Right Command: Citrix maps it to Windows.
            #[cfg(target_os = "windows")]
            Modifier::Meta => Key::Meta,
        };
        self.0.key(
            key,
            if pressed {
                Direction::Press
            } else {
                Direction::Release
            },
        )
    }

    fn key(&mut self, stroke: Stroke, pressed: bool) -> Result<(), Self::Error> {
        #[cfg(target_os = "windows")]
        if let Some(key) = match stroke.windows {
            0xe053 => Some(Key::Delete),
            0xe04b => Some(Key::LeftArrow),
            0xe04d => Some(Key::RightArrow),
            0xe048 => Some(Key::UpArrow),
            0xe050 => Some(Key::DownArrow),
            0xe047 => Some(Key::Home),
            0xe04f => Some(Key::End),
            0xe049 => Some(Key::PageUp),
            0xe051 => Some(Key::PageDown),
            _ => None,
        } {
            return self.0.key(
                key,
                if pressed {
                    Direction::Press
                } else {
                    Direction::Release
                },
            );
        }
        #[cfg(target_os = "macos")]
        let code = stroke.mac;
        #[cfg(target_os = "windows")]
        let code = stroke.windows;
        self.0.raw(
            code,
            if pressed {
                Direction::Press
            } else {
                Direction::Release
            },
        )
    }
}

impl PointerSink for NativeKeyboard {
    type Error = enigo::InputError;
    fn button(&mut self, button: MouseButton, pressed: bool) -> Result<(), Self::Error> {
        let button = match button {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            MouseButton::Middle => Button::Middle,
        };
        self.0.button(
            button,
            if pressed {
                Direction::Press
            } else {
                Direction::Release
            },
        )
    }
}

struct Runtime<'a> {
    input: &'a mut NativeKeyboard,
    timing: &'a Timing,
    cancel: &'a AtomicBool,
    focused: &'a AtomicBool,
}

impl cycle::Driver for Runtime<'_> {
    fn active(&self) -> bool {
        !self.cancel.load(Ordering::SeqCst) && !self.focused.load(Ordering::SeqCst)
    }
    fn pause(&mut self, ms: u64) -> bool {
        wait(Duration::from_millis(ms), self.cancel, Some(self.focused))
    }
    fn perform(&mut self, step: &Step) -> Result<bool, String> {
        let result = match step {
            Step::Wait(ms) => return Ok(self.pause(*ms)),
            Step::Move { x, y } => {
                self.input
                    .0
                    .move_mouse(*x, *y, Coordinate::Abs)
                    .map_err(|e| e.to_string())?;
                true
            }
            Step::Press(shortcut) => {
                let (cancel, focused, hold_ms) = (self.cancel, self.focused, self.timing.hold_ms);
                emit_shortcut(self.input, *shortcut, || {
                    wait(Duration::from_millis(hold_ms), cancel, Some(focused))
                })
                .map_err(|e| e.to_string())?
            }
            Step::Click {
                x,
                y,
                button,
                double,
            } => {
                self.input
                    .0
                    .move_mouse(*x, *y, Coordinate::Abs)
                    .map_err(|e| e.to_string())?;
                if !self.active() {
                    return Ok(false);
                }
                for index in 0..if *double { 2 } else { 1 } {
                    if index > 0 && !self.pause(60) {
                        return Ok(false);
                    }
                    let (cancel, focused, hold_ms) =
                        (self.cancel, self.focused, self.timing.hold_ms);
                    if !cycle::click_button(self.input, *button, || {
                        wait(Duration::from_millis(hold_ms), cancel, Some(focused))
                    })
                    .map_err(|e| e.to_string())?
                    {
                        return Ok(false);
                    }
                }
                true
            }
        };
        Ok(result && self.pause(self.timing.interval_ms))
    }
}

fn run_cycle(
    config: Cycle,
    input: &mut NativeKeyboard,
    timing: &Timing,
    cancel: &AtomicBool,
    focused: &AtomicBool,
    sender: &mpsc::Sender<Status>,
) -> Result<(), String> {
    config.validate()?;
    let (width, height) = input.0.main_display().map_err(|e| e.to_string())?;
    for (index, step) in config.steps.iter().enumerate() {
        if let Step::Click { x, y, .. } | Step::Move { x, y } = step {
            if *x >= width || *y >= height {
                return Err(format!("Шаг {}: координаты вне основного экрана ({width} × {height}). Сценарий не запущен.", index + 1));
            }
        }
    }
    let mut runtime = Runtime {
        input,
        timing,
        cancel,
        focused,
    };
    let result = cycle::run(&config, &mut runtime, |completed, iteration, step| {
        let _ = sender.send(Status::Activity {
            completed,
            total: config.total_steps(),
            message: format!(
                "Цикл {iteration}/{}, выполнен шаг {step}/{}",
                config.repetitions,
                config.steps.len()
            ),
        });
    })
    .map_err(|error| {
        format!("Сценарий остановлен: {error}. Повтор шага автоматически не выполняется.")
    })?;
    let status = if result.finished {
        Status::Finished {
            completed: result.completed_steps,
            message: format!(
                "Сценарий завершён: {} повторов. Проверь результат в целевом окне.",
                config.repetitions
            ),
        }
    } else {
        Status::Stopped(result.completed_steps)
    };
    let _ = sender.send(status);
    Ok(())
}

fn run_motion(
    config: Motion,
    input: &mut NativeKeyboard,
    cancel: &AtomicBool,
    focused: &AtomicBool,
    sender: &mpsc::Sender<Status>,
) -> Result<(), String> {
    config.validate()?;
    let display = input.0.main_display().map_err(|e| e.to_string())?;
    let (x, y) = input.0.location().map_err(|e| e.to_string())?;
    let bounds = Bounds::around(Point { x, y }, config.radius_x, config.radius_y, display)?;
    let started = Instant::now();
    let deadline = if config.until_stopped {
        None
    } else {
        Some(started + Duration::from_secs(config.seconds))
    };
    let active = || !cancel.load(Ordering::SeqCst) && !focused.load(Ordering::SeqCst);
    let alive = || active() && deadline.is_none_or(|deadline| Instant::now() < deadline);
    let pause = |ms| {
        let duration = Duration::from_millis(ms);
        let duration = deadline.map_or(duration, |end| {
            duration.min(end.saturating_duration_since(Instant::now()))
        });
        wait(duration, cancel, Some(focused)) && alive()
    };
    let mut random = SystemRandom;
    let mut moves = 0;
    'movement: while alive() {
        let (x, y) = input.0.location().map_err(|e| e.to_string())?;
        let current = Point { x, y };
        if !bounds.contains(current) {
            let _ = sender.send(Status::Stopped(started.elapsed().as_secs() as usize));
            return Ok(());
        }
        let path = motion::next_path(&config, bounds, current, &mut random)?;
        for (point, delay_ms) in path.points {
            if !pause(delay_ms) {
                break 'movement;
            }
            input
                .0
                .move_mouse(point.x, point.y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
        }
        moves += 1;
        if sender
            .send(Status::Activity {
                completed: started.elapsed().as_secs() as usize,
                total: if config.until_stopped {
                    0
                } else {
                    config.seconds as usize
                },
                message: format!(
                    "Движение мыши: {moves} перемещений, {} с",
                    started.elapsed().as_secs()
                ),
            })
            .is_err()
        {
            return Ok(());
        }
        if !pause(path.pause_ms) {
            break;
        }
    }
    let elapsed = started.elapsed().as_secs() as usize;
    let status = if active() {
        Status::Finished {
            completed: elapsed,
            message: format!("Движение мыши завершено: {moves} перемещений."),
        }
    } else {
        Status::Stopped(elapsed)
    };
    let _ = sender.send(status);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_can_cancel_before_any_input() {
        assert!(!wait(Duration::from_secs(30), &AtomicBool::new(true), None));
    }

    #[test]
    fn returning_to_our_window_stops_input() {
        assert!(!wait(
            Duration::from_secs(30),
            &AtomicBool::new(false),
            Some(&AtomicBool::new(true))
        ));
    }
}
