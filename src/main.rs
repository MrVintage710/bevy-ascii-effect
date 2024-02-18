mod ascii;
pub(crate) mod render;
mod ui;

use ascii::{AsciiCameraBundle, AsciiShaderPlugin};
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    window::close_on_esc,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::{prelude::*, quick::WorldInspectorPlugin};
use ui::{buffer::{AsciiBounds, AsciiBuffer}, button::AsciiButton, command::AsciiUiCommandExtention, AsciiUi, HorizontalAlignment, VerticalAlignment};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            AsciiShaderPlugin,
            WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, init)
        .add_systems(Update, close_on_esc)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .run();
}

pub fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut ascii_ui = AsciiUi::default();

    // ascii_ui.add_node(TestNode::default());

    let camera = commands
        .spawn(AsciiCameraBundle::default())
        .insert(ascii_ui)
        .insert(PanOrbitCamera::default())
        .id();

    commands
        .ascii_ui(camera)
        .aligned(20, 15, HorizontalAlignment::Center, VerticalAlignment::Center,  AsciiButton::from_string("Test"));
    
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

    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         color: Color::BLUE,
    //         intensity: 1600.0,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     transform: Transform::from_xyz(1.0, 0.0, 1.0),
    //     ..Default::default()
    // });

    commands.spawn(SceneBundle {
        scene: asset_server.load("Skull.glb#Scene0"),
        ..default()
    }); 
}
