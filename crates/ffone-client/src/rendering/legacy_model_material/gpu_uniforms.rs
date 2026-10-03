//! Keep numeric material animation out of Bevy's texture/bind-group rebuild path.
//!
//! The shader ABI, curve sampling and material pipeline are unchanged. Only
//! explicitly uniform-only edits use this path; texture/state edits still emit
//! normal asset events. Each extracted asset ID owns its own GPU buffer.

use super::{LegacyModelMaterial, LegacyModelMaterialUniform, LegacyModelPipelineKey};
use bevy::{
    ecs::system::SystemParamItem,
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        erased_render_asset::{AssetExtractionSystems, ErasedRenderAssets, ExtractedAssets},
        render_resource::{
            AsBindGroup, AsBindGroupError, BindGroupLayout, BindGroupLayoutEntry, BindingType,
            Buffer, BufferBindingType, BufferDescriptor, BufferInitDescriptor, BufferUsages,
            CommandEncoderDescriptor, OwnedBindingResource, ShaderStages, ShaderType,
            UnpreparedBindGroup, encase,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};
use std::collections::{HashMap, HashSet};

// Below this point the fixed staging-buffer submission costs more than the
// driver bookkeeping saved by batching. The threshold is based on matched
// Sector V and pool measurements on the same adapter.
const DIRECT_UNIFORM_WRITE_LIMIT: usize = 64;

/// Render-world ownership is deliberately excluded from material equality.
/// Main-world assets never receive an owner; extraction assigns the real ID,
/// including when the caller cloned another material to animate independently.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct UniformOwner(Option<AssetId<LegacyModelMaterial>>);

impl PartialEq for UniformOwner {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Resource)]
pub(crate) struct NativeMaterialUniformUpdates {
    enabled: bool,
    dirty: HashSet<AssetId<LegacyModelMaterial>>,
}

impl NativeMaterialUniformUpdates {
    #[cfg(test)]
    pub(crate) fn enabled_for_test() -> Self {
        Self {
            enabled: true,
            dirty: default(),
        }
    }

    /// Returns false when the caller must emit a normal Modified event.
    pub(crate) fn enqueue(&mut self, id: AssetId<LegacyModelMaterial>) -> bool {
        if self.enabled {
            self.dirty.insert(id);
        }
        self.enabled
    }
}

#[derive(Resource, Default)]
pub struct NativeGpuUniforms {
    buffers: HashMap<AssetId<LegacyModelMaterial>, Buffer>,
    // Retain the newest value while textures are loading. A non-looping curve
    // can finish before its material is prepared and never write another frame.
    pending: HashMap<AssetId<LegacyModelMaterial>, LegacyModelMaterialUniform>,
    staging: Option<Buffer>,
    upload_bytes: Vec<u8>,
    copies: Vec<(Buffer, u64)>,
}

pub(super) fn install(app: &mut App) {
    if app.get_sub_app(RenderApp).is_none() {
        return;
    }
    let baseline = std::env::var_os("FFONE_PERF_OUTPUT").is_some()
        && std::env::var_os("FFONE_PERF_UNIFORM_BASELINE").is_some();
    app.insert_resource(NativeMaterialUniformUpdates {
        enabled: !baseline,
        dirty: default(),
    })
    .add_systems(
        First,
        |mut updates: ResMut<NativeMaterialUniformUpdates>| updates.dirty.clear(),
    );
    app.sub_app_mut(RenderApp)
        .init_resource::<NativeGpuUniforms>()
        .add_systems(
            ExtractSchedule,
            extract_uniforms.after(AssetExtractionSystems),
        )
        .add_systems(
            Render,
            upload_uniforms.in_set(RenderSystems::PrepareBindGroups),
        );
}

fn extract_uniforms(
    mut commands: Commands,
    materials: Extract<Res<Assets<LegacyModelMaterial>>>,
    updates: Extract<Res<NativeMaterialUniformUpdates>>,
) {
    let current = updates
        .dirty
        .iter()
        .filter_map(|id| materials.get(*id).map(|material| (*id, material.uniform)))
        .collect();
    queue_uniform_staging(&mut commands, updates.enabled, current);
}

fn queue_uniform_staging(
    commands: &mut Commands,
    enabled: bool,
    current: HashMap<AssetId<LegacyModelMaterial>, LegacyModelMaterialUniform>,
) {
    // Bevy disables BOTH automatic and final deferred application in Extract.
    // AssetExtractionSystems inserts ExtractedAssets through Commands. Queue
    // our dependent work after those commands, so it runs in ExtractCommands
    // on this frame's assets, before PrepareAssets consumes them. Reading the
    // resource directly in Extract would see last frame's drained collection.
    commands.queue(move |world: &mut World| {
        world.resource_scope(|world, mut gpu: Mut<NativeGpuUniforms>| {
            let mut extracted =
                world.resource_mut::<ExtractedAssets<MeshMaterial3d<LegacyModelMaterial>>>();
            stage_uniforms(enabled, current, &mut extracted, &mut gpu);
        });
    });
}

fn stage_uniforms(
    enabled: bool,
    current: HashMap<AssetId<LegacyModelMaterial>, LegacyModelMaterialUniform>,
    extracted: &mut ExtractedAssets<MeshMaterial3d<LegacyModelMaterial>>,
    gpu: &mut NativeGpuUniforms,
) {
    for id in &extracted.removed {
        gpu.buffers.remove(id);
        gpu.pending.remove(id);
    }
    if !enabled {
        return;
    }
    for (id, material) in &mut extracted.extracted {
        material.uniform_owner = UniformOwner(Some(*id));
        // Also supersede any pending animation with later normal asset edits.
        gpu.pending.insert(*id, material.uniform);
    }
    // Current main-world values supersede an older pending final pose.
    gpu.pending.extend(
        current
            .into_iter()
            .filter(|(id, _)| !extracted.removed.contains(id)),
    );
}

fn uniform_bytes(uniform: &LegacyModelMaterialUniform) -> Vec<u8> {
    // Exactly the serializer used by Bevy's #[uniform(0)] derive.
    let mut data = encase::UniformBuffer::new(Vec::new());
    data.write(uniform)
        .expect("native model uniform has a valid shader layout");
    data.into_inner()
}

fn append_uniform(bytes: &mut Vec<u8>, uniform: &LegacyModelMaterialUniform) -> u64 {
    let offset = bytes.len();
    bytes.resize(
        offset + LegacyModelMaterialUniform::min_size().get() as usize,
        0,
    );
    let mut data = encase::UniformBuffer::new(&mut bytes[offset..]);
    data.write(uniform)
        .expect("native model uniform has a valid shader layout");
    offset as u64
}

fn upload_uniforms(
    mut gpu: ResMut<NativeGpuUniforms>,
    queue: Res<RenderQueue>,
    device: Res<RenderDevice>,
    prepared: Res<ErasedRenderAssets<bevy::pbr::PreparedMaterial>>,
) {
    let NativeGpuUniforms {
        buffers,
        pending,
        staging,
        upload_bytes,
        copies,
    } = &mut *gpu;
    upload_bytes.clear();
    copies.clear();
    pending.retain(|id, uniform| {
        let Some(buffer) = buffers.get(id) else {
            debug_assert!(
                prepared.get(*id).is_none(),
                "prepared native material is missing its persistent uniform buffer"
            );
            return true;
        };
        let offset = append_uniform(upload_bytes, uniform);
        copies.push((buffer.clone(), offset));
        false
    });
    if upload_bytes.is_empty() {
        return;
    }
    let uniform_size = LegacyModelMaterialUniform::min_size().get() as usize;
    if copies.len() <= DIRECT_UNIFORM_WRITE_LIMIT {
        for (buffer, offset) in copies.iter() {
            let start = *offset as usize;
            queue.write_buffer(buffer, 0, &upload_bytes[start..start + uniform_size]);
        }
        copies.clear();
        return;
    }
    // One staging upload replaces hundreds of driver allocations. Copy the
    // exact byte ranges to the existing material buffers; their shader bindings
    // and per-instance ownership remain unchanged.
    let size = upload_bytes.len() as u64;
    if staging.as_ref().is_none_or(|buffer| buffer.size() < size) {
        *staging = Some(device.create_buffer(&BufferDescriptor {
            label: Some("native material batch staging"),
            size: size.next_power_of_two(),
            usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }
    let staging = staging.as_ref().unwrap();
    queue.write_buffer(staging, 0, upload_bytes);
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("native material uniform copies"),
    });
    for (buffer, offset) in copies.iter() {
        encoder.copy_buffer_to_buffer(
            staging,
            *offset,
            buffer,
            0,
            LegacyModelMaterialUniform::min_size().get(),
        );
    }
    // The single queue preserves prior-frame -> upload/copy -> current draw
    // ordering. PrepareBindGroups completes before the render graph submits.
    queue.submit([encoder.finish()]);
    copies.clear();
}

// Derive only the unchanged texture bindings; own binding 0 so an animation
// does not allocate a new uniform buffer (or rebuild four texture bindings).
#[derive(AsBindGroup)]
pub struct NativeMaterialTextureBindings {
    #[texture(1)]
    #[sampler(2)]
    base_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    toon_ramp: Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    bump_texture: Option<Handle<Image>>,
    #[texture(7)]
    #[sampler(8)]
    effect_map: Option<Handle<Image>>,
}

impl AsBindGroup for LegacyModelMaterial {
    fn label() -> &'static str {
        "LegacyModelMaterial"
    }

    type Data = LegacyModelPipelineKey;
    type Param = (
        <NativeMaterialTextureBindings as AsBindGroup>::Param,
        Option<ResMut<'static, NativeGpuUniforms>>,
    );

    fn bind_group_data(&self) -> Self::Data {
        self.into()
    }

    fn unprepared_bind_group(
        &self,
        layout: &BindGroupLayout,
        device: &RenderDevice,
        param: &mut SystemParamItem<'_, '_, Self::Param>,
        force_no_bindless: bool,
    ) -> Result<UnpreparedBindGroup, AsBindGroupError> {
        let textures = NativeMaterialTextureBindings {
            base_texture: self.base_texture.clone(),
            toon_ramp: self.toon_ramp.clone(),
            bump_texture: self.bump_texture.clone(),
            effect_map: self.effect_map.clone(),
        };
        let mut group =
            textures.unprepared_bind_group(layout, device, &mut param.0, force_no_bindless)?;
        let make_buffer = || {
            device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("native model uniform"),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                contents: &uniform_bytes(&self.uniform),
            })
        };
        let buffer = match (self.uniform_owner.0, param.1.as_mut()) {
            (Some(id), Some(gpu)) => gpu.buffers.entry(id).or_insert_with(make_buffer).clone(),
            _ => make_buffer(),
        };
        group
            .bindings
            .push((0, OwnedBindingResource::Buffer(buffer)));
        Ok(group)
    }

    fn bind_group_layout_entries(
        device: &RenderDevice,
        force_no_bindless: bool,
    ) -> Vec<BindGroupLayoutEntry> {
        let mut entries =
            NativeMaterialTextureBindings::bind_group_layout_entries(device, force_no_bindless);
        entries.push(BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(LegacyModelMaterialUniform::min_size()),
            },
            count: None,
        });
        entries
    }
}

#[cfg(test)]
mod tests;
