use super::*;

pub(super) fn stable_native_render_plugin() -> RenderPlugin {
    #[cfg(target_os = "windows")]
    {
        // wgpu's automatic adapter selection picked Vulkan on the supported
        // Windows test machines. That surface can return an unrecoverable
        // generic acquire error after focus/minimize changes, after which all
        // Bevy view uniform buffers become invalid. D3D12 is the native stable
        // presentation path for the Windows-only FFOne runtime.
        RenderPlugin {
            render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                backends: Some(Backends::DX12),
                // `cargo dev` keeps Rust debug assertions for the client, but
                // that must not silently turn an ordinary play-test into a
                // D3D12 debug-layer run. The SDK validation layer is designed
                // for graphics debugging and was one of the largest sampled
                // CPU consumers in a fully resident world. Match wgpu's
                // release default while retaining indirect-command checking;
                // every flag can still be explicitly enabled through WGPU_*
                // environment variables for a dedicated graphics audit.
                instance_flags: InstanceFlags::VALIDATION_INDIRECT_CALL.with_env(),
                ..default()
            })),
            ..default()
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        RenderPlugin::default()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[type_path = "ffone_client::app"]
pub(super) struct DexterHologramMaterial {
    #[uniform(0)]
    pub(super) uniform: DexterHologramUniform,
    #[texture(1)]
    #[sampler(2)]
    pub(super) scene_texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub(super) decal_texture: Handle<Image>,
}

#[derive(Debug, Clone, Copy, ShaderType)]
pub(super) struct DexterHologramUniform {
    pub(super) color: Vec4,
    pub(super) offset: Vec4,
}

impl UiMaterial for DexterHologramMaterial {
    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(dexter_hologram_asset_path())
            .with_source("embedded")
            .into()
    }
}
