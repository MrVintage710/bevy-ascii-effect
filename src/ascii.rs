use bevy::{
    app::Plugin,
    core_pipeline::prepass::DepthPrepass,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
    window::{PrimaryWindow, WindowRef, WindowResized},
};
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, InspectorOptions};

use crate::{
    render::AsciiRendererPlugin,
    ui::{AsciiUi, AsciiUiPlugin},
};

//=============================================================================
//             Acsii Shader Plugin
//=============================================================================

pub struct AsciiShaderPlugin;

//This plugin will add the settings required for the AsciiShader and add the post precess shader to the right spot on the render graph.
impl Plugin for AsciiShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AsciiUiPlugin)
            .register_type::<AsciiCamera>()
            .add_plugins(AsciiRendererPlugin)
            .add_systems(PreUpdate, update_target_resolution);
    }
}

//=============================================================================
//             Ascii Camera Bundle
//=============================================================================

#[derive(Bundle)]
pub struct AsciiCameraBundle {
    pub camera_bundle: Camera3dBundle,
    pub ascii_cam: AsciiCamera,
}

impl Default for AsciiCameraBundle {
    fn default() -> Self {
        AsciiCameraBundle {
            camera_bundle: Camera3dBundle::default(),
            ascii_cam: AsciiCamera::default(),
        }
    }
}

//=============================================================================
//             Shader Settings
//=============================================================================

#[derive(Component, Clone, Reflect, InspectorOptions)]
pub struct AsciiCamera {
    #[inspector(min = 1.0)]
    pub screen_colummns: f32,
    pub should_render: bool,
    #[reflect(ignore)]
    target_resolution: Vec2,
}

impl Default for AsciiCamera {
    fn default() -> Self {
        AsciiCamera {
            screen_colummns: 80.0,
            should_render: true,
            target_resolution: Vec2::ZERO,
        }
    }
}

impl AsciiCamera {
    pub fn buffer(
        &self,
        device: &RenderDevice,
        queue: &RenderQueue,
        width: u32,
    ) -> DynamicUniformBuffer<AsciiShaderSettingsBuffer> {
        let pixels_per_character = self.screen_colummns / width as f32;
        let ascii_buffer = AsciiShaderSettingsBuffer {
            pixels_per_character,
            #[cfg(feature = "webgl2")]
            _webgl2_padding: Vec3::ZERO,
        };

        let mut dyn_buffer: DynamicUniformBuffer<AsciiShaderSettingsBuffer> =
            DynamicUniformBuffer::default();
        let mut writer = dyn_buffer.get_writer(1, device, queue);
        writer.unwrap().write(&ascii_buffer);

        dyn_buffer
    }

    pub fn target_res(&self) -> &Vec2 {
        &self.target_resolution
    }
}

#[derive(ShaderType)]
pub struct AsciiShaderSettingsBuffer {
    pub pixels_per_character: f32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    pub _webgl2_padding: Vec3,
}

//=============================================================================
//             Shader Settings
//=============================================================================

fn update_target_resolution(
    mut ascii_cameras: Query<(&mut AsciiCamera, &Camera)>,
    windows: Query<&Window, Without<PrimaryWindow>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
) {
    for (mut ascii_camera, camera) in ascii_cameras.iter_mut() {
        let res: (f32, f32) = match &camera.target {
            RenderTarget::Window(window_ref) => match window_ref {
                WindowRef::Primary => {
                    let primary_window = primary_window.single();
                    (
                        primary_window.physical_width() as f32,
                        primary_window.physical_height() as f32,
                    )
                }
                WindowRef::Entity(entity) => {
                    let window = windows.get(*entity).unwrap();
                    (
                        window.physical_width() as f32,
                        window.physical_height() as f32,
                    )
                }
            },
            RenderTarget::Image(image) => {
                let image = images.get(image.id()).unwrap();
                (image.width() as f32, image.height() as f32)
            }
            RenderTarget::TextureView(_) => return,
        };

        let pixels_per_character = (res.0 / ascii_camera.screen_colummns).floor();

        let target_resolution = Vec2::new(
            (res.0 / pixels_per_character).floor(),
            (res.1 / pixels_per_character).floor(),
        );

        ascii_camera.target_resolution = target_resolution
    }
}
