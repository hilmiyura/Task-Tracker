//! Duration helpers shared between the backend and (conceptually) the UI.
//!
//! The live timer itself runs in the React frontend; these helpers exist so the
//! backend can format a duration the same way when we later push rows to Sheets.

/// Format a span of seconds as a human label like "1h 5m".
pub fn format_duration(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    format!("{hours}h {minutes}m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_hours_and_minutes() {
        assert_eq!(format_duration(0), "0h 0m");
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(3900), "1h 5m");
    }
}
