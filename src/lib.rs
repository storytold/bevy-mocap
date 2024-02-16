/// This library converts Unreal's LiveLink CSV format to Bevy animation clips.
/// It exports the [`CSVAnimationLoader`] struct, which is an asset loader for these animations.

/// The main module contains the asset loader itself, and a struct defining a clip info JSON file,
/// which is used to point to the CSV and contain additional information to assist with generating
/// the animation clip - such as the name of the root object in the animation clip, the FPS, and
/// the path of a mapping file
mod csv_animation;
mod csv_animation_mapping;
mod csv_frame;

use std::{path::PathBuf, string::FromUtf8Error};

use bevy::{
    animation::AnimationClip,
    asset::{AssetLoader, AsyncReadExt, ReadAssetBytesError},
    log::info,
};

use csv_animation::*;
use csv_animation_mapping::*;
use csv_frame::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClipInfo {
    pub csv: PathBuf,
    pub root_name: String,
    pub fps: Option<f32>,
    pub mapping: Option<PathBuf>,
}

#[derive(Default)]
pub struct CSVAnimationLoader;

#[non_exhaustive]
#[derive(Debug)]
pub enum CSVAnimationLoaderError {
    Io(std::io::Error),
    CSVParseError(CSVParseError),
    ClipInfoParseError(serde_json::Error),
    CSVReadBytesError(ReadAssetBytesError),
    CSVStringError(FromUtf8Error),
    MappingReadBytesError(ReadAssetBytesError),
    MappingParseError(serde_json::Error),
}

impl std::fmt::Display for CSVAnimationLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CSVAnimationLoaderError::Io(e) => {
                f.write_str(&format!("Error accessing the CSV animation files: {e}"))
            }
            CSVAnimationLoaderError::CSVParseError(e) => {
                f.write_str(&format!("Failed to parse the CSV file: {e}"))
            }
            CSVAnimationLoaderError::ClipInfoParseError(e) => {
                f.write_str(&format!("Couldn't parse Clip Info file: {e}"))
            }
            CSVAnimationLoaderError::CSVReadBytesError(e) => {
                f.write_str(&format!("Error while reading the CSV file: {e}"))
            }
            CSVAnimationLoaderError::CSVStringError(e) => f.write_str(&format!(
                "Couldn't process CSV file - is it encoded as Utf8? {e}"
            )),
            CSVAnimationLoaderError::MappingReadBytesError(e) => {
                f.write_str(&format!("Couldn't read mapping file: {e}"))
            }
            CSVAnimationLoaderError::MappingParseError(e) => {
                f.write_str(&format!("Failed to parse mapping file: {e}"))
            }
        }
    }
}

impl std::error::Error for CSVAnimationLoaderError {}

impl AssetLoader for CSVAnimationLoader {
    type Asset = AnimationClip;

    type Settings = ();

    type Error = CSVAnimationLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut clip_info = String::new();
            reader
                .read_to_string(&mut clip_info)
                .await
                .map_err(CSVAnimationLoaderError::Io)?;
            let clip_info: ClipInfo = serde_json::from_str(&clip_info)
                .map_err(CSVAnimationLoaderError::ClipInfoParseError)?;
            let csv_reader = load_context
                .read_asset_bytes(clip_info.csv)
                .await
                .map_err(CSVAnimationLoaderError::CSVReadBytesError)?;
            let csv =
                String::from_utf8(csv_reader).map_err(CSVAnimationLoaderError::CSVStringError)?;
            let mut csv_animation =
                CSVAnimation::parse(&csv).map_err(CSVAnimationLoaderError::CSVParseError)?;

            if let Some(fps) = clip_info.fps {
                csv_animation.fps = fps;
            }

            info!(
                "Parsed Animation - got {} FPS with {} total frames",
                csv_animation.fps,
                csv_animation.frames.len()
            );

            let mapping = if let Some(mapping) = clip_info.mapping {
                let mapping = load_context
                    .read_asset_bytes(mapping)
                    .await
                    .map_err(CSVAnimationLoaderError::MappingReadBytesError)?;
                let mapping: CSVAnimationMapping = serde_json::from_slice(&mapping)
                    .map_err(CSVAnimationLoaderError::MappingParseError)?;
                Some(mapping)
            } else {
                None
            };

            Ok(csv_animation.generate_clip(&clip_info.root_name, mapping.as_ref()))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["morph_anim.json"]
    }
}
