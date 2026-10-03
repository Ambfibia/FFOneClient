//! Retrobution's camera-owned `GlowEffect`, reconstructed from the original
//! `GlowEffect.cs`, serialized mainData component, and D3D9 shader programs.
//! Native correction: normalized blur with intensity applied once during
//! composition prevents the source combiner's exponential overexposure.

use bevy::{
    asset::{AssetPath, embedded_asset, embedded_path},
    core_pipeline::{Core3dSystems, FullscreenShader, schedule::Core3d, tonemapping::tonemapping},
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        camera::ExtractedCamera,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d, FilterMode,
            FragmentState, LoadOp, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler,
            SamplerDescriptor, ShaderStages, ShaderType, StoreOp, TextureDescriptor,
            TextureDimension, TextureFormat, TextureSampleType, TextureUsages, binding_types,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
    },
};

const LEGACY_GLOW_ITERATIONS: usize = 4;

#[derive(Component, Clone, Copy, Debug, ExtractComponent, ShaderType)]
pub struct LegacyGlowSettings {
    /// Exact dynamic `GlowEffect.filterColor`, assigned by `DefaultAmbience`.
    pub filter_color: Vec4,
    /// Exact mainData value `(20/255, 20/255, 20/255, 1)`.
    pub glow_tint: Vec4,
    /// x: intensity; y: blur spread; z/w reserved.
    pub parameters: Vec4,
}

impl Default for LegacyGlowSettings {
    fn default() -> Self {
        Self {
            filter_color: Vec4::ONE,
            glow_tint: Vec4::new(0.078_431_375, 0.078_431_375, 0.078_431_375, 1.0),
            parameters: Vec4::new(1.8, 1.0, 0.0, 0.0),
        }
    }
}

impl LegacyGlowSettings {
    pub fn set_filter_color(&mut self, rgba: [f32; 4]) {
        self.filter_color = Vec4::from_array(rgba);
    }
}

#[derive(Default)]
pub struct LegacyGlowPlugin;

impl Plugin for LegacyGlowPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "legacy_glow.wgsl");
        app.add_plugins((
            ExtractComponentPlugin::<LegacyGlowSettings>::default(),
            UniformComponentPlugin::<LegacyGlowSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(RenderStartup, init_legacy_glow_pipeline)
            .add_systems(
                Render,
                prepare_legacy_glow_textures.in_set(RenderSystems::PrepareResources),
            )
            .add_systems(
                Core3d,
                legacy_glow
                    .in_set(Core3dSystems::PostProcess)
                    .after(tonemapping),
            );
    }
}

#[derive(Component)]
struct LegacyGlowTextures {
    ping: CachedTexture,
    pong: CachedTexture,
}

fn prepare_legacy_glow_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera, &LegacyGlowSettings)>,
) {
    for (entity, camera, _) in &views {
        let Some(viewport) = camera.physical_viewport_size else {
            continue;
        };
        let descriptor = TextureDescriptor {
            label: Some("legacy_glow_quarter_texture"),
            size: Extent3d {
                width: (viewport.x / 4).max(1),
                height: (viewport.y / 4).max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let ping = texture_cache.get(&render_device, descriptor.clone());
        let pong = texture_cache.get(&render_device, descriptor);
        commands
            .entity(entity)
            .insert(LegacyGlowTextures { ping, pong });
    }
}

#[derive(Resource)]
struct LegacyGlowPipeline {
    source_layout: BindGroupLayoutDescriptor,
    composite_layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    downsample: CachedRenderPipelineId,
    blur: [CachedRenderPipelineId; LEGACY_GLOW_ITERATIONS],
    composite: CachedRenderPipelineId,
}

fn init_legacy_glow_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let source_layout = BindGroupLayoutDescriptor::new(
        "legacy_glow_source_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                binding_types::texture_2d(TextureSampleType::Float { filterable: true }),
                binding_types::sampler(
                    bevy::render::render_resource::SamplerBindingType::Filtering,
                ),
                binding_types::uniform_buffer::<LegacyGlowSettings>(true),
            ),
        ),
    );
    let composite_layout = BindGroupLayoutDescriptor::new(
        "legacy_glow_composite_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                binding_types::texture_2d(TextureSampleType::Float { filterable: true }),
                binding_types::sampler(
                    bevy::render::render_resource::SamplerBindingType::Filtering,
                ),
                binding_types::uniform_buffer::<LegacyGlowSettings>(true),
                binding_types::texture_2d(TextureSampleType::Float { filterable: true }),
                binding_types::sampler(
                    bevy::render::render_resource::SamplerBindingType::Filtering,
                ),
            ),
        ),
    );
    let sampler = render_device.create_sampler(&SamplerDescriptor {
        label: Some("legacy_glow_linear_clamp_sampler"),
        address_mode_u: bevy::render::render_resource::AddressMode::ClampToEdge,
        address_mode_v: bevy::render::render_resource::AddressMode::ClampToEdge,
        address_mode_w: bevy::render::render_resource::AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    });
    let shader = asset_server
        .load(AssetPath::from_path_buf(embedded_path!("legacy_glow.wgsl")).with_source("embedded"));
    let vertex = fullscreen_shader.to_vertex_state();
    let queue_pipeline =
        |label: &'static str, layout: BindGroupLayoutDescriptor, entry_point: &'static str| {
            pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some(label.into()),
                layout: vec![layout],
                vertex: vertex.clone(),
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    entry_point: Some(entry_point.into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::Rgba8UnormSrgb,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                    ..default()
                }),
                ..default()
            })
        };
    let downsample = queue_pipeline(
        "legacy_glow_downsample_pipeline",
        source_layout.clone(),
        "downsample",
    );
    let blur = [
        queue_pipeline(
            "legacy_glow_blur_0_pipeline",
            source_layout.clone(),
            "blur_0",
        ),
        queue_pipeline(
            "legacy_glow_blur_1_pipeline",
            source_layout.clone(),
            "blur_1",
        ),
        queue_pipeline(
            "legacy_glow_blur_2_pipeline",
            source_layout.clone(),
            "blur_2",
        ),
        queue_pipeline(
            "legacy_glow_blur_3_pipeline",
            source_layout.clone(),
            "blur_3",
        ),
    ];
    let composite = queue_pipeline(
        "legacy_glow_composite_pipeline",
        composite_layout.clone(),
        "composite",
    );
    commands.insert_resource(LegacyGlowPipeline {
        source_layout,
        composite_layout,
        sampler,
        downsample,
        blur,
        composite,
    });
}

fn legacy_glow(
    view: ViewQuery<(
        &ViewTarget,
        &LegacyGlowSettings,
        &DynamicUniformIndex<LegacyGlowSettings>,
        &LegacyGlowTextures,
    )>,
    pipeline: Res<LegacyGlowPipeline>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<LegacyGlowSettings>>,
    mut render_context: RenderContext,
) {
    let (view_target, _, settings_index, textures) = view.into_inner();
    let Some(downsample_pipeline) = pipeline_cache.get_render_pipeline(pipeline.downsample) else {
        return;
    };
    let Some(blur_pipelines) = pipeline
        .blur
        .iter()
        .map(|id| pipeline_cache.get_render_pipeline(*id))
        .collect::<Option<Vec<&RenderPipeline>>>()
    else {
        return;
    };
    let Some(composite_pipeline) = pipeline_cache.get_render_pipeline(pipeline.composite) else {
        return;
    };
    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    let post_process = view_target.post_process_write();
    let render_device = render_context.render_device();
    let downsample_bind_group = render_device.create_bind_group(
        "legacy_glow_downsample_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipeline.source_layout),
        &BindGroupEntries::sequential((
            post_process.source,
            &pipeline.sampler,
            settings_binding.clone(),
        )),
    );
    let ping_to_pong = render_device.create_bind_group(
        "legacy_glow_ping_to_pong_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipeline.source_layout),
        &BindGroupEntries::sequential((
            &textures.ping.default_view,
            &pipeline.sampler,
            settings_binding.clone(),
        )),
    );
    let pong_to_ping = render_device.create_bind_group(
        "legacy_glow_pong_to_ping_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipeline.source_layout),
        &BindGroupEntries::sequential((
            &textures.pong.default_view,
            &pipeline.sampler,
            settings_binding.clone(),
        )),
    );
    let composite_bind_group = render_device.create_bind_group(
        "legacy_glow_composite_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipeline.composite_layout),
        &BindGroupEntries::sequential((
            post_process.source,
            &pipeline.sampler,
            settings_binding,
            &textures.ping.default_view,
            &pipeline.sampler,
        )),
    );
    let uniform_offset = &[settings_index.index()];

    {
        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("legacy_glow_downsample_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &textures.ping.default_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(default()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_render_pipeline(downsample_pipeline);
        pass.set_bind_group(0, &downsample_bind_group, uniform_offset);
        pass.draw(0..3, 0..1);
    }

    for (iteration, blur_pipeline) in blur_pipelines.into_iter().enumerate() {
        let even = iteration % 2 == 0;
        let (destination, bind_group) = if even {
            (&textures.pong.default_view, &ping_to_pong)
        } else {
            (&textures.ping.default_view, &pong_to_ping)
        };
        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("legacy_glow_cone_tap_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(default()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_render_pipeline(blur_pipeline);
        pass.set_bind_group(0, bind_group, uniform_offset);
        pass.draw(0..3, 0..1);
    }

    {
        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("legacy_glow_filter_and_composite_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(default()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_render_pipeline(composite_pipeline);
        pass.set_bind_group(0, &composite_bind_group, uniform_offset);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests;
