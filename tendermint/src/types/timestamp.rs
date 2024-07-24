// The Timestamp struct is defined in gogoproto v1.3.1 at https://github.com/gogo/protobuf/blob/v1.3.1/protobuf/google/protobuf/timestamp.proto
// and https://github.com/protocolbuffers/protobuf-go/blob/v1.34.2/types/known/timestamppb/timestamp.pb.go

use super::time::Duration;
use chrono::SubsecRound;
use tendermint_proto::Protobuf;

// Slight modification of the RFC3339Nano but it right pads all zeros and drops the time zone info
const SORTABLE_DATE_TIME_FORMAT: &str = "%Y-%m-%dT&H:%M:%S.000000000";

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
    PartialOrd, //TODO: Ord and PartialOrd how is max implemented
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

#[derive(Debug, thiserror::Error)]
pub enum TimestampError {
    #[error("timestamp before 0001-01-01")]
    Underflow,
    #[error("timestamp after 9999-12-31")]
    Overflow,
    #[error("nanoseconds must be in the range 0..1_000_000_000")]
    Nanoseconds,
}

impl Timestamp {
    pub const UNIX_EPOCH: Timestamp = Timestamp {
        seconds: 0,
        nanos: 0,
    };

    pub fn try_new(seconds: i64, nanos: i32) -> Result<Self, TimestampError> {
        let ts = Timestamp { seconds, nanos };
        ts.check()?;
        Ok(ts)
    }

    pub fn timestamp_seconds(&self) -> i64 {
        self.seconds
    }

    pub fn timestamp_nanoseconds(&self) -> i128 {
        self.nanos as i128 + self.seconds as i128 * 1_000_000_000
    }

    /// Returns the number of nanoseconds since the last second boundary
    pub fn nanoseconds(&self) -> i32 {
        self.nanos
    }

    /// Formats a `Timestamp` into a vector of bytes that can be sorted
    // TODO: test this method
    pub fn format_timestamp_bytes(&self) -> Vec<u8> {
        chrono::DateTime::from_timestamp(self.seconds, self.nanos as u32)
            .expect("valid timestamp")
            .round_subsecs(0)
            .format(SORTABLE_DATE_TIME_FORMAT)
            .to_string()
            .as_bytes()
            .to_vec()
    }

    // TODO: this is not the right implementation
    pub fn inverse_format_timestamp_bytes(bytes: &[u8]) -> Result<Timestamp, TimestampError> {
        let s = std::str::from_utf8(bytes).map_err(|_| TimestampError::Overflow)?; //TODO: is this the right error
        let dt = chrono::DateTime::parse_from_str(s, SORTABLE_DATE_TIME_FORMAT)
            .map_err(|_| TimestampError::Overflow)?;
        let ts = dt.timestamp_nanos();
        let seconds = ts / 1_000_000_000;
        let nanos = (ts % 1_000_000_000) as i32;
        Timestamp::try_new(seconds, nanos)
    }

    //TODO: is there a name for this format
    pub fn format_string(&self) -> String {
        chrono::DateTime::from_timestamp(self.seconds, self.nanos as u32)
            .expect("valid timestamp")
            .format(SORTABLE_DATE_TIME_FORMAT)
            .to_string()
    }

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

        let ts = Timestamp { seconds, nanos };
        ts.check().ok()?;
        Some(ts)
    }

    pub fn checked_sub(self, rhs: &Timestamp) -> Option<Duration> {
        let ts_self = self.timestamp_nanoseconds();
        let ts_rhs = rhs.timestamp_nanoseconds();

        let nano_diff = ts_self.checked_sub(ts_rhs)?;

        Duration::try_new_from_nanos(nano_diff).ok()
    }

    // based on https://github.com/protocolbuffers/protobuf-go/blob/c33baa8f3a0d35fd5a39e43c22a50a050f707d34/types/known/timestamppb/timestamp.pb.go#L236C1-L253C2
    fn check(&self) -> Result<(), TimestampError> {
        const MIN_TIMESTAMP: i64 = -62135596800; // Seconds between 1970-01-01T00:00:00Z and 0001-01-01T00:00:00Z, inclusive
        const MAX_TIMESTAMP: i64 = 253402300799; // Seconds between 1970-01-01T00:00:00Z and 9999-12-31T23:59:59Z, inclusive

        if self.seconds < MIN_TIMESTAMP {
            Err(TimestampError::Underflow)
        } else if self.seconds > MAX_TIMESTAMP {
            Err(TimestampError::Overflow)
        } else if self.nanos < 0 || self.nanos >= 1_000_000_000 {
            Err(TimestampError::Nanoseconds)
        } else {
            Ok(())
        }
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
    fn test_timestamp() {
        let ts = Timestamp::try_new(0, 0).unwrap();
        assert_eq!(ts, Timestamp::UNIX_EPOCH);

        let err = Timestamp::try_new(1, 1_000_000_000).unwrap_err();
        assert!(matches!(err, TimestampError::Nanoseconds));

        let err = Timestamp::try_new(1, -1).unwrap_err();
        assert!(matches!(err, TimestampError::Nanoseconds));

        let err = Timestamp::try_new(-62135596801, 0).unwrap_err();
        assert!(matches!(err, TimestampError::Underflow));

        let err = Timestamp::try_new(253402300800, 0).unwrap_err();
        assert!(matches!(err, TimestampError::Overflow));
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
}
