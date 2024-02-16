/// This module handles the mapping between a [`CSVAnimation`] and an animation clipy
use crate::csv_animation::*;
use bevy::{
    animation::{AnimationClip, EntityPath, Keyframes, VariableCurve},
    core::Name,
    math::{Quat, Vec3},
    transform::components::Transform,
    utils::hashbrown::HashMap,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CSVAnimationMapping(pub Vec<(MappingType, String)>);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MappingType {
    Morph(String),
    Bone(BoneMapping),
}

/// A Bone Mapping maps the value from the CSV to a rotation in a specific axis on a bone
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoneMapping {
    /// the hierarchy path to the bone
    path: Vec<String>,
    /// the axis of rotation
    axis: Axis,
    /// an offset - in case "0" in the CSV is different from "0" in the bone hierarchy
    offset: Option<f32>,
    /// scale - to allow you to tone the animation up or down
    scale: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
    #[serde(untagged)]
    Vec(Vec3),
}

impl CSVAnimation {
    pub fn generate_clip(self, name: &str, mapping: Option<&CSVAnimationMapping>) -> AnimationClip {
        // we find the path for the root entity and create a clip
        let path = EntityPath {
            parts: vec![Name::new(name.to_owned())],
        };
        let mut clip = AnimationClip::default();

        // we note the order and name of every morph in the CSV
        let csv_morph_order: HashMap<&str, usize> = self
            .morph_names
            .iter()
            .enumerate()
            .map(|(i, val)| (val.as_str(), i))
            .collect();

        // we generate the animation curve for the morphs themselves
        clip.add_curve_to_path(path, self.generate_morph_curve(mapping, &csv_morph_order));

        // if we have a mapping file, we also generate curves for any bones we're mapped to
        if let Some(mapping) = mapping {
            self.generate_bone_curves(&mut clip, mapping, &csv_morph_order);
        }

        clip
    }

    fn generate_bone_curves(
        &self,
        clip: &mut AnimationClip,
        mapping: &CSVAnimationMapping,
        csv_morph_order: &HashMap<&str, usize>,
    ) {
        // we group all our mappings by the name of the bone, and throw out the non-bone mappings
        let bones = mapping
            .0
            .iter()
            .filter_map(|(k, v)| match k {
                MappingType::Morph(_) => None,
                MappingType::Bone(mapping) => {
                    Some((mapping.path.clone(), (mapping.clone(), v.clone())))
                }
            })
            .group_by(|(key, _)| key.clone());

        for (bone, mappings) in bones.into_iter() {
            // for any bone, we generate the relevant curve and add it to the clip
            let curve = self.generate_bone_curve(mappings.map(|(_, v)| v), csv_morph_order);
            let path = EntityPath {
                parts: bone.into_iter().map(Name::new).collect(),
            };
            clip.add_curve_to_path(path, curve);
        }
    }

    fn generate_bone_curve(
        &self,
        mappings: impl Iterator<Item = (BoneMapping, String)>,
        csv_morph_order: &HashMap<&str, usize>,
    ) -> VariableCurve {
        let mut keyframe_timestamps = Vec::with_capacity(self.frames.len());
        let mut keyframes = Vec::with_capacity(self.frames.len());
        let mappings = mappings.collect::<Vec<_>>();

        let seconds_per_frame = 1.0 / self.fps;
        for (frame, data) in self.frames.iter().enumerate() {
            let time = (frame as f32) * seconds_per_frame;
            keyframe_timestamps.push(time);

            // we set up a transform at rest
            let mut transform =
                Transform::from_rotation(Quat::from_euler(bevy::math::EulerRot::XYZ, 0., 0., 0.));

            // for every mapping - we grab that data from the CSV and
            // rotate the transform accordingly.
            for (mapping, csv_key) in mappings.iter() {
                let result = csv_morph_order
                    .get(csv_key.as_str())
                    .copied()
                    .and_then(|i| data.morphs.get(i))
                    .copied()
                    .unwrap_or(0f32);

                let axis = match mapping.axis {
                    Axis::X => Vec3::X,
                    Axis::Y => Vec3::Y,
                    Axis::Z => Vec3::Z,
                    Axis::Vec(v) => v,
                };
                let offset = mapping.offset.unwrap_or(0f32).to_radians();

                let scale = mapping.scale.unwrap_or(1f32);

                let result = result * 180f32.to_radians() * scale + offset;

                transform.rotate_axis(axis, result);
            }
            keyframes.push(transform.rotation)
        }

        VariableCurve {
            keyframe_timestamps,
            keyframes: Keyframes::Rotation(keyframes),
        }
    }

    fn generate_morph_curve(
        &self,
        mapping: Option<&CSVAnimationMapping>,
        csv_morph_order: &HashMap<&str, usize>,
    ) -> VariableCurve {
        // we figure out how we need to re-order morphs, going from the mesh order to the csv order
        let (num_mesh_morphs, mesh_to_csv_morph_order) = match mapping {
            Some(mapping) => {
                let mesh_to_csv_morph_order: Vec<Option<usize>> = mapping
                    .0
                    .iter()
                    .filter_map(|(mapping, v)| match mapping {
                        MappingType::Morph(_) => {
                            Some(csv_morph_order.get(v.to_string().as_str()).copied())
                        }
                        _ => None,
                    })
                    .collect();
                let num_mesh_morphs: usize = mesh_to_csv_morph_order.len();
                (num_mesh_morphs, mesh_to_csv_morph_order)
            }
            None => (
                self.morph_names.len(),
                self.morph_names
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Some(i))
                    .collect(),
            ),
        };

        let mut keyframe_timestamps = Vec::with_capacity(self.frames.len());
        let mut keyframes = Vec::with_capacity(self.frames.len() * num_mesh_morphs);

        let seconds_per_frame = 1.0 / self.fps;

        for (frame, data) in self.frames.iter().enumerate() {
            let time = (frame as f32) * seconds_per_frame;
            keyframe_timestamps.push(time);
            // we iterate over all the morphs in the mesh, and if they have a matching morph in the CSV - we grab the data for that morph
            for i in 0..num_mesh_morphs {
                let result = mesh_to_csv_morph_order
                    .get(i)
                    .and_then(|i| i.map(|i| data.morphs.get(i).copied().unwrap_or(0.0)))
                    .unwrap_or(0.0);
                keyframes.push(result);
            }
        }

        VariableCurve {
            keyframe_timestamps,
            keyframes: Keyframes::Weights(keyframes),
        }
    }
}

#[cfg(test)]
mod tests {

    use bevy::{
        animation::{AnimationClip, EntityPath, Keyframes},
        core::Name,
        math::Vec3,
    };

    use crate::{CSVAnimation, CSVAnimationMapping};

    #[test]
    fn can_convert_an_animation_into_an_animation_clip() {
        let animation = CSVAnimation::parse(
            "timecode, blendshapecount, shape_1, shape_2, shape_missing
00:00:00:00.00, 0, 1.0, 0, 0.5
00:00:00:01.00, 0, 0, -0.4, 0.2
00:00:00:02.00, 0, 0, 0, 0.4
00:00:01:00.00, 0, 0, 0, 1
00:00:01:01.00, 0, 0, 0, -2",
        )
        .unwrap();

        let clip: AnimationClip = animation.generate_clip("my_node", None);

        assert!(clip.duration() - 1.33333 < 0.1);

        let curves = clip
            .get_curves_by_path(&EntityPath {
                parts: vec![Name::new("my_node")],
            })
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(curves.keyframe_timestamps.len(), 5);

        assert!((curves.keyframe_timestamps[0] - 0.0).abs() < 0.001);
        assert!((curves.keyframe_timestamps[1] - 0.33333).abs() < 0.001);
        assert!((curves.keyframe_timestamps[2] - 0.66666).abs() < 0.01);
        assert!((curves.keyframe_timestamps[3] - 1.0).abs() < 0.01);
        assert!((curves.keyframe_timestamps[4] - 1.33333).abs() < 0.01);

        let Keyframes::Weights(keyframes) = &curves.keyframes else {
            panic!("Keyframes aren't weights")
        };

        assert_eq!(keyframes.len(), 15);

        assert!((keyframes[0] - 1.0).abs() < 0.0001);
        assert!((keyframes[1] - 0.0).abs() < 0.0001);
        assert!((keyframes[2] - 0.5).abs() < 0.0001);
        assert!((keyframes[3] - 0.0).abs() < 0.0001);
        assert!((keyframes[4] + 0.4).abs() < 0.0001);
        assert!((keyframes[5] - 0.2).abs() < 0.0001);
    }

    #[test]
    fn can_convert_an_animation_into_an_animation_clip_with_mapping() {
        let animation = CSVAnimation::parse(
            "timecode, blendshapecount, shape_1, shape_2, shape_missing
00:00:00:00.00, 0, 1.0, 0, 0.5
00:00:00:01.00, 0, 0, -0.4, 0.2
00:00:00:02.00, 0, 0, 0, 0.4
00:00:01:00.00, 0, 0, 0, 1
00:00:01:01.00, 0, 0, 0, -2",
        )
        .unwrap();

        let mapping = CSVAnimationMapping(
            [
                ("\"shape_2\"", "shape_2"),
                ("\"shape_1\"", "shape_1"),
                ("\"shape_extra\"", "shape_extra"),
            ]
            .iter()
            .map(|(key, value)| (serde_json::from_str(key).unwrap(), value.to_string()))
            .collect(),
        );

        let clip: AnimationClip = animation.generate_clip("my_node", Some(&mapping));

        assert!(clip.duration() - 1.33333 < 0.1);

        let curves = clip
            .get_curves_by_path(&EntityPath {
                parts: vec![Name::new("my_node")],
            })
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(curves.keyframe_timestamps.len(), 5);

        assert!((curves.keyframe_timestamps[0] - 0.0).abs() < 0.001);
        assert!((curves.keyframe_timestamps[1] - 0.33333).abs() < 0.001);
        assert!((curves.keyframe_timestamps[2] - 0.66666).abs() < 0.01);
        assert!((curves.keyframe_timestamps[3] - 1.0).abs() < 0.01);
        assert!((curves.keyframe_timestamps[4] - 1.33333).abs() < 0.01);

        let Keyframes::Weights(keyframes) = &curves.keyframes else {
            panic!("Keyframes aren't weights")
        };

        assert_eq!(keyframes.len(), 15);

        assert!((keyframes[1] - 1.0).abs() < 0.0001); // "shape_1" is the second morph in the mesh
        assert!((keyframes[0] - 0.0).abs() < 0.0001); // "shape_2" is the first morph in the mesh
        assert!((keyframes[2] - 0.0).abs() < 0.0001); // the "shape_extra" morph is third, and is always 0.0
        assert!((keyframes[4] - 0.0).abs() < 0.0001);
        assert!((keyframes[3] + 0.4).abs() < 0.0001);
        assert!((keyframes[5] - 0.0).abs() < 0.0001);
    }

    #[test]
    fn can_convert_an_animation_into_an_animation_clip_with_bone_mapping() {
        let animation = CSVAnimation::parse(
            "timecode, blendshapecount, shape_1, shape_2, bone_x
00:00:00:00.00, 0, 0, 1.0, 0
00:00:00:01.00, 0, -0.5, -0.4, 0.5
00:00:00:02.00, 0, 0, 0, 0.4
00:00:01:00.00, 0, 0, 0, 1
00:00:01:01.00, 0, 0, 0, -2",
        )
        .unwrap();

        let mapping: CSVAnimationMapping = CSVAnimationMapping(
            [
                ("\"shape_2\"", "shape_2"),
                ("{ \"path\": [\"bone\"], \"axis\": [0, 0, 1] }", "shape_1"),
                ("{ \"path\": [\"bone\"], \"axis\": \"X\" }", "bone_x"),
            ]
            .iter()
            .map(|(key, value)| (serde_json::from_str(key).unwrap(), value.to_string()))
            .collect(),
        );

        let clip: AnimationClip = animation.generate_clip("my_node", Some(&mapping));

        assert!(clip.duration() - 1.33333 < 0.1);

        let curves = clip
            .get_curves_by_path(&EntityPath {
                parts: vec![Name::new("my_node")],
            })
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(curves.keyframe_timestamps.len(), 5);

        assert!((curves.keyframe_timestamps[0] - 0.0).abs() < 0.001);
        assert!((curves.keyframe_timestamps[1] - 0.33333).abs() < 0.001);
        assert!((curves.keyframe_timestamps[2] - 0.66666).abs() < 0.01);
        assert!((curves.keyframe_timestamps[3] - 1.0).abs() < 0.01);
        assert!((curves.keyframe_timestamps[4] - 1.33333).abs() < 0.01);

        let Keyframes::Weights(keyframes) = &curves.keyframes else {
            panic!("Keyframes aren't weights")
        };

        assert_eq!(keyframes.len(), 5);

        assert!((keyframes[0] - 1.0).abs() < 0.0001);
        assert!((keyframes[1] + 0.4).abs() < 0.0001);

        let curves = clip
            .get_curves_by_path(&EntityPath {
                parts: vec![Name::new("bone")],
            })
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(curves.keyframe_timestamps.len(), 5);

        assert!((curves.keyframe_timestamps[0] - 0.0).abs() < 0.001);
        assert!((curves.keyframe_timestamps[1] - 0.33333).abs() < 0.001);
        assert!((curves.keyframe_timestamps[2] - 0.66666).abs() < 0.01);
        assert!((curves.keyframe_timestamps[3] - 1.0).abs() < 0.01);
        assert!((curves.keyframe_timestamps[4] - 1.33333).abs() < 0.01);

        let Keyframes::Rotation(keyframes) = &curves.keyframes else {
            panic!("Keyframes aren't rotations")
        };

        assert_eq!(keyframes.len(), 5);

        assert!((keyframes[0].xyz() - Vec3::new(0., 0., 0.)).length() < 0.0001);
        assert!((keyframes[1].xyz() - Vec3::new(0.5, 0.5, -0.5)).length() < 0.01);
    }
}
