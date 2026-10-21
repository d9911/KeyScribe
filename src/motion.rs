#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Bounds {
    /// Restrict this version to the primary display, with a one-unit edge margin.
    pub fn around(
        center: Point,
        radius_x: i32,
        radius_y: i32,
        display: (i32, i32),
    ) -> Result<Self, String> {
        if display.0 < 4
            || display.1 < 4
            || center.x < 1
            || center.y < 1
            || center.x >= display.0 - 1
            || center.y >= display.1 - 1
        {
            return Err(
                "Перед стартом помести курсор внутрь основного экрана, не на его край.".into(),
            );
        }
        if !(5..=2000).contains(&radius_x) || !(5..=2000).contains(&radius_y) {
            return Err("Радиусы области должны быть от 5 до 2000.".into());
        }
        Ok(Self {
            left: center.x.saturating_sub(radius_x).max(1),
            top: center.y.saturating_sub(radius_y).max(1),
            right: center.x.saturating_add(radius_x).min(display.0 - 2),
            bottom: center.y.saturating_add(radius_y).min(display.1 - 2),
        })
    }
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.left
            && point.x <= self.right
            && point.y >= self.top
            && point.y <= self.bottom
    }
}

#[derive(Clone, Debug)]
pub struct Motion {
    pub radius_x: i32,
    pub radius_y: i32,
    pub seconds: u64,
    pub until_stopped: bool,
    pub min_pause_ms: u64,
    pub max_pause_ms: u64,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            radius_x: 120,
            radius_y: 80,
            seconds: 60,
            until_stopped: false,
            min_pause_ms: 200,
            max_pause_ms: 1800,
        }
    }
}

impl Motion {
    pub fn validate(&self) -> Result<(), String> {
        if !(5..=2000).contains(&self.radius_x)
            || !(5..=2000).contains(&self.radius_y)
            || !(1..=86400).contains(&self.seconds)
            || self.min_pause_ms > self.max_pause_ms
            || self.max_pause_ms > 60_000
        {
            return Err("Проверь радиусы (5–2000), длительность (1–86400 с) и паузы (0–60000 мс, минимум не больше максимума).".into());
        }
        Ok(())
    }
}

pub trait RandomSource {
    fn next(&mut self) -> Result<u64, String>;
}
pub struct SystemRandom;
impl RandomSource for SystemRandom {
    fn next(&mut self) -> Result<u64, String> {
        getrandom::u64()
            .map_err(|error| format!("Не удалось получить случайные данные ОС: {error}"))
    }
}

fn sample(random: &mut impl RandomSource, low: u64, high: u64) -> Result<u64, String> {
    let width = high - low + 1;
    let limit = u64::MAX - u64::MAX % width;
    loop {
        let value = random.next()?;
        if value < limit {
            return Ok(low + value % width);
        }
    }
}

fn point(bounds: Bounds, random: &mut impl RandomSource) -> Result<Point, String> {
    Ok(Point {
        x: sample(random, bounds.left as u64, bounds.right as u64)? as i32,
        y: sample(random, bounds.top as u64, bounds.bottom as u64)? as i32,
    })
}

#[derive(Debug)]
pub struct Path {
    pub points: Vec<(Point, u64)>,
    pub pause_ms: u64,
}

/// Random targets, both control points, travel time and inter-move pause.
/// A cubic Bezier stays inside the rectangle containing its control points.
pub fn next_path(
    config: &Motion,
    bounds: Bounds,
    start: Point,
    random: &mut impl RandomSource,
) -> Result<Path, String> {
    config.validate()?;
    if !bounds.contains(start) || bounds.left < 0 || bounds.top < 0 {
        return Err("Курсор оказался за пределами области движения.".into());
    }
    let mut end = point(bounds, random)?;
    if end == start {
        end = Point {
            x: if start.x == bounds.left {
                bounds.right
            } else {
                bounds.left
            },
            y: if start.y == bounds.top {
                bounds.bottom
            } else {
                bounds.top
            },
        };
    }
    let a = point(bounds, random)?;
    let b = point(bounds, random)?;
    let duration = sample(random, 250, 1400)?;
    let steps = sample(random, 18, 65)?;
    let mut points = Vec::new();
    let mut previous_ms = 0;
    for index in 1..=steps {
        let t = index as f64 / steps as f64;
        let t = t * t * (3.0 - 2.0 * t);
        let u = 1.0 - t;
        let coordinate = |s: i32, a: i32, b: i32, e: i32| {
            (u.powi(3) * f64::from(s)
                + 3.0 * u.powi(2) * t * f64::from(a)
                + 3.0 * u * t.powi(2) * f64::from(b)
                + t.powi(3) * f64::from(e))
            .round() as i32
        };
        let next = Point {
            x: coordinate(start.x, a.x, b.x, end.x).clamp(bounds.left, bounds.right),
            y: coordinate(start.y, a.y, b.y, end.y).clamp(bounds.top, bounds.bottom),
        };
        let at_ms = duration * index / steps;
        points.push((next, at_ms - previous_ms));
        previous_ms = at_ms;
    }
    Ok(Path {
        points,
        pause_ms: sample(random, config.min_pause_ms, config.max_pause_ms)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Sequence(u64);
    impl RandomSource for Sequence {
        fn next(&mut self) -> Result<u64, String> {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
            Ok(self.0)
        }
    }
    #[test]
    fn trajectories_and_control_points_cannot_leave_the_selected_region() {
        let config = Motion::default();
        let mut current = Point { x: 40, y: 30 };
        let bounds = Bounds::around(current, config.radius_x, config.radius_y, (1920, 1080))
            .expect("bounds");
        let mut random = Sequence(1);
        let mut endpoints = std::collections::HashSet::new();
        for _ in 0..100 {
            let path = next_path(&config, bounds, current, &mut random).expect("path");
            assert!(path.points.iter().all(|(p, _)| bounds.contains(*p)));
            assert!((250..=1400).contains(&path.points.iter().map(|(_, ms)| ms).sum::<u64>()));
            assert!((config.min_pause_ms..=config.max_pause_ms).contains(&path.pause_ms));
            let end = path.points.last().expect("nonempty path").0;
            assert_ne!(end, current);
            endpoints.insert((end.x, end.y));
            current = end;
        }
        assert!(endpoints.len() > 50);
    }
    #[test]
    fn invalid_ranges_and_secondary_display_coordinates_are_rejected() {
        assert!(Bounds::around(Point { x: -20, y: 30 }, 50, 50, (1920, 1080)).is_err());
        assert!(Motion {
            min_pause_ms: 500,
            max_pause_ms: 100,
            ..Motion::default()
        }
        .validate()
        .is_err());
    }
    #[test]
    fn random_source_failure_is_reported_without_a_fallback_pattern() {
        struct Broken;
        impl RandomSource for Broken {
            fn next(&mut self) -> Result<u64, String> {
                Err("no entropy".into())
            }
        }
        let bounds = Bounds::around(Point { x: 50, y: 50 }, 20, 20, (100, 100)).expect("bounds");
        assert_eq!(
            next_path(
                &Motion::default(),
                bounds,
                Point { x: 50, y: 50 },
                &mut Broken
            )
            .expect_err("error"),
            "no entropy"
        );
    }
}
