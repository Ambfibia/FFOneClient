use super::*;

#[cfg(test)]
mod gm_reset_tests {
    use super::*;
    #[test]
    fn reset_clears_only_the_requested_mission_and_its_finished_tasks() {
        let mut runtime=WorldMissionRuntime::default();
        runtime.completed_mission_ids.extend([1,2]);runtime.completed_task_ids.extend([(1,10),(2,20)]);
        runtime.accept_gm_mission_reset(1).unwrap();
        assert!(!runtime.completed_mission_ids.contains(&1));assert!(runtime.completed_mission_ids.contains(&2));assert_eq!(runtime.completed_task_ids.len(),1);
        assert!(runtime.accept_gm_mission_reset(0).is_err());assert!(runtime.accept_gm_mission_reset(2049).is_err());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct WorldMissionRuntime {
    pub(super) inventory_full_notices: Vec<i32>,
    pub(super) dialogue_edges: Vec<(i32, WorldMissionDialogueEdge)>,
    pub(super) active_tasks: Vec<WorldActiveMissionTask>,
    pub(super) completed_mission_ids: BTreeSet<i32>,
    /// Exact `(mission ID, cnMissionNode task ID)` insertion order retained by
    /// `m_CompletedMissionList`; replacing a completion removes then appends.
    pub(super) completed_task_ids: Vec<(i32, i32)>,
    pub(super) repeat_flags: [i64; PcLoadData0104::REPEAT_QUEST_FLAG_COUNT],
    pub(super) selected_mission_id: Option<i32>,
    pub(super) authoritative_current_mission_id: Option<i32>,
    pub(super) pending_requests: Vec<PendingMissionUiRequest>,
    pub(super) pending_current_mission_id: Option<i32>,
    pub(super) automatic_end_task_ids: BTreeSet<i32>,
    pub(super) pending_automatic_reward: Option<WorldMissionAutomaticReward>,
    pub(super) quest_items_changed: bool,
    pub(super) kill_progress_changed: bool,
}

impl WorldMissionRuntime {
    pub fn take_inventory_full_notices(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.inventory_full_notices)
    }
    pub fn take_dialogue_edges(&mut self) -> Vec<(i32, WorldMissionDialogueEdge)> {
        std::mem::take(&mut self.dialogue_edges)
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Administrative mission-flag acknowledgement. Active tasks are removed
    /// by the server's preceding normal task-stop replies; no rewards inferred.
    pub fn accept_gm_mission_completion(&mut self, mission_id: i32) -> Result<(), String> {
        if !(1..=1024).contains(&mission_id) {
            return Err("invalid GM mission ID".to_owned());
        }
        self.completed_mission_ids.insert(mission_id);
        Ok(())
    }

    /// Clear completion only after RustyFusion acknowledges /deletequest.
    pub fn accept_gm_mission_reset(&mut self,mission_id:i32)->Result<(),String> {
        if !(1..=2048).contains(&mission_id) {return Err("invalid GM mission ID".into());}
        self.completed_mission_ids.remove(&mission_id);
        self.completed_task_ids.retain(|(id,_)|*id!=mission_id);
        Ok(())
    }

    /// Rebuilds all mission state transactionally from the exact login blob.
    pub fn seed(
        &mut self,
        load: &PcLoadData0104,
        content: &TutorialMissionContent,
    ) -> Result<(), String> {
        let mut next = Self {
            repeat_flags: load.repeat_quest_flags(),
            ..Self::default()
        };

        // Clean `SetMissionAndTaskFlags` consumes only 16 of the 32 wire
        // words. Bit zero is mission 1, hence missionId = word*64 + bit + 1.
        for (word_index, word) in load.quest_flags().into_iter().take(16).enumerate() {
            let bits = word as u64;
            for bit in 0..64 {
                if bits & (1_u64 << bit) != 0 {
                    let mission_id = (word_index * 64 + bit + 1) as i32;
                    next.completed_mission_ids.insert(mission_id);
                    if let Ok(task) = content.final_serialized_task_for_mission_id(mission_id) {
                        next.completed_task_ids
                            .push((mission_id, task.provenance.task_id));
                    }
                }
            }
        }

        for running in load.running_quests() {
            if running.task_id <= 0 {
                continue;
            }
            let definition = content.mission(running.task_id).map_err(|error| {
                format!(
                    "PC load running task {} is absent from TableData: {error}",
                    running.task_id
                )
            })?;
            // Restore every authoritative task. OpenFusion persists tasks without
            // their slot index, so a Nano mission need not return in slot zero.
            let mut remaining_enemy_counts = definition.provenance.completion_enemy_counts;
            for (definition_index, definition_npc_id) in definition
                .provenance
                .completion_enemy_ids
                .iter()
                .copied()
                .enumerate()
            {
                for (wire_index, wire_npc_id) in running.kill_npc_ids.iter().copied().enumerate() {
                    if definition_npc_id == wire_npc_id {
                        remaining_enemy_counts[definition_index] =
                            running.remaining_kill_counts[wire_index];
                    }
                }
            }
            // `InsertActivateTask` replaces a different active task from the
            // same mission before inserting the restored row.
            next.active_tasks.retain(|active| {
                content.mission(active.task_id).map_or(true, |active| {
                    active.provenance.mission_id != definition.provenance.mission_id
                })
            });
            next.active_tasks.push(WorldActiveMissionTask {
                task_id: running.task_id,
                remaining_enemy_ids: definition.provenance.completion_enemy_ids,
                remaining_enemy_counts,
                remaining_time_millis: (definition.provenance.grant_timer > 0)
                    .then_some(CLEAN_PC_LOAD_TIMER_INITIAL_MILLIS),
            });
        }

        let current = load.current_mission_id();
        next.authoritative_current_mission_id = (current > 0).then_some(current);
        next.selected_mission_id = if current > 0 && next.has_active_mission(current, content) {
            Some(current)
        } else {
            next.last_active_mission_id(content)
        };
        *self = next;
        Ok(())
    }

    #[must_use]
    pub fn active_tasks(&self) -> &[WorldActiveMissionTask] {
        &self.active_tasks
    }

    #[must_use]
    pub fn active_task_ids(&self) -> Vec<i32> {
        self.active_tasks.iter().map(|task| task.task_id).collect()
    }

    #[must_use]
    pub fn selected_mission_id(&self) -> Option<i32> {
        self.selected_mission_id
    }

    #[must_use]
    pub fn selected_task_id(&self, content: &TutorialMissionContent) -> Option<i32> {
        let mission_id = self.selected_mission_id?;
        self.active_tasks.iter().find_map(|active| {
            content
                .mission(active.task_id)
                .ok()
                .filter(|task| task.provenance.mission_id == mission_id)
                .map(|_| active.task_id)
        })
    }

    #[must_use]
    pub fn completed_mission_ids(&self) -> &BTreeSet<i32> {
        &self.completed_mission_ids
    }

    /// Exact completed task nodes consumed by the clean periodic Barker scan.
    pub fn completed_task_ids(&self) -> impl ExactSizeIterator<Item = i32> + '_ {
        self.completed_task_ids.iter().map(|(_, task_id)| *task_id)
    }

    #[must_use]
    pub fn pending_request(&self) -> Option<PendingMissionUiRequest> {
        self.pending_requests.first().copied()
    }

    #[must_use]
    pub fn pending_automatic_reward(&self) -> Option<WorldMissionAutomaticReward> {
        self.pending_automatic_reward
    }

    /// Clears a reward intent only after the caller successfully opened the
    /// exact Reward UI represented by it. A stale acknowledgement cannot
    /// discard a newer task's durable intent.
    pub fn acknowledge_automatic_reward(&mut self, reward: WorldMissionAutomaticReward) -> bool {
        if self.pending_automatic_reward != Some(reward) {
            return false;
        }
        self.pending_automatic_reward = None;
        true
    }

    pub fn record_request(&mut self, request: PendingMissionUiRequest) -> Result<(), String> {
        if self.pending_requests.contains(&request) {
            return Err(format!(
                "world mission request for task {} is already pending",
                request.task_id()
            ));
        }
        self.pending_requests.push(request);
        Ok(())
    }

    pub fn cancel_request(&mut self, request: PendingMissionUiRequest) {
        self.pending_requests.retain(|pending| *pending != request);
        if matches!(request, PendingMissionUiRequest::QuestEnd { .. })
            && self.automatic_end_task_ids.remove(&request.task_id())
        {
            // A local codec/transport failure must allow event-driven clean
            // edges to retry once transport is available again.
            self.quest_items_changed = true;
            self.kill_progress_changed = true;
        }
    }

    /// Advances only the clean client-owned presentation timer.
    pub fn tick(&mut self, delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        let delta_millis = (delta_seconds * 1000.0).round() as i64;
        for task in &mut self.active_tasks {
            if let Some(remaining) = &mut task.remaining_time_millis {
                *remaining = remaining.saturating_sub(delta_millis);
            }
        }
    }

    /// Clean `NpcIconMode.CheckMissionSlot` runs only on ACCEPT. Offer
    /// visibility deliberately remains owned by [`Self::can_start_task`].
    pub fn check_accept_task(
        &self,
        task_id: i32,
        content: &TutorialMissionContent,
    ) -> Result<(), WorldMissionAcceptRejection> {
        let definition = content
            .mission(task_id)
            .map_err(|_| WorldMissionAcceptRejection::UnknownTask { task_id })?;
        let capacity = match definition.mission_type {
            TutorialMissionType::Guide | TutorialMissionType::Nano => 1,
            TutorialMissionType::World => 4,
        };
        let assigned = self
            .active_tasks
            .iter()
            .filter(|active| {
                content
                    .mission(active.task_id)
                    .is_ok_and(|active| active.mission_type == definition.mission_type)
            })
            .count();
        if assigned >= capacity {
            return Err(WorldMissionAcceptRejection::CategoryFull {
                mission_type: definition.mission_type,
                capacity,
            });
        }

        // Instanced (`m_iRequireInstanceID`) and escort/defence (type 6)
        // contracts are ordinary acceptable tasks. Both need a live owner this
        // client does not have to fabricate:
        //
        // * the shard assigns the DEF NPC its matching authored path on
        //   TASK_START, and `collect_automatic_end_requests` already runs the
        //   clean type-6 terminator/escort-proximity completion below;
        // * leaving the required instance is failed by the shard itself, which
        //   emits the authoritative TASK_STOP applied by `apply_server_event`.
        //
        // Failing them closed made every nano-mission chain unfinishable: each
        // one ends on an instanced task, so the awarded Nano, the level and the
        // Fusion Matter cap all stopped advancing.
        Ok(())
    }

    /// Reproduces the client-owned TASK_END edges from clean
    /// `cnMissionManager.Update`, `KillMissionNPC` and
    /// `CheckMissionItemCount`. Each request is registered before it is
    /// returned so a second frame cannot duplicate the clean task checker.
    pub fn collect_automatic_end_requests(
        &mut self,
        quest_inventory: &[ItemBase0104],
        player_position: Option<[f32; 3]>,
        nearby_npcs: &[WorldMissionNearbyNpc],
        content: &TutorialMissionContent,
    ) -> Result<WorldMissionAutomaticActions, String> {
        let kill_progress_changed = std::mem::take(&mut self.kill_progress_changed);
        let quest_items_changed = std::mem::take(&mut self.quest_items_changed);
        let active_tasks = self.active_tasks.clone();
        let mut queued_task_ids = BTreeSet::new();
        let mut requests = Vec::new();
        // Reward UI opening can be deferred by another modal/system popup.
        // Keep returning the exact intent until the presentation owner
        // explicitly acknowledges a successful open.
        let mut reward = self.pending_automatic_reward;

        // `Update`: every expired grant timer requests END with NPC/escort 0,
        // including PC-load rows restored with clean's exact -1-second state.
        for active in &active_tasks {
            if self.task_has_pending_checker(active.task_id) {
                continue;
            }
            let definition = content.mission(active.task_id).map_err(|error| {
                format!(
                    "active task {} is absent from TableData: {error}",
                    active.task_id
                )
            })?;
            if definition.provenance.grant_timer > 0
                && active
                    .remaining_time_millis
                    .is_some_and(|remaining| remaining <= 0)
            {
                queued_task_ids.insert(active.task_id);
                requests.push(task_end_request(active.task_id, 0, 0));
            }
        }

        // `KillMissionNPC`: after a kill packet, a rewardless task with at
        // least one kill requirement and no terminator completes immediately.
        if kill_progress_changed {
            for active in &active_tasks {
                if queued_task_ids.contains(&active.task_id)
                    || self.task_has_pending_checker(active.task_id)
                {
                    continue;
                }
                let definition = content.mission(active.task_id).map_err(|error| {
                    format!(
                        "active task {} is absent from TableData: {error}",
                        active.task_id
                    )
                })?;
                let required_kills = definition
                    .provenance
                    .completion_enemy_counts
                    .iter()
                    .copied()
                    .sum::<i32>();
                if definition.provenance.terminator_npc_type <= 0
                    && required_kills > 0
                    && active
                        .remaining_enemy_counts
                        .iter()
                        .all(|count| *count <= 0)
                {
                    if definition.provenance.reward_id <= 0 {
                        queued_task_ids.insert(active.task_id);
                        requests.push(task_end_request(active.task_id, 0, 0));
                    } else if self.can_complete_task(definition, quest_inventory) {
                        if reward.is_none() {
                            reward = Some(WorldMissionAutomaticReward {
                                task_id: active.task_id,
                                npc_id: 0,
                                escort_npc_id: 0,
                            });
                        }
                        break;
                    }
                }
            }
        }

        // `CheckMissionItemCount` handles only the first qualifying active
        // task for one reward/item event (including the reward-UI branch).
        if quest_items_changed {
            for active in &active_tasks {
                if queued_task_ids.contains(&active.task_id)
                    || self.task_has_pending_checker(active.task_id)
                {
                    continue;
                }
                let definition = content.mission(active.task_id).map_err(|error| {
                    format!(
                        "active task {} is absent from TableData: {error}",
                        active.task_id
                    )
                })?;
                let required_items = definition
                    .provenance
                    .completion_item_counts
                    .iter()
                    .copied()
                    .sum::<i32>();
                let all_items_complete = (0..3).all(|index| {
                    let item_id = definition.provenance.completion_item_ids[index];
                    item_id <= 0
                        || quest_item_count(quest_inventory, item_id)
                            >= definition.provenance.completion_item_counts[index]
                });
                if definition.provenance.terminator_npc_type <= 0
                    && required_items > 0
                    && all_items_complete
                {
                    if definition.provenance.reward_id <= 0 {
                        queued_task_ids.insert(active.task_id);
                        requests.push(task_end_request(active.task_id, 0, 0));
                    } else if self.can_complete_task(definition, quest_inventory) {
                        if reward.is_none() {
                            reward = Some(WorldMissionAutomaticReward {
                                task_id: active.task_id,
                                npc_id: 0,
                                escort_npc_id: 0,
                            });
                        }
                    }
                    break;
                }
            }
        }

        // `Update`: task types 2 and 6 auto-complete when their terminator is
        // inside its own sight range and completion conditions pass. A reward
        // opens Journal Reward instead, so only rewardless rows send END.
        if let Some(player_position) = player_position {
            for active in &active_tasks {
                if queued_task_ids.contains(&active.task_id)
                    || self.task_has_pending_checker(active.task_id)
                {
                    continue;
                }
                let definition = content.mission(active.task_id).map_err(|error| {
                    format!(
                        "active task {} is absent from TableData: {error}",
                        active.task_id
                    )
                })?;
                if definition.provenance.terminator_npc_type <= 0
                    || !matches!(definition.provenance.task_type, 2 | 6)
                {
                    continue;
                }
                let Some(terminator) = nearby_npcs.iter().find(|npc| {
                    npc.npc_type == definition.provenance.terminator_npc_type
                        && distance(player_position, npc.position)
                            < npc.sight_range_server_units as f32 * 0.01
                }) else {
                    continue;
                };
                // Clean returns from this Update pass at the first in-sight
                // type-2/6 task whose completion condition is not ready.
                if !self.can_complete_task(definition, quest_inventory) {
                    break;
                }
                if definition.provenance.reward_id > 0 {
                    if reward.is_none() {
                        reward = Some(WorldMissionAutomaticReward {
                            task_id: active.task_id,
                            npc_id: terminator.npc_id,
                            escort_npc_id: 0,
                        });
                    }
                    break;
                }

                let mut escort_npc_id = 0;
                if definition.provenance.task_type == 6
                    && definition.provenance.escort_def_npc_type > 0
                    && let Some(escort) = nearby_npcs
                        .iter()
                        .find(|npc| npc.npc_type == definition.provenance.escort_def_npc_type)
                {
                    // The clean branch deliberately does not complete while
                    // a found escort is five or more Unity units away.
                    if distance(escort.position, terminator.position) >= 5.0 {
                        break;
                    }
                    escort_npc_id = escort.npc_id;
                }
                queued_task_ids.insert(active.task_id);
                requests.push(task_end_request(
                    active.task_id,
                    terminator.npc_id,
                    escort_npc_id,
                ));
                break;
            }
        }

        for request in &requests {
            let pending = PendingMissionUiRequest::QuestEnd {
                task_id: request.task_id,
                npc_id: request.npc_id,
                box1_choice: 0,
                box2_choice: 0,
            };
            self.record_request(pending)?;
            self.automatic_end_task_ids.insert(request.task_id);
        }
        if self.pending_automatic_reward.is_none() {
            self.pending_automatic_reward = reward;
        }
        Ok(WorldMissionAutomaticActions {
            end_requests: requests,
            reward: self.pending_automatic_reward,
        })
    }

    pub fn begin_select_task(
        &mut self,
        task_id: i32,
        content: &TutorialMissionContent,
    ) -> Result<Option<PcSetCurrentMissionId0104>, String> {
        let definition = content
            .mission(task_id)
            .map_err(|error| format!("cannot select unknown mission task {task_id}: {error}"))?;
        if !self.active_tasks.iter().any(|task| task.task_id == task_id) {
            return Err(format!("cannot select inactive mission task {task_id}"));
        }
        let mission_id = definition.provenance.mission_id;
        self.selected_mission_id = Some(mission_id);
        if self.authoritative_current_mission_id == Some(mission_id)
            || self.pending_current_mission_id == Some(mission_id)
        {
            return Ok(None);
        }
        if self.pending_current_mission_id.is_some() {
            return Err("a set-current-mission request is already pending".to_owned());
        }
        self.pending_current_mission_id = Some(mission_id);
        Ok(Some(PcSetCurrentMissionId0104 { mission_id }))
    }

    pub fn cancel_current_request(&mut self, mission_id: i32) {
        if self.pending_current_mission_id == Some(mission_id) {
            self.pending_current_mission_id = None;
        }
    }

    pub fn apply_event(
        &mut self,
        event: &WorldMissionServerEvent0104,
        content: &TutorialMissionContent,
    ) -> Result<WorldMissionApplyOutcome, String> {
        let mut next = self.clone();
        let outcome = next.apply_event_inner(event, content)?;
        *self = next;
        Ok(outcome)
    }
}
