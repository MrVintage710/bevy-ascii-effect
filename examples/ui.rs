use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, render::mesh::shape::Cube, window::close_on_esc};
use bevy_ascii::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub fn main() {
    let mut app = App::new();
    
    app
        .add_plugins(DefaultPlugins)
        .add_plugins(AsciiShaderPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        
        .add_systems(Startup, init)
        .add_systems(Update, close_on_esc)
        
        .run()
    ;
    
    app.run();
}

fn init(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>
) {
    let mesh = meshes.add(Cube::new(1.0));
    let red_material = materials.add(Color::RED);
    
    commands.spawn(PbrBundle {
        mesh,
        material: red_material,
        ..Default::default()
    });
    
    // light
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
    
    let camera = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        AsciiCamera::default(),
        AsciiUi::default(),
        PanOrbitCamera::default(),
        VisibilityBundle::default(),
    )).id();
    
    commands.ascii_ui_with_parent(camera)
        .aligned(20, 20, HorizontalAlignment::Center, VerticalAlignment::Center, AsciiButton::from_string("Test Button"))
        .relative(-3, -3, 12, 8, AsciiButton::from_string("Inner"))
    ;
    
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 8000.0,
    });
}