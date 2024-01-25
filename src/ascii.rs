use bevy::{
    app::Plugin,
    core_pipeline::prepass::DepthPrepass,
    prelude::*,
    render::{
        render_resource::{DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, InspectorOptions};

use crate::ascii_renderer::AsciiRendererPlugin;

//=============================================================================
//             Acsii Shader Plugin
//=============================================================================

pub struct AsciiShaderPlugin;

//This plugin will add the settings required for the AsciiShader and add the post precess shader to the right spot on the render graph.
impl Plugin for AsciiShaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AsciiCamera>()
            .add_plugins(AsciiRendererPlugin);
    }
}

//=============================================================================
//             Ascii Camera Bundle
//=============================================================================

#[derive(Bundle)]
pub struct AsciiCameraBundle {
    pub camera_bundle: Camera3dBundle,
    pub ascii_cam: AsciiCamera,
    pub depth_prepass: DepthPrepass,
}

impl Default for AsciiCameraBundle {
    fn default() -> Self {
        AsciiCameraBundle {
            camera_bundle: Camera3dBundle::default(),
            depth_prepass: DepthPrepass,
            ascii_cam: AsciiCamera::default(),
        }
    }
}

//=============================================================================
//             Shader Settings
//=============================================================================

#[derive(Component, Clone, Copy, Reflect, InspectorOptions)]
pub struct AsciiCamera {
    #[inspector(min = 24.0)]
    pub pixels_per_character: f32,
}

impl Default for AsciiCamera {
    fn default() -> Self {
        AsciiCamera {
            pixels_per_character: 24.0,
        }
    }
}

impl AsciiCamera {
    pub fn buffer(
        &self,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> DynamicUniformBuffer<AsciiShaderSettingsBuffer> {
        let ascii_buffer = AsciiShaderSettingsBuffer {
            pixels_per_character: self.pixels_per_character,
            #[cfg(feature = "webgl2")]
            _webgl2_padding: Vec3::ZERO,
        };

        let mut dyn_buffer: DynamicUniformBuffer<AsciiShaderSettingsBuffer> =
            DynamicUniformBuffer::default();
        let mut writer = dyn_buffer.get_writer(1, device, queue);
        writer.unwrap().write(&ascii_buffer);

        dyn_buffer
    }
}

#[derive(ShaderType)]
pub struct AsciiShaderSettingsBuffer {
    pub pixels_per_character: f32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    pub _webgl2_padding: Vec3,
}
