// This module defines the types needed to represent a [`CSVFrame`] - including the [`Timestamp`]
use std::{num::ParseFloatError, str::FromStr};

use bevy::reflect::Reflect;

use serde::{Deserialize, Serialize};

// Neither the chrono crate nor the standard library has a good way of parsing SMPT timestamps.
// As a result, we set one up here - and we're sticking to u8s since recording data at above 255 FPS
// is unlikely in this context, and none of the other values should go beyond 60 (or 24 for hours)
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Reflect, Default,
)]
pub struct Timestamp {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub frame: u8,
}

#[derive(Debug)]
pub enum TimestampParseError {
    EmptyTimestamp,
    TooManySegments,
    NumberParseError(ParseFloatError, String),
}

impl std::fmt::Display for TimestampParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimestampParseError::EmptyTimestamp => f.write_str("The timestamp is an empty string"),
            TimestampParseError::TooManySegments => {
                f.write_str("There are too many segments in the timestamp")
            }
            TimestampParseError::NumberParseError(e, s) => f.write_str(&format!(
                "Couldn't parse one of the segments - is {s} a number?: {e}"
            )),
        }
    }
}

impl std::error::Error for TimestampParseError {}

impl FromStr for Timestamp {
    type Err = TimestampParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(TimestampParseError::EmptyTimestamp);
        }

        let mut segments = s.split(':').collect::<Vec<_>>();
        if segments.len() > 4 {
            return Err(TimestampParseError::TooManySegments);
        }

        // The segments start with hours, but we want to start with frames and be robust to missing segments
        segments.reverse();

        let Some(frame) = segments.first() else {
            return Err(TimestampParseError::EmptyTimestamp);
        };

        let frame = frame.parse::<f32>().map(|v| v.floor() as u8).map_err(|e| {
            TimestampParseError::NumberParseError(e, format!("frame failed for {s}"))
        })?;
        let second = segments
            .get(1)
            .unwrap_or(&"0")
            .parse::<f32>()
            .map(|v| v.floor() as u8)
            .map_err(|e| {
                TimestampParseError::NumberParseError(e, format!("second failed for {s}"))
            })?;
        let minute = segments
            .get(2)
            .unwrap_or(&"0")
            .parse::<f32>()
            .map(|v| v.floor() as u8)
            .map_err(|e| {
                TimestampParseError::NumberParseError(e, format!("minute failed for {s}"))
            })?;
        let hour = segments
            .get(3)
            .unwrap_or(&"0")
            .parse::<f32>()
            .map(|v| v.floor() as u8)
            .map_err(|e| {
                TimestampParseError::NumberParseError(e, format!("hour failed for {s}"))
            })?;

        Ok(Self {
            hour,
            minute,
            second,
            frame,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Reflect)]
pub struct CSVFrame {
    pub timestamp: Timestamp,
    pub morphs: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn given_an_empty_timestamp_it_returns_an_empty_timestamp_error() {
        let timestamp = ""
            .parse::<Timestamp>()
            .expect_err("succeeded against expectations");
        assert!(matches!(timestamp, TimestampParseError::EmptyTimestamp))
    }

    #[test]
    fn given_a_timestamp_with_more_than_4_segments_returns_a_too_many_segments_error() {
        let timestamp = "00:00:00:00:00.1234"
            .parse::<Timestamp>()
            .expect_err("succeeded against expectations");
        assert!(matches!(timestamp, TimestampParseError::TooManySegments))
    }

    #[test]
    fn given_a_zero_timestamp_it_is_parsed_correctly() {
        let timestamp = "00:00:00:00"
            .parse::<Timestamp>()
            .expect("Timestamp failed to parse");
        assert_eq!(
            timestamp,
            Timestamp {
                hour: 0,
                minute: 0,
                second: 0,
                frame: 0
            }
        )
    }

    #[test]
    fn given_a_valid_timestamp_it_is_parsed_correctly() {
        let timestamp = "01:02:03:04.123"
            .parse::<Timestamp>()
            .expect("Timestamp failed to parse");
        assert_eq!(
            timestamp,
            Timestamp {
                hour: 1,
                minute: 2,
                second: 3,
                frame: 4
            }
        )
    }
}
