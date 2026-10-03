use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayUserEquipItemDetail {
    pub name: String,
    pub description: String,
    pub level: i32,
    pub point_rating: i32,
    pub group_rating: i32,
    pub defense_rating: i32,
    pub equip_type: Option<i32>,
    pub target_mode: Option<i32>,
    pub rarity: Option<i32>,
    pub tradeable: bool,
    pub vehicle_speed_class: Option<i32>,
}

/// One exact `TableContainer.GetTableData(item_table, item_subtable,
/// item_row_id)` to icon-subtable projection used by
/// `AvatarUtil.GetEquipIconElement`.
///
/// `item_row_id` is deliberately the serialized array index queried by the
/// clean client. `declared_item_number` retains the row's identity field for
/// provenance without incorrectly turning it into the lookup key (the clean
/// Chest table contains a late row whose declared number differs).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayUserEquipIconDefinition {
    pub item_table: u8,
    pub item_subtable: u8,
    pub item_row_id: i32,
    pub declared_item_number: i32,
    pub icon_subtable: u8,
    pub icon_row_id: i32,
    pub icon_type: u8,
    pub icon_number: u32,
    /// Present only when the exact AvatarUtil semantic route is both owned by
    /// the native project manifest and installed on disk.
    pub icon_path: Option<String>,
}

/// Runtime-independent projection of the exact Retrobution Nano table fields
/// consumed by `cnNanoWheel.CheckIcons`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameplayNanoUiDefinition {
    pub nano_id: i16,
    /// Clean `m_iNanoSet` ordering key used by `InventoryManagerScript.SortSlot`.
    pub sort_number: i32,
    /// `m_pNanoStringData[m_iNanoName].m_strName`, used by
    /// `CnGuiNano_info.Update` for the active-Nano HUD.
    pub name: String,
    /// `m_strComment1`, drawn below the Nano name in `Panel_UserClothes`.
    pub attribute: String,
    pub icon_number: u16,
    /// Installed semantic `AvatarUtil` icon for an owned Nano.
    pub icon_path: Option<String>,
    /// Installed semantic forced-type-5 icon used by clean `GetNanoSlot` for
    /// a Nano that is not present in the player's bank.
    pub ready_icon_path: Option<String>,
    pub style: u8,
    pub max_stamina: i16,
}

/// Runtime-independent projection of the exact Retrobution Skill table fields
/// consumed by `cnNanoWheel.CheckIcons`, `GameFrame.SearchNanoSkillTarget`,
/// and `GameFrame.NanoSkillUse`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplaySkillUiDefinition {
    pub skill_id: i16,
    pub skill_type: i32,
    /// Four authored power tiers, in server units; slot boost selects the tier.
    pub values_a: [i32; 4],
    pub icon_number: u16,
    pub active: bool,
    /// XDT `m_iEffectType`.
    pub effect_type: i32,
    /// Projectile contract authored for the skill recipient.
    pub target_effect: i32,
    /// XDT `m_iEffectTarget`.
    pub target: i32,
    /// XDT `m_iTargetType`.
    pub target_type: i32,
    /// XDT `m_iEffectRange`, retained in server centi-units.
    pub range: i32,
    /// XDT `m_iEffectArea`, retained in server centi-units.
    pub area: i32,
    /// XDT `m_iEffectAngle`, retained in degrees.
    pub angle: i32,
    /// XDT `m_iTargetNumber`.
    pub target_number: i32,
    /// XDT `m_iCoolTime`, retained in legacy tenths of a second.
    pub cooldown: i32,
    /// XDT `m_iCoolType` shared cooldown channel.
    pub cool_type: i32,
}

/// Runtime-independent projection of the exact Retrobution SkillBuff table
/// fields consumed by `CnGuiSkillBuffIcon`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplaySkillBuffUiDefinition {
    pub buff_id: i32,
    pub effect_id: i32,
    pub instant_effect_id: i32,
    pub icon_number: u16,
    pub cash_icon_number: u16,
}

/// Exact content shown for one of the three Nano tune cards in
/// `cnMissionJournal`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialNanoJournalSkillUi {
    /// `NanoElement.m_iTune[]` selects a `m_pNanoTuneData` array slot; this is
    /// the selected row's distinct `NanoTuneElement.m_iTuneNumber` value.
    pub tune_id: i32,
    /// Source `NanoTuneElement.m_iSkillID`.
    pub skill_id: i32,
    /// Resolved through `SkillElement.m_iIcon -> m_pSkillIconData`.
    pub icon_number: u16,
    /// Source `m_pNanoTuneStringData[m_iTuneName].m_strName`.
    pub name: String,
    /// Source `m_pNanoTuneStringData[m_iTuneName].m_strComment1`.
    pub type_label: String,
    /// Source `m_pNanoTuneStringData[m_iTuneName].m_strComment`.
    pub description: String,
    /// `NanoTuneElement.m_iReqItemID`.
    pub required_item_id: i32,
    /// `NanoTuneElement.m_iReqItemCount`.
    pub required_item_count: i32,
}

/// Runtime-independent projection of the clean Retrobution Nano mission page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialNanoJournalUi {
    pub nano_id: i32,
    /// Resolved through `NanoElement.m_iIcon1 -> m_pNanoIconData`.
    pub icon_number: u16,
    /// Source `m_pNanoStringData[m_iNanoName].m_strName`.
    pub name: String,
    /// Source `m_pNanoStringData[m_iNanoName].m_strComment1`.
    pub attribute: String,
    /// Source `m_pNanoStringData[m_iNanoName].m_strComment`.
    pub description: String,
    /// `NanoElement.m_iTune` is an exact three-element legacy contract.
    pub skills: [TutorialNanoJournalSkillUi; 3],
}

/// One exact `m_pMessageTable.m_pMessageData[row_id]` record.
///
/// The clean client indexes this array directly and then normalizes failure
/// button types 10/11 to the renderable 1/2 contracts. Keeping both values
/// prevents runtime UI code from either guessing text or losing source
/// provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemMessageDefinition {
    pub row_id: i32,
    pub exact_text: String,
    pub raw_button_type: i32,
    pub runtime_button_type: SystemMessageButtonType,
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct TutorialMissionContent {
    pub provenance: TutorialTableSetProvenance,
    pub(super) missions: BTreeMap<i32, TutorialMissionDefinition>,
    pub(super) mission_tasks_by_npc: BTreeMap<i32, Vec<i32>>,
    pub(super) gameplay_guide_nanocom: BTreeMap<i16, GameplayGuideNanocomDefinition>,
    pub(super) npcs: BTreeMap<i32, TutorialNpcDefinition>,
    pub(super) rewards: BTreeMap<i32, TutorialRewardDefinition>,
    pub(super) scene_events: BTreeMap<i32, TutorialSceneEventDefinition>,
    pub(super) warps: BTreeMap<i32, TutorialWarpDefinition>,
    pub(super) gameplay_warps: BTreeMap<i32, GameplayWarpDefinition>,
    pub(super) gameplay_npcs: BTreeMap<i32, GameplayNpcUiDefinition>,
    pub(super) gameplay_npc_barker_types: BTreeMap<i32, i32>,
    pub(super) gameplay_mission_name_strings: Vec<String>,
    pub(super) gameplay_npc_skill_barkers: BTreeMap<(i32, i16), GameplayNpcSkillBarkerDefinition>,
    pub(super) gameplay_npc_corruption_barkers: BTreeMap<i32, GameplayNpcSkillBarkerDefinition>,
    pub(super) gameplay_npc_portrait_icon_paths: BTreeMap<i32, String>,
    pub(super) gameplay_npc_map_icons: BTreeMap<i32, i32>,
    pub(super) gameplay_npc_sounds: BTreeMap<i32, i32>,
    pub(super) gameplay_npc_services: BTreeMap<i32, (i32, String)>,
    pub(super) gameplay_vendors: BTreeMap<i32, Vec<GameplayVendorItemDefinition>>,
    pub(super) gameplay_vendor_item_metadata: BTreeMap<(i16, i16), GameplayVendorItemMetadataDefinition>,
    pub(super) gameplay_vehicle_speed_classes: BTreeMap<i16, i32>,
    pub(super) gameplay_vehicle_engine_sounds: BTreeMap<i16, String>,
    pub(super) gameplay_vehicle_equip_types: BTreeMap<i16, i32>,
    pub(super) gameplay_xcoms: Vec<GameplayXcomDefinition>,
    pub(super) gameplay_general_items: BTreeMap<i16, GameplayGeneralItemUiDefinition>,
    pub(super) quest_item_names: BTreeMap<i32, String>,
    pub(super) reward_quest_items: BTreeMap<i32, [i32; 3]>,
    pub(super) reward_chest_kinds: BTreeMap<i16, i32>,
    pub(super) gameplay_nanos: BTreeMap<i16, GameplayNanoUiDefinition>,
    pub(super) gameplay_nano_tune_fusion_matter: BTreeMap<i32, i32>,
    pub(super) gameplay_skills: BTreeMap<i16, GameplaySkillUiDefinition>,
    pub(super) gameplay_skill_names: BTreeMap<i16, String>,
    pub(super) gameplay_skill_buffs: BTreeMap<i32, GameplaySkillBuffUiDefinition>,
    pub(super) gameplay_user_equip_icons: BTreeMap<(u8, u8, i32, u8), GameplayUserEquipIconDefinition>,
    pub(super) journal_nanos: BTreeMap<i32, TutorialNanoJournalUi>,
    pub(super) system_messages: BTreeMap<i32, SystemMessageDefinition>,
}
