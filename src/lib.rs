use std::{path::Display, sync::Arc};

use bevy::utils::{default, HashMap, Instant};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MorphID {
    pub id: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RelativeFrameTime(pub f32);

impl MorphID {
    pub fn new(id: usize) -> Self {
        Self {
            id: id
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CSVFrame {
    pub timestamp: RelativeFrameTime,
    pub morphs: HashMap<MorphID, f32>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CSVAnimation {
    pub frames: Vec<CSVFrame>,
    pub morph_names: Vec<String>
}

#[derive(Debug)]
pub enum CSVParseError {
    NoData,
    InvalidCSV(csv::Error)
}

impl std::fmt::Display for CSVParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}

impl std::error::Error for CSVParseError {}

const TIMECODE_COLUMN_ID : usize = 0;
const MORPH_COLUMNS_START_ID : usize = 2;

impl CSVAnimation {
    pub fn parse(csv: &str)  -> Result<CSVAnimation, CSVParseError> {
        if csv.is_empty() {
            return Err(CSVParseError::NoData);
        }

        let mut buffer = csv.as_bytes();

        let mut csv_content = csv::ReaderBuilder::new().has_headers(false).from_reader(buffer);
        
        let mut records = csv_content.records();

        let first_row = match records.next() {
            Some(Ok(v)) => v,
            Some(Err(e)) => {
                return Err(CSVParseError::InvalidCSV(e));
            },
            None => {
                return Err(CSVParseError::NoData);
            }
        };

        let morph_names = first_row.iter().enumerate().filter_map(|(i, v)| if i >= MORPH_COLUMNS_START_ID { Some(v.trim().to_string()) } else { None }).collect();

        for result in csv_content.records() {
            let record = match result {
                Ok(value) => value,
                Err(error) => {
                    return Err(CSVParseError::InvalidCSV(error));
                }
            };
        }

        Ok(CSVAnimation {
            frames: vec![],
            morph_names,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{CSVAnimation, CSVParseError};

    #[test]
    fn given_no_data_it_emits_a_no_data_error() {
        assert!(matches!(CSVAnimation::parse("").err().unwrap(), CSVParseError::NoData))
    }

    #[test]
    fn given_an_invalid_csv_it_emits_an_invalid_csv_error() {
        assert!(matches!(CSVAnimation::parse("an invalid csv file
        seems to be here... right now?
        I wonder, will it be caught"), Err(CSVParseError::InvalidCSV(_))))
    }

    #[test]
    fn given_a_header_row_it_sets_correct_headers() {
        let result = CSVAnimation::parse("timecode, blendshapecount, shape_1, shape_2").unwrap();
        assert_eq!(result.morph_names.get(0).unwrap(), "shape_1");
        assert_eq!(result.morph_names.get(1).unwrap(), "shape_2");
    }
}
