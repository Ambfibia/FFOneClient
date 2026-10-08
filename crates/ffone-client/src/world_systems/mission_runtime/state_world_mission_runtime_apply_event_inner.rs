use super::*;

impl WorldMissionRuntime {


    pub(super) fn apply_event_inner(
        &mut self,
        event: &WorldMissionServerEvent0104,
        content: &TutorialMissionContent,
    ) -> Result<WorldMissionApplyOutcome, String> {
        let mut outcome = WorldMissionApplyOutcome::default();
        match event {
            WorldMissionServerEvent0104::TaskStartSuccess(reply) => {
                let correlated = self.pending_requests.iter().position(|pending| {
                    matches!(pending, PendingMissionUiRequest::TaskStart { task_id, .. } if *task_id == reply.task_id)
                });
                let definition = content
                    .mission(reply.task_id)
                    .map_err(|error| format!("unknown started task {}: {error}", reply.task_id))?;
                if let Some(index) = correlated {
                    self.pending_requests.remove(index);
                    outcome.ui_resolution =
                        Some(WorldMissionUiResolution::TaskStartAccepted(reply.task_id));
                }
                // Clean `InsertActivateTask` eliminates the current task from
                // this mission before installing the authoritative START row.
                // Reapplying the same task also refreshes its shared node's
                // InitializeBeforeStart/remaining-time state.
                self.remove_active_mission_tasks(definition.provenance.mission_id, content);
                self.active_tasks.push(WorldActiveMissionTask {
                    task_id: reply.task_id,
                    remaining_enemy_ids: definition.provenance.completion_enemy_ids,
                    remaining_enemy_counts: definition.provenance.completion_enemy_counts,
                    remaining_time_millis: (definition.provenance.grant_timer > 0)
                        .then_some(i64::from(reply.remaining_time).saturating_mul(1000)),
                });
                if !self.has_active_mission_id(self.selected_mission_id, content)
                    && !self.pending_requests.iter().any(|pending| {
                        matches!(pending, PendingMissionUiRequest::TaskStart { .. })
                            && content.mission(pending.task_id()).is_ok_and(|task| {
                                Some(task.provenance.mission_id) == self.selected_mission_id
                            })
                    })
                {
                    self.selected_mission_id = Some(definition.provenance.mission_id);
                }
                self.dialogue_edges
                    .push((reply.task_id, WorldMissionDialogueEdge::Start));
            }
            WorldMissionServerEvent0104::TaskStartFailure(reply) => {
                let index = self.require_pending_task(reply.task_id, true)?;
                self.pending_requests.remove(index);
                outcome.ui_resolution =
                    Some(WorldMissionUiResolution::RequestRejected(reply.task_id));
            }
            WorldMissionServerEvent0104::TaskEndSuccess(reply) => {
                let correlated = self.pending_requests.iter().position(|pending| {
                    matches!(pending, PendingMissionUiRequest::QuestEnd { task_id, .. } if *task_id == reply.task_id)
                });
                let automatic = self.automatic_end_task_ids.remove(&reply.task_id);
                let definition = content
                    .mission(reply.task_id)
                    .map_err(|error| format!("unknown ended task {}: {error}", reply.task_id))?;
                if !self
                    .active_tasks
                    .iter()
                    .any(|task| task.task_id == reply.task_id)
                {
                    return Err(format!(
                        "task-end success owns inactive task {}",
                        reply.task_id
                    ));
                }
                if let Some(index) = correlated {
                    self.pending_requests.remove(index);
                    if !automatic {
                        outcome.ui_resolution =
                            Some(WorldMissionUiResolution::QuestEndAccepted(reply.task_id));
                    }
                }
                let mission_id = definition.provenance.mission_id;
                let is_final =
                    content
                        .is_final_serialized_task(reply.task_id)
                        .map_err(|error| {
                            format!("cannot classify ended task {}: {error}", reply.task_id)
                        })?;
                self.remove_active_mission_tasks(mission_id, content);
                // Clean removes a previous completion for this mission before
                // deciding whether this exact END row completes it again.
                self.completed_mission_ids.remove(&mission_id);
                self.completed_task_ids
                    .retain(|(completed_mission_id, _)| *completed_mission_id != mission_id);
                let completes_mission = is_final || definition.provenance.outgoing_task_id == 0;
                self.dialogue_edges.push((
                    reply.task_id,
                    if completes_mission {
                        WorldMissionDialogueEdge::Complete
                    } else {
                        WorldMissionDialogueEdge::Success
                    },
                ));
                outcome.completion_kind = Some(if completes_mission {
                    WorldMissionCompletionKind::Mission
                } else {
                    WorldMissionCompletionKind::Task
                });
                if completes_mission {
                    self.completed_mission_ids.insert(mission_id);
                    self.completed_task_ids.push((mission_id, reply.task_id));
                    self.set_repeat_completed(definition.provenance.repeat_flag)?;
                }
                // Clean still tries a positive outgoing row after an
                // IsFinalTask completion. A source-owned dangling ID is not a
                // reducer error: GetTask returns null and no packet is sent.
                if definition.provenance.outgoing_task_id > 0
                    && let Ok(outgoing) = content.mission(definition.provenance.outgoing_task_id)
                {
                    outcome.follow_up_start =
                        Some(self.queue_follow_up_start(outgoing.provenance.task_id));
                }
                if completes_mission && self.selected_mission_id == Some(mission_id) {
                    self.selected_mission_id = self.last_active_mission_id(content);
                }
            }
            WorldMissionServerEvent0104::TaskEndFailure(reply) => {
                let correlated = self.pending_requests.iter().position(|pending| {
                    matches!(pending, PendingMissionUiRequest::QuestEnd { task_id, .. } if *task_id == reply.task_id)
                });
                // OpenFusion fails instanced tasks proactively after login and
                // on instance departure. The loaded active task owns that event
                // even though the client has not requested its completion.
                let proactive = matches!(reply.error_code, 1 | 11 | 12)
                    && self
                        .active_tasks
                        .iter()
                        .any(|task| task.task_id == reply.task_id);
                if let Some(index) = correlated {
                    self.pending_requests.remove(index);
                } else if !proactive {
                    return Err(format!(
                        "uncorrelated task-end failure for task {}",
                        reply.task_id
                    ));
                }
                let automatic = self.automatic_end_task_ids.remove(&reply.task_id);
                let definition = content
                    .mission(reply.task_id)
                    .map_err(|error| format!("unknown failed task {}: {error}", reply.task_id))?;
                if reply.error_code == 13 {
                    self.inventory_full_notices.push(reply.task_id);
                }
                if matches!(reply.error_code, 1 | 11 | 12) {
                    self.dialogue_edges
                        .push((reply.task_id, WorldMissionDialogueEdge::Failure));
                    let mission_id = definition.provenance.mission_id;
                    self.remove_active_mission_tasks(mission_id, content);
                    let failure_outgoing = definition.provenance.failure_outgoing_task_id;
                    if failure_outgoing > 0 {
                        // Failure chains are independent from success mission
                        // ownership. Missing source rows fail closed exactly as
                        // clean GetTask(null): remove the failed task, send none.
                        if let Ok(outgoing) = content.mission(failure_outgoing) {
                            outcome.follow_up_start =
                                Some(self.queue_follow_up_start(outgoing.provenance.task_id));
                        }
                    } else if content
                        .is_final_serialized_task(reply.task_id)
                        .map_err(|error| {
                            format!("cannot classify failed task {}: {error}", reply.task_id)
                        })?
                        && self.selected_mission_id == Some(mission_id)
                    {
                        self.selected_mission_id = self.last_active_mission_id(content);
                    }
                }
                if correlated.is_some() && !automatic {
                    outcome.ui_resolution =
                        Some(WorldMissionUiResolution::RequestRejected(reply.task_id));
                }
            }
            WorldMissionServerEvent0104::TaskStopSuccess(reply) => {
                let ui_pending = self.pending_requests.iter().any(|pending| {
                    matches!(
                        pending,
                        PendingMissionUiRequest::TaskStop { task_id }
                            if *task_id == reply.task_id
                    )
                });
                let removed_mission_id = content
                    .mission(reply.task_id)
                    .ok()
                    .map(|definition| definition.provenance.mission_id);
                if let Some(mission_id) = removed_mission_id {
                    self.completed_mission_ids.remove(&mission_id);
                    self.completed_task_ids
                        .retain(|(completed_mission_id, _)| *completed_mission_id != mission_id);
                }
                let selected_was_removed = removed_mission_id
                    .is_some_and(|mission_id| self.selected_mission_id == Some(mission_id));
                // Stop replies are authoritative world state even when no UI
                // request is pending (for example another client/session edge).
                self.remove_active_task(reply.task_id, content);
                if selected_was_removed || self.active_tasks.is_empty() {
                    self.selected_mission_id = self.last_active_mission_id(content);
                }
                if ui_pending {
                    outcome.ui_resolution =
                        Some(WorldMissionUiResolution::TaskStopAccepted(reply.task_id));
                }
            }
            WorldMissionServerEvent0104::TaskStopFailure(_) => {
                let mut stop_requests =
                    self.pending_requests
                        .iter()
                        .enumerate()
                        .filter_map(|(index, pending)| match pending {
                            PendingMissionUiRequest::TaskStop { task_id } => {
                                Some((index, *task_id))
                            }
                            PendingMissionUiRequest::TaskStart { .. }
                            | PendingMissionUiRequest::QuestEnd { .. } => None,
                        });
                let Some((index, task_id)) = stop_requests.next() else {
                    return Ok(outcome);
                };
                if stop_requests.next().is_none() {
                    self.pending_requests.remove(index);
                    outcome.ui_resolution =
                        Some(WorldMissionUiResolution::RequestRejected(task_id));
                }
            }
            WorldMissionServerEvent0104::KillQuestNpc(reply) => {
                self.kill_progress_changed = true;
                for task in &mut self.active_tasks {
                    for index in 0..3 {
                        if task.remaining_enemy_ids[index] == reply.npc_type_id {
                            task.remaining_enemy_counts[index] =
                                task.remaining_enemy_counts[index].saturating_sub(1);
                        }
                    }
                }
            }
            // Inventory and currency owners apply this post-state before the
            // next END_SUCC. The mission reducer reads item progress live.
            WorldMissionServerEvent0104::RewardItem(_) => {
                self.quest_items_changed = true;
            }
            WorldMissionServerEvent0104::SetCurrentMission(reply) => {
                if self.pending_current_mission_id != Some(reply.mission_id) {
                    return Err(format!(
                        "uncorrelated set-current-mission reply {}",
                        reply.mission_id
                    ));
                }
                // Selection is applied at input time. A delayed acknowledgement
                // must retire its request even if END has removed that mission,
                // and must not overwrite a newer local selection.
                self.authoritative_current_mission_id = Some(reply.mission_id);
                self.pending_current_mission_id = None;
            }
        }
        Ok(outcome)
    }

    pub fn can_start_task(
        &self,
        definition: &TutorialMissionDefinition,
        player_level: i32,
        guide: i32,
        owned_nanos: &BTreeSet<i32>,
        quest_inventory: &[ItemBase0104],
        content: &TutorialMissionContent,
    ) -> bool {
        let row = &definition.provenance;
        if self.pending_requests.iter().any(|pending| {
            content.mission(pending.task_id()).is_ok_and(|pending| {
                pending.provenance.mission_id == definition.provenance.mission_id
            })
        }) {
            return false;
        }
        if row.repeat_flag == 0 {
            if self.completed_mission_ids.contains(&row.mission_id) {
                return false;
            }
        } else if self.repeat_completed(row.repeat_flag) {
            return false;
        }
        if self.has_active_mission(row.mission_id, content) {
            return false;
        }
        if row
            .required_missions
            .iter()
            .any(|mission| *mission > 0 && !self.completed_mission_ids.contains(mission))
        {
            return false;
        }
        if (row.required_level > 0 && player_level < row.required_level)
            || (row.required_level_max > 0 && player_level < row.required_level_max)
        {
            return false;
        }
        if row
            .required_nanos
            .iter()
            .any(|nano| *nano > 0 && !owned_nanos.contains(nano))
        {
            return false;
        }
        if row.required_guide > 0 && guide != row.required_guide {
            return false;
        }
        for index in 0..3 {
            if row.start_item_ids[index] > 0
                && quest_item_count(quest_inventory, row.start_item_ids[index])
                    < row.start_item_counts[index]
            {
                return false;
            }
        }
        row.trigger_task_id <= 0
            || self
                .active_tasks
                .iter()
                .any(|task| task.task_id == row.trigger_task_id)
    }

    pub fn can_complete_task(
        &self,
        definition: &TutorialMissionDefinition,
        quest_inventory: &[ItemBase0104],
    ) -> bool {
        let Some(active) = self
            .active_tasks
            .iter()
            .find(|task| task.task_id == definition.provenance.task_id)
        else {
            return false;
        };
        if definition.provenance.completion_check_timer > 0
            && active
                .remaining_time_millis
                .is_none_or(|remaining| remaining <= 0)
        {
            return false;
        }
        for index in 0..3 {
            if definition.provenance.completion_enemy_ids[index] > 0
                && active.remaining_enemy_counts[index] > 0
            {
                return false;
            }
            let item_id = definition.provenance.completion_item_ids[index];
            if item_id > 0
                && definition.provenance.completion_item_counts[index]
                    - quest_item_count(quest_inventory, item_id)
                    > 0
            {
                return false;
            }
        }
        true
    }

    pub fn npc_entries(
        &self,
        npc_type: i32,
        npc_id: i32,
        npc_position: impl Into<String>,
        player_level: i32,
        guide: i32,
        owned_nanos: &BTreeSet<i32>,
        quest_inventory: &[ItemBase0104],
        content: &TutorialMissionContent,
    ) -> Result<(Vec<MissionUiEntry>, Vec<MissionUiEntry>), String> {
        let position = npc_position.into();
        let mut available = Vec::new();
        let mut completed = Vec::new();
        let mut offered_missions = BTreeSet::new();
        for definition in content.missions() {
            if definition.provenance.start_npc_type == npc_type
                && offered_missions.insert(definition.provenance.mission_id)
                && self.can_start_task(
                    definition,
                    player_level,
                    guide,
                    owned_nanos,
                    quest_inventory,
                    content,
                )
            {
                available.push(
                    content
                        .mission_entry(definition.provenance.task_id, npc_id, position.clone())
                        .map_err(|error| error.to_string())?,
                );
            }
            if definition.provenance.terminator_npc_type == npc_type
                && self.can_complete_task(definition, quest_inventory)
            {
                completed.push(
                    content
                        .mission_entry(definition.provenance.task_id, npc_id, position.clone())
                        .map_err(|error| error.to_string())?,
                );
            }
        }
        available.sort_by_key(|entry| {
            content
                .mission_row_index(entry.task_id)
                .unwrap_or(usize::MAX)
        });
        completed.sort_by_key(|entry| {
            content
                .mission_row_index(entry.task_id)
                .unwrap_or(usize::MAX)
        });
        Ok((available, completed))
    }

    pub fn journal_entries(
        &self,
        content: &TutorialMissionContent,
    ) -> Result<(Vec<MissionUiEntry>, Vec<MissionUiEntry>), String> {
        let active = content
            .active_journal_entries(&self.active_task_ids(), "")
            .map_err(|error| error.to_string())?;
        let completed_task_ids = self.completed_task_ids().collect::<Vec<_>>();
        let completed = content
            .completed_journal_entries(&completed_task_ids, "")
            .map_err(|error| error.to_string())?;
        Ok((active, completed))
    }

    /// Exact `cnGUINanocom` source fields for the selected task. Formatting is
    /// deferred until after every TableData label has been localized.
    pub fn current_objective(
        &self,
        quest_inventory: &[ItemBase0104],
        content: &TutorialMissionContent,
    ) -> Result<Option<WorldMissionCurrentObjective>, String> {
        let Some(task_id) = self.selected_task_id(content) else {
            return Ok(None);
        };
        let definition = content
            .mission(task_id)
            .map_err(|error| error.to_string())?;
        let active = self
            .active_tasks
            .iter()
            .find(|task| task.task_id == task_id)
            .ok_or_else(|| format!("selected task {task_id} is not active"))?;
        let mut enemies = Vec::new();
        for index in 0..3 {
            let npc_type = definition.provenance.completion_enemy_ids[index];
            let needed = definition.provenance.completion_enemy_counts[index];
            if npc_type <= 0 || needed <= 0 {
                continue;
            }
            let name = content
                .gameplay_npc(npc_type)
                .map(|npc| npc.name.as_str())
                .ok_or_else(|| format!("mission task {task_id} has unknown enemy {npc_type}"))?;
            let complete = (needed - active.remaining_enemy_counts[index]).min(needed);
            enemies.push(WorldMissionObjectiveProgress {
                content_id: npc_type,
                name: name.to_owned(),
                complete,
                needed,
            });
        }
        let mut quest_items = Vec::new();
        for index in 0..3 {
            let item_id = definition.provenance.completion_item_ids[index];
            let needed = definition.provenance.completion_item_counts[index];
            if item_id <= 0 || needed <= 0 {
                continue;
            }
            let name = content.quest_item_name(item_id).ok_or_else(|| {
                format!("mission task {task_id} has unknown quest item {item_id}")
            })?;
            let complete = quest_item_count(quest_inventory, item_id).min(needed);
            quest_items.push(WorldMissionObjectiveProgress {
                content_id: item_id,
                name: name.to_owned(),
                complete,
                needed,
            });
        }
        Ok(Some(WorldMissionCurrentObjective {
            task_id,
            title: definition.title.clone(),
            objective: definition.objective.clone(),
            remaining_time_seconds: if definition.provenance.grant_timer > 0 {
                active
                    .remaining_time_millis
                    .map(|remaining| remaining / 1_000)
            } else {
                None
            },
            enemies,
            quest_items,
        }))
    }

    #[must_use]
    pub fn npc_has_available_or_completable_mission(
        &self,
        npc_type: i32,
        player_level: i32,
        guide: i32,
        owned_nanos: &BTreeSet<i32>,
        quest_inventory: &[ItemBase0104],
        content: &TutorialMissionContent,
    ) -> (bool, bool) {
        self.npc_mission_availability(npc_type,None,player_level,guide,owned_nanos,quest_inventory,content)
    }

    #[must_use]
    pub fn npc_mission_markers_on(
        &self,npc_type:i32,surface:MissionMarkerSurface,player_level:i32,guide:i32,
        owned_nanos:&BTreeSet<i32>,quest_inventory:&[ItemBase0104],content:&TutorialMissionContent,
    )->(bool,bool) {
        self.npc_mission_availability(npc_type,Some(surface),player_level,guide,owned_nanos,quest_inventory,content)
    }

    fn npc_mission_availability(
        &self,npc_type:i32,surface:Option<MissionMarkerSurface>,player_level:i32,guide:i32,
        owned_nanos:&BTreeSet<i32>,quest_inventory:&[ItemBase0104],content:&TutorialMissionContent,
    )->(bool,bool) {
        let mut available = false;
        let mut advance_available = false;
        for definition in content.missions_for_npc(npc_type) {
            if surface.is_some_and(|surface|!definition.provenance.marker_visibility.visible_on(surface)) {continue;}
            available |= definition.provenance.start_npc_type == npc_type
                && self.can_start_task(
                    definition,
                    player_level,
                    guide,
                    owned_nanos,
                    quest_inventory,
                    content,
                );
            // Clean `NPCTypeHasMissionAdvanceAvailable` matches an active task
            // to the NPC's terminate list; it does not require kills/items or
            // the completion timer to be ready yet.
            advance_available |= definition.provenance.terminator_npc_type == npc_type
                && self
                    .active_tasks
                    .iter()
                    .any(|active| active.task_id == definition.provenance.task_id);
        }
        (available, advance_available)
    }

    /// Computes the WorldMap New/Advance predicates for every mission NPC in
    /// one source-order pass.
    ///
    /// `WorldMapMode` projects thousands of `clientnpc.asset` placements,
    /// including duplicate NPC types. Calling the single-NPC reducer for each
    /// placement would rescan the complete mission table thousands of times
    /// per frame, so production builds this value catalog once and then keeps
    /// the clean per-placement draw order separate.
    #[must_use]
    pub fn mission_availability_by_npc(
        &self,
        player_level: i32,
        guide: i32,
        owned_nanos: &BTreeSet<i32>,
        quest_inventory: &[ItemBase0104],
        content: &TutorialMissionContent,
    ) -> BTreeMap<i32, (bool, bool)> {
        let mut availability = BTreeMap::<i32, (bool, bool)>::new();
        for definition in content.missions() {
            let row = &definition.provenance;
            if !row.marker_visibility.visible_on(MissionMarkerSurface::WorldMap) {continue;}
            if row.start_npc_type > 0
                && self.can_start_task(
                    definition,
                    player_level,
                    guide,
                    owned_nanos,
                    quest_inventory,
                    content,
                )
            {
                availability.entry(row.start_npc_type).or_default().0 = true;
            }
            if row.terminator_npc_type > 0
                && self
                    .active_tasks
                    .iter()
                    .any(|active| active.task_id == row.task_id)
            {
                availability.entry(row.terminator_npc_type).or_default().1 = true;
            }
        }
        availability
    }

    pub(super) fn require_pending_task(&self, task_id: i32, start: bool) -> Result<usize, String> {
        self.pending_requests
            .iter()
            .position(|pending| match pending {
                PendingMissionUiRequest::TaskStart {
                    task_id: pending, ..
                } => start && *pending == task_id,
                PendingMissionUiRequest::QuestEnd {
                    task_id: pending, ..
                } => !start && *pending == task_id,
                PendingMissionUiRequest::TaskStop { .. } => false,
            })
            .ok_or_else(|| {
                format!(
                    "uncorrelated task-{} reply for task {task_id}",
                    if start { "start" } else { "end" }
                )
            })
    }

    pub(super) fn queue_follow_up_start(&mut self, task_id: i32) -> PcTaskStartRequest0104 {
        let pending = PendingMissionUiRequest::TaskStart { task_id, npc_id: 0 };
        if !self.pending_requests.contains(&pending) {
            self.pending_requests.push(pending);
        }
        PcTaskStartRequest0104 {
            task_id,
            npc_id: 0,
            escort_npc_id: 0,
        }
    }

    pub(super) fn clear_task_runtime_state(&mut self, task_id: i32) {
        self.pending_requests
            .retain(|pending| pending.task_id() != task_id);
        self.automatic_end_task_ids.remove(&task_id);
        if self
            .pending_automatic_reward
            .is_some_and(|reward| reward.task_id == task_id)
        {
            self.pending_automatic_reward = None;
        }
    }

    pub(super) fn remove_active_mission_tasks(
        &mut self,
        mission_id: i32,
        content: &TutorialMissionContent,
    ) -> Vec<i32> {
        let removed = self
            .active_tasks
            .iter()
            .filter_map(|active| {
                content
                    .mission(active.task_id)
                    .ok()
                    .filter(|task| task.provenance.mission_id == mission_id)
                    .map(|_| active.task_id)
            })
            .collect::<Vec<_>>();
        let removed_set = removed.iter().copied().collect::<BTreeSet<_>>();
        self.active_tasks
            .retain(|active| !removed_set.contains(&active.task_id));
        for task_id in &removed {
            self.clear_task_runtime_state(*task_id);
        }
        removed
    }

    pub(super) fn remove_active_task(
        &mut self,
        task_id: i32,
        content: &TutorialMissionContent,
    ) -> Option<i32> {
        let removed_mission = self.active_tasks.iter().find_map(|active| {
            (active.task_id == task_id).then(|| {
                content
                    .mission(active.task_id)
                    .ok()
                    .map(|task| task.provenance.mission_id)
            })?
        });
        self.active_tasks.retain(|task| task.task_id != task_id);
        self.clear_task_runtime_state(task_id);
        removed_mission
    }

    pub(super) fn task_has_pending_checker(&self, task_id: i32) -> bool {
        self.pending_requests
            .iter()
            .any(|pending| pending.task_id() == task_id)
    }

    pub(super) fn has_active_mission(&self, mission_id: i32, content: &TutorialMissionContent) -> bool {
        self.active_tasks.iter().any(|active| {
            content
                .mission(active.task_id)
                .is_ok_and(|task| task.provenance.mission_id == mission_id)
        })
    }
}

impl WorldMissionRuntime {


    pub(super) fn has_active_mission_id(
        &self,
        mission_id: Option<i32>,
        content: &TutorialMissionContent,
    ) -> bool {
        mission_id.is_some_and(|mission_id| self.has_active_mission(mission_id, content))
    }

    pub(super) fn last_active_mission_id(&self, content: &TutorialMissionContent) -> Option<i32> {
        self.active_tasks.iter().rev().find_map(|active| {
            content
                .mission(active.task_id)
                .ok()
                .map(|task| task.provenance.mission_id)
        })
    }

    pub(super) fn repeat_completed(&self, repeat_flag: i32) -> bool {
        repeat_flag_position(repeat_flag).is_some_and(|(word, bit)| {
            self.repeat_flags
                .get(word)
                .is_some_and(|flags| ((*flags as u64) >> bit) & 1 != 0)
        })
    }

    pub(super) fn set_repeat_completed(&mut self, repeat_flag: i32) -> Result<(), String> {
        if repeat_flag <= 0 {
            return Ok(());
        }
        let (word, bit) = repeat_flag_position(repeat_flag)
            .ok_or_else(|| format!("repeat flag {repeat_flag} exceeds PC load capacity"))?;
        let Some(flags) = self.repeat_flags.get_mut(word) else {
            return Err(format!(
                "repeat flag {repeat_flag} exceeds PC load capacity"
            ));
        };
        *flags = ((*flags as u64) | (1_u64 << bit)) as i64;
        Ok(())
    }

}
