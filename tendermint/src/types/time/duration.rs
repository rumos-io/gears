// The Duration struct is defined in gogoproto v1.3.1 at https://github.com/gogo/protobuf/blob/v1.3.1/protobuf/google/protobuf/duration.proto
// and https://github.com/protocolbuffers/protobuf-go/blob/v1.34.2/types/known/durationpb/duration.pb.go

const SECONDS_PER_HOUR: i64 = 60 * 60;
const NANOS_PER_SECOND: i32 = 1_000_000_000;
const ABS_DURATION_SECONDS: i64 = 315_576_000_000; // 10000yr * 365.25day/yr * 24hr/day * 60min/hr * 60sec/min
const ABS_DURATION_HOURS: i64 = ABS_DURATION_SECONDS / SECONDS_PER_HOUR;
const ABS_DURATION_NANOSECONDS: i128 =
    ABS_DURATION_SECONDS as i128 * NANOS_PER_SECOND as i128 + NANOS_PER_SECOND as i128 - 1;

/// A Duration represents a signed, fixed-length span of time represented
/// as a count of seconds and fractions of seconds at nanosecond
/// resolution. It is independent of any calendar and concepts like "day"
/// or "month". It is related to Timestamp in that the difference between
/// two Timestamp values is a Duration and it can be added
/// from a Timestamp. Range is approximately +-10,000 years.
///
/// In JSON format, the Duration type is encoded as a string rather than an
/// object, where the string ends in the suffix "s" (indicating seconds) and
/// is preceded by the number of seconds, with nanoseconds expressed as
/// fractional seconds. For example, 3 seconds with 0 nanoseconds should be
/// encoded in JSON format as "3s", while 3 seconds and 1 nanosecond should
/// be expressed in JSON format as "3.000000001s", and 3 seconds and 1
/// microsecond should be expressed in JSON format as "3.000001s".
#[derive(Clone, Copy, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
#[serde(
    try_from = "super::serializers::SerdeDuration",
    into = "super::serializers::SerdeDuration"
)]
pub struct Duration {
    /// Signed seconds of the span of time. Must be from -315,576,000,000
    /// to +315,576,000,000 inclusive. Note: these bounds are computed from:
    /// 60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
    #[prost(int64, tag = "1")]
    pub(crate) seconds: i64,
    /// Signed fractions of a second at nanosecond resolution of the span
    /// of time. Durations less than one second are represented with a 0
    /// `seconds` field and a positive or negative `nanos` field. For durations
    /// of one second or more, a non-zero value for the `nanos` field must be
    /// of the same sign as the `seconds` field. Must be from -999,999,999
    /// to +999,999,999 inclusive.
    #[prost(int32, tag = "2")]
    pub(crate) nanos: i32,
}

/// Errors that can occur when creating a new `Duration`.
#[derive(Debug, thiserror::Error)]
pub enum DurationError {
    #[error("duration exceeds -315,576,000,001 seconds (approx -10,000 years)")]
    Underflow,
    #[error("duration exceeds 315,576,000,001 seconds (approx 10,000 years")]
    Overflow,
    #[error("nanoseconds must be in the range [-999_999_999_999, 999_999_999_999] inclusive")]
    Nanoseconds,
    #[error("for durations of one second or more, a non-zero value for the nanoseconds field must be of the same sign as the seconds field")]
    Sign,
}

/// Represents fractions of a second at nanosecond resolution. Guaranteed to be in the range [-999,999,999, 999,999,999] inclusive.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Nanoseconds(i32);

impl Nanoseconds {
    pub const MIN: Self = Self(-NANOS_PER_SECOND + 1);
    pub const MAX: Self = Self(NANOS_PER_SECOND - 1);
    pub const ZERO: Self = Self(0);
}

impl From<Nanoseconds> for i32 {
    fn from(Nanoseconds(nanos): Nanoseconds) -> i32 {
        nanos
    }
}

/// Represents the number of whole hours in a `Duration`. Guaranteed to be in the range [-87,660,000, 87,660,000] inclusive.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DurationHours(i64);

impl DurationHours {
    pub const MIN: Self = Self(-ABS_DURATION_HOURS);
    pub const MAX: Self = Self(ABS_DURATION_HOURS);
}

impl From<DurationHours> for i64 {
    fn from(DurationHours(hours): DurationHours) -> i64 {
        hours
    }
}

/// Represents the number of whole seconds in a `Duration`. Guaranteed to be in the range [-315,576,000,000, 315,576,000,000] inclusive.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DurationSeconds(i64);

impl DurationSeconds {
    pub const MIN: Self = Self(-ABS_DURATION_SECONDS);
    pub const MAX: Self = Self(ABS_DURATION_SECONDS);
}

impl From<DurationSeconds> for i64 {
    fn from(DurationSeconds(secs): DurationSeconds) -> i64 {
        secs
    }
}

/// Represents the number of nanoseconds in a `Duration`.
/// Guaranteed to be in the range [-315,576,000,000,999,999,999, 315,576,000,000,999,999,999] inclusive.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DurationNanoseconds(i128);

impl DurationNanoseconds {
    pub const MIN: Self = Self(-ABS_DURATION_NANOSECONDS);
    pub const MAX: Self = Self(ABS_DURATION_NANOSECONDS);
}

impl From<DurationNanoseconds> for i128 {
    fn from(DurationNanoseconds(nanos): DurationNanoseconds) -> i128 {
        nanos
    }
}

impl Duration {
    /// The minimum possible `Duration` value.
    pub const MIN: Duration = Duration {
        seconds: -ABS_DURATION_SECONDS,
        nanos: -NANOS_PER_SECOND + 1,
    };

    /// The maximum possible `Duration` value.
    pub const MAX: Duration = Duration {
        seconds: ABS_DURATION_SECONDS,
        nanos: NANOS_PER_SECOND - 1,
    };
    /// Creates a new `Duration` from the given number of seconds and nanoseconds.
    /// Returns an error if `seconds` is out of the range [-315,576,000,000, 315,576,000,000]
    /// or if `nanos` is out of the range [-999_999_999, 999_999_999]. Also returns an error
    /// if `nanos` is non-zero and has a different sign than `seconds`.
    pub fn try_new(seconds: i64, nanos: i32) -> Result<Duration, DurationError> {
        let duration = Duration { seconds, nanos };
        duration.check()?;
        Ok(duration)
    }

    /// Creates a new `Duration` from the given number of seconds
    pub const fn new_from_secs(secs: i32) -> Duration {
        // NOTE: this is safe because:
        // i32::MAX < ABS_DURATION_SECONDS
        // i32::MIN > -ABS_DURATION_SECONDS
        Duration {
            seconds: secs as i64,
            nanos: 0,
        }
    }

    /// Creates a new `Duration` from the given number of nanoseconds
    pub const fn new_from_nanos(nanos: i64) -> Duration {
        let seconds = nanos / NANOS_PER_SECOND as i64;
        let nanos = (nanos % NANOS_PER_SECOND as i64) as i32; // cast is safe because of modulo operation

        // NOTE: this is safe, the seconds are always valid:
        // i64::MAX / NANOS_PER_SECOND < ABS_DURATION_SECONDS
        // i64::MIN / NANOS_PER_SECOND > -ABS_DURATION_SECONDS
        Duration { seconds, nanos }
    }

    /// Creates a new `Duration` from the given number of nanoseconds
    /// Returns an error if the number of nanoseconds is out of the range [-315,576,000,000,999,999,999, 315,576,000,000,999,999,999,] inclusive.
    pub fn try_new_from_nanos(nanos: i128) -> Result<Duration, DurationError> {
        let seconds = i64::try_from(nanos / NANOS_PER_SECOND as i128).map_err(|_| {
            if nanos > 0 {
                DurationError::Overflow
            } else {
                DurationError::Underflow
            }
        })?;

        let nanos = (nanos % NANOS_PER_SECOND as i128) as i32; // cast is safe because of modulo operation
        Self::try_new(seconds, nanos)
    }

    /// Returns the number of whole hours in the `Duration`.
    pub fn duration_hours(&self) -> DurationHours {
        // NOTE: we can ignore the nanoseconds here. Since the nanoseconds are always less
        // than a second, they will never be enough to push the total seconds over an hour boundary.
        DurationHours(self.seconds / SECONDS_PER_HOUR)
    }

    /// Returns the number of whole seconds in the `Duration`.
    pub fn duration_seconds(&self) -> DurationSeconds {
        DurationSeconds(self.seconds)
    }

    /// Returns the fractional seconds component of the `Duration` in nanoseconds.
    pub fn nanoseconds(&self) -> Nanoseconds {
        Nanoseconds(self.nanos)
    }

    // based on https://github.com/protocolbuffers/protobuf-go/blob/c33baa8f3a0d35fd5a39e43c22a50a050f707d34/types/known/durationpb/duration.pb.go#L225C1-L244C2
    fn check(&self) -> Result<(), DurationError> {
        if self.seconds < -ABS_DURATION_SECONDS {
            Err(DurationError::Underflow)
        } else if self.seconds > ABS_DURATION_SECONDS {
            Err(DurationError::Overflow)
        } else if self.nanos <= -NANOS_PER_SECOND || self.nanos >= NANOS_PER_SECOND {
            Err(DurationError::Nanoseconds)
        } else if (self.seconds > 0 && self.nanos < 0) || (self.seconds < 0 && self.nanos > 0) {
            Err(DurationError::Sign)
        } else {
            Ok(())
        }
    }

    /// Returns the total number of nanoseconds in the `Duration`.
    pub fn duration_nanoseconds(&self) -> DurationNanoseconds {
        DurationNanoseconds(i128::from(self.seconds) * 1_000_000_000 + i128::from(self.nanos))
    }
}

impl TryFrom<tendermint_proto::google::protobuf::Duration> for Duration {
    type Error = DurationError;

    fn try_from(
        duration: tendermint_proto::google::protobuf::Duration,
    ) -> Result<Self, Self::Error> {
        Self::try_new(duration.seconds, duration.nanos)
    }
}

impl From<Duration> for inner::Duration {
    fn from(Duration { seconds, nanos }: Duration) -> Self {
        Self { seconds, nanos }
    }
}

pub mod inner {
    pub use tendermint_proto::google::protobuf::Duration;
}

#[cfg(test)]
mod tests {
    use std::i32;

    use super::*;

    #[test]
    fn test_try_new() {
        let duration = Duration::try_new(1, 0).unwrap();
        assert_eq!(
            duration,
            Duration {
                seconds: 1,
                nanos: 0
            }
        );

        let duration =
            Duration::try_new(DurationSeconds::MIN.into(), Nanoseconds::MIN.into()).unwrap();
        assert_eq!(duration, Duration::MIN);

        let duration =
            Duration::try_new(DurationSeconds::MAX.into(), Nanoseconds::MAX.into()).unwrap();
        assert_eq!(duration, Duration::MAX);

        let dur_error = Duration::try_new(i64::from(DurationSeconds::MIN) - 1, 0).unwrap_err();
        assert!(matches!(dur_error, DurationError::Underflow));

        let dur_error = Duration::try_new(i64::from(DurationSeconds::MAX) + 1, 0).unwrap_err();
        assert!(matches!(dur_error, DurationError::Overflow));

        let dur_error = Duration::try_new(0, i32::from(Nanoseconds::MIN) - 1).unwrap_err();
        assert!(matches!(dur_error, DurationError::Nanoseconds));
    }

    #[test]
    fn test_new_from_secs() {
        let duration = Duration::new_from_secs(i32::MAX);
        assert_eq!(
            duration,
            Duration {
                seconds: 2147483647,
                nanos: 0
            }
        );

        let duration = Duration::new_from_secs(i32::MIN);
        assert_eq!(
            duration,
            Duration {
                seconds: -2147483648,
                nanos: 0
            }
        );
    }

    #[test]
    fn test_new_from_nanos() {
        let duration = Duration::new_from_nanos(1_000_000_001);
        assert_eq!(
            duration,
            Duration {
                seconds: 1,
                nanos: 1
            }
        );

        let duration = Duration::new_from_nanos(-1_000_000_001);
        assert_eq!(
            duration,
            Duration {
                seconds: -1,
                nanos: -1
            }
        );

        let duration = Duration::new_from_nanos(1_000_000_000);
        assert_eq!(
            duration,
            Duration {
                seconds: 1,
                nanos: 0
            }
        );

        let duration = Duration::new_from_nanos(-1_000_000_000);
        assert_eq!(
            duration,
            Duration {
                seconds: -1,
                nanos: 0
            }
        );

        let duration = Duration::new_from_nanos(0);
        assert_eq!(
            duration,
            Duration {
                seconds: 0,
                nanos: 0
            }
        );

        let duration = Duration::new_from_nanos(i64::MIN);
        assert_eq!(
            duration,
            Duration {
                seconds: -9_223_372_036,
                nanos: -854_775_808
            }
        );

        let duration = Duration::new_from_nanos(i64::MAX);
        assert_eq!(
            duration,
            Duration {
                seconds: 9_223_372_036,
                nanos: 854_775_807
            }
        );
    }

    #[test]
    fn test_try_new_from_nanos() {
        let duration = Duration::try_new_from_nanos(1_000_000_001).unwrap();
        assert_eq!(
            duration,
            Duration {
                seconds: 1,
                nanos: 1
            }
        );

        let duration = Duration::try_new_from_nanos(-1_000_000_001).unwrap();
        assert_eq!(
            duration,
            Duration {
                seconds: -1,
                nanos: -1
            }
        );

        let duration = Duration::try_new_from_nanos(1_000_000_000).unwrap();
        assert_eq!(
            duration,
            Duration {
                seconds: 1,
                nanos: 0
            }
        );

        let duration = Duration::try_new_from_nanos(-1_000_000_000).unwrap();
        assert_eq!(
            duration,
            Duration {
                seconds: -1,
                nanos: 0
            }
        );

        let duration = Duration::try_new_from_nanos(0).unwrap();
        assert_eq!(
            duration,
            Duration {
                seconds: 0,
                nanos: 0
            }
        );

        let duration = Duration::try_new_from_nanos(DurationNanoseconds::MAX.into()).unwrap();
        assert_eq!(duration, Duration::MAX);

        let duration = Duration::try_new_from_nanos(DurationNanoseconds::MIN.into()).unwrap();
        assert_eq!(duration, Duration::MIN);

        let dur_error =
            Duration::try_new_from_nanos(i128::from(DurationNanoseconds::MIN) - 1).unwrap_err();
        assert!(matches!(dur_error, DurationError::Underflow));

        let dur_error =
            Duration::try_new_from_nanos(i128::from(DurationNanoseconds::MAX) + 1).unwrap_err();
        assert!(matches!(dur_error, DurationError::Overflow));
    }

    #[test]
    fn test_duration_hours() {
        let duration = Duration::try_new(3 * SECONDS_PER_HOUR + 1, 0).unwrap();
        assert_eq!(duration.duration_hours(), DurationHours(3));

        let duration = Duration::try_new(3 * SECONDS_PER_HOUR - 1, 0).unwrap();
        assert_eq!(duration.duration_hours(), DurationHours(2));

        let duration = Duration::try_new(-3 * SECONDS_PER_HOUR - 1, 0).unwrap();
        assert_eq!(duration.duration_hours(), DurationHours(-3));

        let duration = Duration::try_new(-3 * SECONDS_PER_HOUR + 1, 0).unwrap();
        assert_eq!(duration.duration_hours(), DurationHours(-2));

        let duration = Duration::try_new(0, 0).unwrap();
        assert_eq!(duration.duration_hours(), DurationHours(0));

        let duration = Duration::MAX;
        assert_eq!(duration.duration_hours(), DurationHours::MAX);

        let duration = Duration::MIN;
        assert_eq!(duration.duration_hours(), DurationHours::MIN);
    }

    #[test]
    fn test_duration_seconds() {
        let duration = Duration::try_new(3, 1).unwrap();
        assert_eq!(duration.duration_seconds(), DurationSeconds(3));

        let duration = Duration::try_new(-3, -1).unwrap();
        assert_eq!(duration.duration_seconds(), DurationSeconds(-3));

        let duration = Duration::try_new(0, 0).unwrap();
        assert_eq!(duration.duration_seconds(), DurationSeconds(0));

        let duration = Duration::MAX;
        assert_eq!(duration.duration_seconds(), DurationSeconds::MAX);

        let duration = Duration::MIN;
        assert_eq!(duration.duration_seconds(), DurationSeconds::MIN);
    }

    #[test]
    fn test_nanoseconds() {
        let duration = Duration::try_new(3, 1).unwrap();
        assert_eq!(duration.nanoseconds(), Nanoseconds(1));

        let duration = Duration::try_new(-3, -1).unwrap();
        assert_eq!(duration.nanoseconds(), Nanoseconds(-1));

        let duration = Duration::try_new(0, 0).unwrap();
        assert_eq!(duration.nanoseconds(), Nanoseconds(0));

        let duration = Duration::try_new(0, 999_999_999).unwrap();
        assert_eq!(duration.nanoseconds(), Nanoseconds::MAX);

        let duration = Duration::try_new(0, -999_999_999).unwrap();
        assert_eq!(duration.nanoseconds(), Nanoseconds::MIN);
    }

    #[test]
    fn test_duration_nanoseconds() {
        let duration = Duration::try_new(3, 1).unwrap();
        assert_eq!(
            duration.duration_nanoseconds(),
            DurationNanoseconds(3_000_000_001)
        );

        let duration = Duration::try_new(-3, -1).unwrap();
        assert_eq!(
            duration.duration_nanoseconds(),
            DurationNanoseconds(-3_000_000_001)
        );

        let duration = Duration::try_new(0, 0).unwrap();
        assert_eq!(duration.duration_nanoseconds(), DurationNanoseconds(0));

        let duration = Duration::try_new(0, 999_999_999).unwrap();
        assert_eq!(
            duration.duration_nanoseconds(),
            DurationNanoseconds(999_999_999)
        );

        let duration = Duration::try_new(0, -999_999_999).unwrap();
        assert_eq!(
            duration.duration_nanoseconds(),
            DurationNanoseconds(-999_999_999)
        );

        let duration = Duration::MAX;
        assert_eq!(duration.duration_nanoseconds(), DurationNanoseconds::MAX);

        let duration = Duration::MIN;
        assert_eq!(duration.duration_nanoseconds(), DurationNanoseconds::MIN);
    }

    #[test]
    fn test_serialization() {
        let duration = Duration::try_new(3, 1).unwrap();
        let serialized = serde_json::to_string(&duration).unwrap();
        assert_eq!(serialized, r#""3.000000001s""#);

        let deserialized: Duration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, duration);

        //--------------------------------------------

        let duration = Duration::try_new(3, 0).unwrap();
        let serialized = serde_json::to_string(&duration).unwrap();
        assert_eq!(serialized, r#""3s""#);

        let deserialized: Duration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, duration);

        //--------------------------------------------

        let duration = Duration::try_new(-3, -1).unwrap();
        let serialized = serde_json::to_string(&duration).unwrap();
        assert_eq!(serialized, r#""-3.000000001s""#);

        let deserialized: Duration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, duration);

        //--------------------------------------------

        let duration = Duration::try_new(3, 1000).unwrap();
        let serialized = serde_json::to_string(&duration).unwrap();
        assert_eq!(serialized, r#""3.000001s""#);

        let deserialized: Duration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, duration);

        //--------------------------------------------
        // no trailing zeros

        let serialized = r#""3.000001000s""#;

        serde_json::from_str::<Duration>(&serialized).unwrap_err();

        //--------------------------------------------
        // too big

        let serialized = r#""3.1234567891s""#;

        serde_json::from_str::<Duration>(&serialized).unwrap_err();

        //--------------------------------------------
        // empty apart from 's' suffix

        let serialized = r#""s""#;

        serde_json::from_str::<Duration>(&serialized).unwrap_err();

        //--------------------------------------------
        // empty

        let serialized = r#""""#;

        serde_json::from_str::<Duration>(&serialized).unwrap_err();

        //--------------------------------------------
        // (really) empty

        let serialized = r#""#;

        serde_json::from_str::<Duration>(&serialized).unwrap_err();
    }
}
