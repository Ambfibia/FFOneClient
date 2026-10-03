use ffone_skinned_model::{NativeSampler, PublishedMipPolicy, TextureColorSpace};
use serde::{Deserialize, Serialize};

pub const CHARACTER_CREATION_ROOT: &str = "data/character_creation";
pub const CHARACTER_CREATION_NAME_WHEEL_PATH: &str = "data/character_creation/name_wheel.json";
pub const CHARACTER_CREATION_APPEARANCE_PATH: &str = "data/character_creation/appearance.json";
pub const CHARACTER_CREATION_AVATAR_ITEMS_PATH: &str = "data/character_creation/avatar_items.json";
pub const CHARACTER_CREATION_RUNTIME_TEXTURES_PATH: &str =
    "data/character_creation/runtime_textures.json";
pub const CHARACTER_CREATION_NAME_WHEEL_SCHEMA: &str = "ffone.character-creation.name-wheel.v1";
pub const CHARACTER_CREATION_APPEARANCE_SCHEMA: &str = "ffone.character-creation.appearance.v1";
pub const CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA: &str = "ffone.character-creation.avatar-items.v1";
pub const CHARACTER_CREATION_RUNTIME_TEXTURES_SCHEMA: &str =
    "ffone.character-creation.runtime-textures.v2";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationProvenance {
    pub source_build: String,
    pub table_set: CharacterCreationAssetReference,
    pub player_equipment_catalog: CharacterCreationAssetReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationAssetReference {
    pub path: String,
    pub bytes: u64,
    pub blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationNameWheel {
    pub schema: String,
    pub protocol: u16,
    pub provenance: CharacterCreationProvenance,
    pub first_names: Vec<NameWheelEntry>,
    pub middle_names: Vec<NameWheelEntry>,
    pub last_names: Vec<NameWheelEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NameWheelEntry {
    pub code: u16,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterGender {
    Male,
    Female,
}

impl CharacterGender {
    pub const fn protocol_code(self) -> u8 {
        match self {
            Self::Male => 1,
            Self::Female => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterAppearanceCategory {
    Face,
    Hair,
    Shirt,
    Pants,
    Shoes,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationAppearance {
    pub schema: String,
    pub protocol: u16,
    pub provenance: CharacterCreationProvenance,
    pub constraints: CharacterAppearanceConstraints,
    pub color_contract: CharacterColorContract,
    pub texture_rules: CharacterTextureRules,
    pub maxima: CharacterCreationMaxima,
    pub creation_rows: Vec<CharacterCreationRow>,
    pub choices: Vec<CharacterCreationChoice>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterAppearanceConstraints {
    pub gender_codes: Vec<u8>,
    pub body_codes: Vec<u8>,
    pub height_codes: Vec<u8>,
    pub skin_color_codes: Vec<u8>,
    pub hair_color_codes: Vec<u8>,
    pub eye_color_codes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterColorContract {
    pub source_asset: String,
    pub source_path_id: i64,
    pub serialized_color_space: String,
    pub runtime_uniform_policy: String,
    pub actor_skin_tint_multiplier: f32,
    pub skin: Vec<CharacterPaletteColor>,
    pub hair: Vec<CharacterPaletteColor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterPaletteColor {
    pub code: u8,
    pub rgba: [f32; 4],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterTextureRules {
    pub face_eye_suffix_by_code: Vec<CharacterTextureSuffix>,
    pub hair_eye_suffix: String,
    pub male_skin_texture: AvatarTextureReference,
    pub female_skin_texture: AvatarTextureReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterTextureSuffix {
    pub code: u8,
    pub suffix: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterRuntimeTextureContract {
    pub true_name: String,
    pub native_asset: CharacterCreationAssetReference,
    pub native_png_sha256: String,
    pub source: CharacterRuntimeTextureSource,
    pub usage_color_space: TextureColorSpace,
    pub usage_color_space_source: String,
    pub sampler: NativeSampler,
    pub published_mip_policy: PublishedMipPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterRuntimeTextureSource {
    pub asset: String,
    pub container_route: String,
    pub path_id: i64,
    pub width: u32,
    pub height: u32,
    pub texture_format: i32,
    pub texture_format_name: String,
    pub complete_image_size: u64,
    pub source_chain_sha256: String,
    pub mip_map: bool,
    pub source_mip_count: u32,
    pub image_count: u32,
    pub texture_dimension: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationRuntimeTextures {
    pub schema: String,
    pub protocol: u16,
    pub provenance: CharacterCreationProvenance,
    pub coverage: CharacterRuntimeTextureCoverage,
    pub textures: Vec<CharacterRuntimeTextureContract>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterRuntimeTextureCoverage {
    pub creator_choices: u64,
    pub creator_required_textures: u64,
    pub creator_published_textures: u64,
    pub avatar_texture_references: u64,
    pub avatar_verified_unique_routes: u64,
    pub avatar_published_routes: u64,
    pub avatar_deferred_verified_routes: u64,
    pub avatar_missing_true_names: Vec<String>,
    pub avatar_ambiguous_true_names: Vec<String>,
    pub avatar_missing_source_metadata: Vec<String>,
    pub source_metadata_textures: u64,
    pub source_metadata_unreadable: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationMaxima {
    pub male_face: u16,
    pub female_face: u16,
    pub male_hair: u16,
    pub female_hair: u16,
    pub male_shirts: u16,
    pub female_shirts: u16,
    pub male_pants: u16,
    pub female_pants: u16,
    pub male_shoes: u16,
    pub female_shoes: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationRow {
    pub creation_index: u16,
    pub male_face: u32,
    pub female_face: u32,
    pub male_hair: u32,
    pub female_hair: u32,
    pub male_shirt: u32,
    pub female_shirt: u32,
    pub male_pants: u32,
    pub female_pants: u32,
    pub male_shoes: u32,
    pub female_shoes: u32,
    pub weapon: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationChoice {
    pub category: CharacterAppearanceCategory,
    pub gender: CharacterGender,
    pub creation_index: u16,
    pub value: u32,
    pub label: String,
    pub icon: Option<AvatarIconReference>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvatarItemCategory {
    Back,
    Glasses,
    Hat,
    Head,
    Face,
    Pants,
    Shirt,
    Shoes,
    Vehicle,
    Weapon,
}

impl AvatarItemCategory {
    pub const fn table_name(self) -> &'static str {
        match self {
            Self::Back => "m_pBackItemTable",
            Self::Glasses => "m_pGlassItemTable",
            Self::Hat => "m_pHatItemTable",
            Self::Head => "m_pHeadItemTable",
            Self::Face => "m_pFaceItemTable",
            Self::Pants => "m_pPantsItemTable",
            Self::Shirt => "m_pShirtsItemTable",
            Self::Shoes => "m_pShoesItemTable",
            Self::Vehicle => "m_pVehicleItemTable",
            Self::Weapon => "m_pWeaponItemTable",
        }
    }

    pub const fn equipment_category(self) -> &'static str {
        match self {
            Self::Face => "mask",
            Self::Back => "back",
            Self::Glasses => "glasses",
            Self::Hat => "hat",
            Self::Head => "head",
            Self::Pants => "pants",
            Self::Shirt => "shirt",
            Self::Shoes => "shoes",
            Self::Vehicle => "vehicle",
            Self::Weapon => "weapon",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationAvatarItems {
    pub schema: String,
    pub protocol: u16,
    pub provenance: CharacterCreationProvenance,
    pub lookup_complete: bool,
    pub counts: CharacterCreationAvatarItemCounts,
    pub items: Vec<AvatarItemLookup>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterCreationAvatarItemCounts {
    pub categories: u64,
    pub items: u64,
    pub model_references: u64,
    pub resolved_models: u64,
    pub texture_references: u64,
    pub resolved_textures: u64,
    pub icon_references: u64,
    pub resolved_icons: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvatarItemLookup {
    pub category: AvatarItemCategory,
    pub item_number: u32,
    pub level: u16,
    pub required_gender: u8,
    /// Legacy `m_iEquipType`. Hats use values 0..=5 to select the exact
    /// hair/face/glasses visibility policy; other categories normally use 0.
    #[serde(default)]
    pub equip_type: u8,
    pub name: String,
    pub description: String,
    pub icon: Option<AvatarIconReference>,
    pub male: AvatarItemVisual,
    pub female: AvatarItemVisual,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvatarItemVisual {
    pub source_model_true_name: Option<String>,
    pub model_status: NativeLookupStatus,
    pub models: Vec<AvatarModelReference>,
    pub primary_texture: Option<AvatarTextureReference>,
    pub secondary_texture: Option<AvatarTextureReference>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeLookupStatus {
    VerifiedUnique,
    VerifiedVariants,
    Missing,
    Ambiguous,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvatarModelReference {
    pub true_name: String,
    pub exact_route: String,
    pub native_asset: CharacterCreationAssetReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvatarTextureReference {
    pub true_name: String,
    pub status: NativeLookupStatus,
    pub candidates: Vec<CharacterCreationAssetReference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvatarIconReference {
    pub icon_type: u8,
    pub icon_number: u32,
    pub true_name: String,
    pub status: NativeLookupStatus,
    pub candidates: Vec<CharacterCreationAssetReference>,
}
