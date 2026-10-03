use super::*;

impl Plugin for NativeCharacterCreationUiPlugin {
    fn build(&self, app: &mut App) {
        let deferred_in_production = app
            .world()
            .get_resource::<State<crate::ui_startup::NativeUiStartupPhase>>()
            .is_some_and(|phase| phase.get() == &crate::ui_startup::NativeUiStartupPhase::Deferred);

        app.init_resource::<CharacterCreationUiModel>()
            .init_resource::<RetrobutionAudioMix>()
            .init_resource::<CharacterCreationUiOutbox>()
            .init_resource::<CharacterNameLists>()
            .init_resource::<CharacterCreationRandom>()
            .configure_sets(
                Update,
                (
                    CharacterCreationUiSet::Assets,
                    CharacterCreationUiSet::Layout,
                    CharacterCreationUiSet::Interaction,
                    CharacterCreationUiSet::Bind,
                    CharacterCreationUiSet::Audio,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                update_character_creation_asset_status.in_set(CharacterCreationUiSet::Assets),
            )
            .add_systems(
                Update,
                update_character_creation_layout.in_set(CharacterCreationUiSet::Layout),
            )
            .add_systems(
                Update,
                (
                    apply_pending_appearance_randomization,
                    handle_character_creation_interactions,
                    repeat_character_creation_camera_controls,
                    edit_custom_character_name,
                )
                    .chain()
                    .in_set(CharacterCreationUiSet::Interaction),
            )
            .add_systems(
                Update,
                (
                    bind_character_creation_visibility,
                    bind_character_creation_controls,
                    bind_character_creation_clothing_icons,
                    bind_character_creation_text,
                    bind_character_creation_colors,
                )
                    .in_set(CharacterCreationUiSet::Bind)
                    .before(LocalizationSet::Apply),
            )
            .add_systems(
                Update,
                control_character_creation_music.in_set(CharacterCreationUiSet::Audio),
            )
            // Like selection, the creation update set is phase-gated. Camera
            // ownership must keep running for one transition frame so an old
            // order-100 camera cannot survive into gameplay or selection.
            .add_systems(Update, sync_character_creation_camera_activity);

        if deferred_in_production {
            app.configure_sets(
                Update,
                CharacterCreationUiSet::Assets.run_if(character_creation_assets_active),
            )
            .configure_sets(
                Update,
                (
                    CharacterCreationUiSet::Layout,
                    CharacterCreationUiSet::Interaction,
                    CharacterCreationUiSet::Bind,
                )
                    .chain()
                    .in_set(CharacterCreationStartupSet),
            )
            .configure_sets(
                Update,
                CharacterCreationStartupSet.run_if(in_state(
                    crate::ui_startup::NativeUiStartupPhase::CharacterCreation,
                )),
            )
            .add_systems(
                OnEnter(crate::ui_startup::NativeUiStartupPhase::CharacterSelection),
                spawn_character_creation_ui,
            )
            .add_systems(
                OnEnter(crate::ui_startup::NativeUiStartupPhase::CharacterCreation),
                spawn_character_creation_ui,
            );
        } else {
            app.add_systems(Startup, spawn_character_creation_ui);
        }
    }
}
