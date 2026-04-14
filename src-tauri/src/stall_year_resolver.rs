//! Resolves Project: Gorgon's year-less timestamp format ("Mon Apr 13 14:29")
//! into real ISO 8601 datetimes. The game logs don't include the year, so we
//! infer it from context (current time for live tailing, filename date for
//! imports) and handle year-boundary crossings with a monotonic walk.

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};

static MONTHS: &[(&str, u32)] = &[
    ("Jan", 1), ("Feb", 2), ("Mar", 3), ("Apr", 4), ("May", 5), ("Jun", 6),
    ("Jul", 7), ("Aug", 8), ("Sep", 9), ("Oct", 10), ("Nov", 11), ("Dec", 12),
];

/// Parse "Mon Apr 13 14:29" into (month, day, hour, minute).
pub fn parse_game_timestamp(ts: &str) -> Option<(u32, u32, u32, u32)> {
    let parts: Vec<&str> = ts.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    let month = MONTHS
        .iter()
        .find(|(n, _)| *n == parts[1])
        .map(|(_, m)| *m)?;
    let day: u32 = parts[2].parse().ok()?;
    let time_parts: Vec<&str> = parts[3].split(':').collect();
    if time_parts.len() != 2 {
        return None;
    }
    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    Some((month, day, hour, minute))
}

fn format_iso(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Option<String> {
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let dt = date.and_hms_opt(hour, minute, 0)?;
    Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// Resolve a list of game timestamps (oldest first) into ISO strings.
///
/// `base_year` is the year of the OLDEST entry. Walking forward, a backward
/// month jump (e.g., Dec → Jan) increments the year to handle books that
/// span a year boundary.
pub fn resolve_timestamps_oldest_first(
    timestamps: &[&str],
    base_year: i32,
) -> Vec<Option<String>> {
    let mut out = Vec::with_capacity(timestamps.len());
    let mut year = base_year;
    let mut prev_month: Option<u32> = None;

    for ts in timestamps {
        let Some((m, d, h, min)) = parse_game_timestamp(ts) else {
            out.push(None);
            continue;
        };
        if let Some(pm) = prev_month {
            if m < pm {
                year += 1;
            }
        }
        out.push(format_iso(year, m, d, h, min));
        prev_month = Some(m);
    }
    out
}

/// Choose the base year (year of the oldest entry) for a live-tailed book.
///
/// Starts from `now.year()` and walks back if the oldest entry's (month, day)
/// is in the future relative to now — meaning the book contains entries from
/// the previous calendar year.
pub fn base_year_for_live(oldest_ts: &str) -> i32 {
    let now = Local::now().naive_local();
    base_year_for_live_at(oldest_ts, now)
}

fn base_year_for_live_at(oldest_ts: &str, now: NaiveDateTime) -> i32 {
    let year = now.year();
    let Some((m, d, h, min)) = parse_game_timestamp(oldest_ts) else {
        return year;
    };
    let Some(candidate) = NaiveDate::from_ymd_opt(year, m, d).and_then(|d| d.and_hms_opt(h, min, 0))
    else {
        return year;
    };
    if candidate > now {
        year - 1
    } else {
        year
    }
}

/// For backfill: infer the year of a single row from its `created_at`.
/// Events are never in the future at insert time, so if the parsed (month, day)
/// is later in the year than `created_at`, it must be from the previous year.
pub fn backfill_year(event_timestamp: &str, created_year: i32, created_month: u32, created_day: u32) -> Option<String> {
    let (m, d, h, min) = parse_game_timestamp(event_timestamp)?;
    let year = if (m, d) > (created_month, created_day) {
        created_year - 1
    } else {
        created_year
    };
    format_iso(year, m, d, h, min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_game_timestamp() {
        assert_eq!(parse_game_timestamp("Mon Apr 13 14:29"), Some((4, 13, 14, 29)));
        assert_eq!(parse_game_timestamp("Sat Dec 31 23:59"), Some((12, 31, 23, 59)));
        assert_eq!(parse_game_timestamp("garbage"), None);
    }

    #[test]
    fn resolves_single_year_book() {
        let input = ["Mon Apr 13 10:00", "Mon Apr 13 14:00", "Tue Apr 14 09:00"];
        let result = resolve_timestamps_oldest_first(&input, 2026);
        assert_eq!(result[0].as_deref(), Some("2026-04-13 10:00:00"));
        assert_eq!(result[1].as_deref(), Some("2026-04-13 14:00:00"));
        assert_eq!(result[2].as_deref(), Some("2026-04-14 09:00:00"));
    }

    #[test]
    fn resolves_year_boundary_crossing() {
        let input = ["Wed Dec 28 10:00", "Fri Dec 30 12:00", "Sun Jan 1 00:01", "Thu Jan 5 15:00"];
        let result = resolve_timestamps_oldest_first(&input, 2025);
        assert_eq!(result[0].as_deref(), Some("2025-12-28 10:00:00"));
        assert_eq!(result[1].as_deref(), Some("2025-12-30 12:00:00"));
        assert_eq!(result[2].as_deref(), Some("2026-01-01 00:01:00"));
        assert_eq!(result[3].as_deref(), Some("2026-01-05 15:00:00"));
    }

    #[test]
    fn base_year_for_live_in_past() {
        let now = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap().and_hms_opt(12, 0, 0).unwrap();
        assert_eq!(base_year_for_live_at("Mon Apr 13 10:00", now), 2026);
    }

    #[test]
    fn base_year_for_live_wraps_to_previous_year() {
        // now = Jan 3 2026, oldest = Dec 28 → must be 2025
        let now = NaiveDate::from_ymd_opt(2026, 1, 3).unwrap().and_hms_opt(12, 0, 0).unwrap();
        assert_eq!(base_year_for_live_at("Wed Dec 28 10:00", now), 2025);
    }

    #[test]
    fn backfill_same_year() {
        // event Apr 13, row inserted Apr 14 → same year
        assert_eq!(
            backfill_year("Mon Apr 13 14:29", 2026, 4, 14).as_deref(),
            Some("2026-04-13 14:29:00"),
        );
    }

    #[test]
    fn backfill_previous_year() {
        // event Dec 30, row inserted Jan 5 → previous year
        assert_eq!(
            backfill_year("Tue Dec 30 10:00", 2026, 1, 5).as_deref(),
            Some("2025-12-30 10:00:00"),
        );
    }
}
