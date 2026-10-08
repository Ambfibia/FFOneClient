use super::*;

#[derive(Debug, Resource)]
pub(super) struct EditorState {
    pub(super) reveal_selection: bool,
    pub(super) kind: CatalogKind,
    pub(super) selected: usize,
    pub(super) search: String,
    pub(super) search_focused: bool,
    pub(super) strings_open: bool,
    pub(super) xdt_open: bool,
    pub(super) missions_open: bool,
    pub(super) world_open: Option<bool>,
    pub(super) viewer_tabs: BTreeMap<CatalogKind, (usize, String)>,
    pub(super) details_open: bool,
    pub(super) npc_inspector: NpcInspectorTab,
    pub(super) equipment_female: bool,
    pub(super) equipment_category: Option<ffone_runtime_contracts::AvatarItemCategory>,
    pub(super) animation_page: usize,
    pub(super) clip_index: usize,
    pub(super) pose_mode: EditorPoseMode,
    pub(super) paused: bool,
    pub(super) looping: bool,
    pub(super) speed: f32,
    pub(super) turntable: bool,
    pub(super) playback_revision: u64,
}

impl EditorState {
    pub(super) fn npc_editing(&self)->bool {
        self.kind==CatalogKind::Npc && self.npc_inspector==NpcInspectorTab::Edit
            && !self.strings_open && !self.xdt_open && self.world_open.is_none()
    }
    pub(super) fn new(catalog: &EditorCatalog) -> Self {
        let selected = catalog
            .entries
            .iter()
            .position(|entry| entry.semantic_id == "npc/npc_dexter")
            .unwrap_or_default();
        let mut state = Self {
            reveal_selection: false,
            kind: catalog.entries[selected].kind,
            selected,
            search: String::new(),
            search_focused: false,
            strings_open: false,
            xdt_open: false,
            missions_open: false,
            world_open: None,
            viewer_tabs: BTreeMap::new(),
            details_open: false,
            npc_inspector: NpcInspectorTab::Details,
            equipment_female: false,
            equipment_category: None,
            animation_page: 0,
            clip_index: 0,
            pose_mode: EditorPoseMode::Default,
            paused: true,
            looping: true,
            speed: 1.0,
            turntable: false,
            playback_revision: 1,
        };
        state.choose_default_clip(catalog);
        state
    }

    pub(super) fn filtered(&self, catalog: &EditorCatalog) -> Vec<usize> {
        let needle = self.search.trim().to_lowercase();
        catalog
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.kind == self.kind)
            .filter(|(_, entry)| {
                self.kind != CatalogKind::Equipment
                    || self.equipment_category.is_none_or(|category| {
                        entry
                            .semantic_id
                            .starts_with(&format!("equipment/{}/", category_slug(category)))
                    })
            })
            .filter(|(_, entry)| {
                needle.is_empty()
                    || entry.display_name.to_lowercase().contains(&needle)
                    || entry.logical_name.to_lowercase().contains(&needle)
                    || entry.semantic_id.to_lowercase().contains(&needle)
                    || entry
                        .network_id
                        .is_some_and(|id| id.to_string().contains(&needle))
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(super) fn select(&mut self, catalog: &EditorCatalog, index: usize) {
        if self.selected == index {
            return;
        }
        self.selected = index;
        self.animation_page = 0;
        self.pose_mode = EditorPoseMode::Default;
        self.paused = true;
        self.choose_default_clip(catalog);
        self.bump_playback_revision();
    }

    pub(super) fn select_relative_entry(&mut self, catalog: &EditorCatalog, down: bool) {
        let filtered = self.filtered(catalog);
        if filtered.is_empty() {
            return;
        }
        let position = filtered.iter().position(|&index| index == self.selected);
        let next = match position {
            Some(position) if down => (position + 1).min(filtered.len() - 1),
            Some(position) => position.saturating_sub(1),
            None if down => 0,
            None => filtered.len() - 1,
        };
        self.select(catalog, filtered[next]);
        self.reveal_selection = true;
    }

    pub(super) fn choose_default_clip(&mut self, catalog: &EditorCatalog) {
        let animations = &catalog.entries[self.selected].animations;
        self.clip_index = animations
            .iter()
            .position(|clip| clip == "stand1")
            .unwrap_or_default()
            .min(animations.len().saturating_sub(1));
        self.animation_page = self.clip_index / ANIMATION_SLOTS;
    }

    pub(super) fn activate_default_pose(&mut self, catalog: &EditorCatalog) {
        self.choose_default_clip(catalog);
        self.pose_mode = EditorPoseMode::Default;
        self.paused = true;
        self.bump_playback_revision();
    }

    pub(super) fn activate_t_pose(&mut self) {
        self.pose_mode = EditorPoseMode::TPose;
        self.paused = true;
        self.bump_playback_revision();
    }

    pub(super) fn select_clip(&mut self, catalog: &EditorCatalog, index: usize) {
        if index >= catalog.entries[self.selected].animations.len()
            || (self.clip_index == index && self.pose_mode == EditorPoseMode::Clip)
        {
            return;
        }
        self.clip_index = index;
        self.pose_mode = EditorPoseMode::Clip;
        self.paused = false;
        self.bump_playback_revision();
    }

    pub(super) fn toggle_playback(&mut self, catalog: &EditorCatalog) {
        if catalog.entries[self.selected].animations.is_empty() {
            return;
        }
        if self.pose_mode != EditorPoseMode::Clip {
            self.pose_mode = EditorPoseMode::Clip;
            self.paused = false;
            self.bump_playback_revision();
        } else {
            self.paused = !self.paused;
        }
    }

    pub(super) fn bump_playback_revision(&mut self) {
        self.playback_revision = self.playback_revision.wrapping_add(1).max(1);
    }
}

#[derive(Debug, Default, Resource)]
pub(super) struct EditorRuntimeStatus {
    pub(super) error: Option<String>,
    pub(super) ready: bool,
    pub(super) animation_time: f32,
    pub(super) animation_duration: f32,
}

pub(super) fn update_runtime_status(
    prepared: Res<PreparedAnimations>,
    clips: Res<Assets<AnimationClip>>,
    gltfs: Res<Assets<Gltf>>,
    preview: Res<ModelPreview>,
    players: Query<(&AnimationPlayer, &EditorAnimationApplied)>,
    scene_statuses: Query<&LegacyCharacterSceneStatus>,
    hnpc_statuses: Query<&NetworkHnpcRigAppearanceStatus0104>,
    deferred_reveals: Query<(), With<LegacyCharacterSceneDeferredReveal>>,
    material_errors: Query<&LegacyMaterialMetadataError>,
    mut status: ResMut<EditorRuntimeStatus>,
) {
    status.error = material_errors.iter().next().map(|error| error.0.clone());
    if status.error.is_none() {
        status.error = scene_statuses.iter().find_map(|scene| match scene {
            LegacyCharacterSceneStatus::Blocked(reason) => Some(format!("{reason:?}")),
            _ => None,
        });
    }
    if status.error.is_none() {
        status.error = hnpc_statuses.iter().find_map(|state| match state {
            NetworkHnpcRigAppearanceStatus0104::Blocked(error) => Some(error.clone()),
            _ => None,
        });
    }
    status.ready = status.error.is_none()
        && prepared
            .0
            .as_ref()
            .is_some_and(|value| value.generation == preview.generation)
        && (scene_statuses
            .iter()
            .any(|scene| matches!(scene, LegacyCharacterSceneStatus::Ready { .. }))
            || hnpc_statuses
                .iter()
                .any(|state| matches!(state, NetworkHnpcRigAppearanceStatus0104::Ready)))
        && deferred_reveals.is_empty();
    status.animation_time = 0.0;
    status.animation_duration = 0.0;
    for (player, applied) in &players {
        if applied.generation != preview.generation {
            continue;
        }
        if let Some(active) = player.animation(applied.node) {
            status.animation_time = active.seek_time();
        }
        let Some(gltf) = preview.gltf.as_ref().and_then(|handle| gltfs.get(handle)) else {
            continue;
        };
        if let Some((_, clip_handle)) = gltf.named_animations.iter().find(|(_, handle)| {
            prepared
                .0
                .as_ref()
                .and_then(|set| set.nodes.iter().find(|(_, node)| **node == applied.node))
                .is_some_and(|(name, _)| gltf.named_animations.get(name.as_str()) == Some(*handle))
        }) && let Some(clip) = clips.get(clip_handle)
        {
            status.animation_duration = clip.duration();
        }
        break;
    }
}

pub(super) fn reveal_keyboard_selection(
    mut state: ResMut<EditorState>,
    catalog: Res<EditorCatalog>,
    mut viewport: Single<(&mut ScrollPosition, &ComputedNode), With<CatalogScroll>>,
) {
    if !state.reveal_selection {
        return;
    }
    if viewport.1.size().y <= 0.0 {
        return;
    }
    state.reveal_selection = false;
    let filtered = state.filtered(&catalog);
    let Some(row) = filtered.iter().position(|&index| index == state.selected) else {
        return;
    };
    let (scroll, computed) = &mut *viewport;
    let height = computed.size().y * computed.inverse_scale_factor();
    let top = row as f32 * (CATALOG_ROW_HEIGHT + CATALOG_ROW_GAP);
    let bottom = top + CATALOG_ROW_HEIGHT;
    let next = if top < scroll.y {
        top
    } else if bottom > scroll.y + height {
        (bottom - height).max(0.0)
    } else {
        scroll.y
    };
    if scroll.y != next {
        scroll.y = next;
    }
}
