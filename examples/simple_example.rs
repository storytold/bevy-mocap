use bevy::{pbr::DirectionalLightShadowMap, prelude::*};

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .run();
}
