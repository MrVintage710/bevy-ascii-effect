//=============================================================================
//             Ascii Shader Node
//=============================================================================

#[derive(Default)]
pub struct AsciiShaderNode;
impl AsciiShaderNode {
    const NAME: &'static str = "AsciiShaderNode";
}

impl ViewNode for AsciiShaderNode {
    type ViewQuery = (Entity, &'static ViewTarget, &'static AsciiCamera);

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        view_query: bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let (entity, view_target, ascii_camera) = view_query;

        // Get the pipeline resource that contains the global data we need
        // to create the render pipeline
        let ascii_pipeline_resource = world.resource::<AsciiShaderPipeline>();
        let pixel_pipeline_resource = world.resource::<PixelShaderPipeline>();

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the pipeline from the cache
        let Some(ascii_pipeline) =
            pipeline_cache.get_render_pipeline(ascii_pipeline_resource.pipeline_id)
        else {
            return Ok(());
        };

        let Some(pixel_pipeline) =
            pipeline_cache.get_render_pipeline(pixel_pipeline_resource.pipeline_id)
        else {
            return Ok(());
        };

        // let acsii_camera = world.resource::<AsciiCamera>();
        // Get the settings uniform binding
        let settings_uniforms = ascii_camera.buffer(
            render_context.render_device(),
            world.get_resource::<RenderQueue>().unwrap(),
        );

        let Some(low_res_texture) = pixel_pipeline_resource.low_res_textures.get(&entity) else {
            return Ok(());
        };

        let Some(settings_binding) = settings_uniforms.binding() else {
            return Ok(());
        };

        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let post_process = view_target.post_process_write();

        pixel_pass(
            low_res_texture,
            render_context,
            pixel_pipeline,
            pixel_pipeline_resource,
            &post_process,
        );

        // The bind_group gets created each frame.
        //
        // Normally, you would create a bind_group in the Queue set,
        // but this doesn't work with the post_process_write().
        // The reason it doesn't work is because each post_process_write will alternate the source/destination.
        // The only way to have the correct source/destination for the bind_group
        // is to make sure you get it during the node execution.
        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &ascii_pipeline_resource.layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                low_res_texture,
                // use the font texture
                &ascii_pipeline_resource.font_texture,
                // Use the sampler created for the pipeline
                &ascii_pipeline_resource.sampler,
                // Set the settings binding
                settings_binding.clone(),
            )),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ascii_post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // We need to specify the post process destination view here
                // to make sure we write to the appropriate texture.
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
        // using the pipeline/bind_group created above
        render_pass.set_render_pipeline(ascii_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

fn pixel_pass(
    low_res_texture: &TextureView,
    render_context: &mut RenderContext,
    pixel_pipeline: &RenderPipeline,
    pixel_pipeline_resource: &PixelShaderPipeline,
    post_process: &PostProcessWrite,
) {
    let pixel_bind_group = render_context.render_device().create_bind_group(
        "pixel_shader_bind_group",
        &pixel_pipeline_resource.layout,
        &BindGroupEntries::sequential((post_process.source, &pixel_pipeline_resource.sampler)),
    );

    let mut pixel_render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("pixel_shader_render_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            // We need to specify the post process destination view here
            // to make sure we write to the appropriate texture.
            view: low_res_texture,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
    });

    pixel_render_pass.set_render_pipeline(pixel_pipeline);
    pixel_render_pass.set_bind_group(0, &pixel_bind_group, &[]);
    pixel_render_pass.draw(0..3, 0..1);
}

//=============================================================================
//             Ascii Shader Pipeline
//=============================================================================

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct AsciiShaderPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    font_texture: TextureView,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for AsciiShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let queue = world.get_resource::<RenderQueue>().unwrap();
        let render_device = world.resource::<RenderDevice>();

        //We need to create the bind group
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                //This is the screen texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // This is the texture for the ascii font
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler that will be used to sample the screen texture
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // The settings uniform that will control the effect
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: bevy::render::render_resource::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(AsciiShaderSettingsBuffer::min_size()),
                    },
                    count: None,
                },
            ],
            label: Some("AsciiShaderPipeline::bind_group_layout"),
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.resource::<AssetServer>().load("ascii.wgsl");

        let font_texture = Image::from_buffer(
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", "font.png")),
            ImageType::Format(ImageFormat::Png),
            CompressedImageFormats::default(),
            true,
            ImageSampler::nearest(),
        )
        .expect("There was an error reading an internal texture.");

        let texture = render_device.create_texture(&font_texture.texture_descriptor);
        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &font_texture.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * font_texture.width()),
                rows_per_image: Some(font_texture.height()),
            },
            Extent3d {
                width: font_texture.width(),
                height: font_texture.height(),
                depth_or_array_layers: 1,
            },
        );

        let font_texture = texture.create_view(&TextureViewDescriptor {
            label: "ascii_font_texture".into(),
            ..Default::default()
        });

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("ascii_post_process_shader".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        AsciiShaderPipeline {
            layout,
            sampler,
            font_texture,
            pipeline_id,
        }
    }
}

//=============================================================================
//             Pixel Shader Pipeline
//=============================================================================

#[derive(Resource)]
pub struct PixelShaderPipeline {
    low_res_textures: HashMap<Entity, TextureView>,
    target_size: Vec2,
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PixelShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let queue = world.get_resource::<RenderQueue>().unwrap();
        let render_device = world.resource::<RenderDevice>();

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        //We need to create the bind group
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                //This is the screen texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("downsize_shader_bind_group_layout"),
        });

        let shader = world.resource::<AssetServer>().load("pixel.wgsl");

        // let low_res_texture = render_device.create_texture(&TextureDescriptor {
        //     label: "low_res_texture".into(),
        //     size: Extent3d {
        //         width: 1,
        //         height: 1,
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: TextureDimension::D2,
        //     format: TextureFormat::bevy_default(),
        //     usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        //     view_formats: &[TextureFormat::bevy_default()],
        // });

        // let low_res_texture = low_res_texture.create_view(&TextureViewDescriptor {
        //     label: Some("low_res_texture"),
        //     ..TextureViewDescriptor::default()
        // });

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("downsize_shader_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        PixelShaderPipeline {
            low_res_textures: HashMap::new(),
            target_size: Vec2 { x: 1.0, y: 1.0 },
            layout,
            sampler,
            pipeline_id,
        }
    }
}

//=============================================================================
//             Extract Step
//=============================================================================

pub fn extract_camera(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera, &AsciiCamera)>>,
) {
    for (entity, camera, pixel_camera) in &cameras {
        if camera.is_active {
            commands.get_or_spawn(entity).insert(pixel_camera.clone());
        }
    }
}

//=============================================================================
//             Prepare Step
//=============================================================================

// Thiw will calculate the target resolution for the effect. If this resolution changes,
// it will remake the texture.
pub fn prepare_pixel_shader(
    mut acsii_cameras: Query<(Entity, &AsciiCamera)>,
    mut pixel_shader_pipeline: ResMut<PixelShaderPipeline>,
    mut render_device: ResMut<RenderDevice>,
    windows: Res<ExtractedWindows>,
) {
    let window = windows.windows.get(&windows.primary.unwrap()).unwrap();

    for (entity, ascii_camera) in acsii_cameras.iter() {
        let target_resolution = Vec2::new(
            (window.physical_width as f32 / ascii_camera.pixels_per_character).floor(),
            (window.physical_height as f32 / ascii_camera.pixels_per_character).floor(),
        );

        if target_resolution != pixel_shader_pipeline.target_size
            || !pixel_shader_pipeline.low_res_textures.contains_key(&entity)
        {
            pixel_shader_pipeline.target_size = target_resolution;
            let low_res_texture = render_device
                .create_texture(&TextureDescriptor {
                    label: "low_res_texture".into(),
                    size: Extent3d {
                        width: target_resolution.x as u32,
                        height: target_resolution.y as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::bevy_default(),
                    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[TextureFormat::bevy_default()],
                })
                .create_view(&TextureViewDescriptor {
                    label: Some("low_res_texture"),
                    ..TextureViewDescriptor::default()
                });

            pixel_shader_pipeline
                .low_res_textures
                .insert(entity, low_res_texture);
        }
    }
}
