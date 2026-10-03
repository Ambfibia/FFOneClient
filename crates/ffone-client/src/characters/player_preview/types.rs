use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum NativePlayerPreviewStage {
    #[default]
    Selection,
    Creation,
    /// Clean `Panel_UserClothes` normal Item/Nano-book presentation. This is
    /// deliberately distinct from `eGui_NanoMachine`, whose camera layout is
    /// owned by the Nano-station UI.
    Inventory,
    TryOn,
    Barber,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativePlayerPartKind {
    Face,
    Hair,
    Shirt,
    Pants,
    Shoes,
    Hat,
    Glasses,
    Back,
    Weapon,
    Vehicle,
}

/// How one table-owned player part enters the common actor instance.
///
/// Most categories have one fixed policy, but legacy Back rows are mixed:
/// `m_iEquipType == 0` uses `AttachGO(back01)`, while equip type one is a
/// skinned cape sampled by the actor skeleton. Keeping that distinction on
/// the resolved part prevents preview and world consumers from guessing from
/// a GLB name or from the mere presence of a skin.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NativePlayerPartAssembly {
    SharedSkin,
    RigidAttachment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativePlayerPartLook {
    pub kind: NativePlayerPartKind,
    pub assembly: NativePlayerPartAssembly,
    /// Exact legacy GameObject route certified by the native shared-rig
    /// contract. The GLB path alone is not a stable ownership key because one
    /// published model can have gender-specific route evidence.
    pub exact_route: String,
    pub glb: String,
    pub primary_texture: Option<NativePlayerTexture>,
    /// Exact `m_pstrSTextureString` passed to `AttachGO` for attachments.
    /// Body parts still route their `sub` material to the global skin texture,
    /// matching `SetBody`.
    pub secondary_texture: Option<NativePlayerTexture>,
}

impl NativePlayerPartLook {
    #[must_use]
    pub const fn uses_shared_skin(&self) -> bool {
        matches!(self.assembly, NativePlayerPartAssembly::SharedSkin)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativePlayerLook {
    pub identity: String,
    pub gender: PlayerRigGender,
    pub parts: Vec<NativePlayerPartLook>,
    pub skin_texture: Option<NativePlayerTexture>,
    pub skin_color: LinearRgba,
    pub hair_color: LinearRgba,
    /// Exact `cnAvatarStatus.SetFaceHand` animation family derived from the
    /// equipped WeaponItemTable row. Every independent preview and gameplay
    /// rig consumes this same value.
    pub weapon_animation_profile: Option<PlayerWeaponAnimationProfile>,
    /// Five authored height poses sampled by the shared skeleton.
    pub height_selector: i8,
    /// Three authored body poses sampled by the shared skeleton.
    pub body_selector: i8,
}

impl NativePlayerLook {
    pub fn validate(&self) -> Result<(), String> {
        if self.identity.trim().is_empty() {
            return Err("native player look identity is empty".to_owned());
        }
        if self.parts.is_empty() {
            return Err("native player look contains no model parts".to_owned());
        }
        let mut kinds = std::collections::BTreeSet::new();
        for part in &self.parts {
            if !kinds.insert(part.kind) {
                return Err(format!("native player look repeats {:?} part", part.kind));
            }
            validate_exact_route(&part.exact_route)?;
            validate_asset_path(&part.glb, ".glb")?;
            if let Some(texture) = &part.primary_texture {
                validate_player_texture(texture)?;
            }
            if let Some(texture) = &part.secondary_texture {
                validate_player_texture(texture)?;
            }
            let assembly_is_valid = match part.kind {
                NativePlayerPartKind::Face
                | NativePlayerPartKind::Hair
                | NativePlayerPartKind::Shirt
                | NativePlayerPartKind::Pants
                | NativePlayerPartKind::Shoes => part.uses_shared_skin(),
                NativePlayerPartKind::Hat
                | NativePlayerPartKind::Glasses
                | NativePlayerPartKind::Weapon
                | NativePlayerPartKind::Vehicle => !part.uses_shared_skin(),
                NativePlayerPartKind::Back => true,
            };
            if !assembly_is_valid {
                return Err(format!(
                    "native player {:?} part {:?} has invalid {:?} assembly",
                    part.kind, part.exact_route, part.assembly
                ));
            }
        }
        if let Some(texture) = &self.skin_texture {
            validate_player_texture(texture)?;
        }
        validate_palette_color("skin", self.skin_color)?;
        validate_palette_color("hair", self.hair_color)?;
        if !(0..=4).contains(&self.height_selector) {
            return Err(format!(
                "height selector {} is outside 0..=4",
                self.height_selector
            ));
        }
        if !(0..=2).contains(&self.body_selector) {
            return Err(format!(
                "body selector {} is outside 0..=2",
                self.body_selector
            ));
        }
        Ok(())
    }
}

/// Orders external look selection before the preview's rebuild/bind pipeline.
///
/// The main client changes UI state and preview identity in the same frame. An
/// explicit boundary prevents the renderer from observing a half-transitioned
/// selection/creation model.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum NativePlayerPreviewSet {
    Rebuild,
}

#[derive(Component)]
pub(super) struct NativePlayerPreviewRoot;

#[derive(Component, Clone)]
pub(super) struct NativePlayerPreviewPart {
    pub(super) kind: NativePlayerPartKind,
    pub(super) exact_route: String,
    pub(super) generation: u64,
}

#[derive(Component)]
pub(super) struct PendingNativePlayerPreviewAttachment {
    pub(super) rig_root: Entity,
    pub(super) socket_full_path: String,
    pub(super) socket_local_scale_override: Option<Vec3>,
}

#[derive(Component)]
/// Camera marker exposed to native authoring tools that supply their own viewport.
/// Gameplay consumers keep the production camera policy unchanged.
pub struct NativePlayerPreviewCamera;

#[derive(Component)]
pub(super) struct NativePlayerPreviewLight;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NativePlayerPreviewGeometry {
    pub(super) gender: PlayerRigGender,
    pub(super) parts: Vec<(
        NativePlayerPartKind,
        NativePlayerPartAssembly,
        String,
        String,
    )>,
    pub(super) weapon_animation_profile: Option<PlayerWeaponAnimationProfile>,
}

impl From<&NativePlayerLook> for NativePlayerPreviewGeometry {
    fn from(look: &NativePlayerLook) -> Self {
        Self {
            gender: look.gender,
            weapon_animation_profile: look.weapon_animation_profile,
            parts: look
                .parts
                .iter()
                .map(|part| {
                    (
                        part.kind,
                        part.assembly,
                        part.exact_route.clone(),
                        part.glb.clone(),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Default)]
pub(super) struct NativePlayerBindingDiagnostics {
    pub(super) seen: usize,
    pub(super) bound: usize,
    pub(super) no_part: usize,
    pub(super) wrong_generation: usize,
    pub(super) no_look: usize,
    pub(super) no_metadata: usize,
    pub(super) no_material: usize,
}

pub struct NativePlayerPreviewPlugin;

#[derive(Resource)]
pub struct NativePlayerTryOnPreviewImage(pub Handle<Image>);

#[derive(Resource)]
pub struct NativePlayerBarberPreviewImage(pub Handle<Image>);

impl Plugin for NativePlayerPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NativePlayerPreviewModel>()
            .init_resource::<NativePlayerPreviewRuntime>()
            .init_resource::<NativePlayerPreviewAudit>()
            .add_systems(Startup, setup_native_player_preview_stage)
            .add_systems(
                PostUpdate,
                inventory_target::sync_inventory_preview_resolution
                    .before(bevy::camera::CameraUpdateSystems),
            )
            .configure_sets(Update, NativePlayerPreviewSet::Rebuild)
            .add_systems(
                Update,
                (
                    rebuild_native_player_preview,
                    bind_native_player_preview_attachments,
                    bind_native_player_preview_render_layers,
                    recover_native_player_preview_idle_animation,
                    bind_native_player_preview_materials,
                    update_native_player_preview_status,
                    sync_native_player_preview_transform,
                )
                    .chain()
                    .in_set(NativePlayerPreviewSet::Rebuild),
            );
    }
}
