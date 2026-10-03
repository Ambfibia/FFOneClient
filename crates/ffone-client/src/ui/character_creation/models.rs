use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct CharacterCreationUiModel {
    pub visible: bool,
    pub screen: CharacterCreationScreen,
    pub ui_scale: f32,
    pub fullscreen: bool,
    pub music_enabled: bool,
    /// Set after `SaveCharName` succeeds. The legacy client randomizes the
    /// initial appearance once when `CnGuiCharCreation` first opens.
    pub randomize_on_appearance_open: bool,
    pub slot: Option<u8>,
    pub pc_uid: Option<i64>,
    pub appearance: CharacterAppearance,
    pub hair_label: String,
    pub face_label: String,
    pub option_counts: CharacterCreationOptionCounts,
    pub color_palettes: [Vec<Color>; 3],
    pub color_pages: [u8; 3],
    pub name_mode: CharacterNameMode,
    pub name_indices: [usize; 3],
    pub custom_name: String,
    pub custom_name_focused: bool,
    pub asset_status: CharacterCreationAssetStatus,
    pub name_table: CharacterCreationCapability,
    pub custom_name_filter: CharacterCreationCapability,
    pub creation_items: CharacterCreationCapability,
    pub starter_icons: CharacterCreationCapability,
    /// Exact five-icon windows for shirt, pants and shoes.
    pub starter_icon_paths: [[Option<String>; 5]; 3],
    pub preview: CharacterCreationPreviewStatus,
    pub reserve_name: CharacterCreationCapability,
    pub save_appearance: CharacterCreationCapability,
    pub blocker: Option<CharacterCreationPending>,
    pub validation_error: Option<CharacterNameValidationError>,
}

impl Default for CharacterCreationUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            screen: CharacterCreationScreen::Name,
            ui_scale: 1.0,
            fullscreen: false,
            music_enabled: true,
            randomize_on_appearance_open: false,
            slot: None,
            pc_uid: None,
            appearance: CharacterAppearance::default(),
            hair_label: "HAIR 2".to_owned(),
            face_label: "FACE 2".to_owned(),
            option_counts: CharacterCreationOptionCounts::default(),
            color_palettes: [
                SKIN_COLORS.to_vec(),
                HAIR_COLORS.to_vec(),
                EYE_COLORS.to_vec(),
            ],
            color_pages: [0; 3],
            name_mode: CharacterNameMode::Generated,
            name_indices: [1; 3],
            custom_name: String::new(),
            custom_name_focused: false,
            asset_status: CharacterCreationAssetStatus::Loading,
            name_table: CharacterCreationCapability::Pending(
                CharacterCreationPending::NameTableNotPublished,
            ),
            custom_name_filter: CharacterCreationCapability::Pending(
                CharacterCreationPending::CustomNameFilterNotPublished,
            ),
            creation_items: CharacterCreationCapability::Pending(
                CharacterCreationPending::CreationItemCatalogNotPublished,
            ),
            starter_icons: CharacterCreationCapability::Pending(
                CharacterCreationPending::StarterClothingIconsNotPublished,
            ),
            starter_icon_paths: std::array::from_fn(|_| std::array::from_fn(|_| None)),
            preview: CharacterCreationPreviewStatus::PlayerAssemblyPending,
            reserve_name: CharacterCreationCapability::Pending(
                CharacterCreationPending::ReserveNameNetworkCommand,
            ),
            save_appearance: CharacterCreationCapability::Pending(
                CharacterCreationPending::SaveAppearanceNetworkCommand,
            ),
            blocker: None,
            validation_error: None,
        }
    }
}

impl CharacterCreationUiModel {
    pub fn set_gender(&mut self, gender: CharacterGender) {
        self.appearance.set_gender(gender, self.option_counts);
    }

    pub fn step_appearance(&mut self, field: AppearanceField, delta: i8) {
        self.appearance.step(field, delta, self.option_counts);
    }

    pub fn set_skin_color(&mut self, index: u8) {
        if usize::from(index) < self.color_palettes[0].len() {
            self.appearance.skin_color = index + 1;
        }
    }

    pub fn set_hair_color(&mut self, index: u8) {
        if usize::from(index) < self.color_palettes[1].len() {
            self.appearance.hair_color = index + 1;
        }
    }

    pub fn set_eye_color(&mut self, index: u8) {
        if usize::from(index) < self.color_palettes[2].len() {
            self.appearance.eye_color = index + 1;
        }
    }

    pub fn scroll_name(
        &mut self,
        names: &CharacterNameLists,
        part: CharacterNamePart,
        delta: i8,
    ) -> bool {
        let list = names.list(part);
        if list.len() <= 2 || delta == 0 {
            return false;
        }
        let selectable = list.len() - 1;
        let current = self.name_indices[part as usize].clamp(1, selectable);
        self.name_indices[part as usize] =
            1 + (current as isize - 1 + delta as isize).rem_euclid(selectable as isize) as usize;
        true
    }

    pub fn visible_name_indices(
        &self,
        names: &CharacterNameLists,
        part: CharacterNamePart,
    ) -> Option<[usize; 5]> {
        let list = names.list(part);
        if list.len() <= 2 {
            return None;
        }
        let selectable = list.len() - 1;
        let center = self.name_indices[part as usize].clamp(1, selectable);
        Some(std::array::from_fn(|row| {
            let delta = row as isize - 2;
            1 + (center as isize - 1 + delta).rem_euclid(selectable as isize) as usize
        }))
    }

    pub fn generated_name(
        &self,
        names: &CharacterNameLists,
    ) -> Result<GeneratedCharacterName, CharacterNameValidationError> {
        if !names.valid() {
            return Err(CharacterNameValidationError::NameTableUnavailable);
        }
        let [first_index, middle_index, last_index] = self.name_indices;
        let first = names
            .first
            .get(first_index)
            .ok_or(CharacterNameValidationError::NameTableUnavailable)?
            .clone();
        let middle = names
            .middle
            .get(middle_index)
            .ok_or(CharacterNameValidationError::NameTableUnavailable)?;
        let last = names
            .last
            .get(last_index)
            .ok_or(CharacterNameValidationError::NameTableUnavailable)?;
        if first.is_empty() {
            return Err(CharacterNameValidationError::Empty);
        }
        let last = compose_legacy_last_name(middle, last);
        if last.is_empty() {
            return Err(CharacterNameValidationError::MissingLastPart);
        }
        Ok(GeneratedCharacterName {
            first,
            last,
            first_index,
            middle_index,
            last_index,
        })
    }

    pub fn custom_name(&self) -> Result<CustomCharacterName, CharacterNameValidationError> {
        validate_custom_name(&self.custom_name)
    }
}
