//! Serialize/deserialize Timestamp type from and into string:

// TODO: this module should be re-written now that timestamp::Timestamp has the required constraints
use core::fmt;

use serde::{de::Error as _, ser::Error, Deserialize, Deserializer, Serialize, Serializer};
use time::{
    format_description::well_known::Rfc3339 as Rfc3339Format, macros::offset, OffsetDateTime,
};

use crate::types::time::timestamp::Timestamp;

use super::duration::{Duration, DurationError};

/// Helper struct to serialize and deserialize Timestamp into an RFC3339-compatible string
/// This is required because the serde `with` attribute is only available to fields of a struct but
/// not the whole struct.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rfc3339(#[serde(with = "crate::types::time::serializers")] Timestamp);

impl From<Timestamp> for Rfc3339 {
    fn from(value: Timestamp) -> Self {
        Rfc3339(value)
    }
}
impl From<Rfc3339> for Timestamp {
    fn from(value: Rfc3339) -> Self {
        value.0
    }
}

/// Deserialize string into Timestamp
pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let value_string = String::deserialize(deserializer)?;
    let t = OffsetDateTime::parse(&value_string, &Rfc3339Format).map_err(D::Error::custom)?;
    let t = t.to_offset(offset!(UTC));
    if !matches!(t.year(), 1..=9999) {
        return Err(D::Error::custom("date is out of range"));
    }
    let seconds = t.unix_timestamp();
    // Safe to convert to i32 because .nanosecond()
    // is guaranteed to return a value in 0..1_000_000_000 range.
    let nanos = t.nanosecond() as i32;
    Ok(Timestamp { seconds, nanos })
}

/// Serialize from Timestamp into string
pub fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.nanos < 0 || value.nanos > 999_999_999 {
        return Err(S::Error::custom("invalid nanoseconds in time"));
    }
    let total_nanos = value.seconds as i128 * 1_000_000_000 + value.nanos as i128;
    let datetime = OffsetDateTime::from_unix_timestamp_nanos(total_nanos)
        .map_err(|_| S::Error::custom("invalid time"))?;
    to_rfc3339_nanos(datetime).serialize(serializer)
}

/// Serialization helper for converting an [`OffsetDateTime`] object to a string.
///
/// This reproduces the behavior of Go's `time.RFC3339Nano` format,
/// ie. a RFC3339 date-time with left-padded subsecond digits without
///     trailing zeros and no trailing dot.
pub fn to_rfc3339_nanos(t: OffsetDateTime) -> String {
    // Can't use OffsetDateTime::format because the feature enabling it
    // currently requires std (https://github.com/time-rs/time/issues/400)

    // Preallocate enough string capacity to fit the shortest possible form,
    // yyyy-mm-ddThh:mm:ssZ
    let mut buf = String::with_capacity(20);

    fmt_as_rfc3339_nanos(t, &mut buf).unwrap();

    buf
}

/// Helper for formatting an [`OffsetDateTime`] value.
///
/// This function can be used to efficiently format date-time values
/// in [`Display`] or [`Debug`] implementations.
///
/// The format reproduces Go's `time.RFC3339Nano` format,
/// ie. a RFC3339 date-time with left-padded subsecond digits without
///     trailing zeros and no trailing dot.
///
/// [`Display`]: core::fmt::Display
/// [`Debug`]: core::fmt::Debug
pub fn fmt_as_rfc3339_nanos(t: OffsetDateTime, f: &mut impl fmt::Write) -> fmt::Result {
    let t = t.to_offset(offset!(UTC));
    let nanos = t.nanosecond();
    if nanos == 0 {
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z",
            year = t.year(),
            month = t.month() as u8,
            day = t.day(),
            hour = t.hour(),
            minute = t.minute(),
            second = t.second(),
        )
    } else {
        let mut secfrac = nanos;
        let mut secfrac_width = 9;
        while secfrac % 10 == 0 {
            secfrac /= 10;
            secfrac_width -= 1;
        }
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{secfrac:0sfw$}Z",
            year = t.year(),
            month = t.month() as u8,
            day = t.day(),
            hour = t.hour(),
            minute = t.minute(),
            second = t.second(),
            secfrac = secfrac,
            sfw = secfrac_width,
        )
    }
}

#[derive(Debug)]
pub(crate) struct SerdeDuration {
    seconds: i64,
    nanos: i32,
}

impl TryFrom<SerdeDuration> for Duration {
    type Error = DurationError;

    fn try_from(dur: SerdeDuration) -> Result<Duration, Self::Error> {
        Duration::try_new(dur.seconds, dur.nanos)
    }
}

impl From<Duration> for SerdeDuration {
    fn from(dur: Duration) -> SerdeDuration {
        SerdeDuration {
            seconds: dur.duration_seconds().into(),
            nanos: dur.nanoseconds().into(),
        }
    }
}

impl Serialize for SerdeDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let seconds = self.seconds;
        let nanos = self.nanos;
        let nanos = if nanos == 0 {
            "".to_string()
        } else {
            let mut secfrac = nanos.abs();
            let mut secfrac_width: usize = 9;
            while secfrac % 10 == 0 {
                secfrac /= 10;
                secfrac_width -= 1;
            }
            format!(".{secfrac:00$}", secfrac_width)
        };
        serializer.serialize_str(&format!("{}{}s", seconds, nanos))
    }
}

impl<'de> Deserialize<'de> for SerdeDuration {
    fn deserialize<D>(deserializer: D) -> Result<SerdeDuration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut s = String::deserialize(deserializer)?;
        if s.pop() != Some('s') {
            return Err(D::Error::custom("missing 's' suffix"));
        }

        let mut s = s.splitn(2, '.');

        let seconds = s
            .next()
            .expect("splitn(2,...) always returns at least one element");
        let seconds: i64 = seconds
            .parse()
            .map_err(|e| D::Error::custom(format!("invalid seconds, {e}")))?;

        let nanos = s.next();
        let nanos = match nanos {
            Some(n) => {
                if n.ends_with('0') {
                    return Err(D::Error::custom(
                        "invalid nanoseconds - contains trailing zero(s)",
                    ));
                } else {
                    // pad with trailing zeros if there are fewer than 9 digits
                    format!("{:0<9}", n)
                }
            }
            None => "0".to_string(),
        };

        let mut nanos = nanos
            .parse::<i32>()
            .map_err(|e| D::Error::custom(format!("invalid nanoseconds, {e}")))?;
        if seconds < 0 {
            nanos = -nanos;
        }

        Ok(SerdeDuration { seconds, nanos })
    }
}
