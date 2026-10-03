use super::*;

/// Keeps a deliberate Enter-the-game action across the final selection-loading
/// frames. The selection screen can already be painted while its presentation
/// barrier is settling; dropping the input in that window made the button look
/// permanently unresponsive until the player clicked it again.
#[derive(Debug, Default, Resource)]
pub(in super::super) struct BufferedCharacterEntry(pub(super) Option<i64>);

impl BufferedCharacterEntry {
    pub(in super::super) fn queue(&mut self, pc_uid: i64, loading: &GameplayLoadingState) {
        // A second click during the shard handshake must never become another
        // entry request after WorldReady releases the loading screen.
        if loading.scope == Some(ResourceLoadingScope::CharacterSelection) {
            self.0 = Some(pc_uid);
        }
    }

    pub(in super::super) fn take_when_ready(&mut self, loading: &GameplayLoadingState) -> Option<i64> {
        (!loading.visible).then(|| self.0.take()).flatten()
    }

    pub(in super::super) fn clear(&mut self) {
        self.0 = None;
    }
}

pub(in super::super) struct CharacterFlowPlugin;

impl Plugin for CharacterFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BufferedCharacterEntry>()
            .init_resource::<CharacterCreationAssetLease>()
            .add_systems(
                Update,
                sync_character_selection_ui
                    .after(NetworkSessionLifecycleSet::Apply)
                    .before(CharacterSelectionUiSet::Layout)
                    .before(CharacterSelectionPortraitsSet::Rebuild),
            )
            .add_systems(
                Update,
                handle_character_selection_ui_actions
                    .after(CharacterSelectionUiSet::Interaction)
                    .before(CharacterSelectionUiSet::Bind),
            )
            .add_systems(
                Update,
                select_character_from_keyboard
                    .after(CharacterSelectionUiSet::Layout)
                    .before(CharacterSelectionUiSet::Bind),
            )
            .add_systems(
                Update,
                sync_character_creation_ui
                    .after(NetworkSessionLifecycleSet::Apply)
                    .before(CharacterCreationUiSet::Layout),
            )
            .add_systems(
                Update,
                sync_native_player_preview
                    .after(sync_character_selection_ui)
                    .after(sync_character_creation_ui)
                    .after(sync_user_equip_ui_context)
                    .after(consume_user_equip_ui_outbox)
                    .after(CharacterSelectionUiSet::Interaction)
                    .after(CharacterCreationUiSet::Interaction)
                    .after(UserEquipUiSet::Interaction)
                    .after(VendorUiSet::Interaction)
                    .before(VendorUiSet::Bind)
                    .before(NativePlayerPreviewSet::Rebuild)
                    .before(CharacterSelectionUiSet::Bind)
                    .before(CharacterCreationUiSet::Bind)
                    .before(UserEquipUiSet::Bind),
            )
            .add_systems(
                Update,
                handle_character_creation_ui_actions
                    .after(CharacterCreationUiSet::Interaction)
                    .after(sync_native_player_preview)
                    .before(NativePlayerPreviewSet::Rebuild)
                    .before(CharacterCreationUiSet::Bind),
            );
    }
}

#[derive(SystemParam)]
pub(super) struct VendorTryOnOwners<'w> {
    pub(super) popup: Option<ResMut<'w, ffone_client::vendor_ui::VendorItemPopupState>>,
    pub(super) vendor: Option<Res<'w, VendorUiState>>,
    pub(super) content: Option<Res<'w, TutorialMissionContent>>,
    pub(super) guide: Option<Res<'w, GuideRuntime>>,
}
