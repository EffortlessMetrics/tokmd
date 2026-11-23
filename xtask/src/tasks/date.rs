//! Shared date helpers for xtask.
//!
//! These helpers wrap the [`time`] crate so the rest of xtask does not have to
//! care about format descriptions or UTC-offset handling. The former
//! implementation used `chrono`, which is now unmaintained
//! (`RUSTSEC-2026-0543`); `time` is already a workspace dependency and covers
//! every date operation xtask needs.

use time::format_description::well_known::Rfc2822;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime};

/// The canonical `YYYY-MM-DD` format used across xtask policy files.
pub(crate) const DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[year]-[month]-[day]");

/// Today's date in UTC.
pub(crate) fn today_utc() -> Date {
    OffsetDateTime::now_utc().date()
}

/// Parse a `YYYY-MM-DD` date.
pub(crate) fn parse_date(value: &str) -> Result<Date, time::error::Parse> {
    Date::parse(value, &DATE_FORMAT)
}

/// Format a date as `YYYY-MM-DD`.
pub(crate) fn format_date(date: Date) -> String {
    date.format(&DATE_FORMAT).unwrap_or_default()
}

/// Format today's date as `YYYY-MM-DD`.
pub(crate) fn today_utc_formatted() -> String {
    format_date(today_utc())
}

/// The current instant as an RFC 3339 string (for example
/// `2026-11-23T09:15:00.123456789Z`).
pub(crate) fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Parse an RFC 2822 timestamp (for example `Tue, 24 Feb 2026 16:57:08 GMT`).
///
/// Returns the instant as an [`OffsetDateTime`].
pub(crate) fn parse_rfc2822(value: &str) -> Result<OffsetDateTime, time::error::Parse> {
    PrimitiveDateTime::parse(value, &Rfc2822).map(|dt| dt.assume_utc())
}
