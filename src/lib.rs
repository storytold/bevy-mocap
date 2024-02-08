use std::{path::Display, sync::Arc};

use bevy::utils::{default, HashMap, Instant};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MorphID {
    id: usize,

}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RelativeFrameTime(f32);

impl MorphID {
    pub fn new(id: usize) -> Self {
        Self {
            id: id
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CSVFrame {
    timestamp: RelativeFrameTime,
    morphs: HashMap<MorphID, f32>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CSVAnimation {
    frames: Vec<CSVFrame>,
    morph_names: Vec<String>
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CSVParseError {
    NoData
}

impl std::fmt::Display for CSVParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}

impl std::error::Error for CSVParseError {}

impl CSVAnimation {
    pub fn parse(csv: &str) -> Result<CSVAnimation, CSVParseError> {
        Err(CSVParseError::NoData)
    }
}

#[cfg(test)]
mod tests {
    use crate::{CSVAnimation, CSVParseError};

    #[test]
    fn given_no_data_the_csv_doesnt_parse() {
        assert_eq!(CSVAnimation::parse("").err().unwrap(), CSVParseError::NoData)
    }
}
