use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct CharacterSelectionUiModel {
    pub visible: bool,
    pub slots: [CharacterSlotUi; 4],
    pub selected_slot: Option<usize>,
    pub hovered_slot: Option<usize>,
    pub music_enabled: bool,
    pub fullscreen: bool,
    pub ui_scale: f32,
    pub create: CharacterSelectionCapability,
    pub delete: CharacterSelectionCapability,
    pub preview: CharacterPreviewStatus,
    /// Readiness of the complete static selection screen (images, fonts,
    /// button sounds and music). The application-level loading barrier keeps
    /// the screen covered until this reaches `Ready` together with every
    /// occupied avatar renderer.
    pub asset_status: CharacterSelectionAssetStatus,
    /// Exact `CnCharSelectionMode::RotateLeft/RotateRight` yaw accumulator.
    ///
    /// The native player-assembly renderer consumes this once it is ready.
    /// Keeping the value in the UI model lets the original repeat-button
    /// control remain functional without substituting a legacy or NPC preview.
    pub preview_yaw_degrees: f32,
    /// `CnGuiCharSelection::bDeletePressed`: while set, the exact modal blocks
    /// the underlying selection controls until Cancel or Delete is pressed.
    pub delete_confirmation_pc_uid: Option<i64>,
    /// Text from the focused legacy `delname` field (maximum 32 characters).
    pub delete_name_input: String,
    /// The clean dialog remains modal after emitting its delete request and
    /// rejects a second click until the correlated login-server reply arrives.
    pub delete_request_pending: bool,
    pub status: Option<String>,
}

impl Default for CharacterSelectionUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            slots: std::array::from_fn(|_| CharacterSlotUi::Empty),
            selected_slot: None,
            hovered_slot: None,
            music_enabled: true,
            fullscreen: false,
            ui_scale: 1.0,
            create: CharacterSelectionCapability::Pending(
                CharacterSelectionPending::CreateCharacterNetworkCommand,
            ),
            delete: CharacterSelectionCapability::Pending(
                CharacterSelectionPending::DeleteCharacterNetworkCommand,
            ),
            preview: CharacterPreviewStatus::PlayerAssemblyPending,
            asset_status: CharacterSelectionAssetStatus::Loading,
            preview_yaw_degrees: 0.0,
            delete_confirmation_pc_uid: None,
            delete_name_input: String::new(),
            delete_request_pending: false,
            status: None,
        }
    }
}

impl CharacterSelectionUiModel {
    pub fn set_slots(&mut self, slots: [CharacterSlotUi; 4], preferred_uid: Option<i64>) {
        self.slots = slots;
        self.selected_slot = preferred_uid
            .and_then(|uid| {
                self.slots.iter().position(|slot| {
                    matches!(
                        slot,
                        CharacterSlotUi::Occupied(character) if character.pc_uid == uid
                    )
                })
            })
            .or_else(|| {
                self.selected_slot
                    .filter(|index| self.is_selectable(*index))
            })
            .or_else(|| self.first_selectable_slot());
        if self
            .hovered_slot
            .is_some_and(|index| !self.is_selectable(index))
        {
            self.hovered_slot = None;
        }
    }

    pub fn select_slot(&mut self, index: usize) -> bool {
        // The selected slot is drawn as `GUI.Label(CharButtonOver)` in the
        // source and therefore cannot generate another click or sound.
        if !self.is_selectable(index) || self.selected_slot == Some(index) {
            return false;
        }
        self.selected_slot = Some(index);
        true
    }

    pub fn selected_character(&self) -> Option<&OccupiedCharacterSlotUi> {
        self.selected_slot
            .and_then(|index| self.slots.get(index))
            .and_then(CharacterSlotUi::occupied)
    }

    pub fn selected_background(&self) -> CharacterLocationBackground {
        self.selected_character()
            .map_or(CharacterLocationBackground::Future, |slot| slot.background)
    }

    pub fn first_selectable_slot(&self) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| matches!(slot, CharacterSlotUi::Occupied(_)))
    }

    /// First writable OpenFusion character slot, using the protocol's
    /// one-based `iPC_UID` slot numbering (`1..=4`), not the UI array index.
    pub fn first_creatable_protocol_slot(&self) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| matches!(slot, CharacterSlotUi::Empty))
            .map(|index| index + 1)
    }

    pub fn is_selectable(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .is_some_and(|slot| matches!(slot, CharacterSlotUi::Occupied(_)))
    }

    pub(super) fn delete_confirmation_name_matches(&self) -> bool {
        let Some(pc_uid) = self.delete_confirmation_pc_uid else {
            return false;
        };
        self.slots.iter().any(|slot| {
            let Some(character) = slot.occupied() else {
                return false;
            };
            character.pc_uid == pc_uid
                && character
                    .display_name
                    .split_whitespace()
                    .next()
                    .is_some_and(|first_name| first_name == self.delete_name_input)
        })
    }

    pub fn complete_delete_request(&mut self, pc_uid: i64) {
        if self.delete_confirmation_pc_uid == Some(pc_uid) {
            self.delete_confirmation_pc_uid = None;
            self.delete_name_input.clear();
        }
        self.delete_request_pending = false;
    }

    pub fn reject_delete_request(&mut self) {
        self.delete_request_pending = false;
    }

    pub fn reset_delete_confirmation(&mut self) {
        self.delete_confirmation_pc_uid = None;
        self.delete_name_input.clear();
        self.delete_request_pending = false;
    }
}
