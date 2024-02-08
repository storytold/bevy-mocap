use std::{num::ParseFloatError, path::Display, str::FromStr, sync::Arc};

use bevy::{time::{self, Time}, utils::{default, HashMap}};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    hour: u8,
    minute: u8,
    second: u8,
    frame: u8,
}

#[derive(Debug)]
pub enum TimestampParseError {
    EmptyTimestamp,
    TooManySegments,
    NumberParseError(ParseFloatError, String),
}

impl std::fmt::Display for TimestampParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
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

        segments.reverse();

        let Some(frame) = segments.first() else {
            return Err(TimestampParseError::EmptyTimestamp);
        };

        let frame = frame.parse::<f32>().map(|v| v.floor() as u8).map_err(|e| TimestampParseError::NumberParseError(e, format!("frame failed for {s}")))?;
        let second = segments.get(1).unwrap_or(&"0").parse::<f32>().map(|v| v.floor() as u8).map_err(|e| TimestampParseError::NumberParseError(e, format!("second failed for {s}")))?;
        let minute = segments.get(2).unwrap_or(&"0").parse::<f32>().map(|v| v.floor() as u8).map_err(|e| TimestampParseError::NumberParseError(e, format!("minute failed for {s}")))?;
        let hour = segments.get(3).unwrap_or(&"0").parse::<f32>().map(|v| v.floor() as u8).map_err(|e| TimestampParseError::NumberParseError(e, format!("hour failed for {s}")))?;

        Ok(Self { hour, minute, second, frame })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CSVFrame {
    pub timestamp: Timestamp,
    pub morphs: Vec<f32>
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CSVAnimation {
    pub frames: Vec<CSVFrame>,
    pub morph_names: Vec<String>,
    pub fps: u8
}

#[derive(Debug)]
pub enum CSVParseError {
    NoData,
    InvalidCSV(csv::Error),
    NoMorphs,
    MisssingKnownHeaders,
    NoFrames,
    TimecodeParseError(TimestampParseError),
    MorphParseError(ParseFloatError),
}

impl std::fmt::Display for CSVParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}

impl std::error::Error for CSVParseError {}

const SKIP_COLUMNS_BEFORE_MORPH : usize = 1;

impl CSVAnimation {
    pub fn parse(csv: &str)  -> Result<CSVAnimation, CSVParseError> {
        if csv.is_empty() {
            return Err(CSVParseError::NoData);
        }

        let buffer = csv.as_bytes();

        let mut csv_content = csv::ReaderBuilder::new().has_headers(false).from_reader(buffer);
        
        let mut records = csv_content.records();

        let header_row = records.next().ok_or(CSVParseError::NoData)?.map_err(|e| CSVParseError::InvalidCSV(e))?;

        let morph_names : Vec<String> = header_row.iter().enumerate().filter_map(|(i, v)| if i > SKIP_COLUMNS_BEFORE_MORPH { Some(v.trim().to_string()) } else { None }).collect();

        if morph_names.is_empty() {
            return Err(CSVParseError::NoMorphs);
        }

        let mut frames = Vec::new();
        let mut fps : u8 = 0;


        for result in records {
            let record = result.map_err(CSVParseError::InvalidCSV)?;

            let mut iter = record.iter();

            let timestamp = iter.next().ok_or(CSVParseError::MisssingKnownHeaders)?; // This error should be impossible since the CSV parser validates that all rows have the same length, and we already confirmed there is more than 0 characters in the first row.
            let timestamp : Timestamp = timestamp.parse().map_err(CSVParseError::TimecodeParseError)?;
            for _ in 0..SKIP_COLUMNS_BEFORE_MORPH {
                let _ = iter.next().ok_or(CSVParseError::MisssingKnownHeaders)?; // similarly, we already validate that there are a single morph in the CSV, so the skipped columns shouldn't be valid CSVs.
            }

            let morphs = iter.map(|v| v.trim().parse::<f32>().map_err(CSVParseError::MorphParseError)).collect::<Result<Vec<_>,_>>()?;

            if fps < timestamp.frame {
                fps = timestamp.frame;
            }

            frames.push(CSVFrame { timestamp, morphs });
        }

        if frames.is_empty() {
            return Err(CSVParseError::NoFrames);
        }

        Ok(CSVAnimation {
            frames,
            morph_names,
            fps: fps + 1
        })
    }
}

#[cfg(test)]
mod tests {
    use bevy::time::{self, Time};

    use crate::{CSVAnimation, CSVParseError, Timestamp, TimestampParseError};
    #[test]
    fn given_an_empty_timestamp_it_returns_an_empty_timestamp_error() {
        let timestamp = "".parse::<Timestamp>().expect_err("succeeded against expectations");
        assert!(matches!(timestamp, TimestampParseError::EmptyTimestamp))
    }

    #[test]
    fn given_a_timestamp_with_more_than_4_segments_returns_a_too_many_segments_error() {
        let timestamp = "00:00:00:00:00.1234".parse::<Timestamp>().expect_err("succeeded against expectations");
        assert!(matches!(timestamp, TimestampParseError::TooManySegments))
    }

    #[test]
    fn given_a_zero_timestamp_it_is_parsed_correctly() {
        let timestamp = "00:00:00:00".parse::<Timestamp>().expect("Timestamp failed to parse");
        assert_eq!(timestamp, Timestamp { hour: 0, minute: 0, second: 0, frame: 0 })
    }

    #[test]
    fn given_a_valid_timestamp_it_is_parsed_correctly() {
        let timestamp = "01:02:03:04.123".parse::<Timestamp>().expect("Timestamp failed to parse");
        assert_eq!(timestamp, Timestamp { hour: 1, minute: 2, second: 3, frame: 4 })
    }

    #[test]
    fn given_no_data_it_emits_a_no_data_error() {
        assert!(matches!(CSVAnimation::parse("").err().unwrap(), CSVParseError::NoData))
    }

    #[test]
    fn given_an_invalid_csv_it_emits_an_invalid_csv_error() {
        let result = CSVAnimation::parse("an, invalid, csv, file
        seems to be here... right now?
        I wonder, will it be caught");
        if !matches!(result, Err(CSVParseError::InvalidCSV(_))) {
            panic!( "got {result:?} instead of a CSVParseError");
        }
    }

    #[test]
    fn given_a_header_row_it_sets_correct_headers() {
        let result = CSVAnimation::parse("timecode, blendshapecount, shape_1, shape_2
        00:00:00:00, 0, 0, 0").unwrap();
        assert_eq!(result.morph_names.first().unwrap(), "shape_1");
        assert_eq!(result.morph_names.get(1).unwrap(), "shape_2");
    }

    #[test]
    fn given_no_morph_headers_it_emits_a_no_morph_error() {
        let result = CSVAnimation::parse("timecode, blendshapecount").unwrap_err();
        assert!(matches!(result, CSVParseError::NoMorphs));
    }

    #[test]
    fn given_no_frames_it_emits_a_no_frame_error() {
        let result = CSVAnimation::parse("timecode, blendshapecount, shape_1, shape_2").unwrap_err();
        assert!(matches!(result, CSVParseError::NoFrames));
    }
    
    
    #[test]
    fn given_a_single_frame_it_has_a_time_of_0() {
        let result = CSVAnimation::parse("timecode, blendshapecount, shape_1, shape_2
00:00:00:00.00, 0, 0, 0").unwrap();
        assert_eq!(result.frames.first().unwrap().timestamp.frame, 0);
    }

    #[test]
    fn given_multiple_frames_can_deduce_fps() {
        let result = CSVAnimation::parse("timecode, blendshapecount, shape_1, shape_2
00:00:00:00.00, 0, 0, 0
00:00:00:01.00, 0, 0, 0
00:00:00:02.00, 0, 0, 0
00:00:01:00.00, 0, 0, 0
00:00:01:01.00, 0, 0, 0").unwrap();
        assert_eq!(result.fps, 3);
    }

    #[test]
    fn given_a_single_frame_it_parses_morphs_correctly() {
        let result = CSVAnimation::parse("timecode, blendshapecount, shape_1, shape_2
00:00:00:00.00, 0, 0.1, 0.5").unwrap();
        assert!(&result.frames.first().unwrap().morphs[0] - 0.1 < 0.0001);
        assert!(&result.frames.first().unwrap().morphs[1] - 0.5 < 0.0001);
    }
}
