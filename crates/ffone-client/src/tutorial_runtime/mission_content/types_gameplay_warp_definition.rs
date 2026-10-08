use super::*;

pub type TutorialMissionContentResult<T> = Result<T, TutorialMissionContentError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialTableSetProvenance {
    pub asset_path: String,
    pub source_path: String,
    pub bytes: u64,
    pub blake3: String,
    pub schema: String,
    pub table_key: String,
    pub table_name: String,
}

/// Exact `m_iHMissionType` contract consumed by
/// `cnMissionJournal.SortMission`: 1 Guide, 2 Nano, 3 Normal/world.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum TutorialMissionType {
    Guide = 1,
    Nano = 2,
    World = 3,
}

impl TutorialMissionType {
    #[must_use]
    pub const fn xdt_value(self) -> i32 {
        self as i32
    }

    /// MessageData calls the Normal/world category "Quest".
    #[must_use]
    pub const fn legacy_label(self) -> &'static str {
        match self {
            Self::Guide => "Guide",
            Self::Nano => "Nano",
            Self::World => "Quest",
        }
    }
}

impl TryFrom<i32> for TutorialMissionType {
    type Error = TutorialMissionContentError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Guide),
            2 => Ok(Self::Nano),
            3 => Ok(Self::World),
            _ => Err(invalid(format!(
                "unsupported XDT m_iHMissionType {value}; expected 1 Guide, 2 Nano, or 3 Normal/world"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialMissionRowProvenance {
    pub row_index: usize,
    pub task_id: i32,
    pub mission_id: i32,
    pub title_string_id: i32,
    pub objective_string_id: i32,
    pub mission_type: i32,
    pub marker_visibility: MissionMarkerVisibility,
    /// XDT `m_iSTNanoID`; `cnMissionJournal` uses this as the Nano table
    /// index for mission type 2.
    pub nano_id: i32,
    pub journal_row_id: i32,
    pub required_level: i32,
    pub required_level_max: i32,
    pub repeat_flag: i32,
    /// Exact four-way completed-mission ambient bark string IDs.
    pub barker_text_ids: [i32; 4],
    pub required_missions: [i32; 2],
    pub required_nanos: [i32; 5],
    pub required_guide: i32,
    pub start_item_ids: [i32; 3],
    pub start_item_counts: [i32; 3],
    /// Exact clean `SetMissionMessage` inputs for TASK_START_SUCC.
    pub start_message_npc_type: i32,
    pub start_message_type: i32,
    pub start_message_string_id: i32,
    pub trigger_task_id: i32,
    pub grant_timer: i32,
    pub completion_check_timer: i32,
    pub completion_enemy_ids: [i32; 3],
    pub completion_enemy_counts: [i32; 3],
    pub completion_item_ids: [i32; 3],
    pub completion_item_counts: [i32; 3],
    /// XDT `m_iCSUDEFNPCID`; clean task type 6 uses this table ID for
    /// escort proximity and the escort runtime-ID request field.
    pub escort_def_npc_type: i32,
    /// XDT `m_iRequireInstanceID`; leaving such an instance is an explicit
    /// mission-failure edge, not a normal completion condition.
    pub required_instance_id: i32,
    pub difficulty_type: i32,
    pub journal_npc_type: i32,
    pub start_npc_type: i32,
    pub terminator_npc_type: i32,
    /// XDT `m_iSTGrantWayPoint`; Retrobution uses this independently from the
    /// journal/start/terminator NPCs for its selected-mission ES668 marker.
    pub grant_waypoint_npc_type: i32,
    pub task_type: i32,
    pub outgoing_task_id: i32,
    /// Exact clean `SetMissionMessage` inputs for TASK_END_SUCC.
    pub success_message_npc_type: i32,
    pub success_message_type: i32,
    pub success_message_string_id: i32,
    /// Exact XDT `m_iFOutgoingTask`. Clean starts this task after END_FAIL
    /// codes 1, 11, or 12; it is independent from the success chain and may
    /// legitimately resolve to a task owned by another mission.
    pub failure_outgoing_task_id: i32,
    /// Exact clean `SetMissionMessage` inputs for terminal TASK_END_FAIL
    /// codes 1, 11, and 12.
    pub failure_message_npc_type: i32,
    pub failure_message_type: i32,
    pub failure_message_string_id: i32,
    pub reward_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialMissionJournalProvenance {
    pub row_index: usize,
    pub offer_description_string_id: i32,
    pub active_task_description_string_id: i32,
    pub mission_summary_string_id: i32,
    pub mission_complete_summary_string_id: i32,
    pub completion_description_string_id: i32,
}

/// All mission-journal strings remain separate so each legacy UI mode can
/// consume the same field that Retrobution consumed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialMissionJournalText {
    /// XDT `m_iDetaileMissionDesc`, shown by Allow.
    pub offer_description: String,
    /// XDT `m_iDetailedTaskDesc`, appended to the current objective by Active.
    pub active_task_description: String,
    /// XDT `m_iMissionSummary`.
    pub mission_summary: String,
    /// XDT `m_iMissionCompleteSummary`.
    pub mission_complete_summary: String,
    /// XDT `m_iDetaileMissionCompleteSummary`, shown by Reward.
    pub completion_description: String,
    pub provenance: TutorialMissionJournalProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialMissionDefinition {
    pub title: String,
    pub objective: String,
    pub mission_type: TutorialMissionType,
    pub journal: TutorialMissionJournalText,
    pub start_nanocom_message: Option<TutorialMissionNanocomMessage>,
    pub success_nanocom_message: Option<TutorialMissionNanocomMessage>,
    pub failure_nanocom_message: Option<TutorialMissionNanocomMessage>,
    pub start_dialogue: Option<TutorialMissionDialogue>,
    pub success_dialogue: Option<TutorialMissionDialogue>,
    pub failure_dialogue: Option<TutorialMissionDialogue>,
    pub provenance: TutorialMissionRowProvenance,
}

/// NPC-owned quest speech, independent of NanoCom message flags and portraits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialMissionDialogue {
    pub npc_type: i32,
    pub string_id: i32,
    pub text: String,
}

/// Source-owned passive type-9 NanoCom copy attached to one mission edge.
/// The `message_type & 2` gate is applied while loading TableData, matching
/// clean `cnMissionManager.SetMissionMessage`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialMissionNanocomMessage {
    pub npc_type: i32,
    pub message_type: i32,
    pub string_id: i32,
    pub text: String,
}

/// One clean `GuideTableElement` projected for the login NanoCom producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayGuideNanocomDefinition {
    pub raw_mentor: i16,
    pub npc_type: i32,
    pub login_mail_string_id: i32,
    pub login_mail_text: String,
    pub login_no_mail_string_id: i32,
    pub login_no_mail_text: String,
    pub level_up_string_id: i32,
    pub level_up_text: String,
}

impl TutorialMissionDefinition {
    #[must_use]
    pub fn offer_description(&self) -> &str {
        &self.journal.offer_description
    }

    /// `cnMissionJournal` renders the current objective, two line breaks, then
    /// `m_iDetailedTaskDesc` for an active task.
    #[must_use]
    pub fn active_description(&self) -> String {
        format!(
            "{}\n\n{}",
            self.objective, self.journal.active_task_description
        )
    }

    #[must_use]
    pub fn completion_description(&self) -> &str {
        &self.journal.completion_description
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TutorialRewardItem {
    pub item_type: i32,
    pub item_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialRewardDefinition {
    pub reward_id: i32,
    pub cash: i32,
    pub fusion_matter: i32,
    pub box1_choice: i32,
    pub box2_choice: i32,
    pub primary_items: [TutorialRewardItem; 4],
    pub secondary_items: [TutorialRewardItem; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialSceneTextProvenance {
    pub event_row_index: usize,
    pub element_index: usize,
    pub event: i32,
    pub line: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialSceneTextDefinition {
    pub text: String,
    pub provenance: TutorialSceneTextProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialNpcRowProvenance {
    pub row_index: usize,
    /// XDT `m_iNpcNumber`; the legacy runtime usually calls this the NPC type.
    pub npc_type: i32,
    /// XDT category field `m_iNpcType`.
    pub npc_class: i32,
    pub name_string_id: i32,
    pub name_row_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialNpcDefinition {
    pub name: String,
    pub provenance: TutorialNpcRowProvenance,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TutorialWarpTarget {
    pub map_id: i32,
    /// Raw XDT/server position. X/Y form the horizontal plane and Z is height;
    /// this must be converted through `ProtocolPosition`, not as a Unity Vec3.
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialWarpRowProvenance {
    pub row_index: usize,
    pub npc_type: i32,
    pub warp_id: i32,
    pub raw_limit_task_id: i32,
    pub mission_id: i32,
    pub is_instance: i32,
    pub cost: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialWarpDefinition {
    /// Resolved through `m_pNpcData.m_iNpcName -> m_pNpcStringData`.
    pub label: String,
    pub target: TutorialWarpTarget,
    pub required_task_id: Option<i32>,
    pub provenance: TutorialWarpRowProvenance,
}

/// Runtime-independent projection of one exact `m_pWarpData` row, indexed by
/// `m_iWarpNumber`. Unlike [`TutorialWarpDefinition`], this retains the full
/// normal-world gate and destination contract and does not require the source
/// NPC to belong to the tutorial-only catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplayWarpDefinition {
    pub row_index: usize,
    pub warp_id: i32,
    pub npc_type: i32,
    pub target: TutorialWarpTarget,
    pub warp_group_type: i32,
    pub limit_level: i32,
    pub limit_task_id: i32,
    pub limit_item_id: i32,
    pub limit_item_type: i32,
    pub limit_use_item_id: i32,
    pub limit_use_item_type: i32,
    pub mission_id: i32,
    pub is_instance: i32,
    pub cost: i32,
}

/// Client/Unity world position for clean group-distance gating.
///
/// Axes are `x/z` horizontal and `y` vertical. The NPC side comes from the
/// source NPC transform, while group-roster coordinates must enter through
/// [`GameplayWarpServerPosition::to_client`] so the clean axis swap and scale
/// cannot be guessed at the call site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameplayWarpWorldPosition {
    pub map_number: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl GameplayWarpWorldPosition {
    #[must_use]
    pub const fn new(map_number: i32, x: f32, y: f32, z: f32) -> Self {
        Self {
            map_number,
            x,
            y,
            z,
        }
    }

    pub(super) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

/// Lossless protocol/group-roster position consumed by clean `CoordUtil`.
///
/// `ServerToClient` maps `(server X, server Y, server Z)` to
/// `(client x, client y, client z) = (X, Z, Y) * 0.01`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplayWarpServerPosition {
    pub map_number: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GameplayWarpServerPosition {
    #[must_use]
    pub const fn new(map_number: i32, x: i32, y: i32, z: i32) -> Self {
        Self {
            map_number,
            x,
            y,
            z,
        }
    }

    /// Exact clean `CoordUtil.ServerToClient` projection.
    #[must_use]
    pub fn to_client(self) -> GameplayWarpWorldPosition {
        GameplayWarpWorldPosition::new(
            self.map_number,
            self.x as f32 * 0.01,
            self.z as f32 * 0.01,
            self.y as f32 * 0.01,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameplayWarpGroupMember {
    pub pc_uid: i64,
    pub position: GameplayWarpServerPosition,
}

/// Authoritative non-TableData inputs consumed by one normal NPC warp click.
///
/// `limit_task_is_active` is exactly
/// `cnMissionManager.IsExistTaskInActiveMission(limit_task_id)`: completed
/// tasks do not satisfy it. `active_mission_ids` supplies the separate clean
/// same-mission fallback that can clear message 114.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameplayWarpEligibilityInput<'a> {
    /// Runtime NPC entity ID written to `iNPC_ID`, not the TableData NPC type.
    pub runtime_npc_id: i32,
    pub cash: i32,
    pub level: i32,
    pub limit_task_is_active: bool,
    pub active_mission_ids: &'a [i32],
    pub local_pc_uid: i64,
    pub npc_position: GameplayWarpWorldPosition,
    pub group_members: &'a [GameplayWarpGroupMember],
}

/// Clean SystemMessage row selected by a failed normal NPC warp gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum GameplayWarpDenial {
    InsufficientCash = 110,
    LevelTooLow = 111,
    MissingLimitItem = 112,
    MissingUseItem = 113,
    TaskOrMissionLocked = 114,
    GroupMemberTooFar = 115,
}

impl GameplayWarpDenial {
    #[must_use]
    pub const fn system_message_id(self) -> i32 {
        self as i32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameplayWarpEligibility {
    Ready(PcWarpUseNpcRequest0104),
    Denied(GameplayWarpDenial),
}

impl GameplayWarpDefinition {
    /// Resolves the clean normal-world gates and exact 0104 request fields.
    /// This does not play the 1.5-second effect and never sends or teleports.
    ///
    /// Evidence ledger (2026-08-07): primary container `main.unity3d`,
    /// 7,000,415 bytes, SHA-256
    /// `59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F`;
    /// object `Assembly - CSharp.dll`, reusable extraction
    /// `patched/cache/extracted-bundles/ed793e024e70bdfc/Assembly - CSharp.dll`,
    /// 1,517,568 bytes, SHA-256
    /// `33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB`.
    /// Decompiled with repository FFSpy 7.2.0 (`ilspycmd -t NpcIconMode`,
    /// `-t cnMissionManager`, and `-t CoordUtil`) and traced through
    /// `WarpOK`, `ReceiveCheckToActiveTask`, and `ServerToClient`; no
    /// intentional gate divergence.
    pub fn resolve_normal_eligibility(
        &self,
        input: GameplayWarpEligibilityInput<'_>,
        mut find_item_slot: impl FnMut(GameplayWarpItemLookup) -> Option<i32>,
    ) -> Result<GameplayWarpEligibility, GameplayWarpResolveError> {
        if input.cash < self.cost {
            return Ok(GameplayWarpEligibility::Denied(
                GameplayWarpDenial::InsufficientCash,
            ));
        }
        if input.level < self.limit_level {
            return Ok(GameplayWarpEligibility::Denied(
                GameplayWarpDenial::LevelTooLow,
            ));
        }

        let item_location = if self.limit_item_type == 8 {
            GameplayWarpInventoryLocation::Quest
        } else {
            GameplayWarpInventoryLocation::Inventory
        };
        let mut item_slot_1 = 0;
        if self.limit_item_id != 0 {
            item_slot_1 = find_item_slot(GameplayWarpItemLookup {
                item_location,
                start_slot: 0,
                item_id: self.limit_item_id,
                item_type: self.limit_item_type,
            })
            .unwrap_or(-1);
            if item_slot_1 < 0 {
                return Ok(GameplayWarpEligibility::Denied(
                    GameplayWarpDenial::MissingLimitItem,
                ));
            }
        }

        let mut item_slot_2 = 0;
        if self.limit_use_item_id != 0 {
            // Source quirk: `WarpOK` selects this location from
            // `m_iLimit_ItemType`, not `m_iLimit_UseItemType`.
            item_slot_2 = find_item_slot(GameplayWarpItemLookup {
                item_location,
                start_slot: 0,
                item_id: self.limit_use_item_id,
                item_type: self.limit_use_item_type,
            })
            .unwrap_or(-1);
            if item_slot_2 < 0 {
                return Ok(GameplayWarpEligibility::Denied(
                    GameplayWarpDenial::MissingUseItem,
                ));
            }
        }

        if self.limit_task_id != 0
            && !input.limit_task_is_active
            && !input.active_mission_ids.contains(&self.mission_id)
        {
            return Ok(GameplayWarpEligibility::Denied(
                GameplayWarpDenial::TaskOrMissionLocked,
            ));
        }

        if self.warp_group_type == 1 {
            if !input.npc_position.is_finite() {
                return Err(GameplayWarpResolveError::NonFiniteNpcPosition {
                    map_number: input.npc_position.map_number,
                });
            }
            for member in input.group_members {
                if member.pc_uid == input.local_pc_uid {
                    continue;
                }
                let member_position = member.position.to_client();
                let delta_x = input.npc_position.x - member_position.x;
                let delta_z = input.npc_position.z - member_position.z;
                if (delta_x * delta_x + delta_z * delta_z).sqrt() > 5.0 {
                    return Ok(GameplayWarpEligibility::Denied(
                        GameplayWarpDenial::GroupMemberTooFar,
                    ));
                }
            }
        }

        Ok(GameplayWarpEligibility::Ready(PcWarpUseNpcRequest0104 {
            npc_id: input.runtime_npc_id,
            warp_id: self.warp_id,
            e_il_1: 4,
            item_slot_1,
            e_il_2: 4,
            item_slot_2,
        }))
    }
}

/// Runtime-independent UI projection for every positive primary
/// `m_pNpcData.m_iNpcNumber`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XdtF32(pub(super) u32);

impl XdtF32 {
    pub(super) fn from_value(value: f32) -> Self {
        Self(value.to_bits())
    }

    #[must_use]
    pub fn value(self) -> f32 {
        f32::from_bits(self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayNpcUiDefinition {
    pub npc_type: i32,
    pub name: String,
    /// Exact `m_iNpcName` row identity. `NpcIconMode.NpcGreetingBubble`
    /// resolves its click greeting through this same string row.
    pub greeting_string_id: i32,
    /// Exact `m_pNpcStringData[m_iNpcName].m_strComment` click greeting.
    pub greeting: String,
    /// Exact optional `m_iBarkerNumber` row and its four random lines.
    pub barker: Option<GameplayNpcBarkerDefinition>,
    /// Exact `m_iTeam`; team `1` is the friendly/talkable branch.
    pub team: i32,
    /// Exact `m_iNpcLevel` rendered by `CnGuiMonster_info.DoWindowMob`.
    pub npc_level: i32,
    /// Exact `m_iNpcStyle` used by `cnVirtualServer.GetStyleEffect` and the
    /// target HUD's active-Nano matchup.
    pub npc_style: i32,
    /// Exact `m_iEffect`: the projectile type for ordinary attacks and the
    /// persistent `MakeGameIcon` effect for service NPC rows. Zero is the
    /// clean no-effect branch.
    pub attack_effect: i32,
    /// Exact service/category selector `m_iNpcType` used by `NpcIconMode`.
    pub service_category: i32,
    /// Exact optional `m_iServiceNumber`. The clean Rule route depends on the
    /// numeric value (`9` or `10`); the localized service string alone is not
    /// sufficient to select the correct RulesTable page.
    pub service_number: Option<i32>,
    /// Exact clean `NpcTableElement.m_iNpcType` used as the NPC-class gate by
    /// `NpcContainer.GetConeList`. The source has no separate
    /// `m_iNpcClass`; retaining this semantic alias prevents targeting code
    /// from depending on the UI-service field name.
    pub npc_class: i32,
    /// Exact `m_iAiType`; positive rows own NPC_INTERACTION open/close packets.
    pub ai_type: i32,
    /// Exact `m_iMesh` row identity. Compact/custom table fixtures may omit
    /// fields which are irrelevant to their UI-only projection; the complete
    /// production TableData always supplies this value.
    pub mesh_id: Option<i32>,
    /// Exact `m_fScale`, retained as source float bits so the catalog remains
    /// equality-comparable without rounding the authored value.
    pub table_scale: Option<XdtF32>,
    /// Exact `m_iAtkRange` in protocol centiunits.
    pub attack_range_server_units: Option<i32>,

    /// Exact `NpcStringTable.m_strComment2` selected by `m_iComment`.
    /// `cnTrans.Trans` passes this owner to `AvatarUtil.CallVoicePlay` for
    /// `eVoice.MoveOK`; keeping the semantic owner avoids constructing an
    /// English or Russian audio path in gameplay code.
    pub move_voice_owner: String,
    /// Exact `m_iRadius` in protocol centiunits. `NpcMoveController` converts
    /// this with `* 0.01` before `NpcContainer.GetConeList`.
    pub radius_server_units: i32,
    /// Exact `m_iHeight` in protocol centiunits.
    pub height_server_units: i32,
    /// Exact normal-world talk distance `m_iSightRange`, in protocol
    /// centiunits. The tutorial-only six-unit override is not applied here.
    pub sight_range_server_units: i32,
    /// Exact `m_pNpcData.m_iHP`. Zero is retained for noncombat rows and must
    /// not be converted into a fabricated renderable health bar.
    pub max_hp: i32,
}

/// Exact `NpcBarkerTableElement` retained in the random order used by clean
/// `NpcMoveController.Update`: name, comment, comment1, comment2.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayNpcBarkerDefinition {
    pub string_id: i32,
    pub lines: [String; 4],
}

/// Exact `SkillStringTable.m_strComment1` line selected by
/// `NpcMoveController.SkillReady` or `ReceiveCorruptionSkillReady`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayNpcSkillBarkerDefinition {
    pub string_id: i32,
    pub text: String,
}

/// Exact `cnGUINanocom.RenderMinimap` TableData inputs for one NPC type.
///
/// The gameplay minimap cannot infer its visibility branch from team or from
/// the icon artwork: clean code gates class-zero rows with `m_iSound` before
/// it considers `m_iMapIcon` or a mission replacement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplayNpcMinimapDefinition {
    pub npc_type: i32,
    pub npc_class: i32,
    pub sound: i32,
    pub map_icon: i32,
}

impl GameplayNpcUiDefinition {
    /// Returns a renderable group-health fraction only when TableData owns a
    /// positive maximum. `None` is the explicit gate for a noncombat zero.
    #[must_use]
    pub fn hp_fraction(&self, current_hp: i32) -> Option<f32> {
        if self.max_hp <= 0 {
            return None;
        }
        Some((f64::from(current_hp.max(0)) / f64::from(self.max_hp)).clamp(0.0, 1.0) as f32)
    }

    #[must_use]
    pub fn radius(&self) -> f32 {
        self.radius_server_units as f32 * 0.01
    }

    #[must_use]
    pub fn height(&self) -> f32 {
        self.height_server_units as f32 * 0.01
    }

    #[must_use]
    pub fn sight_range(&self) -> f32 {
        self.sight_range_server_units as f32 * 0.01
    }

    #[must_use]
    pub fn attack_range(&self) -> Option<f32> {
        self.attack_range_server_units
            .map(|range| range as f32 * 0.01)
    }
}

/// One exact `m_pXComData` row retained with its array index. The clean
/// resurrection request sends this row index, not `m_iXcomNumber`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplayXcomDefinition {
    pub row_index: i32,
    pub xcom_number: i32,
    pub zone: i32,
    pub position: [i32; 3],
}

/// Exact GeneralItem row fields consumed by QuickSlot, ResurrectMode and
/// UserEquip's targeted `GumPopup`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayGeneralItemUiDefinition {
    pub item_id: i16,
    /// Clean `GeneralItemElement.m_iItemType` (the semantic subtype, not the
    /// outer protocol `sItemType`, which is always 7 for this table).
    pub item_type: i32,
    pub link_skill: Option<i32>,
    /// Clean `GeneralItemElement.m_iStimPackAttri`. `GumPopup.EnableGum`
    /// compares this value with the equipped Nano's style before enabling
    /// that Nano's `GIVE` button.
    pub stim_pack_attribute: Option<i32>,
    /// Resolved through `m_iIcon -> m_pItemIconData`.
    pub icon_type: Option<u8>,
    pub icon_number: Option<u32>,
    /// Installed semantic route used by the native QuickSlot image.
    pub icon_path: Option<String>,
}

/// One exact clean `VendorTableScript` row. Source array order is retained
/// through `row_index`; `sort_number` is not a unique key in Retrobution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplayVendorItemDefinition {
    pub row_index: usize,
    pub npc_number: i32,
    pub sort_number: i32,
    pub item_type: i16,
    pub item_id: i16,
    pub sell_cost: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GameplayVendorItemMetadataDefinition {
    pub(super) text_key_prefix: String,
    pub(super) try_on_gender: i32,
    pub(super) try_on_guide: i32,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) level: i32,
    pub(super) buy_price: i32,
    pub(super) sell_price: i32,
    pub(super) sellable: bool,
    pub(super) general_item_type: Option<i32>,
    pub(super) battery_recharge: Option<i32>,
    pub(super) stack_size: Option<i32>,
    pub(super) icon_path: Option<String>,
    pub(super) point_rating: i32,
    pub(super) group_rating: i32,
    pub(super) defense_rating: i32,
    pub(super) equip_type: Option<i32>,
    pub(super) target_mode: Option<i32>,
    pub(super) rarity: Option<i32>,
    pub(super) tradeable: bool,
}
