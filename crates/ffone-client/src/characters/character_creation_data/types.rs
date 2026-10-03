use super::*;

pub type CharacterCreationDataResult<T> = Result<T, CharacterCreationDataError>;

/// Exact `SetHat` visibility contract recovered from the primary 0104 client.
/// `face_variant`/`hair_variant` select the serialized `_type01`/`_type02`
/// models; `None` means the hair renderer is removed entirely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LegacyHatPolicy {
    pub(super) face_variant: u8,
    pub(super) hair_variant: Option<u8>,
    pub(super) glasses_visible: bool,
}

impl Default for LegacyHatPolicy {
    fn default() -> Self {
        Self {
            face_variant: 1,
            hair_variant: Some(1),
            glasses_visible: true,
        }
    }
}

impl LegacyHatPolicy {
    pub(super) fn from_equip_type(equip_type: u8) -> CharacterCreationDataResult<Self> {
        match equip_type {
            0 => Ok(Self::default()),
            1 => Ok(Self {
                hair_variant: Some(2),
                ..Self::default()
            }),
            2 => Ok(Self {
                face_variant: 2,
                hair_variant: None,
                glasses_visible: true,
            }),
            3 => Ok(Self {
                face_variant: 2,
                hair_variant: None,
                glasses_visible: false,
            }),
            4 => Ok(Self {
                glasses_visible: false,
                ..Self::default()
            }),
            5 => Ok(Self {
                hair_variant: Some(2),
                glasses_visible: false,
                ..Self::default()
            }),
            _ => invalid(format!(
                "hat equip type {equip_type} is outside the primary client contract 0..=5"
            )),
        }
    }
}

/// Immutable, contract-verified native data for selection and creation.
pub struct CharacterCreationData {
    pub(super) asset_root: PathBuf,
    pub(super) locator: AssetLocator,
    pub(super) names: CharacterCreationNameWheel,
    pub(super) appearance: CharacterCreationAppearance,
    pub(super) customization: crate::character_customization::CharacterCustomization,
    pub(super) avatar_items: CharacterCreationAvatarItems,
    pub(super) runtime_textures: CharacterCreationRuntimeTextures,
    pub(super) choices: BTreeMap<(DataGender, CharacterAppearanceCategory, u16), CharacterCreationChoice>,
    pub(super) items: BTreeMap<(AvatarItemCategory, u32), AvatarItemLookup>,
    pub(super) creator_model_fallbacks: BTreeMap<(DataGender, AvatarItemCategory, u32), AvatarModelReference>,
    pub(super) item_models: BTreeMap<(String, String), Vec<CharacterCreationAssetReference>>,
    pub(super) textures_by_stem: BTreeMap<String, Vec<String>>,
    pub(super) runtime_texture_contracts: BTreeMap<String, CharacterRuntimeTextureContract>,
    pub(super) vehicles: BTreeMap<u32, NativePlayerPartLook>,
}

/// Shared immutable runtime owner used by gameplay consumers in addition to
/// character selection/creation. The underlying catalogs are expensive to
/// validate, so ordinary-world remote players reuse the startup instance.
#[derive(Clone, Resource)]
pub struct CharacterCreationDataResource(pub Arc<CharacterCreationData>);

impl CharacterCreationData {
    pub fn open(asset_root: impl AsRef<Path>) -> CharacterCreationDataResult<Self> {
        let asset_root = asset_root.as_ref().to_path_buf();
        let locator =
            AssetLocator::open(&asset_root).map_err(CharacterCreationDataError::Invalid)?;
        let names: CharacterCreationNameWheel =
            read_json(&asset_root.join(CHARACTER_CREATION_NAME_WHEEL_PATH))?;
        let mut appearance: CharacterCreationAppearance =
            read_json(&asset_root.join(CHARACTER_CREATION_APPEARANCE_PATH))?;
        let avatar_items: CharacterCreationAvatarItems =
            read_json(&asset_root.join(CHARACTER_CREATION_AVATAR_ITEMS_PATH))?;
        let mut runtime_textures: CharacterCreationRuntimeTextures =
            read_json(&asset_root.join(CHARACTER_CREATION_RUNTIME_TEXTURES_PATH))?;
        validate_documents(&names, &appearance, &avatar_items, &runtime_textures)?;
        let customization: crate::character_customization::CharacterCustomization =
            read_json(&asset_root.join(crate::character_customization::CUSTOMIZATION_PATH))?;
        customization
            .validate()
            .map_err(CharacterCreationDataError::Invalid)?;
        appearance.color_contract.skin =
            crate::character_customization::native_palette(&customization.skin);
        appearance.color_contract.hair =
            crate::character_customization::native_palette(&customization.hair);
        appearance.constraints.skin_color_codes =
            (1..=customization.creation_skin_count as u8).collect();
        appearance.constraints.hair_color_codes = (1..=customization.hair.len() as u8).collect();
        appearance.constraints.eye_color_codes = (1..=customization.eye.len() as u8).collect();
        appearance.texture_rules.face_eye_suffix_by_code = (1..=customization.eye.len() as u8)
            .map(|code| ffone_runtime_contracts::CharacterTextureSuffix {
                code,
                suffix: char::from(b'a' + code - 1).to_string(),
            })
            .collect();
        runtime_textures.textures.extend(
            customization
                .textures
                .iter()
                .map(|texture| texture.runtime_contract()),
        );

        let player_item_proof = &avatar_items.provenance.player_equipment_catalog;
        for proof in [
            &names.provenance.player_equipment_catalog,
            &appearance.provenance.player_equipment_catalog,
            &runtime_textures.provenance.player_equipment_catalog,
        ] {
            if proof != player_item_proof {
                return invalid("character-creation documents disagree on the player item catalog");
            }
        }
        let player_item_catalog_path = normalize_relative_path(&player_item_proof.path)?;
        if player_item_catalog_path != PLAYER_ITEM_SET_CATALOG_PATH {
            return invalid(format!(
                "character-creation data references obsolete player item catalog {}",
                player_item_proof.path
            ));
        }
        let player_item_catalog_bytes = locator
            .read_verified(
                &player_item_catalog_path,
                Some(player_item_proof.bytes),
                &player_item_proof.blake3,
            )
            .map_err(CharacterCreationDataError::Invalid)?;
        let player_item_catalog: PlayerItemRouteCatalog =
            serde_json::from_slice(&player_item_catalog_bytes).map_err(|source| {
                CharacterCreationDataError::Json {
                    path: asset_root.join(&player_item_catalog_path),
                    source,
                }
            })?;
        if player_item_catalog.schema != PLAYER_ITEM_SET_CATALOG_SCHEMA
            || player_item_catalog.models.is_empty()
        {
            return invalid("player item route catalog has an invalid identity");
        }
        let mut item_models = BTreeMap::new();
        for route in player_item_catalog.models {
            if route.source_route.is_empty() || route.resource_set.is_empty() {
                return invalid(format!(
                    "player item route {}/{} has incomplete ownership",
                    route.category, route.true_name
                ));
            }
            let path = normalize_relative_path(&route.model.path)?;
            if Path::new(&path)
                .extension()
                .and_then(|value| value.to_str())
                != Some("glb")
            {
                return invalid(format!("player item model route is not a GLB: {path}"));
            }
            let key = (
                route.category.to_ascii_lowercase(),
                route.true_name.to_ascii_lowercase(),
            );
            let model = CharacterCreationAssetReference {
                path,
                bytes: route.model.bytes,
                blake3: route.model.blake3,
            };
            item_models.entry(key).or_insert_with(Vec::new).push(model);
        }
        for models in item_models.values_mut() {
            models.sort_by(|left, right| left.path.cmp(&right.path));
            models.dedup();
        }

        let mut choices = BTreeMap::new();
        for choice in &appearance.choices {
            let key = (choice.gender, choice.category, choice.creation_index);
            if choices.insert(key, choice.clone()).is_some() {
                return invalid(format!(
                    "appearance catalog repeats {:?}/{:?}/{}",
                    choice.gender, choice.category, choice.creation_index
                ));
            }
        }

        let mut items = BTreeMap::new();
        for item in &avatar_items.items {
            let key = (item.category, item.item_number);
            if items.insert(key, item.clone()).is_some() {
                return invalid(format!(
                    "avatar item catalog repeats {:?}/{}",
                    item.category, item.item_number
                ));
            }
        }

        let rig_catalog = NativePlayerRigCatalog::open(&asset_root).map_err(|error| {
            CharacterCreationDataError::Invalid(format!(
                "native shared-rig creator contract is invalid: {error}"
            ))
        })?;
        let mut creator_model_fallbacks = BTreeMap::new();
        for gender_contract in &rig_catalog.contract().genders {
            let gender = match gender_contract.gender {
                PlayerRigGender::Male => DataGender::Male,
                PlayerRigGender::Female => DataGender::Female,
            };
            for choice in &gender_contract.creator_choices {
                let category = match choice.appearance_category {
                    CharacterAppearanceCategory::Face => AvatarItemCategory::Face,
                    CharacterAppearanceCategory::Hair => AvatarItemCategory::Head,
                    CharacterAppearanceCategory::Shirt => AvatarItemCategory::Shirt,
                    CharacterAppearanceCategory::Pants => AvatarItemCategory::Pants,
                    CharacterAppearanceCategory::Shoes => AvatarItemCategory::Shoes,
                };
                let part = rig_catalog
                    .part_by_exact_route(gender_contract.gender, &choice.exact_route)
                    .map_err(CharacterCreationDataError::Invalid)?;
                if part.glb != choice.glb {
                    return invalid(format!(
                        "{:?}/{:?}/{} shared-rig choice GLB differs from its exact-route contract",
                        gender, choice.appearance_category, choice.creation_index
                    ));
                }
                let bytes = locator
                    .read(&part.glb)
                    .map_err(CharacterCreationDataError::MissingRoute)?;
                let reference = AvatarModelReference {
                    true_name: part.true_name.clone(),
                    exact_route: part.exact_route.clone(),
                    native_asset: CharacterCreationAssetReference {
                        path: part.glb.clone(),
                        bytes: bytes.len() as u64,
                        blake3: blake3::hash(&bytes).to_hex().to_string(),
                    },
                };
                let key = (gender, category, choice.item_number);
                if let Some(previous) = creator_model_fallbacks.insert(key, reference.clone())
                    && previous != reference
                {
                    return invalid(format!(
                        "{gender:?}/{category:?}/{} resolves inconsistent shared-rig models",
                        choice.item_number
                    ));
                }
            }
        }

        let mut textures_by_stem = BTreeMap::<String, Vec<String>>::new();
        let mut runtime_texture_contracts = BTreeMap::new();
        for contract in &runtime_textures.textures {
            validate_runtime_texture_contract(&locator, contract)?;
            let path = normalize_relative_path(&contract.native_asset.path)?;
            textures_by_stem
                .entry(strip_native_collision_suffix(&contract.true_name).to_ascii_lowercase())
                .or_default()
                .push(path.clone());
            if let Some(previous) = runtime_texture_contracts.get(&path) {
                if !compatible_shared_texture_contract(previous, contract) {
                    return invalid(format!(
                        "shared runtime texture route {path} has incompatible sampler or image proof"
                    ));
                }
            } else {
                runtime_texture_contracts.insert(path.clone(), contract.clone());
            }
        }
        for paths in textures_by_stem.values_mut() {
            paths.sort();
            paths.dedup();
        }

        let mut data = Self {
            asset_root,
            locator,
            names,
            appearance,
            customization,
            avatar_items,
            runtime_textures,
            choices,
            items,
            creator_model_fallbacks,
            item_models,
            textures_by_stem,
            runtime_texture_contracts,
            vehicles: BTreeMap::new(),
        };
        data.load_personal_vehicles()?;
        Ok(data)
    }

    pub fn asset_root(&self) -> &Path {
        &self.asset_root
    }

    pub fn names_document(&self) -> &CharacterCreationNameWheel {
        &self.names
    }

    pub fn appearance_document(&self) -> &CharacterCreationAppearance {
        &self.appearance
    }

    pub fn avatar_items_document(&self) -> &CharacterCreationAvatarItems {
        &self.avatar_items
    }

    pub fn runtime_textures_document(&self) -> &CharacterCreationRuntimeTextures {
        &self.runtime_textures
    }

    pub fn name_lists(&self) -> CharacterNameLists {
        CharacterNameLists {
            first: self
                .names
                .first_names
                .iter()
                .map(|entry| entry.value.clone())
                .collect(),
            middle: self
                .names
                .middle_names
                .iter()
                .map(|entry| entry.value.clone())
                .collect(),
            last: self
                .names
                .last_names
                .iter()
                .map(|entry| entry.value.clone())
                .collect(),
        }
    }

    pub fn option_counts(&self) -> CharacterCreationDataResult<CharacterCreationOptionCounts> {
        let maxima = &self.appearance.maxima;
        Ok(CharacterCreationOptionCounts {
            male_hair: to_u8("male hair maximum", maxima.male_hair)?,
            female_hair: to_u8("female hair maximum", maxima.female_hair)?,
            male_face: to_u8("male face maximum", maxima.male_face)?,
            female_face: to_u8("female face maximum", maxima.female_face)?,
            male_shirt: to_u8("male shirt maximum", maxima.male_shirts)?,
            female_shirt: to_u8("female shirt maximum", maxima.female_shirts)?,
            male_pants: to_u8("male pants maximum", maxima.male_pants)?,
            female_pants: to_u8("female pants maximum", maxima.female_pants)?,
            male_shoes: to_u8("male shoes maximum", maxima.male_shoes)?,
            female_shoes: to_u8("female shoes maximum", maxima.female_shoes)?,
        })
    }

    /// Exact five-icon carousel used by `CnGuiCharCreation`: two previous
    /// choices, the selected choice, and two following choices with wrapping.
    pub fn clothing_icon_window(
        &self,
        appearance: &CharacterAppearance,
        field: AppearanceField,
    ) -> CharacterCreationDataResult<[String; 5]> {
        let category = match field {
            AppearanceField::Shirt => CharacterAppearanceCategory::Shirt,
            AppearanceField::Pants => CharacterAppearanceCategory::Pants,
            AppearanceField::Shoes => CharacterAppearanceCategory::Shoes,
            _ => {
                return invalid(format!(
                    "{field:?} does not have a creator clothing-icon carousel"
                ));
            }
        };
        let counts = self.option_counts()?;
        let maximum = counts.count(appearance.gender, field) + 1;
        let minimum = 2_u8;
        let span = i16::from(maximum - minimum + 1);
        let center = appearance.selector(field);
        if !(minimum..=maximum).contains(&center) {
            return invalid(format!(
                "{field:?} selector {center} is outside {minimum}..={maximum}"
            ));
        }
        let gender = data_gender(appearance.gender);
        let mut routes = Vec::with_capacity(5);
        for offset in -2_i16..=2 {
            let selector =
                minimum + (i16::from(center) - i16::from(minimum) + offset).rem_euclid(span) as u8;
            let choice = self.choice(gender, category, selector)?;
            let icon = choice.icon.as_ref().ok_or_else(|| {
                CharacterCreationDataError::MissingRoute(format!(
                    "{gender:?}/{category:?}/{selector} has no native icon"
                ))
            })?;
            if icon.status != NativeLookupStatus::VerifiedUnique || icon.candidates.len() != 1 {
                return Err(CharacterCreationDataError::MissingRoute(format!(
                    "icon {} route is {:?} with {} candidates",
                    icon.true_name,
                    icon.status,
                    icon.candidates.len()
                )));
            }
            let reference = &icon.candidates[0];
            self.validate_reference(reference, NativeAssetKind::Texture)?;
            routes.push(reference.path.clone());
        }
        routes.try_into().map_err(|routes: Vec<String>| {
            CharacterCreationDataError::Invalid(format!(
                "creator icon window has {} entries instead of 5",
                routes.len()
            ))
        })
    }

    pub fn appearance_label(
        &self,
        appearance: &CharacterAppearance,
        field: AppearanceField,
    ) -> CharacterCreationDataResult<String> {
        let category = match field {
            AppearanceField::Hair => CharacterAppearanceCategory::Hair,
            AppearanceField::Face => CharacterAppearanceCategory::Face,
            _ => {
                return invalid(format!(
                    "{field:?} does not have a table-driven appearance label"
                ));
            }
        };
        Ok(self
            .choice(
                data_gender(appearance.gender),
                category,
                appearance.selector(field),
            )?
            .label
            .clone())
    }

    pub fn compose_generated_name(
        &self,
        first_index: usize,
        middle_index: usize,
        last_index: usize,
    ) -> CharacterCreationDataResult<GeneratedCharacterName> {
        let first = self
            .names
            .first_names
            .get(first_index)
            .ok_or_else(|| {
                CharacterCreationDataError::Invalid(format!(
                    "first-name code {first_index} is outside the Retrobution table"
                ))
            })?
            .value
            .clone();
        let middle = &self
            .names
            .middle_names
            .get(middle_index)
            .ok_or_else(|| {
                CharacterCreationDataError::Invalid(format!(
                    "middle-name code {middle_index} is outside the Retrobution table"
                ))
            })?
            .value;
        let last = &self
            .names
            .last_names
            .get(last_index)
            .ok_or_else(|| {
                CharacterCreationDataError::Invalid(format!(
                    "last-name code {last_index} is outside the Retrobution table"
                ))
            })?
            .value;
        if first.is_empty() {
            return invalid("generated first name is empty");
        }
        let last = compose_legacy_last_name(middle, last);
        if last.is_empty() {
            return invalid("generated last name is empty");
        }
        Ok(GeneratedCharacterName {
            first,
            last,
            first_index,
            middle_index,
            last_index,
        })
    }

    pub fn ui_palettes(&self) -> [Vec<bevy::prelude::Color>; 3] {
        self.customization.ui_palettes()
    }

    pub fn skin_color(&self, code: u8) -> CharacterCreationDataResult<LinearRgba> {
        palette_color("skin", code, &self.appearance.color_contract.skin)
    }

    pub fn hair_color(&self, code: u8) -> CharacterCreationDataResult<LinearRgba> {
        palette_color("hair", code, &self.appearance.color_contract.hair)
    }

    pub fn resolve_creator(
        &self,
        pc_uid: i64,
        name_check: i8,
        first_name: &str,
        last_name: &str,
        appearance: &CharacterAppearance,
    ) -> CharacterCreationDataResult<ResolvedCreatorSelection> {
        self.validate_appearance(appearance)?;
        let gender = data_gender(appearance.gender);
        let face = self.choice(gender, CharacterAppearanceCategory::Face, appearance.face)?;
        let hair = self.choice(gender, CharacterAppearanceCategory::Hair, appearance.hair)?;
        let shirt = self.choice(gender, CharacterAppearanceCategory::Shirt, appearance.shirt)?;
        let pants = self.choice(gender, CharacterAppearanceCategory::Pants, appearance.pants)?;
        let shoes = self.choice(gender, CharacterAppearanceCategory::Shoes, appearance.shoes)?;

        let style = PcStyle0104 {
            pc_uid,
            name_check,
            first_name: FixedUtf16::from_str(first_name).map_err(|error| {
                CharacterCreationDataError::Invalid(format!(
                    "creator first name is not protocol-safe: {error:?}"
                ))
            })?,
            last_name: FixedUtf16::from_str(last_name).map_err(|error| {
                CharacterCreationDataError::Invalid(format!(
                    "creator last name is not protocol-safe: {error:?}"
                ))
            })?,
            gender: appearance.gender as i8,
            face_style: to_i8("face style", face.value)?,
            hair_style: to_i8("hair style", hair.value)?,
            hair_color: appearance.hair_color as i8,
            skin_color: appearance.skin_color as i8,
            eye_color: appearance.eye_color as i8,
            height: appearance.height as i8,
            body: appearance.body as i8,
            class: 0,
        };
        let equipped = OnItem0104 {
            upper_body_id: to_i16("shirt item", shirt.value)?,
            lower_body_id: to_i16("pants item", pants.value)?,
            foot_id: to_i16("shoes item", shoes.value)?,
            ..Default::default()
        };
        let selected_indices = OnItemIndex0104 {
            upper_body_index: appearance.shirt as i16,
            lower_body_index: appearance.pants as i16,
            foot_index: appearance.shoes as i16,
            face_style_index: appearance.face as i16,
            hair_style_index: appearance.hair as i16,
        };
        let look = self.resolve_look(
            format!("{first_name} {last_name}").trim().to_owned(),
            style.gender,
            style.face_style,
            style.hair_style,
            style.hair_color,
            style.skin_color,
            style.eye_color,
            style.height,
            style.body,
            LegacyHatPolicy::default(),
            &[
                (
                    AvatarItemCategory::Shirt,
                    shirt.value,
                    NativePlayerPartKind::Shirt,
                ),
                (
                    AvatarItemCategory::Pants,
                    pants.value,
                    NativePlayerPartKind::Pants,
                ),
                (
                    AvatarItemCategory::Shoes,
                    shoes.value,
                    NativePlayerPartKind::Shoes,
                ),
            ],
        )?;
        Ok(ResolvedCreatorSelection {
            style,
            equipped,
            selected_indices,
            look,
        })
    }

    pub fn resolve_try_on_character(
        &self,
        character: &CharacterSummary,
        item: ffone_protocol::ItemBase0104,
    ) -> CharacterCreationDataResult<NativePlayerLook> {
        if !(0..=6).contains(&item.item_type) {
            return Err(CharacterCreationDataError::MissingRoute(
                "unsupported try-on item type".into(),
            ));
        }
        let mut draft = character.clone();
        draft.equipment[item.item_type as usize] = EquippedItem0104 {
            item_type: item.item_type,
            item_id: item.item_id,
            option: item.option,
            time_limit: item.time_limit,
        };
        for equipped in &mut draft.equipment {
            let appearance = ((equipped.option as u32 >> 16) & 0xffff) as i16;
            if appearance > 0 {
                equipped.item_id = appearance;
            }
            equipped.option = 0;
        }
        self.resolve_character_summary(&draft)
    }

    pub fn resolve_character_summary(
        &self,
        character: &CharacterSummary,
    ) -> CharacterCreationDataResult<NativePlayerLook> {
        let equipment = |slot: CharacterEquipSlot0104| {
            let item: EquippedItem0104 = character.equipment[slot as usize];
            u32::try_from(item.item_id).unwrap_or_default()
        };
        self.resolve_protocol_player_look(
            format!("character-{}-slot-{}", character.pc_uid, character.slot),
            character.style.gender,
            character.style.face_style,
            character.style.hair_style,
            character.style.hair_color,
            character.style.skin_color,
            character.style.eye_color,
            character.style.height,
            character.style.body,
            equipment,
        )
    }

    /// Resolves a live shard `sPCAppearanceData` through the same validated
    /// shared-rig and equipment contracts as the login character roster.
    pub fn resolve_pc_appearance(
        &self,
        appearance: &PcAppearance0104,
    ) -> CharacterCreationDataResult<NativePlayerLook> {
        let equipment = |slot: CharacterEquipSlot0104| {
            u32::try_from(appearance.equipment[slot as usize].item_id).unwrap_or_default()
        };
        self.resolve_protocol_player_look(
            format!("network-player-{}", appearance.id),
            appearance.style.gender,
            appearance.style.face_style,
            appearance.style.hair_style,
            appearance.style.hair_color,
            appearance.style.skin_color,
            appearance.style.eye_color,
            appearance.style.height,
            appearance.style.body,
            equipment,
        )
    }
}
