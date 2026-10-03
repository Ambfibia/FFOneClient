use super::*;

impl CharacterCreationData {


    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_protocol_player_look(
        &self,
        identity: String,
        gender_code: i8,
        face_style: i8,
        hair_style: i8,
        hair_color: i8,
        skin_color: i8,
        eye_color: i8,
        height: i8,
        body: i8,
        equipment: impl Fn(CharacterEquipSlot0104) -> u32,
    ) -> CharacterCreationDataResult<NativePlayerLook> {
        let gender = protocol_gender(gender_code)?;
        let hat_item_number = equipment(CharacterEquipSlot0104::Head);
        let hat_policy = if hat_item_number == 0 {
            LegacyHatPolicy::default()
        } else {
            let hat = self
                .items
                .get(&(AvatarItemCategory::Hat, hat_item_number))
                .ok_or_else(|| {
                    CharacterCreationDataError::MissingRoute(format!(
                        "Hat item {hat_item_number} is not present in the native avatar catalog"
                    ))
                })?;
            LegacyHatPolicy::from_equip_type(hat.equip_type)?
        };
        // Build the common skinned actor first. Hat/Glasses/Hand and Back
        // equipType=0 entries are rigid `AttachGO` socket models. Back
        // equipType=1 entries are resolved below too, but join the same
        // ActorSkinCombiner rig as the body instead of the `back01` socket.
        let mut parts = Vec::new();
        for (category, value, kind) in [
            (
                AvatarItemCategory::Shirt,
                equipment(CharacterEquipSlot0104::UpperBody),
                NativePlayerPartKind::Shirt,
            ),
            (
                AvatarItemCategory::Pants,
                equipment(CharacterEquipSlot0104::LowerBody),
                NativePlayerPartKind::Pants,
            ),
            (
                AvatarItemCategory::Shoes,
                equipment(CharacterEquipSlot0104::Foot),
                NativePlayerPartKind::Shoes,
            ),
        ] {
            if value != 0 {
                parts.push((category, value, kind));
            }
        }
        let mut look = self.resolve_look(
            identity,
            gender_code,
            face_style,
            hair_style,
            hair_color,
            skin_color,
            eye_color,
            height,
            body,
            hat_policy,
            &parts,
        )?;
        for (slot, kind) in [
            (
                CharacterEquipSlot0104::UpperBody,
                NativePlayerPartKind::Shirt,
            ),
            (
                CharacterEquipSlot0104::LowerBody,
                NativePlayerPartKind::Pants,
            ),
            (CharacterEquipSlot0104::Foot, NativePlayerPartKind::Shoes),
        ] {
            if equipment(slot) == 0 {
                look.parts.push(self.resolve_unequipped_part(gender, kind)?);
            }
        }
        for (slot, category, kind) in [
            (
                CharacterEquipSlot0104::Head,
                AvatarItemCategory::Hat,
                NativePlayerPartKind::Hat,
            ),
            (
                CharacterEquipSlot0104::Face,
                AvatarItemCategory::Glasses,
                NativePlayerPartKind::Glasses,
            ),
            (
                CharacterEquipSlot0104::Back,
                AvatarItemCategory::Back,
                NativePlayerPartKind::Back,
            ),
        ] {
            let item_number = equipment(slot);
            if item_number != 0
                && (slot != CharacterEquipSlot0104::Face || hat_policy.glasses_visible)
            {
                look.parts.push(self.resolve_equipment_part(
                    category,
                    item_number,
                    gender,
                    kind,
                )?);
            }
        }
        let weapon = equipment(CharacterEquipSlot0104::Hand);
        if weapon != 0 {
            let weapon_item = self
                .items
                .get(&(AvatarItemCategory::Weapon, weapon))
                .ok_or_else(|| {
                    CharacterCreationDataError::MissingRoute(format!(
                        "Weapon item {weapon} is not present in the native avatar catalog"
                    ))
                })?;
            look.weapon_animation_profile =
                PlayerWeaponAnimationProfile::from_equip_type(i32::from(weapon_item.equip_type));
            look.parts.push(self.resolve_equipment_part(
                AvatarItemCategory::Weapon,
                weapon,
                gender,
                NativePlayerPartKind::Weapon,
            )?);
        }
        look.parts.sort_by_key(|part| part.kind);
        look.validate()
            .map_err(CharacterCreationDataError::Invalid)?;
        Ok(look)
    }

    /// Resolve one rigid weapon attachment through the same contract-verified
    /// avatar-item route used by selection previews. Tutorial playback uses
    /// this narrow API for its exact item IDs instead of duplicating catalog
    /// lookup rules in the world renderer.
    pub fn resolve_weapon_attachment(
        &self,
        item_number: u32,
        gender: PlayerRigGender,
    ) -> CharacterCreationDataResult<NativePlayerPartLook> {
        let gender = match gender {
            PlayerRigGender::Male => DataGender::Male,
            PlayerRigGender::Female => DataGender::Female,
        };
        self.resolve_equipment_part(
            AvatarItemCategory::Weapon,
            item_number,
            gender,
            NativePlayerPartKind::Weapon,
        )
    }

    /// Vehicles use the common table mesh for both actor genders.
    pub fn resolve_vehicle_attachment(&self, item_number: u32) -> CharacterCreationDataResult<NativePlayerPartLook> {
        self.vehicles.get(&item_number).cloned().ok_or_else(|| CharacterCreationDataError::MissingRoute(format!("vehicle {item_number} has no validated native appearance")))
    }

    pub(super) fn load_personal_vehicles(&mut self) -> CharacterCreationDataResult<()> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Vehicle { item_id: u32, model: String, texture: String }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Catalog { schema: String, vehicles: Vec<Vehicle> }
        let catalog: Catalog = read_json(&self.asset_root.join("characters/player/shared/vehicles.json"))?;
        if catalog.schema != "ffone.personal-vehicle-catalog.v1" || catalog.vehicles.is_empty() {
            return invalid("invalid personal vehicle catalog");
        }
        for entry in catalog.vehicles {
            let model = normalize_relative_path(&entry.model)?;
            if entry.item_id == 0 || !model.ends_with(".glb") || !self.asset_root.join(&model).is_file() {
                return invalid(format!("invalid vehicle {} model", entry.item_id));
            }
            let texture = self.native_player_texture(&normalize_relative_path(&entry.texture)?)?;
            let part = NativePlayerPartLook {
                kind: NativePlayerPartKind::Vehicle,
                assembly: NativePlayerPartAssembly::RigidAttachment,
                exact_route: format!("vehicle/{}", entry.item_id),
                glb: model,
                primary_texture: Some(texture.clone()),
                secondary_texture: Some(texture),
            };
            if self.vehicles.insert(entry.item_id, part).is_some() {
                return invalid(format!("duplicate vehicle {}", entry.item_id));
            }
        }
        Ok(())
    }

    pub(super) fn resolve_unequipped_part(
        &self,
        gender: DataGender,
        kind: NativePlayerPartKind,
    ) -> CharacterCreationDataResult<NativePlayerPartLook> {
        let prefix = match gender {
            DataGender::Male => "m",
            DataGender::Female => "f",
        };
        let category = match kind {
            NativePlayerPartKind::Shirt => "shirt",
            NativePlayerPartKind::Pants => "pants",
            NativePlayerPartKind::Shoes => "shoes",
            _ => {
                return invalid(format!("{kind:?} has no legacy unequipped naked model"));
            }
        };
        let true_name = format!("{prefix}_{category}_naked");
        let models = self
            .item_models
            .get(&(category.to_owned(), true_name.clone()))
            .ok_or_else(|| {
                CharacterCreationDataError::MissingRoute(format!(
                    "player item catalog has no {category}/{true_name} model"
                ))
            })?;
        if models.len() != 1 {
            return invalid(format!(
                "player item catalog resolves {category}/{true_name} to {} models",
                models.len()
            ));
        }
        let model = &models[0];
        self.locator
            .read_verified(&model.path, Some(model.bytes), &model.blake3)
            .map_err(CharacterCreationDataError::MissingRoute)?;
        let glb = model.path.clone();
        let naked_texture = self.unique_texture_stem(&format!("{prefix}_naked"))?;
        Ok(NativePlayerPartLook {
            kind,
            assembly: NativePlayerPartAssembly::SharedSkin,
            exact_route: format!("wear/{true_name}.nif"),
            glb,
            primary_texture: Some(naked_texture.clone()),
            secondary_texture: Some(naked_texture),
        })
    }

    pub(super) fn resolve_equipment_part(
        &self,
        category: AvatarItemCategory,
        item_number: u32,
        gender: DataGender,
        kind: NativePlayerPartKind,
    ) -> CharacterCreationDataResult<NativePlayerPartLook> {
        let item = self.items.get(&(category, item_number)).ok_or_else(|| {
            CharacterCreationDataError::MissingRoute(format!(
                "{category:?} item {item_number} is not present in the native avatar catalog"
            ))
        })?;
        let visual = match gender {
            DataGender::Male => &item.male,
            DataGender::Female => &item.female,
        };
        let model = self.resolve_model(item, visual, gender, None)?;
        self.validate_reference(&model.native_asset, NativeAssetKind::Model)?;
        let primary_texture = visual
            .primary_texture
            .as_ref()
            .map(|texture| self.published_texture_reference(texture))
            .transpose()?;
        let secondary_texture = visual
            .secondary_texture
            .as_ref()
            .map(|texture| self.published_texture_reference(texture))
            .transpose()?;
        let assembly = if category == AvatarItemCategory::Back && item.equip_type == 1 {
            NativePlayerPartAssembly::SharedSkin
        } else {
            NativePlayerPartAssembly::RigidAttachment
        };
        Ok(NativePlayerPartLook {
            kind,
            assembly,
            exact_route: model.exact_route.clone(),
            glb: model.native_asset.path.clone(),
            // Legacy item models deliberately carry white `_MainTex`
            // placeholders. The corresponding SetBody/AttachGO branch fills
            // materials named `main`/`sub` from these table-driven references.
            primary_texture: primary_texture.flatten(),
            secondary_texture: secondary_texture.flatten(),
        })
    }

    pub(super) fn validate_appearance(
        &self,
        appearance: &CharacterAppearance,
    ) -> CharacterCreationDataResult<()> {
        let constraints = &self.appearance.constraints;
        for (label, value, allowed) in [
            (
                "gender",
                appearance.gender as u8,
                constraints.gender_codes.as_slice(),
            ),
            ("body", appearance.body, constraints.body_codes.as_slice()),
            (
                "height",
                appearance.height,
                constraints.height_codes.as_slice(),
            ),
            (
                "skin color",
                appearance.skin_color,
                constraints.skin_color_codes.as_slice(),
            ),
            (
                "hair color",
                appearance.hair_color,
                constraints.hair_color_codes.as_slice(),
            ),
            (
                "eye color",
                appearance.eye_color,
                constraints.eye_color_codes.as_slice(),
            ),
        ] {
            if !allowed.contains(&value) {
                return invalid(format!(
                    "{label} value {value} is outside the exact Retrobution constraint"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn choice(
        &self,
        gender: DataGender,
        category: CharacterAppearanceCategory,
        selector: u8,
    ) -> CharacterCreationDataResult<&CharacterCreationChoice> {
        self.choices
            .get(&(gender, category, u16::from(selector)))
            .ok_or_else(|| {
                CharacterCreationDataError::Invalid(format!(
                    "no exact Retrobution choice for {gender:?}/{category:?}/{selector}"
                ))
            })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_look(
        &self,
        identity: String,
        gender_code: i8,
        face_style: i8,
        hair_style: i8,
        hair_color: i8,
        skin_color: i8,
        eye_color: i8,
        height: i8,
        body: i8,
        hat_policy: LegacyHatPolicy,
        equipped: &[(AvatarItemCategory, u32, NativePlayerPartKind)],
    ) -> CharacterCreationDataResult<NativePlayerLook> {
        let gender = protocol_gender(gender_code)?;
        let eye_color = u8::try_from(eye_color)
            .map_err(|_| CharacterCreationDataError::Invalid("negative eye color".to_owned()))?;
        let mut parts = vec![self.resolve_part(
            AvatarItemCategory::Face,
            positive_style("face", face_style)?,
            gender,
            NativePlayerPartKind::Face,
            PartTextureRule::Face { eye_color },
            Some(hat_policy.face_variant),
        )?];
        if let Some(hair_variant) = hat_policy.hair_variant {
            parts.push(self.resolve_part(
                AvatarItemCategory::Head,
                positive_style("hair", hair_style)?,
                gender,
                NativePlayerPartKind::Hair,
                PartTextureRule::Hair,
                Some(hair_variant),
            )?);
        }
        for &(category, item_number, kind) in equipped {
            parts.push(self.resolve_part(
                category,
                item_number,
                gender,
                kind,
                PartTextureRule::Static,
                None,
            )?);
        }

        let skin_reference = match gender {
            DataGender::Male => &self.appearance.texture_rules.male_skin_texture,
            DataGender::Female => &self.appearance.texture_rules.female_skin_texture,
        };
        let skin_texture = Some(self.unique_texture_reference(skin_reference)?);
        let look = NativePlayerLook {
            identity,
            gender: player_rig_gender(gender),
            parts,
            skin_texture,
            skin_color: self.skin_color(u8::try_from(skin_color).map_err(|_| {
                CharacterCreationDataError::Invalid("negative skin color".to_owned())
            })?)?,
            hair_color: self.hair_color(u8::try_from(hair_color).map_err(|_| {
                CharacterCreationDataError::Invalid("negative hair color".to_owned())
            })?)?,
            weapon_animation_profile: None,
            height_selector: height,
            body_selector: body,
        };
        look.validate()
            .map_err(CharacterCreationDataError::Invalid)?;
        Ok(look)
    }

    pub(super) fn resolve_part(
        &self,
        category: AvatarItemCategory,
        item_number: u32,
        gender: DataGender,
        kind: NativePlayerPartKind,
        texture_rule: PartTextureRule,
        preferred_model_variant: Option<u8>,
    ) -> CharacterCreationDataResult<NativePlayerPartLook> {
        let item = self.items.get(&(category, item_number)).ok_or_else(|| {
            CharacterCreationDataError::MissingRoute(format!(
                "native avatar lookup has no {:?} item {}",
                category, item_number
            ))
        })?;
        let visual = match gender {
            DataGender::Male => &item.male,
            DataGender::Female => &item.female,
        };
        let model = self.resolve_model(item, visual, gender, preferred_model_variant)?;
        let primary_texture = match texture_rule {
            PartTextureRule::Static => visual
                .primary_texture
                .as_ref()
                .map(|texture| self.unique_texture_reference(texture))
                .transpose()?,
            PartTextureRule::Face { eye_color } => {
                let base = visual
                    .primary_texture
                    .as_ref()
                    .map(|texture| texture.true_name.as_str())
                    .ok_or_else(|| {
                        CharacterCreationDataError::MissingRoute(format!(
                            "{:?} item {} has no face texture basename",
                            category, item_number
                        ))
                    })?;
                let suffix = self
                    .appearance
                    .texture_rules
                    .face_eye_suffix_by_code
                    .iter()
                    .find(|entry| entry.code == eye_color)
                    .ok_or_else(|| {
                        CharacterCreationDataError::Invalid(format!(
                            "eye color {eye_color} has no exact texture suffix"
                        ))
                    })?;
                Some(self.unique_texture_stem(&format!("{base}_{}", suffix.suffix))?)
            }
            PartTextureRule::Hair => {
                let base = visual
                    .primary_texture
                    .as_ref()
                    .map(|texture| texture.true_name.as_str())
                    .ok_or_else(|| {
                        CharacterCreationDataError::MissingRoute(format!(
                            "{:?} item {} has no hair texture basename",
                            category, item_number
                        ))
                    })?;
                Some(self.unique_texture_stem(&format!(
                    "{base}_{}",
                    self.appearance.texture_rules.hair_eye_suffix
                ))?)
            }
        };
        let secondary_texture = visual
            .secondary_texture
            .as_ref()
            .map(|texture| self.unique_texture_reference(texture))
            .transpose()?;
        Ok(NativePlayerPartLook {
            kind,
            assembly: NativePlayerPartAssembly::SharedSkin,
            exact_route: model.exact_route.clone(),
            glb: model.native_asset.path.clone(),
            primary_texture,
            secondary_texture,
        })
    }

    pub(super) fn resolve_model<'a>(
        &'a self,
        item: &AvatarItemLookup,
        visual: &'a AvatarItemVisual,
        gender: DataGender,
        preferred_variant: Option<u8>,
    ) -> CharacterCreationDataResult<&'a ffone_runtime_contracts::AvatarModelReference> {
        let fallback = self
            .creator_model_fallbacks
            .get(&(gender, item.category, item.item_number));
        if matches!(
            visual.model_status,
            NativeLookupStatus::VerifiedUnique | NativeLookupStatus::VerifiedVariants
        ) {
            let model = if let Some(variant) = preferred_variant {
                let suffix = format!("_type{variant:02}");
                visual
                    .models
                    .iter()
                    .find(|model| model.true_name.to_ascii_lowercase().ends_with(&suffix))
                    .ok_or_else(|| {
                        CharacterCreationDataError::MissingRoute(format!(
                            "{:?} item {} has no exact {suffix} native GLB",
                            item.category, item.item_number
                        ))
                    })?
            } else {
                visual
                    .models
                    .iter()
                    .find(|model| model.true_name.to_ascii_lowercase().ends_with("_type01"))
                    .or_else(|| visual.models.first())
                    .ok_or_else(|| {
                        CharacterCreationDataError::MissingRoute(format!(
                            "{:?} item {} has no native GLB",
                            item.category, item.item_number
                        ))
                    })?
            };
            self.validate_reference(&model.native_asset, NativeAssetKind::Model)?;
            if preferred_variant.unwrap_or(1) == 1
                && let Some(fallback) = fallback
                && (fallback.exact_route != model.exact_route
                    || fallback.native_asset != model.native_asset)
            {
                return invalid(format!(
                    "{gender:?}/{:?}/{} avatar-item model differs from the complete shared-rig creator contract",
                    item.category, item.item_number
                ));
            }
            return Ok(model);
        }
        if visual.model_status == NativeLookupStatus::Missing {
            if preferred_variant.is_some_and(|variant| variant != 1) {
                return Err(CharacterCreationDataError::MissingRoute(format!(
                    "{gender:?}/{:?} item {} has no exact requested model variant",
                    item.category, item.item_number
                )));
            }
            let fallback = fallback.ok_or_else(|| {
                CharacterCreationDataError::MissingRoute(format!(
                    "{gender:?}/{:?} item {} is missing and has no exact shared-rig fallback",
                    item.category, item.item_number
                ))
            })?;
            self.validate_reference(&fallback.native_asset, NativeAssetKind::Model)?;
            return Ok(fallback);
        }
        Err(CharacterCreationDataError::MissingRoute(format!(
            "{gender:?}/{:?} item {} model route is {:?}",
            item.category, item.item_number, visual.model_status
        )))
    }

    pub(super) fn unique_texture_reference(
        &self,
        texture: &AvatarTextureReference,
    ) -> CharacterCreationDataResult<NativePlayerTexture> {
        if texture.status != NativeLookupStatus::VerifiedUnique || texture.candidates.len() != 1 {
            return Err(CharacterCreationDataError::MissingRoute(format!(
                "texture {} route is {:?} with {} candidates",
                texture.true_name,
                texture.status,
                texture.candidates.len()
            )));
        }
        let reference = &texture.candidates[0];
        self.validate_reference(reference, NativeAssetKind::Texture)?;
        self.native_player_texture(&reference.path)
    }

    pub(super) fn published_texture_reference(
        &self,
        texture: &AvatarTextureReference,
    ) -> CharacterCreationDataResult<Option<NativePlayerTexture>> {
        if texture.status != NativeLookupStatus::VerifiedUnique || texture.candidates.len() != 1 {
            return Ok(None);
        }
        let reference = &texture.candidates[0];
        self.validate_reference(reference, NativeAssetKind::Texture)?;
        Ok(self
            .runtime_texture_contracts
            .get(&reference.path)
            .cloned()
            .map(|contract| NativePlayerTexture {
                path: reference.path.clone(),
                contract,
            }))
    }

    pub(super) fn unique_texture_stem(&self, stem: &str) -> CharacterCreationDataResult<NativePlayerTexture> {
        let routes = self
            .textures_by_stem
            .get(&stem.to_ascii_lowercase())
            .ok_or_else(|| {
                CharacterCreationDataError::MissingRoute(format!(
                    "native texture stem {stem} is absent from the runtime texture contract"
                ))
            })?;
        if routes.len() != 1 {
            return Err(CharacterCreationDataError::MissingRoute(format!(
                "native texture stem {stem} is ambiguous: {} routes",
                routes.len()
            )));
        }
        self.native_player_texture(&routes[0])
    }

    pub(super) fn native_player_texture(
        &self,
        path: &str,
    ) -> CharacterCreationDataResult<NativePlayerTexture> {
        let contract = self.runtime_texture_contracts.get(path).ok_or_else(|| {
            CharacterCreationDataError::MissingRoute(format!(
                "native texture {path} has no exact runtime sampler contract"
            ))
        })?;
        Ok(NativePlayerTexture {
            path: path.to_owned(),
            contract: contract.clone(),
        })
    }

    pub(super) fn validate_reference(
        &self,
        reference: &CharacterCreationAssetReference,
        kind: NativeAssetKind,
    ) -> CharacterCreationDataResult<()> {
        let path = normalize_relative_path(&reference.path)?;
        if Path::new(&path)
            .extension()
            .and_then(|value| value.to_str())
            != Some(kind.extension())
        {
            return invalid(format!(
                "native route {path} has the wrong extension for {kind:?}"
            ));
        }
        self.locator
            .read_verified(&path, Some(reference.bytes), &reference.blake3)
            .map_err(CharacterCreationDataError::Invalid)?;
        Ok(())
    }

}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> CharacterCreationDataResult<T> {
    let bytes = fs::read(path).map_err(|source| CharacterCreationDataError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| CharacterCreationDataError::Json {
        path: path.to_path_buf(),
        source,
    })
}
