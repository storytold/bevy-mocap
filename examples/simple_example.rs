#![allow(clippy::type_complexity)]

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use bevy_facial_mocap::CSVAnimationLoader;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        // We add the CSV Animation Loader here
        .init_asset_loader::<CSVAnimationLoader>()
        .add_systems(Startup, setup)
        .add_systems(Update, setup_animation)
        .run();
}

#[derive(Component)]
struct AnimationRunning(Handle<AnimationClip>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("example.gltf#Scene0"),
        ..default()
    });
}

fn setup_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut players: Query<Entity, (With<MorphWeights>, Without<AnimationRunning>)>,
) {
    for entity in &mut players {
        // And then we can load the animation just like any other animation clip
        let handle = asset_server.load("recorded_test.morph_anim.json");
        let mut player = AnimationPlayer::default();
        player.play(handle.clone()).repeat();

        commands
            .entity(entity)
            .insert((AnimationRunning(handle), player));
    }
}
