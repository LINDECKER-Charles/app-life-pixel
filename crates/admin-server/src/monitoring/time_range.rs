//! The time range every monitoring view shares, as the console keeps it in its URL: `from` and
//! `to` are `now`, `now-<n><unit>` (`s`, `m`, `h`, `d`, `w`), Unix seconds or RFC 3339; a step is
//! seconds or `<n><unit>`.

use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

/// `from` when the console sends none.
pub const DEFAULT_FROM: &str = "now-24h";
/// `to` when the console sends none.
pub const DEFAULT_TO: &str = "now";
/// The longest range a view reads.
pub const MAX_RANGE: Duration = Duration::days(90);
/// The points a series has when the console sends no step.
const DEFAULT_POINTS: i64 = 200;
/// The most points a series has.
const MAX_POINTS: i64 = 1_000;
/// The shortest step: the scrape interval.
const MIN_STEP_SECONDS: i64 = 15;
const NOW: &str = "now";

/// A range of time, `from` before `to`, at most [`MAX_RANGE`] long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeRange {
    /// Its start.
    pub from: OffsetDateTime,
    /// Its end.
    pub to: OffsetDateTime,
}

impl TimeRange {
    /// The range `from`..`to`, relative expressions read at `now`; the defaults for what is
    /// missing.
    #[must_use]
    pub fn parse(from: Option<&str>, to: Option<&str>, now: OffsetDateTime) -> Option<Self> {
        let from = parse_instant(from.unwrap_or(DEFAULT_FROM), now)?;
        let to = parse_instant(to.unwrap_or(DEFAULT_TO), now)?;
        let is_valid = from < to && to - from <= MAX_RANGE;
        is_valid.then_some(Self { from, to })
    }

    /// Its length.
    #[must_use]
    pub fn length(&self) -> Duration {
        self.to - self.from
    }

    /// The range of the same length just before it.
    #[must_use]
    pub fn previous(&self) -> Self {
        Self {
            from: self.from - self.length(),
            to: self.from,
        }
    }

    /// The step of a series over the range: `requested` when given, never finer than 1,000
    /// points nor than 15 seconds; 200 points otherwise.
    #[must_use]
    pub fn step(&self, requested: Option<&str>) -> Option<Duration> {
        let seconds = self.length().whole_seconds();
        let finest = (seconds + MAX_POINTS - 1) / MAX_POINTS;
        let chosen = match requested {
            Some(text) => parse_duration(text)?.whole_seconds(),
            None => (seconds + DEFAULT_POINTS - 1) / DEFAULT_POINTS,
        };
        Some(Duration::seconds(chosen.max(finest).max(MIN_STEP_SECONDS)))
    }
}

/// `text` read as an instant at `now`.
fn parse_instant(text: &str, now: OffsetDateTime) -> Option<OffsetDateTime> {
    let text = text.trim();
    if text == NOW {
        return Some(now);
    }
    if let Some(ago) = text.strip_prefix("now-") {
        return Some(now - parse_duration(ago)?);
    }
    if let Ok(seconds) = text.parse::<i64>() {
        return OffsetDateTime::from_unix_timestamp(seconds).ok();
    }
    OffsetDateTime::parse(text, &Rfc3339).ok()
}

/// `<n>` seconds, or `<n><unit>` with `s`, `m`, `h`, `d` or `w`; above zero.
fn parse_duration(text: &str) -> Option<Duration> {
    let text = text.trim();
    let split = text
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(text.len());
    let (digits, unit) = text.split_at(split);
    let count: i64 = digits.parse().ok().filter(|count| *count > 0)?;
    let unit_seconds = match unit {
        "" | "s" => 1,
        "m" => 60,
        "h" => 3_600,
        "d" => 86_400,
        "w" => 604_800,
        _ => return None,
    };
    count.checked_mul(unit_seconds).map(Duration::seconds)
}

/// `duration` as PromQL writes it: whole seconds, such as `3600s`.
#[must_use]
pub fn promql_duration(duration: Duration) -> String {
    format!("{}s", duration.whole_seconds().max(1))
}

/// `instant` as Unix seconds, as the sources read it.
#[must_use]
pub fn unix_seconds(instant: OffsetDateTime) -> String {
    instant.unix_timestamp().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap()
    }

    #[test]
    fn a_range_defaults_to_the_last_day_and_reads_every_form() {
        let range = TimeRange::parse(None, None, now()).unwrap();
        assert_eq!((range.from, range.to), (now() - Duration::hours(24), now()));
        let written = TimeRange::parse(Some("2027-01-14T08:00:00Z"), Some("1800000000"), now());
        assert_eq!(written.unwrap().to, now());
        let week = TimeRange::parse(Some("now-1w"), Some("now-30m"), now()).unwrap();
        assert_eq!(week.length(), Duration::weeks(1) - Duration::minutes(30));
        assert_eq!(week.previous().to, week.from);
    }

    #[test]
    fn a_range_reversed_empty_too_long_or_unreadable_is_refused() {
        for (from, to) in [
            ("now", "now-1h"),
            ("now", "now"),
            ("now-91d", "now"),
            ("yesterday", "now"),
            ("now-5y", "now"),
            ("now-0h", "now"),
        ] {
            assert_eq!(
                TimeRange::parse(Some(from), Some(to), now()),
                None,
                "{from}"
            );
        }
    }

    #[test]
    fn a_step_keeps_a_series_between_15_seconds_and_1000_points() {
        let day = TimeRange::parse(Some("now-24h"), None, now()).unwrap();
        assert_eq!(day.step(None), Some(Duration::seconds(432)));
        assert_eq!(day.step(Some("1s")), Some(Duration::seconds(87)));
        assert_eq!(day.step(Some("1h")), Some(Duration::hours(1)));
        let hour = TimeRange::parse(Some("now-1h"), None, now()).unwrap();
        assert_eq!(hour.step(None), Some(Duration::seconds(18)));
        assert_eq!(hour.step(Some("5")), Some(Duration::seconds(15)));
        assert_eq!(hour.step(Some("5x")), None);
    }
}
