// ABOUTME: Parses cumulative process CPU clocks and derives interval CPU usage.
// ABOUTME: Rejects incomplete or nonmonotonic readings before reporting a profile result.
use anyhow::{Context as _, Result, ensure};
use std::time::Duration;

#[cfg(windows)]
pub(super) fn cpu_ticks(_value: &str) -> Result<Duration> {
    anyhow::bail!("Windows 100ns CPU ticks are not supported yet")
}

pub(super) fn cpu_clock(value: &str) -> Result<Duration> {
    let value = value.trim();
    ensure!(
        !value.is_empty() && value.len() <= 64,
        "invalid CPU clock length"
    );
    let (days, clock) = match value.split_once('-') {
        Some((days, clock)) => (days.parse::<u64>().context("CPU clock days")?, clock),
        None => (0, value),
    };
    let fields: Vec<_> = clock.split(':').collect();
    ensure!(
        matches!(fields.len(), 2 | 3),
        "CPU clock must contain minutes and seconds"
    );
    let seconds = fields[fields.len() - 1]
        .parse::<f64>()
        .context("CPU clock seconds")?;
    ensure!(
        seconds.is_finite() && (0. ..60.).contains(&seconds),
        "CPU clock seconds out of range"
    );
    let minutes = fields[fields.len() - 2]
        .parse::<u64>()
        .context("CPU clock minutes")?;
    let hours = if fields.len() == 3 {
        ensure!(minutes < 60, "CPU clock minutes out of range");
        fields[0].parse::<u64>().context("CPU clock hours")?
    } else {
        0
    };
    let total = days
        .checked_mul(86_400)
        .and_then(|value| {
            hours
                .checked_mul(3_600)
                .and_then(|hours| value.checked_add(hours))
        })
        .and_then(|value| {
            minutes
                .checked_mul(60)
                .and_then(|minutes| value.checked_add(minutes))
        })
        .context("CPU clock overflow")?;
    Duration::from_secs(total)
        .checked_add(Duration::try_from_secs_f64(seconds)?)
        .context("CPU clock duration overflow")
}

pub(super) fn process_cpu(before: Duration, after: Duration, wall: Duration) -> Result<f64> {
    ensure!(!wall.is_zero(), "CPU interval has zero wall duration");
    let used = after
        .checked_sub(before)
        .context("CPU clock moved backwards")?;
    Ok(used.as_secs_f64() * 100. / wall.as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ps_cumulative_cpu_clock_units() -> Result<()> {
        for (text, seconds) in [
            ("0:05.25", 5.25),
            ("01:02:03.50", 3723.5),
            ("1-02:03:04.12", 93784.12),
        ] {
            assert_eq!(cpu_clock(text)?, Duration::from_secs_f64(seconds));
        }
        Ok(())
    }

    #[test]
    fn cpu_percentage_uses_interval_delta_and_may_exceed_one_core() -> Result<()> {
        let first = Duration::from_secs(10);
        let wall = Duration::from_secs(4);
        assert_eq!(
            process_cpu(first, first + Duration::from_secs(2), wall)?,
            50.
        );
        assert_eq!(
            process_cpu(first, first + Duration::from_secs(10), wall)?,
            250.
        );
        Ok(())
    }

    #[test]
    fn refuses_invalid_or_reversed_clock_readings() {
        for text in ["", "1", "0:no", "0:-1", "00:99.00", "nan", "1:00:00:00"] {
            assert!(cpu_clock(text).is_err(), "accepted invalid clock {text:?}");
        }
        assert!(
            process_cpu(
                Duration::from_secs(2),
                Duration::from_secs(1),
                Duration::from_secs(1)
            )
            .is_err()
        );
        assert!(process_cpu(Duration::ZERO, Duration::from_secs(1), Duration::ZERO).is_err());
    }
}
