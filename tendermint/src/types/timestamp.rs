// The Timestamp struct is defined in gogoproto v1.3.1 at https://github.com/gogo/protobuf/blob/v1.3.1/protobuf/google/protobuf/timestamp.proto
// and https://github.com/protocolbuffers/protobuf-go/blob/v1.34.2/types/known/timestamppb/timestamp.pb.go

use super::time::Duration;
use chrono::SubsecRound;
use tendermint_proto::Protobuf;

// Slight modification of the RFC3339Nano but it right pads all zeros and drops the time zone info
const SORTABLE_DATE_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.000000000";

/// A Timestamp represents a point in time independent of any time zone or local
/// calendar, encoded as a count of seconds and fractions of seconds at
/// nanosecond resolution. The count is relative to an epoch at UTC midnight on
/// January 1, 1970, in the proleptic Gregorian calendar which extends the
/// Gregorian calendar backwards to year one.
///
/// All minutes are 60 seconds long. Leap seconds are "smeared" so that no leap
/// second table is needed for interpretation, using a [24-hour linear
/// smear](https://developers.google.com/time/smear).
///
/// The range is from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59.999999999Z. By
/// restricting to that range, we ensure that we can convert to and from [RFC
/// 3339](https://www.ietf.org/rfc/rfc3339.txt) date strings.
///
/// In JSON format, the Timestamp type is encoded as a string in the
/// [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format. That is, the
/// format is "{year}-{month}-{day}T{hour}:{min}:{sec}[.{frac_sec}]Z"
/// where {year} is always expressed using four digits while {month}, {day},
/// {hour}, {min}, and {sec} are zero-padded to two digits each. The fractional
/// seconds, which can go up to 9 digits (i.e. up to 1 nanosecond resolution),
/// are optional. The "Z" suffix indicates the timezone ("UTC"); the timezone
/// is required. A proto3 JSON serializer should always use UTC (as indicated by
/// "Z") when printing the Timestamp type and a proto3 JSON parser should be
/// able to accept both UTC and other timezones (as indicated by an offset).
///
/// For example, "2017-01-15T01:30:15.01Z" encodes 15.01 seconds past
/// 01:30 UTC on January 15, 2017.
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    serde::Serialize,
    serde::Deserialize,
    Copy,
    Ord,
    PartialOrd,
)]
#[serde(
    from = "crate::types::serializers::timestamp::Rfc3339",
    into = "crate::types::serializers::timestamp::Rfc3339"
)]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag = "1")]
    pub(crate) seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag = "2")]
    pub(crate) nanos: i32,
}

/// Errors that can occur when creating a `Timestamp`
#[derive(Debug, thiserror::Error)]
pub enum NewTimestampError {
    #[error("timestamp before 0001-01-01")]
    Underflow,
    #[error("timestamp after 9999-12-31")]
    Overflow,
    #[error("{0}")]
    Nanoseconds(#[from] NewNanosecondsError),
}

/// Errors that can occur when parsing a `Timestamp`
#[derive(Debug, thiserror::Error)]
pub enum TimestampParseError {
    #[error("{0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("{0}")]
    Format(String),
    #[error("timestamp before 0001-01-01 or timestamp after 9999-12-31")]
    OutOfRange,
}

impl From<chrono::ParseError> for TimestampParseError {
    fn from(err: chrono::ParseError) -> Self {
        match err.kind() {
            chrono::format::ParseErrorKind::OutOfRange => TimestampParseError::OutOfRange,
            _ => TimestampParseError::Format(err.to_string()),
        }
    }
}

impl From<NewTimestampSecondsError> for NewTimestampError {
    fn from(err: NewTimestampSecondsError) -> Self {
        match err {
            NewTimestampSecondsError::Underflow => NewTimestampError::Underflow,
            NewTimestampSecondsError::Overflow => NewTimestampError::Overflow,
        }
    }
}

impl From<NewTimestampSecondsError> for TimestampParseError {
    fn from(_: NewTimestampSecondsError) -> Self {
        TimestampParseError::OutOfRange
    }
}

/// Represents seconds of UTC time since Unix epoch,
/// 1970-01-01T00:00:00Z. Guaranteed to be from 0001-01-01T00:00:00Z to
/// 9999-12-31T23:59:59Z inclusive which corresponds to the range -62135596800..253402300800
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TimestampSeconds(i64);

impl TimestampSeconds {
    pub const MIN: Self = Self(-62135596800); // Seconds between 1970-01-01T00:00:00Z and 0001-01-01T00:00:00Z, inclusive
    pub const MAX: Self = Self(253402300799); // Seconds between 1970-01-01T00:00:00Z and 9999-12-31T23:59:59Z, inclusive
}

#[derive(Debug, thiserror::Error)]
pub enum NewTimestampSecondsError {
    #[error("timestamp seconds must be greater than or equal to -62135596800")]
    Underflow,
    #[error("timestamp seconds must be less than or equal to 253402300799")]
    Overflow,
}

impl TryFrom<i64> for TimestampSeconds {
    type Error = NewTimestampSecondsError;

    fn try_from(seconds: i64) -> Result<Self, Self::Error> {
        let seconds = Self(seconds);
        if seconds < Self::MIN {
            Err(NewTimestampSecondsError::Underflow)
        } else if seconds > Self::MAX {
            Err(NewTimestampSecondsError::Overflow)
        } else {
            Ok(seconds)
        }
    }
}

impl From<TimestampSeconds> for i64 {
    fn from(TimestampSeconds(seconds): TimestampSeconds) -> i64 {
        seconds
    }
}

/// Represents non-negative fractions of a second at nanosecond resolution. Guaranteed to be in the range 0..1_000_000_000
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Nanoseconds(u32);

impl Nanoseconds {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(999_999_999);
    pub const ZERO: Self = Self(0);
}

#[derive(Debug, thiserror::Error)]
#[error("nanoseconds must be less than 1,000,000,000 and non-negative")]
pub struct NewNanosecondsError;

impl TryFrom<u32> for Nanoseconds {
    type Error = NewNanosecondsError;

    fn try_from(nanos: u32) -> Result<Self, Self::Error> {
        let nanos = Self(nanos);
        if nanos > Self::MAX {
            Err(NewNanosecondsError)
        } else {
            Ok(nanos)
        }
    }
}

impl TryFrom<i32> for Nanoseconds {
    type Error = NewNanosecondsError;

    fn try_from(nanos: i32) -> Result<Self, Self::Error> {
        if nanos < 0 {
            Err(NewNanosecondsError)
        } else {
            Self::try_from(nanos as u32)
        }
    }
}

impl From<Nanoseconds> for i32 {
    fn from(Nanoseconds(nanos): Nanoseconds) -> i32 {
        nanos as i32
    }
}

/// Represents nanoseconds of UTC time since Unix epoch,
/// 1970-01-01T00:00:00Z. Guaranteed to be from 0001-01-01T00:00:00Z to
/// 9999-12-31T23:59:59Z inclusive which corresponds to the range --62135596800_000_000_000..253402300799_999_999_999
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TimestampNanoseconds(i128);

impl TimestampNanoseconds {
    pub const MIN: Self = Self(-62135596800_000_000_000); // Nanoseconds between 1970-01-01T00:00:00Z and 0001-01-01T00:00:00Z, inclusive
    pub const MAX: Self = Self(253402300799_999_999_999); // Nanoseconds between 1970-01-01T00:00:00Z and 9999-12-31T23:59:59Z, inclusive
}

impl From<TimestampNanoseconds> for i128 {
    fn from(TimestampNanoseconds(nanos): TimestampNanoseconds) -> i128 {
        nanos
    }
}

impl Timestamp {
    /// `Timestamp` of Unix epoch
    pub const UNIX_EPOCH: Timestamp = Timestamp {
        seconds: 0,
        nanos: 0,
    };

    /// Creates a new `Timestamp` from the given seconds and nanoseconds.
    /// `seconds` represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive which corresponds to the range -62135596800..253402300800
    /// `nanos` represents non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    pub fn try_new(seconds: i64, nanos: i32) -> Result<Self, NewTimestampError> {
        let seconds = TimestampSeconds::try_from(seconds)?;
        let nanos = Nanoseconds::try_from(nanos)?;
        Ok(Self::new(seconds, nanos))
    }

    /// Creates a new `Timestamp` from the given seconds and nanoseconds.
    pub fn new(seconds: TimestampSeconds, nanos: Nanoseconds) -> Self {
        Self {
            seconds: seconds.into(),
            nanos: nanos.into(),
        }
    }

    /// Returns the number of whole seconds since Unix epoch
    pub fn timestamp_seconds(&self) -> TimestampSeconds {
        if self.seconds < 0 && self.nanos > 0 {
            TimestampSeconds(self.seconds + 1)
        } else {
            TimestampSeconds(self.seconds)
        }
    }

    /// Returns the number of nanoseconds since Unix epoch
    pub fn timestamp_nanoseconds(&self) -> TimestampNanoseconds {
        TimestampNanoseconds(self.nanos as i128 + self.seconds as i128 * 1_000_000_000)
    }

    /// Returns the number of nanoseconds since the last second boundary
    pub fn nanoseconds(&self) -> Nanoseconds {
        Nanoseconds(self.nanos as u32)
    }

    /// Formats a `Timestamp` into `Vec<u8>` that can be sorted.
    /// The bytes are in the format "YYYY-MM-DDTHH:MM:SS.000000000" encoded as UTF-8.
    /// The time is rounded to the nearest second.
    pub fn format_bytes_rounded(&self) -> Vec<u8> {
        self.format_string_rounded().into_bytes()
    }

    /// Formats a `Timestamp` into a `String` that can be sorted.
    /// The `String` is in the format "YYYY-MM-DDTHH:MM:SS.000000000".
    /// The time is rounded to the nearest second.
    pub fn format_string_rounded(&self) -> String {
        chrono::DateTime::from_timestamp(self.seconds, self.nanos as u32)
            .expect("`Self` is within `chrono::DateTime` range")
            .round_subsecs(0)
            .format(SORTABLE_DATE_TIME_FORMAT)
            .to_string()
    }

    /// Returns a `Timestamp` from a byte slice formatted as in `format_bytes`.
    pub fn try_from_formatted_bytes(bytes: &[u8]) -> Result<Timestamp, TimestampParseError> {
        let s = std::str::from_utf8(bytes)?;
        let dt = chrono::NaiveDateTime::parse_from_str(s, SORTABLE_DATE_TIME_FORMAT)?;
        let seconds = TimestampSeconds::try_from(dt.and_utc().timestamp())?;
        let nanoseconds = Nanoseconds::ZERO; //NOTE: SORTABLE_DATE_TIME_FORMAT ensures nanoseconds are always zero
        Ok(Timestamp::new(seconds, nanoseconds))
    }

    /// Adds a `Duration` to the `Timestamp`, returning a new `Timestamp`.
    /// Returns `None` if the resulting `Timestamp` is outside the valid range.
    pub fn checked_add(self, rhs: Duration) -> Option<Self> {
        let mut seconds = self.seconds.checked_add(rhs.seconds)?;
        let mut nanos = self.nanos.checked_add(rhs.nanos)?;

        if nanos >= 1_000_000_000 {
            seconds = seconds.checked_add(1)?;
            nanos -= 1_000_000_000;
        } else if nanos < 0 {
            seconds = seconds.checked_sub(1)?;
            nanos += 1_000_000_000;
        }

        Timestamp::try_new(seconds, nanos).ok()
    }

    /// Subtracts a `Timestamp` from another `Timestamp`, returning a new `Duration`.
    /// Equivalently, calculates the `Duration` between the two `Timestamp`'s.
    /// Returns `None` if the difference is outside the range of `Duration`.
    pub fn checked_sub(self, rhs: &Timestamp) -> Option<Duration> {
        let ts_self: i128 = self.timestamp_nanoseconds().into();
        let ts_rhs = rhs.timestamp_nanoseconds().into();

        let nano_diff = ts_self.checked_sub(ts_rhs)?;

        Duration::try_new_from_nanos(nano_diff).ok()
    }
}

impl From<inner::Timestamp> for Timestamp {
    fn from(inner::Timestamp { seconds, nanos }: inner::Timestamp) -> Self {
        Self { seconds, nanos }
    }
}

impl From<Timestamp> for inner::Timestamp {
    fn from(Timestamp { seconds, nanos }: Timestamp) -> Self {
        Self { seconds, nanos }
    }
}

impl Protobuf<Timestamp> for Timestamp {}

pub mod inner {
    pub use tendermint_proto::google::protobuf::Timestamp;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_try_new() {
        let ts = Timestamp::try_new(0, 0).unwrap();
        assert_eq!(ts, Timestamp::UNIX_EPOCH);

        let err = Timestamp::try_new(1, 1_000_000_000).unwrap_err();
        assert!(matches!(err, NewTimestampError::Nanoseconds(_)));

        let err = Timestamp::try_new(1, -1).unwrap_err();
        assert!(matches!(err, NewTimestampError::Nanoseconds(_)));

        let err = Timestamp::try_new(-62135596801, 0).unwrap_err();
        assert!(matches!(err, NewTimestampError::Underflow));

        let err = Timestamp::try_new(253402300800, 0).unwrap_err();
        assert!(matches!(err, NewTimestampError::Overflow));
    }

    #[test]
    fn test_checked_add() {
        let ts = Timestamp::try_new(0, 0).unwrap();
        let dur = Duration::try_new(1, 0).unwrap();
        let ts = ts.checked_add(dur).unwrap();
        assert_eq!(ts, Timestamp::try_new(1, 0).unwrap());

        let ts = Timestamp::try_new(0, 0).unwrap();
        let dur = Duration::try_new(0, -1).unwrap();
        let ts = ts.checked_add(dur).unwrap();
        assert_eq!(ts, Timestamp::try_new(-1, 999_999_999).unwrap());

        let ts = Timestamp::try_new(0, 999_999_999).unwrap();
        let dur = Duration::try_new(0, 999_999_999).unwrap();
        let ts = ts.checked_add(dur).unwrap();
        assert_eq!(ts, Timestamp::try_new(1, 999_999_998).unwrap());

        let ts = Timestamp::try_new(253402300799, 999_999_999).unwrap();
        let dur = Duration::try_new(0, 1).unwrap();
        let ts = ts.checked_add(dur);
        assert!(ts.is_none());
    }

    #[test]
    fn test_timestamp_nanoseconds() {
        let ts = Timestamp::try_new(1, 234).unwrap();
        let ts: i128 = ts.timestamp_nanoseconds().into();
        assert_eq!(ts, 1_000_000_234);

        let ts = Timestamp::try_new(-1, 234).unwrap();
        let ts: i128 = ts.timestamp_nanoseconds().into();
        assert_eq!(ts, -999_999_766);
    }

    #[test]
    fn test_timestamp_seconds() {
        let ts = Timestamp::try_new(1, 234).unwrap();
        let ts: i64 = ts.timestamp_seconds().into();
        assert_eq!(ts, 1);

        let ts = Timestamp::try_new(-1, 234).unwrap();
        let ts: i64 = ts.timestamp_seconds().into();
        assert_eq!(ts, 0);

        let ts = Timestamp::try_new(-1, 0).unwrap();
        let ts: i64 = ts.timestamp_seconds().into();
        assert_eq!(ts, -1);
    }

    #[test]
    fn test_checked_sub() {
        let ts1 = Timestamp::try_new(1, 0).unwrap();
        let ts2 = Timestamp::try_new(0, 0).unwrap();
        let dur = ts1.checked_sub(&ts2).unwrap();
        assert_eq!(dur, Duration::try_new(1, 0).unwrap());

        let dur = ts2.checked_sub(&ts1).unwrap();
        assert_eq!(dur, Duration::try_new(-1, 0).unwrap());

        let ts1 = Timestamp::try_new(1, 0).unwrap();
        let ts2 = Timestamp::try_new(0, 1).unwrap();
        let dur = ts1.checked_sub(&ts2).unwrap();
        assert_eq!(dur, Duration::try_new(0, 999_999_999).unwrap());

        let dur = ts2.checked_sub(&ts1).unwrap();
        assert_eq!(dur, Duration::try_new(0, -999_999_999).unwrap());
    }

    #[test]
    fn test_format_string_rounded() {
        let ts = Timestamp::try_new(1, 234_000_000).unwrap();
        assert_eq!(ts.format_string_rounded(), "1970-01-01T00:00:01.000000000");

        let ts = Timestamp::try_new(-1, 234_000_000).unwrap();
        assert_eq!(ts.format_string_rounded(), "1969-12-31T23:59:59.000000000");

        let ts = Timestamp::try_new(-1, 734_000_000).unwrap();
        assert_eq!(ts.format_string_rounded(), "1970-01-01T00:00:00.000000000");
    }

    #[test]
    fn test_try_from_formatted_bytes() {
        let bytes = b"1969-12-31T23:59:59.000000000";
        let ts2 = Timestamp::try_from_formatted_bytes(bytes.as_slice()).unwrap();
        let ts3 = Timestamp::try_new(-1, 0).unwrap();
        assert_eq!(ts3, ts2);

        // nanoseconds should be zero
        let bytes = b"1969-12-31T23:59:59.100000000";
        let ts2 = Timestamp::try_from_formatted_bytes(bytes.as_slice()).unwrap_err();
        assert!(matches!(ts2, TimestampParseError::Format(_)));

        // too big
        let bytes = b"+9999999-01-01T23:59:59.000000000";
        let ts2 = Timestamp::try_from_formatted_bytes(bytes.as_slice()).unwrap_err();
        assert!(matches!(ts2, TimestampParseError::OutOfRange));

        // leaving off the `+` sign should be a format error
        let bytes = b"9999999-01-01T23:59:59.000000000";
        let ts2 = Timestamp::try_from_formatted_bytes(bytes.as_slice()).unwrap_err();
        assert!(matches!(ts2, TimestampParseError::Format(_)));

        // too small
        let bytes = b"-9999999-01-01T23:59:59.000000000";
        let ts2 = Timestamp::try_from_formatted_bytes(bytes.as_slice()).unwrap_err();
        assert!(matches!(ts2, TimestampParseError::OutOfRange));
    }
}
