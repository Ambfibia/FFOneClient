//! Published clean-client `all_hnpc.asset` appearances for ordinary-world
//! HNPCs.
//!
//! The runtime document is native data under `assets/game`: legacy bundles and
//! extraction caches are provenance only and are never opened here. Body
//! pieces use the shared player skeleton; rigid equipment keeps the exact
//! `ActorSkinCombiner.AttachGO` socket path in the network-world consumer.

use std::{collections::BTreeMap, path::Path, sync::Arc};

use bevy::color::LinearRgba;
use ffone_runtime_contracts::{
    AvatarItemCategory, CHARACTER_CREATION_AVATAR_ITEMS_PATH,
    CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA, CHARACTER_CREATION_RUNTIME_TEXTURES_PATH,
    CHARACTER_CREATION_RUNTIME_TEXTURES_SCHEMA, CharacterCreationAssetReference,
    CharacterCreationAvatarItems, CharacterCreationRuntimeTextures,
    CharacterRuntimeTextureContract, CharacterRuntimeTextureSource, PlayerRigGender,
};
use ffone_skinned_model::{
    NativeSampler, PublishedMipPolicy, SamplerMagFilter, SamplerMinFilter, SamplerWrapMode,
    TextureColorSpace,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    assets::AssetLocator,
    player_preview::{
        NativePlayerLook, NativePlayerPartAssembly, NativePlayerPartKind, NativePlayerPartLook,
        NativePlayerTexture,
    },
    player_shared_rig::NativePlayerRigCatalog,
    tutorial_player_presentation::PlayerWeaponAnimationProfile,
};

pub const HNPC_RUNTIME_CATALOG_PATH: &str = "data/hnpc/catalog.json";
const HNPC_RUNTIME_CATALOG_SCHEMA: &str = "ffone.hnpc-runtime-catalog.v1";

#[derive(Clone, Debug)]
pub struct HnpcRuntimeAppearance {
    pub legacy_type: i32,
    pub look: Option<NativePlayerLook>,
}

#[derive(Clone, Debug)]
pub struct HnpcRuntimeCatalog {
    appearances: BTreeMap<usize, HnpcRuntimeAppearance>,
    rig_catalog: NativePlayerRigCatalog,
    animation_ends: [Arc<BTreeMap<String, f32>>; 2],
    animation_sounds: [Arc<[crate::network_world_runtime::NetworkNpcAnimationSoundEvent0104]>; 2],
}

impl HnpcRuntimeCatalog {
    pub fn open(
        locator: &AssetLocator,
        base_rig_catalog: &NativePlayerRigCatalog,
    ) -> Result<Self, String> {
        let document: HnpcRuntimeCatalogDocument = locator.read_json(HNPC_RUNTIME_CATALOG_PATH)?;
        Self::from_document(locator, base_rig_catalog, document)
    }

    /// Validate an unsaved native catalog using the same asset and rig contract as runtime loading.
    pub fn from_json(
        locator: &AssetLocator,
        base_rig_catalog: &NativePlayerRigCatalog,
        value: serde_json::Value,
    ) -> Result<Self, String> {
        let document = serde_json::from_value(value).map_err(|e| format!("HNPC catalog: {e}"))?;
        Self::from_document(locator, base_rig_catalog, document)
    }

    fn from_document(
        locator: &AssetLocator,
        base_rig_catalog: &NativePlayerRigCatalog,
        document: HnpcRuntimeCatalogDocument,
    ) -> Result<Self, String> {
        document.validate_provenance()?;

        let palette: HnpcPalette = locator.read_json("data/hnpc/palette.json")?;
        if palette.schema != "ffone.hnpc-palette.v1" {
            return Err(format!(
                "HNPC palette source has schema {:?}, expected {:?}",
                palette.schema, "ffone.hnpc-palette.v1"
            ));
        }

        let mut textures = load_textures(locator, document.textures)?;
        load_shared_skin_textures(locator, &mut textures)?;
        let weapon_equip_types = load_weapon_equip_types(locator)?;
        let mut rig_catalog = base_rig_catalog.clone();
        let mut appearances = BTreeMap::new();
        for (position, appearance) in document.appearances.into_iter().enumerate() {
            if appearance.index != position {
                return Err(format!(
                    "HNPC appearance index drift at position {position}: {}",
                    appearance.index
                ));
            }
            let expected_gender = if appearance.legacy_type.rem_euclid(2) == 0 {
                PlayerRigGender::Female
            } else {
                PlayerRigGender::Male
            };
            if appearance.gender != expected_gender {
                return Err(format!(
                    "HNPC appearance {} gender {:?} contradicts legacy iType {} parity",
                    appearance.index, appearance.gender, appearance.legacy_type
                ));
            }
            if !(0..=4).contains(&appearance.height) || !(0..=2).contains(&appearance.shape) {
                return Err(format!(
                    "HNPC appearance {} has invalid height/shape selectors {}/{}",
                    appearance.index, appearance.height, appearance.shape
                ));
            }

            let mut native_parts = Vec::new();
            let mut seen_kinds = BTreeMap::new();
            let mut weapon_animation_profile = None;
            for part in appearance.parts {
                part.validate_identity(locator)?;
                if seen_kinds
                    .insert(part.kind, part.exact_route.clone())
                    .is_some()
                {
                    return Err(format!(
                        "HNPC appearance {} repeats {:?}",
                        appearance.index, part.kind
                    ));
                }
                let Some(native_kind) = part.kind.native_kind() else {
                    return Err(format!(
                        "HNPC appearance {} uses unsupported left-hand weapon route {:?}",
                        appearance.index, part.exact_route
                    ));
                };
                if native_kind == NativePlayerPartKind::Weapon {
                    let equip_types = weapon_equip_types
                        .get(&part.exact_route)
                        .ok_or_else(|| {
                            format!(
                                "HNPC appearance {} weapon route {:?} has no WeaponItemTable equip type",
                                appearance.index, part.exact_route
                            )
                        })?;
                    if equip_types.len() != 1 {
                        return Err(format!(
                            "HNPC appearance {} weapon route {:?} ambiguously maps to equip types {:?}",
                            appearance.index, part.exact_route, equip_types
                        ));
                    }
                    let equip_type = *equip_types.iter().next().expect("one equip type");
                    let profile = PlayerWeaponAnimationProfile::from_equip_type(equip_type)
                        .ok_or_else(|| {
                            format!(
                                "HNPC appearance {} weapon route {:?} uses unsupported equip type {equip_type}",
                                appearance.index, part.exact_route
                            )
                        })?;
                    if let Some(previous) = weapon_animation_profile.replace(profile)
                        && previous != profile
                    {
                        return Err(format!(
                            "HNPC appearance {} mixes weapon animation profiles",
                            appearance.index
                        ));
                    }
                }
                if let Some(clothes_index) = part.kind.clothes_index() {
                    if part.actor_skin_combiner_clothes_index != Some(clothes_index) {
                        return Err(format!(
                            "HNPC appearance {} route {:?} has wrong ActorSkinCombiner clothes index",
                            appearance.index, part.exact_route
                        ));
                    }
                    rig_catalog.register_hnpc_skinned_part(
                        appearance.gender,
                        part.exact_route.clone(),
                        part.true_name.clone(),
                        part.native_asset.path.clone(),
                        part.native_asset.bytes,
                        &part.native_asset.blake3,
                        clothes_index,
                    )?;
                } else if part.actor_skin_combiner_clothes_index.is_some() {
                    return Err(format!(
                        "HNPC rigid route {:?} unexpectedly declares a clothes index",
                        part.exact_route
                    ));
                }
                let assembly = if part.kind.clothes_index().is_some() {
                    NativePlayerPartAssembly::SharedSkin
                } else {
                    NativePlayerPartAssembly::RigidAttachment
                };
                native_parts.push(NativePlayerPartLook {
                    kind: native_kind,
                    assembly,
                    exact_route: part.exact_route,
                    glb: part.native_asset.path,
                    primary_texture: part
                        .textures
                        .first()
                        .map(|name| resolve_texture(&textures, name))
                        .transpose()?,
                    secondary_texture: part
                        .textures
                        .get(1)
                        .map(|name| resolve_texture(&textures, name))
                        .transpose()?,
                });
            }

            let look = if native_parts.is_empty() {
                None
            } else {
                let skin_name = match appearance.gender {
                    PlayerRigGender::Male => "m_skin",
                    PlayerRigGender::Female => "f_skin",
                };
                let look = NativePlayerLook {
                    identity: format!(
                        "HNPC appearance {} type {}",
                        appearance.index, appearance.legacy_type
                    ),
                    gender: appearance.gender,
                    parts: native_parts,
                    skin_texture: Some(resolve_texture(&textures, skin_name)?),
                    skin_color: palette_color(&palette.skin, appearance.skin_color),
                    hair_color: palette_color(&palette.hair, appearance.hair_color),
                    weapon_animation_profile,
                    height_selector: appearance.height,
                    body_selector: appearance.shape,
                };
                look.validate()?;
                Some(look)
            };
            appearances.insert(
                appearance.index,
                HnpcRuntimeAppearance {
                    legacy_type: appearance.legacy_type,
                    look,
                },
            );
        }
        if appearances.is_empty() {
            return Err(format!(
                "production HNPC catalog has {} appearances, expected a non-empty set",
                appearances.len()
            ));
        }
        let mut animation_ends = Vec::new();
        let mut animation_sounds = Vec::new();
        for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
            let bytes = locator.read(&rig_catalog.gender(gender)?.skeleton_glb)?;
            animation_ends.push(read_hnpc_animation_ends(&bytes)?);
            animation_sounds.push(
                crate::network_world_runtime::parse_network_npc_animation_sound_events(&bytes)?
                    .into(),
            );
        }
        Ok(Self {
            appearances,
            animation_ends: animation_ends.try_into().unwrap(),
            animation_sounds: animation_sounds.try_into().unwrap(),
            rig_catalog,
        })
    }

    #[must_use]
    pub fn appearance(&self, index: usize) -> Option<&HnpcRuntimeAppearance> {
        self.appearances.get(&index)
    }

    /// Create an isolated preview catalog; the original runtime catalog stays unchanged.
    pub fn with_appearance_override(
        &self, index: usize, appearance: HnpcRuntimeAppearance,
    ) -> Result<Self, String> {
        if let Some(look) = &appearance.look {
            look.validate()?;
            let gender = if appearance.legacy_type.rem_euclid(2) == 0 { PlayerRigGender::Female } else { PlayerRigGender::Male };
            if look.gender != gender { return Err("HNPC gender contradicts legacy type parity".into()); }
        }
        let mut result = self.clone();
        result.appearances.insert(index, appearance);
        Ok(result)
    }

    #[must_use]
    pub fn rig_catalog(&self) -> &NativePlayerRigCatalog {
        &self.rig_catalog
    }

    pub fn animation_ends(&self, gender: PlayerRigGender) -> Arc<BTreeMap<String, f32>> {
        Arc::clone(
            &self.animation_ends[match gender {
                PlayerRigGender::Male => 0,
                PlayerRigGender::Female => 1,
            }],
        )
    }

    pub fn animation_sounds(
        &self,
        gender: PlayerRigGender,
    ) -> Arc<[crate::network_world_runtime::NetworkNpcAnimationSoundEvent0104]> {
        Arc::clone(
            &self.animation_sounds[match gender {
                PlayerRigGender::Male => 0,
                PlayerRigGender::Female => 1,
            }],
        )
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.appearances.len()
    }
}

fn read_hnpc_animation_ends(bytes: &[u8]) -> Result<Arc<BTreeMap<String, f32>>, String> {
    if bytes.len() < 20 || &bytes[..4] != b"glTF" || &bytes[16..20] != b"JSON" {
        return Err("HNPC skeleton has no GLB JSON chunk".to_owned());
    }
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let json = bytes
        .get(
            20..20_usize
                .checked_add(length)
                .ok_or("HNPC GLB length overflow")?,
        )
        .ok_or("HNPC GLB JSON chunk is truncated")?;
    let document: serde_json::Value =
        serde_json::from_slice(json).map_err(|error| error.to_string())?;
    let mut ends = BTreeMap::new();
    for animation in document["animations"]
        .as_array()
        .ok_or("HNPC skeleton has no animations")?
    {
        let Some(name) = animation["name"].as_str() else {
            continue;
        };
        let Some(events) = animation
            .pointer("/extras/nonTrs/events")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for event in events.iter().filter(|event| event["functionName"] == "end") {
            let time = event["time"]
                .as_f64()
                .ok_or("HNPC animation end has no time")? as f32;
            if !time.is_finite() || time <= 0.0 {
                return Err(format!("invalid HNPC end time for {name}"));
            }
            ends.entry(name.to_owned())
                .and_modify(|end: &mut f32| *end = end.min(time))
                .or_insert(time);
        }
    }
    Ok(Arc::new(ends))
}

fn load_weapon_equip_types(
    locator: &AssetLocator,
) -> Result<BTreeMap<String, std::collections::BTreeSet<i32>>, String> {
    let document: CharacterCreationAvatarItems =
        locator.read_json(CHARACTER_CREATION_AVATAR_ITEMS_PATH)?;
    if document.schema != CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA {
        return Err(format!(
            "HNPC weapon source has schema {:?}, expected {:?}",
            document.schema, CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA
        ));
    }
    let mut routes = BTreeMap::new();
    for item in document
        .items
        .iter()
        .filter(|item| item.category == AvatarItemCategory::Weapon)
    {
        let equip_type = i32::from(item.equip_type);
        for visual in [&item.male, &item.female] {
            for model in &visual.models {
                routes
                    .entry(model.exact_route.clone())
                    .or_insert_with(std::collections::BTreeSet::new)
                    .insert(equip_type);
            }
        }
    }
    Ok(routes)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcRuntimeCatalogDocument {
    schema: String,
    #[serde(default)]
    provenance: Option<HnpcRuntimeProvenance>,
    appearances: Vec<HnpcAppearanceDocument>,
    textures: Vec<HnpcTextureDocument>,
}

impl HnpcRuntimeCatalogDocument {
    fn validate_provenance(&self) -> Result<(), String> {
        if self.schema == "ffone.hnpc-runtime-catalog.v2" {
            if self.provenance.is_some()
                || self.textures.iter().any(|texture| texture.source.is_some())
            {
                return Err(
                    "native HNPC v2 must keep source lineage outside the runtime".to_owned(),
                );
            }
            return Ok(());
        }
        if self.schema != HNPC_RUNTIME_CATALOG_SCHEMA {
            return Err(format!(
                "unsupported HNPC catalog schema {:?}; expected {:?}",
                self.schema, HNPC_RUNTIME_CATALOG_SCHEMA
            ));
        }
        self.provenance
            .as_ref()
            .ok_or_else(|| "v1 HNPC provenance is missing".to_owned())?
            .validate()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcRuntimeProvenance {
    source_alias: String,
    source_build: String,
    raw_container: HnpcSourceProof,
    serialized_asset: HnpcSerializedAssetProof,
    focused_object_dump: HnpcDigestProof,
    player_item_catalog: String,
    conversion_version: String,
    skin_remap_policy: String,
    intentional_divergence: String,
}

impl HnpcRuntimeProvenance {
    fn validate(&self) -> Result<(), String> {
        if self.source_alias != "primary"
            || self.source_build != "retrobution-20260613"
            || self.raw_container.path != "TableData.resourceFile"
            || self.raw_container.bytes != 784_963
            || self.raw_container.sha256
                != "6d4cea151152e2ab75b7d16590bda00fba172600a163e0b3a5318564df0d7e3b"
            || self.serialized_asset.identity
                != "CustomAssetBundle-1dca92eecee4742d985b799d8226666d"
            || self.serialized_asset.object != "all_hnpc.asset"
            || self.serialized_asset.path_id != 6
            || self.serialized_asset.bytes != 6_814_737
            || self.serialized_asset.sha256
                != "17aedfd846239896f5ae998555254b679ff5666cf0e07eb68c6c14127c9ccdb1"
            || self.focused_object_dump.bytes != 216_598
            || self.focused_object_dump.sha256
                != "262edf172a5012a6f0b55f0d5680a3bc0590c3c8ca2f8782054464767da1b380"
            || self.player_item_catalog != "characters/player/items/catalog.json"
            || self.conversion_version != HNPC_RUNTIME_CATALOG_SCHEMA
            || self.skin_remap_policy != "published-glb-joint-path-to-verified-actor-bone-v1"
            || self.intentional_divergence.is_empty()
        {
            return Err(
                "HNPC catalog clean-primary provenance is incomplete or drifted".to_owned(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcSourceProof {
    path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcDigestProof {
    bytes: u64,
    sha256: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcSerializedAssetProof {
    identity: String,
    object: String,
    path_id: i64,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcAppearanceDocument {
    index: usize,
    legacy_type: i32,
    gender: PlayerRigGender,
    hair_color: i32,
    height: i8,
    shape: i8,
    skin_color: i32,
    parts: Vec<HnpcPartDocument>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
enum HnpcPartKind {
    Face,
    Hair,
    Shirt,
    Pants,
    Shoes,
    Hat,
    Glasses,
    Back,
    LeftWeapon,
    RightWeapon,
}

impl HnpcPartKind {
    const fn native_kind(self) -> Option<NativePlayerPartKind> {
        Some(match self {
            Self::Face => NativePlayerPartKind::Face,
            Self::Hair => NativePlayerPartKind::Hair,
            Self::Shirt => NativePlayerPartKind::Shirt,
            Self::Pants => NativePlayerPartKind::Pants,
            Self::Shoes => NativePlayerPartKind::Shoes,
            Self::Hat => NativePlayerPartKind::Hat,
            Self::Glasses => NativePlayerPartKind::Glasses,
            Self::Back => NativePlayerPartKind::Back,
            Self::RightWeapon => NativePlayerPartKind::Weapon,
            Self::LeftWeapon => return None,
        })
    }

    const fn clothes_index(self) -> Option<u8> {
        match self {
            Self::Face => Some(3),
            Self::Hair => Some(4),
            Self::Shirt => Some(2),
            Self::Pants => Some(1),
            Self::Shoes => Some(0),
            Self::Hat | Self::Glasses | Self::Back | Self::LeftWeapon | Self::RightWeapon => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcPartDocument {
    kind: HnpcPartKind,
    exact_route: String,
    source_route: String,
    resource_set: String,
    true_name: String,
    native_asset: CharacterCreationAssetReference,
    textures: Vec<String>,
    #[serde(default)]
    actor_skin_combiner_clothes_index: Option<u8>,
}

impl HnpcPartDocument {
    fn validate_identity(&self, locator: &AssetLocator) -> Result<(), String> {
        if !self.exact_route.starts_with("wear/")
            || !self.exact_route.ends_with(".nif")
            || self.source_route.is_empty()
            || self.resource_set.is_empty()
            || self.true_name.is_empty()
            || Path::new(&self.native_asset.path)
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("glb")
            || self.textures.len() > 2
        {
            return Err(format!(
                "HNPC part {:?} has incomplete native ownership",
                self.exact_route
            ));
        }
        locator
            .read_verified(
                &self.native_asset.path,
                Some(self.native_asset.bytes),
                &self.native_asset.blake3,
            )
            .map(|_| ())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcTextureDocument {
    true_name: String,
    path: String,
    bytes: u64,
    sha256: String,
    #[serde(default)]
    source: Option<HnpcTextureSourceDocument>,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
    #[serde(default)]
    sampler: Option<NativeSampler>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HnpcTextureSourceDocument {
    alias: String,
    bundle: String,
    serialized_asset: String,
    path_id: i64,
    container_route: String,
    width: u32,
    height: u32,
    texture_format: i32,
    complete_image_size: u64,
    source_chain_sha256: String,
    mip_map: bool,
    source_mip_count: u32,
    image_count: u32,
    texture_dimension: i32,
    filter_mode: i32,
    wrap_mode: i32,
    anisotropy_level: u32,
}

fn load_textures(
    locator: &AssetLocator,
    documents: Vec<HnpcTextureDocument>,
) -> Result<BTreeMap<String, NativePlayerTexture>, String> {
    let mut textures = BTreeMap::new();
    for mut document in documents {
        let native_sampler = document.sampler.take();
        let source = match document.source.take() {
            Some(source) => source,
            None => {
                if native_sampler.is_none() || document.width == 0 || document.height == 0 {
                    return Err(format!(
                        "native HNPC texture {:?} needs dimensions and sampler",
                        document.true_name
                    ));
                }
                // This legacy-shaped internal adapter carries only native ownership.
                // No serialized Unity identity or source build is required by v2.
                HnpcTextureSourceDocument {
                    alias: "native".to_owned(),
                    bundle: document.path.clone(),
                    serialized_asset: document.true_name.clone(),
                    path_id: 0,
                    container_route: document.path.clone(),
                    width: document.width,
                    height: document.height,
                    texture_format: 4,
                    complete_image_size: u64::from(document.width)
                        .checked_mul(u64::from(document.height))
                        .and_then(|pixels| pixels.checked_mul(4))
                        .ok_or_else(|| {
                            format!(
                                "native HNPC texture {:?} byte size overflows",
                                document.true_name
                            )
                        })?,
                    source_chain_sha256: document.sha256.clone(),
                    mip_map: false,
                    source_mip_count: 1,
                    image_count: 1,
                    texture_dimension: 2,
                    filter_mode: 1,
                    wrap_mode: 0,
                    anisotropy_level: 1,
                }
            }
        };
        if document.true_name.is_empty()
            || !matches!(source.alias.as_str(), "primary" | "native")
            || source.bundle.is_empty()
            || source.serialized_asset.is_empty()
            || source.container_route.is_empty()
            || source.source_mip_count == 0
            || Path::new(&document.path)
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("png")
        {
            return Err(format!(
                "HNPC runtime texture {:?} has incomplete primary ownership",
                document.true_name
            ));
        }
        let bytes = locator.read(&document.path)?;
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        if bytes.len() as u64 != document.bytes || sha256 != document.sha256 {
            return Err(format!(
                "HNPC runtime texture {:?} failed byte/SHA-256 verification",
                document.path
            ));
        }
        let key = document.true_name.to_ascii_lowercase();
        let mag_filter = if source.filter_mode == 0 {
            SamplerMagFilter::Nearest
        } else {
            SamplerMagFilter::Linear
        };
        let min_filter = match (source.filter_mode, source.mip_map) {
            (0, true) => SamplerMinFilter::NearestMipmapNearest,
            (1, true) => SamplerMinFilter::LinearMipmapNearest,
            (2, true) => SamplerMinFilter::LinearMipmapLinear,
            (0, false) => SamplerMinFilter::Nearest,
            _ => SamplerMinFilter::Linear,
        };
        let wrap = if source.wrap_mode == 1 {
            SamplerWrapMode::ClampToEdge
        } else {
            SamplerWrapMode::Repeat
        };
        let mut contract = CharacterRuntimeTextureContract {
            true_name: document.true_name.clone(),
            native_asset: CharacterCreationAssetReference {
                path: document.path.clone(),
                bytes: document.bytes,
                blake3: blake3::hash(&bytes).to_hex().to_string(),
            },
            native_png_sha256: document.sha256,
            source: CharacterRuntimeTextureSource {
                asset: format!("{}/{}", source.bundle, source.serialized_asset),
                container_route: source.container_route,
                path_id: source.path_id,
                width: source.width,
                height: source.height,
                texture_format: source.texture_format,
                texture_format_name: format!("legacy-format-{}", source.texture_format),
                complete_image_size: source.complete_image_size,
                source_chain_sha256: source.source_chain_sha256,
                mip_map: false,
                source_mip_count: 1,
                image_count: source.image_count,
                texture_dimension: source.texture_dimension,
            },
            usage_color_space: TextureColorSpace::Srgb,
            usage_color_space_source: "all_hnpc ActorSkinCombiner _MainTex override".to_owned(),
            sampler: NativeSampler {
                name: document.true_name,
                mag_filter,
                min_filter,
                wrap_s: wrap,
                wrap_t: wrap,
                legacy_filter_mode: source.filter_mode,
                legacy_wrap_mode: source.wrap_mode,
                anisotropy_level: source.anisotropy_level,
                mip_map_bias: 0.0,
            },
            published_mip_policy: PublishedMipPolicy::BaseLevelOnly,
        };
        if let Some(sampler) = native_sampler {
            contract.sampler = sampler;
        }
        let texture = NativePlayerTexture {
            path: document.path,
            contract,
        };
        if textures.insert(key.clone(), texture).is_some() {
            return Err(format!("HNPC runtime texture true name repeats: {key}"));
        }
    }
    if textures.is_empty() {
        return Err(format!(
            "production HNPC catalog has {} textures, expected a non-empty set",
            textures.len()
        ));
    }
    Ok(textures)
}

fn resolve_texture(
    textures: &BTreeMap<String, NativePlayerTexture>,
    name: &str,
) -> Result<NativePlayerTexture, String> {
    textures
        .get(&name.to_ascii_lowercase())
        .cloned()
        .ok_or_else(|| format!("HNPC texture {name:?} has no production contract"))
}

fn load_shared_skin_textures(
    locator: &AssetLocator,
    textures: &mut BTreeMap<String, NativePlayerTexture>,
) -> Result<(), String> {
    let document: CharacterCreationRuntimeTextures =
        locator.read_json(CHARACTER_CREATION_RUNTIME_TEXTURES_PATH)?;
    if document.schema != CHARACTER_CREATION_RUNTIME_TEXTURES_SCHEMA {
        return Err(format!(
            "HNPC shared skin texture source has schema {:?}, expected {:?}",
            document.schema, CHARACTER_CREATION_RUNTIME_TEXTURES_SCHEMA
        ));
    }
    for true_name in ["m_skin", "f_skin"] {
        let matches = document
            .textures
            .iter()
            .filter(|contract| contract.true_name.eq_ignore_ascii_case(true_name))
            .collect::<Vec<_>>();
        let [contract] = matches.as_slice() else {
            return Err(format!(
                "character-creation runtime textures resolve {true_name:?} {} times",
                matches.len()
            ));
        };
        let bytes = locator.read_verified(
            &contract.native_asset.path,
            Some(contract.native_asset.bytes),
            &contract.native_asset.blake3,
        )?;
        if format!("{:x}", Sha256::digest(&bytes)) != contract.native_png_sha256 {
            return Err(format!(
                "shared HNPC skin texture {true_name:?} failed SHA-256 verification"
            ));
        }
        textures.insert(
            true_name.to_owned(),
            NativePlayerTexture {
                path: contract.native_asset.path.clone(),
                contract: (*contract).clone(),
            },
        );
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HnpcPalette {
    schema: String,
    skin: Vec<[f32; 4]>,
    hair: Vec<[f32; 4]>,
}

fn palette_color(palette: &[[f32; 4]], selector: i32) -> LinearRgba {
    if selector < 0 {
        return LinearRgba::WHITE;
    }
    // HNPC selectors index the complete native palette from zero. Player
    // creation codes are one-based and expose only a subset of these colors.
    palette
        .get(selector as usize)
        .map_or(LinearRgba::WHITE, |rgba| {
            LinearRgba::new(rgba[0], rgba[1], rgba[2], rgba[3])
        })
}

#[cfg(test)]
mod tests;
