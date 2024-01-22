mod ascii;

use ascii::{AsciiCamera, AsciiShaderPlugin};
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    window::close_on_esc,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin, AsciiShaderPlugin))
        .add_systems(Startup, init)
        .add_systems(Update, close_on_esc)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .run();
}

pub fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        AsciiCamera {
            pixels_per_character: 24.0,
        },
        PanOrbitCamera::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("Skull.glb#Scene0"),
        ..default()
    });
}
