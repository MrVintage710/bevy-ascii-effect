use bevy::{
    app::Plugin,
    asset::AssetServer,
    core_pipeline::{
        core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state, CorePipelinePlugin,
    },
    ecs::world::FromWorld,
    prelude::*,
    render::{
        self,
        extract_component::{ComponentUniforms, ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner},
        render_phase::CachedRenderPipelinePhaseItem,
        render_resource::{
            encase::internal::WriteInto, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, DynamicUniformBuffer, Extent3d, FragmentState,
            ImageCopyTexture, ImageCopyTextureBase, ImageDataLayout, MultisampleState, Operations,
            Origin3d, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, Texture,
            TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
            TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::{
            BevyDefault, CompressedImageFormats, Image, ImageFormat, ImageSampler, ImageType,
        },
        view::{ExtractedWindows, PostProcessWrite, ViewTarget},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::HashMap,
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
