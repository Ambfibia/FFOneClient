use super::*;

pub(in super::super) struct TutorialChoreographyAdapterPlugin;

impl Plugin for TutorialChoreographyAdapterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialChoreographyExecution>()
            .init_resource::<TutorialProjectileRandomStream>()
            .add_systems(
                Update,
                (
                    drive_tutorial_auxiliary_choreography,
                    apply_tutorial_choreography_events,
                    apply_pending_tutorial_exit,
                    apply_tutorial_player_cinematic_turn,
                    apply_tutorial_choreography_camera,
                    sync_tutorial_voice_subtitle_context.in_set(TutorialVoiceSubtitleSet::Context),
                )
                    .chain()
                    .before(TutorialCinematicTitleSet::Bind)
                    .before(process_tutorial_effect_runtime)
                    .after(drive_local_tutorial)
                    .after(TutorialNanoPresentationSet::ApplyCommands)
                    .run_if(in_state(ClientState::Tutorial)),
            )
            .add_systems(
                Update,
                apply_tutorial_npc_subtarget_camera
                    .after(apply_tutorial_choreography_camera)
                    .after(sync_tutorial_mission_interaction)
                    .after(LegacyMovementSet::CameraPose)
                    .after(NativeWorldSet::ResolveCameraOcclusion)
                    .before(ensure_and_update_legacy_skybox)
                    .before(ensure_and_update_legacy_fusion_star)
                    .run_if(in_state(ClientState::Tutorial)),
            )
            .add_systems(
                Update,
                resolve_pending_tutorial_actor_effects
                    .after(drive_tutorial_auxiliary_choreography)
                    .after(TutorialActorSet::GroundActors)
                    .before(process_tutorial_effect_runtime)
                    .run_if(in_state(ClientState::Tutorial)),
            )
            .add_systems(
                Update,
                sync_tutorial_choreography_visibility
                    .after(apply_tutorial_choreography_events)
                    .after(gameplay_ui_actions::gm_runtime::pump),
            );
    }
}
