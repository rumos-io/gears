// The Duration struct is defined in gogoproto v1.3.1 at https://github.com/gogo/protobuf/blob/v1.3.1/protobuf/google/protobuf/duration.proto
// and https://github.com/protocolbuffers/protobuf-go/blob/v1.34.2/types/known/durationpb/duration.pb.go

/// A Duration represents a signed, fixed-length span of time represented
/// as a count of seconds and fractions of seconds at nanosecond
/// resolution. It is independent of any calendar and concepts like "day"
/// or "month". It is related to Timestamp in that the difference between
/// two Timestamp values is a Duration and it can be added or subtracted
/// from a Timestamp. Range is approximately +-10,000 years.
///
/// TODO: make the statement below true!
///
/// In JSON format, the Duration type is encoded as a string rather than an
/// object, where the string ends in the suffix "s" (indicating seconds) and
/// is preceded by the number of seconds, with nanoseconds expressed as
/// fractional seconds. For example, 3 seconds with 0 nanoseconds should be
/// encoded in JSON format as "3s", while 3 seconds and 1 nanosecond should
/// be expressed in JSON format as "3.000000001s", and 3 seconds and 1
/// microsecond should be expressed in JSON format as "3.000001s".
#[derive(Clone, Copy, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, thiserror::Error)]
pub enum DurationError {
    #[error("duration exceeds -10000 years")]
    Underflow,
    #[error("duration exceeds 10000 years")]
    Overflow,
    #[error("nanoseconds must be in the range -999_999_999_999..1_000_000_000")]
    Nanoseconds,
    #[error("for durations of one second or more, a non-zero value for the nanoseconds field must be of the same sign as the seconds field")]
    Sign,
}

impl Duration {
    pub fn try_new(seconds: i64, nanos: i32) -> Result<Duration, DurationError> {
        let duration = Duration { seconds, nanos };
        duration.check()?;
        Ok(duration)
    }

    pub const fn new_from_secs(secs: i32) -> Duration {
        // i32::MAX < ABS_DURATION
        // i32::MIN > -ABS_DURATION
        // so this is safe
        Duration {
            seconds: secs as i64,
            nanos: 0,
        }
    }

    pub const fn new_from_nanos(nanos: i64) -> Duration {
        let seconds = nanos / 1_000_000_000;
        let nanos = (nanos % 1_000_000_000) as i32; // cast is safe because of modulo operation

        // i64::MAX / 1_000_000_000 is < ABS_DURATION
        // i64::MIN / 1_000_000_000 is > -ABS_DURATION
        // so seconds are always valid
        Duration { seconds, nanos }
    }

    pub fn try_new_from_nanos(nanos: i128) -> Result<Duration, DurationError> {
        let seconds = i64::try_from(nanos / 1_000_000_000).map_err(|_| {
            if nanos > 0 {
                DurationError::Overflow
            } else {
                DurationError::Underflow
            }
        })?;

        let nanos = (nanos % 1_000_000_000) as i32; //cast is safe because of modulo operation

        Ok(Duration { seconds, nanos })
    }

    /// Returns the total number of whole hours in the `Duration`.
    pub fn num_hours(&self) -> i64 {
        // NOTE: we can ignore the nanoseconds here. Since the nanoseconds are always less
        // than a second, they will never be enough to push the total seconds over an hour boundary.
        self.seconds / 3600
    }

    pub fn duration_seconds(&self) -> i64 {
        self.seconds
    }

    /// Returns the number of nanoseconds since the last second boundary //TODO: better description
    pub fn nanoseconds(&self) -> i32 {
        self.nanos
    }

    // based on https://github.com/protocolbuffers/protobuf-go/blob/c33baa8f3a0d35fd5a39e43c22a50a050f707d34/types/known/durationpb/duration.pb.go#L225C1-L244C2
    fn check(&self) -> Result<(), DurationError> {
        const ABS_DURATION: i64 = 315576000000; // 10000yr * 365.25day/yr * 24hr/day * 60min/hr * 60sec/min

        if self.seconds < -ABS_DURATION {
            Err(DurationError::Underflow)
        } else if self.seconds > ABS_DURATION {
            Err(DurationError::Overflow)
        } else if self.nanos <= -1_000_000_000 || self.nanos >= 1_000_000_000 {
            Err(DurationError::Nanoseconds)
        } else if (self.seconds > 0 && self.nanos < 0) || (self.seconds < 0 && self.nanos > 0) {
            Err(DurationError::Sign)
        } else {
            Ok(())
        }
    }

    pub fn to_nanoseconds(&self) -> i128 {
        i128::from(self.seconds) * 1_000_000_000 + i128::from(self.nanos)
    }
}

impl From<tendermint_proto::google::protobuf::Duration> for Duration {
    fn from(inner::Duration { seconds, nanos }: inner::Duration) -> Self {
        Self { seconds, nanos }
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
