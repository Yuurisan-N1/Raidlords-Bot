use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn draw_u64() -> u64 {
    let mut raw = [0u8; 8];

    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut raw).is_ok() {
            return u64::from_le_bytes(raw);
        }
    }

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    nanos ^ ((std::process::id() as u64) << 32)
}

pub fn jitter_secs(max: u64) -> u64 {
    if max == 0 {
        return 0;
    }

    draw_u64() % (max + 1)
}

pub struct Pace {
    min_ms: u64,
    jitter_ms: u64,
    last: Mutex<Option<Instant>>,
}

impl Pace {
    pub fn new(min_ms: u64, jitter_ms: u64) -> Pace {
        Pace {
            min_ms,
            jitter_ms,
            last: Mutex::new(None),
        }
    }

    pub fn wait(&self) {
        let gap = Duration::from_millis(self.min_ms + self.draw());

        let mut guard = match self.last.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some(last) = *guard {
            let spent = last.elapsed();

            if spent < gap {
                std::thread::sleep(gap - spent);
            }
        }

        *guard = Some(Instant::now());
    }

    fn draw(&self) -> u64 {
        jitter_secs(self.jitter_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quiet_pace_does_not_slow_the_run_down() {
        let pace = Pace::new(0, 0);
        let started = Instant::now();

        for _ in 0..5 {
            pace.wait();
        }

        assert!(started.elapsed() < Duration::from_millis(120));
    }

    #[test]
    fn two_calls_are_never_closer_than_the_floor() {
        let pace = Pace::new(60, 0);

        pace.wait();
        let started = Instant::now();
        pace.wait();

        assert!(started.elapsed() >= Duration::from_millis(55));
    }

    #[test]
    fn the_jitter_spreads_the_gap_without_going_below_the_floor() {
        let pace = Pace::new(20, 40);
        let mut seen = std::collections::HashSet::new();

        for _ in 0..20 {
            seen.insert(pace.draw());
        }

        assert!(seen.len() > 1);
        assert!(seen.iter().all(|value| *value <= 40));
    }

    #[test]
    fn the_cycle_jitter_stays_inside_the_window() {
        let mut seen = std::collections::HashSet::new();

        for _ in 0..40 {
            let secs = jitter_secs(90);
            assert!(secs <= 90);
            seen.insert(secs);
        }

        assert!(seen.len() > 1);
        assert_eq!(jitter_secs(0), 0);
    }
}
