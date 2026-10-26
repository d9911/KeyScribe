use crate::shortcut::Shortcut;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Press(Shortcut),
    Click {
        x: i32,
        y: i32,
        button: MouseButton,
        double: bool,
    },
    Move {
        x: i32,
        y: i32,
    },
    Wait(u64),
}

impl Step {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Press(_) => "Клавиши",
            Self::Click { .. } => "Клик",
            Self::Move { .. } => "Движение мыши",
            Self::Wait(_) => "Пауза",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cycle {
    pub steps: Vec<Step>,
    pub repetitions: u32,
    pub between_ms: u64,
}

impl Default for Cycle {
    fn default() -> Self {
        Self {
            steps: vec![Step::Wait(500)],
            repetitions: 2,
            between_ms: 1000,
        }
    }
}

impl Cycle {
    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() || self.steps.len() > 200 {
            return Err("В сценарии должно быть от 1 до 200 шагов.".into());
        }
        if !(1..=10_000).contains(&self.repetitions) || self.between_ms > 3_600_000 {
            return Err("Допустимо 1–10000 повторов и пауза между ними до часа.".into());
        }
        for (index, step) in self.steps.iter().enumerate() {
            let valid = match step {
                Step::Press(shortcut) => {
                    shortcut.trigger.is_some_and(|key| key.stroke().is_some())
                        && !shortcut.is_reserved()
                }
                Step::Click { x, y, .. } | Step::Move { x, y } => *x >= 0 && *y >= 0,
                Step::Wait(ms) => (10..=3_600_000).contains(ms),
            };
            if !valid {
                return Err(format!("Некорректный шаг {}. Проверь клавишу, координаты или паузу. {} зарезервированы приложением.", index + 1, crate::shortcut::RESERVED_LABEL));
            }
        }
        Ok(())
    }

    pub fn total_steps(&self) -> usize {
        self.steps.len() * self.repetitions as usize
    }
}

pub trait Driver {
    fn active(&self) -> bool;
    fn perform(&mut self, step: &Step) -> Result<bool, String>;
    fn pause(&mut self, milliseconds: u64) -> bool;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub completed_steps: usize,
    pub finished: bool,
}

/// No retries: cancellation or an error must not repeat a potentially destructive step.
pub fn run(
    cycle: &Cycle,
    driver: &mut impl Driver,
    mut progress: impl FnMut(usize, u32, usize),
) -> Result<Outcome, String> {
    cycle.validate()?;
    let mut completed_steps = 0;
    for repetition in 1..=cycle.repetitions {
        for (index, step) in cycle.steps.iter().enumerate() {
            if !driver.active() || !driver.perform(step)? {
                return Ok(Outcome {
                    completed_steps,
                    finished: false,
                });
            }
            completed_steps += 1;
            progress(completed_steps, repetition, index + 1);
        }
        if repetition < cycle.repetitions && !driver.pause(cycle.between_ms) {
            return Ok(Outcome {
                completed_steps,
                finished: false,
            });
        }
    }
    Ok(Outcome {
        completed_steps,
        finished: true,
    })
}

pub trait PointerSink {
    type Error;
    fn button(&mut self, button: MouseButton, pressed: bool) -> Result<(), Self::Error>;
}

pub fn click_button<S: PointerSink>(
    sink: &mut S,
    button: MouseButton,
    hold: impl FnOnce() -> bool,
) -> Result<bool, S::Error> {
    let pressed = sink.button(button, true);
    let completed = pressed.is_ok() && hold();
    let released = sink.button(button, false);
    pressed?;
    released?;
    Ok(completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Fake {
        events: Vec<Step>,
        pauses: Vec<u64>,
        stop_after: usize,
        fail_at: Option<usize>,
    }
    impl Driver for Fake {
        fn active(&self) -> bool {
            self.events.len() < self.stop_after
        }
        fn perform(&mut self, step: &Step) -> Result<bool, String> {
            self.events.push(step.clone());
            if self.fail_at == Some(self.events.len()) {
                Err("test failure".into())
            } else {
                Ok(true)
            }
        }
        fn pause(&mut self, ms: u64) -> bool {
            self.pauses.push(ms);
            self.active()
        }
    }
    #[test]
    fn repeats_in_order_and_waits_only_between_repetitions() {
        let cycle = Cycle {
            steps: vec![
                Step::Move { x: 20, y: 30 },
                Step::Click {
                    x: 20,
                    y: 30,
                    button: MouseButton::Left,
                    double: false,
                },
                Step::Wait(75),
            ],
            repetitions: 2,
            between_ms: 400,
        };
        let mut fake = Fake {
            stop_after: 100,
            ..Fake::default()
        };
        assert_eq!(
            run(&cycle, &mut fake, |_, _, _| {}).expect("run"),
            Outcome {
                completed_steps: 6,
                finished: true
            }
        );
        assert_eq!(
            fake.events,
            [cycle.steps.clone(), cycle.steps.clone()].concat()
        );
        assert_eq!(fake.pauses, [400]);
    }
    #[test]
    fn cancellation_and_errors_never_retry_an_action() {
        let cycle = Cycle {
            steps: vec![Step::Wait(10)],
            repetitions: 4,
            between_ms: 1,
        };
        let mut fake = Fake {
            stop_after: 1,
            ..Fake::default()
        };
        assert_eq!(
            run(&cycle, &mut fake, |_, _, _| {}).expect("stop"),
            Outcome {
                completed_steps: 1,
                finished: false
            }
        );
        let mut fake = Fake {
            stop_after: 100,
            fail_at: Some(1),
            ..Fake::default()
        };
        assert!(run(&cycle, &mut fake, |_, _, _| {}).is_err());
        assert_eq!(fake.events.len(), 1);
    }
    #[test]
    fn validates_the_entire_cycle_before_performing_any_step() {
        let cycle = Cycle {
            steps: vec![Step::Wait(10), Step::Move { x: -1, y: 0 }],
            ..Cycle::default()
        };
        let mut fake = Fake {
            stop_after: 100,
            ..Fake::default()
        };
        assert!(run(&cycle, &mut fake, |_, _, _| {}).is_err());
        assert!(fake.events.is_empty());
    }

    #[test]
    fn mouse_button_is_released_after_cancellation_or_failed_press() {
        struct Pointer {
            pressed: Vec<bool>,
            fail: bool,
        }
        impl PointerSink for Pointer {
            type Error = &'static str;
            fn button(&mut self, _: MouseButton, pressed: bool) -> Result<(), Self::Error> {
                self.pressed.push(pressed);
                if pressed && self.fail {
                    Err("failed press")
                } else {
                    Ok(())
                }
            }
        }
        let mut pointer = Pointer {
            pressed: vec![],
            fail: false,
        };
        assert_eq!(
            click_button(&mut pointer, MouseButton::Left, || false),
            Ok(false)
        );
        assert_eq!(pointer.pressed, [true, false]);
        let mut pointer = Pointer {
            pressed: vec![],
            fail: true,
        };
        assert_eq!(
            click_button(&mut pointer, MouseButton::Left, || panic!(
                "no hold after error"
            )),
            Err("failed press")
        );
        assert_eq!(pointer.pressed, [true, false]);
    }
}
