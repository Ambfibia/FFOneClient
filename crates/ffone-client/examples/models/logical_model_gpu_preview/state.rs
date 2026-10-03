use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CharacterRuntimeConfig {
    pub(super) kind: CharacterKind,
    pub(super) true_root: String,
    pub(super) npc_scale: Option<f32>,
}

impl CharacterRuntimeConfig {
    pub(super) fn policy(&self) -> LegacyCharacterRootPolicy {
        match self.kind {
            CharacterKind::Npc => LegacyCharacterRootPolicy::Npc {
                table_scale: self
                    .npc_scale
                    .expect("validated NPC character config has a table scale"),
            },
            CharacterKind::Nano => LegacyCharacterRootPolicy::Nano,
            CharacterKind::Player => LegacyCharacterRootPolicy::Player,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PreviewOutlineMode {
    Source,
    Hidden,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_character_runtime_status(
    config: Res<PreviewConfig>,
    handles: Option<Res<CharacterSceneHandles>>,
    statuses: Query<&LegacyCharacterSceneStatus>,
    local_transforms: Query<&Transform>,
    global_transforms: Query<&GlobalTransform>,
    mut state: ResMut<RuntimeState>,
    shared: Res<SharedReport>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(character) = config.character.as_ref() else {
        return;
    };
    if state.terminal {
        return;
    }
    let Some(handles) = handles else {
        return;
    };
    let Ok(status) = statuses.get(handles.scene) else {
        fail_runtime(
            &mut state,
            &shared,
            &mut app_exit,
            "character runtime scene lost its normalization status".to_owned(),
        );
        return;
    };
    match status {
        LegacyCharacterSceneStatus::Pending => {}
        LegacyCharacterSceneStatus::Blocked(block) => {
            fail_runtime(
                &mut state,
                &shared,
                &mut app_exit,
                character_scene_block_message(block),
            );
        }
        LegacyCharacterSceneStatus::Ready {
            root,
            authored_root,
            applied_root,
        } => {
            let Ok(gameplay_root) = local_transforms.get(handles.gameplay_root) else {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    "character runtime gameplay root lost its Transform".to_owned(),
                );
                return;
            };
            let Ok(visual_container) = local_transforms.get(handles.visual_container) else {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    "character runtime visual container lost its Transform".to_owned(),
                );
                return;
            };
            let Ok(actual_root) = local_transforms.get(*root) else {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    "normalized exact character root lost its Transform".to_owned(),
                );
                return;
            };
            if *actual_root != *applied_root {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    format!(
                        "normalized exact character root drifted after WorldInstanceReady: expected {applied_root:?}, actual {actual_root:?}"
                    ),
                );
                return;
            }
            let container_world = global_transforms
                .get(handles.visual_container)
                .ok()
                .and_then(TransformSnapshot::from_global);
            let root_world = global_transforms
                .get(*root)
                .ok()
                .and_then(TransformSnapshot::from_global);
            let one_character_half_turn =
                is_exact_character_runtime_chain(*gameplay_root, *visual_container, *actual_root);
            if !one_character_half_turn {
                fail_runtime(
                    &mut state,
                    &shared,
                    &mut app_exit,
                    "character runtime chain does not contain exactly one +Z to -Z visual-container half-turn"
                        .to_owned(),
                );
                return;
            }
            state.scene_ready = true;
            shared.update(|report| {
                report.character_runtime = Some(CharacterRuntimeReport {
                    kind: character.kind,
                    exact_root_name: character.true_root.clone(),
                    npc_table_scale: character.npc_scale,
                    gameplay_root_local: TransformSnapshot::from_local(*gameplay_root),
                    visual_container_local: TransformSnapshot::from_local(*visual_container),
                    authored_root_local: TransformSnapshot::from_local(*authored_root),
                    applied_root_local: TransformSnapshot::from_local(*applied_root),
                    actual_root_local: TransformSnapshot::from_local(*actual_root),
                    visual_container_world: container_world,
                    root_world,
                    one_character_half_turn,
                });
            });
        }
    }
}

pub(super) fn is_exact_character_runtime_chain(
    gameplay_root: Transform,
    visual_container: Transform,
    applied_root: Transform,
) -> bool {
    gameplay_root.translation == Vec3::ZERO
        && gameplay_root.rotation.abs_diff_eq(Quat::IDENTITY, 1.0e-6)
        && gameplay_root.scale == Vec3::ONE
        && visual_container.translation == Vec3::ZERO
        && visual_container.scale == Vec3::ONE
        && (visual_container.rotation * Vec3::Z).abs_diff_eq(Vec3::NEG_Z, 1.0e-6)
        && (visual_container.rotation * Vec3::Y).abs_diff_eq(Vec3::Y, 1.0e-6)
        && applied_root.translation == Vec3::ZERO
        && applied_root.rotation.abs_diff_eq(Quat::IDENTITY, 1.0e-6)
        && applied_root.scale.is_finite()
        && applied_root.scale.min_element() > 0.0
}

#[derive(Resource)]
pub(super) struct RuntimeState {
    pub(super) started: Instant,
    pub(super) frames: u64,
    pub(super) scene_ready: bool,
    pub(super) gltf_loaded_with_dependencies: bool,
    pub(super) gltf_inspected: bool,
    pub(super) animation_resolved: bool,
    pub(super) animation_started: bool,
    pub(super) animation_started_frame: Option<u64>,
    pub(super) animations_loaded: u64,
    pub(super) exact_animation_names: Vec<String>,
    pub(super) sampled_players: u64,
    pub(super) prepared_animation: Option<PreparedAnimation>,
    pub(super) ready_since_frame: Option<u64>,
    pub(super) camera_view: PreviewCameraView,
    pub(super) camera_retry_count: u8,
    pub(super) camera_framed: bool,
    pub(super) capture_issued: bool,
    pub(super) blank_capture_attempts: u32,
    pub(super) screenshot_saved_frame: Option<u64>,
    pub(super) captured_png: Option<CapturedPng>,
    pub(super) terminal: bool,
}

impl RuntimeState {
    pub(super) fn new(camera_view: PreviewCameraView) -> Self {
        Self {
            started: Instant::now(),
            frames: 0,
            scene_ready: false,
            gltf_loaded_with_dependencies: false,
            gltf_inspected: false,
            animation_resolved: false,
            animation_started: false,
            animation_started_frame: None,
            animations_loaded: 0,
            exact_animation_names: Vec::new(),
            sampled_players: 0,
            prepared_animation: None,
            ready_since_frame: None,
            camera_view,
            camera_retry_count: 0,
            camera_framed: false,
            capture_issued: false,
            blank_capture_attempts: 0,
            screenshot_saved_frame: None,
            captured_png: None,
            terminal: false,
        }
    }
}

pub(super) fn fail_runtime(
    state: &mut RuntimeState,
    shared: &SharedReport,
    app_exit: &mut MessageWriter<AppExit>,
    error: String,
) {
    if state.terminal {
        return;
    }
    state.terminal = true;
    shared.fail(error);
    app_exit.write(AppExit::error());
}

pub(super) fn succeed_runtime(
    state: &mut RuntimeState,
    shared: &SharedReport,
    app_exit: &mut MessageWriter<AppExit>,
) {
    if state.terminal {
        return;
    }
    state.terminal = true;
    shared.succeed();
    app_exit.write(AppExit::Success);
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CharacterRuntimeReport {
    pub(super) kind: CharacterKind,
    pub(super) exact_root_name: String,
    pub(super) npc_table_scale: Option<f32>,
    pub(super) gameplay_root_local: TransformSnapshot,
    pub(super) visual_container_local: TransformSnapshot,
    pub(super) authored_root_local: TransformSnapshot,
    pub(super) applied_root_local: TransformSnapshot,
    pub(super) actual_root_local: TransformSnapshot,
    pub(super) visual_container_world: Option<TransformSnapshot>,
    pub(super) root_world: Option<TransformSnapshot>,
    pub(super) one_character_half_turn: bool,
}

impl CharacterRuntimeReport {
    pub(super) fn as_json(&self) -> serde_json::Value {
        json!({
            "kind": self.kind.as_str(),
            "exactRootName": self.exact_root_name,
            "npcTableScale": self.npc_table_scale,
            "gameplayRootLocal": self.gameplay_root_local.as_json(),
            "visualContainerLocal": self.visual_container_local.as_json(),
            "authoredRootLocal": self.authored_root_local.as_json(),
            "appliedRootLocal": self.applied_root_local.as_json(),
            "actualRootLocal": self.actual_root_local.as_json(),
            "visualContainerWorld": self.visual_container_world.map(TransformSnapshot::as_json),
            "rootWorld": self.root_world.map(TransformSnapshot::as_json),
            "oneCharacterHalfTurn": self.one_character_half_turn,
        })
    }
}
