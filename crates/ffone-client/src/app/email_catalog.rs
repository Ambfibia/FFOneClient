//! Email item tables, guide mail catalog and login guide Nanocom.

use super::combi::{combi_table_i32_0104, combi_table_string_0104};
use super::guide_nanocom;
use super::local_inventory::LocalInventoryRuntime;
use bevy::prelude::*;
use ffone_client::{
    assets::AssetLocator,
    combi_ui::COMBI_RECIPE_TABLE_PATH,
    email_runtime::{EmailItemCatalog0104, EmailItemCatalogMetadata0104},
    email_ui::EmailGuideMessage,
    guide_runtime::GuideRuntime,
    guide_ui::GuideMentor,
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    nanocom_message_ui::NanocomMessageUiModel,
    tutorial_mission_content::{TutorialMissionContent, TutorialMissionType},
    user_equip_ui::{UserEquipCatalogQuery, UserEquipItemCatalog, UserEquipItemIds},
    world_mission_runtime::WorldMissionRuntime,
};
use ffone_protocol::ItemBase0104;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const EMAIL_SYSTEM_MESSAGE_ID_BASE_0104: u64 = 0x454d_4149_0000_0000;
pub(super) const EMAIL_UI_MODE_SOUND_TRUE_NAME_0104: &str = "InvenLooping";
pub(super) const EMAIL_UI_MODE_SOUND_GAIN_0104: f32 = 0.7;
pub(super) const EMAIL_ITEM_TABLES_0104: [(&str, i16); 10] = [
    ("m_pWeaponItemTable", 0),
    ("m_pShirtsItemTable", 1),
    ("m_pPantsItemTable", 2),
    ("m_pShoesItemTable", 3),
    ("m_pHatItemTable", 4),
    ("m_pGlassItemTable", 5),
    ("m_pBackItemTable", 6),
    ("m_pGeneralItemTable", 7),
    ("m_pChestItemTable", 9),
    ("m_pVehicleItemTable", 10),
];

#[derive(Clone, Debug)]
pub(super) struct EmailGuideTableRow0104 {
    pub(super) task_id: i32,
    pub(super) start_message_type: i32,
    pub(super) start_sender: i32,
    pub(super) start_copy: String,
    pub(super) start_string_id: i32,
    pub(super) subject_string_id: i32,
    pub(super) subject: String,
    pub(super) mentor_copy: [Option<String>; 5],
    pub(super) mentor_string_ids: [i32; 5],
}

#[derive(Clone, Debug, Resource)]
pub(super) struct EmailProductionCatalog0104 {
    pub(super) items: BTreeMap<(i16, i16), EmailItemCatalogMetadata0104>,
    pub(super) guide_rows: Vec<EmailGuideTableRow0104>,
}

impl EmailProductionCatalog0104 {
    pub(super) fn open(assets: &AssetLocator, content: &TutorialMissionContent) -> Result<Self, String> {
        let bytes = assets.read(COMBI_RECIPE_TABLE_PATH)?;
        Self::from_table_set_bytes(&bytes, |item| {
            UserEquipCatalogQuery::from_non_empty_item(item)
                .ok()
                .and_then(|query| UserEquipItemCatalog::resolve_icon(content, query))
                .map(|icon| icon.runtime_path().to_owned())
        })
    }

    pub(super) fn from_table_set_bytes(
        bytes: &[u8],
        mut resolve_icon_path: impl FnMut(ItemBase0104) -> Option<String>,
    ) -> Result<Self, String> {
        let document: serde_json::Value = ffone_client::xdt::from_slice(bytes)
            .map_err(|error| format!("invalid Email TableData JSON: {error}"))?;
        let tables = document
            .get("tables")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "Email TableData root has no tables array".to_owned())?;
        let mut consolidated = tables.iter().filter(|table| {
            table.get("name").and_then(serde_json::Value::as_str)
                == Some("npc_imports_consolidated")
        });
        let value = consolidated
            .next()
            .and_then(|table| table.get("value"))
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "Email TableData has no consolidated value object".to_owned())?;
        if consolidated.next().is_some() {
            return Err("Email TableData has duplicate consolidated tables".to_owned());
        }

        let mut items = BTreeMap::new();
        for (table_name, item_type) in EMAIL_ITEM_TABLES_0104 {
            let table = value
                .get(table_name)
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| format!("Email TableData is missing {table_name}"))?;
            let rows = table
                .get("m_pItemData")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("{table_name}.m_pItemData is not an array"))?;
            for (row_index, row) in rows.iter().enumerate() {
                let item_id = i16::try_from(row_index).map_err(|_| {
                    format!("{table_name} row {row_index} does not fit protocol i16")
                })?;
                let row_context = format!("{table_name}.m_pItemData[{row_index}]");
                let row = row
                    .as_object()
                    .ok_or_else(|| format!("{row_context} is not an object"))?;
                let tradeable = combi_table_i32_0104(row, "m_iTradeAble", &row_context)? != 0;
                let general_item_type = (item_type == 7)
                    .then(|| combi_table_i32_0104(row, "m_iItemType", &row_context))
                    .transpose()?;
                let item = ItemBase0104 {
                    item_type,
                    item_id,
                    option: 0,
                    time_limit: 0,
                };
                let icon_path = resolve_icon_path(item);
                let metadata = EmailItemCatalogMetadata0104 {
                    icon_path,
                    tradeable: Some(tradeable),
                    general_item_type,
                };
                if items.insert((item_type, item_id), metadata).is_some() {
                    return Err(format!(
                        "duplicate Email item metadata for type {item_type} row {item_id}"
                    ));
                }
            }
        }

        let mission_table = value
            .get("m_pMissionTable")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "Email TableData is missing m_pMissionTable".to_owned())?;
        let mission_rows = mission_table
            .get("m_pMissionData")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "m_pMissionTable.m_pMissionData is not an array".to_owned())?;
        let mission_strings = mission_table
            .get("m_pMissionStringData")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "m_pMissionTable.m_pMissionStringData is not an array".to_owned())?;
        let resolve_mission_string = |index: i32, context: &str| -> Result<String, String> {
            let index = usize::try_from(index)
                .map_err(|_| format!("{context} references negative string index"))?;
            let row = mission_strings
                .get(index)
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| format!("{context} references missing mission string {index}"))?;
            combi_table_string_0104(row, "m_pstrNameString", context).map(str::to_owned)
        };
        let mut seen_tasks = BTreeSet::new();
        let mut guide_rows = Vec::new();
        for (row_index, row) in mission_rows.iter().enumerate() {
            let row_context = format!("m_pMissionTable.m_pMissionData[{row_index}]");
            let row = row
                .as_object()
                .ok_or_else(|| format!("{row_context} is not an object"))?;
            let task_id = combi_table_i32_0104(row, "m_iHTaskID", &row_context)?;
            if task_id <= 0 {
                continue;
            }
            if !seen_tasks.insert(task_id) {
                return Err(format!("duplicate Email guide task row {task_id}"));
            }
            let email_ids = row
                .get("m_iMentorEmailID")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| format!("{row_context}.m_iMentorEmailID is not an array"))?;
            if email_ids.len() != 5 {
                return Err(format!(
                    "{row_context}.m_iMentorEmailID has {} entries instead of 5",
                    email_ids.len()
                ));
            }
            let mut mentor_copy: [Option<String>; 5] = std::array::from_fn(|_| None);
            let mut mentor_string_ids = [0; 5];
            for (mentor_index, value) in email_ids.iter().enumerate() {
                let string_id = value.as_i64().ok_or_else(|| {
                    format!("{row_context}.m_iMentorEmailID[{mentor_index}] is not an integer")
                })?;
                let string_id = i32::try_from(string_id).map_err(|_| {
                    format!("{row_context}.m_iMentorEmailID[{mentor_index}] does not fit i32")
                })?;
                if string_id > 0 {
                    mentor_string_ids[mentor_index] = string_id;
                    mentor_copy[mentor_index] = Some(resolve_mission_string(
                        string_id,
                        &format!("{row_context}.m_iMentorEmailID[{mentor_index}]"),
                    )?);
                }
            }
            let start_message_type = combi_table_i32_0104(row, "m_iSTMessageType", &row_context)?;
            let has_start_mail = matches!(start_message_type, 4 | 6);
            if !has_start_mail && mentor_copy.iter().all(Option::is_none) {
                continue;
            }
            guide_rows.push(EmailGuideTableRow0104 {
                task_id,
                start_message_type,
                start_sender: combi_table_i32_0104(row, "m_iSTMessageSendNPC", &row_context)?,
                start_string_id: combi_table_i32_0104(row, "m_iSTMessageTextID", &row_context)?,
                start_copy: if has_start_mail {
                    resolve_mission_string(
                        combi_table_i32_0104(row, "m_iSTMessageTextID", &row_context)?,
                        &row_context,
                    )?
                } else {
                    String::new()
                },
                subject_string_id: combi_table_i32_0104(row, "m_iHMissionName", &row_context)?,
                subject: resolve_mission_string(
                    combi_table_i32_0104(row, "m_iHMissionName", &row_context)?,
                    &format!("{row_context}.m_iHMissionName"),
                )?,
                mentor_copy,
                mentor_string_ids,
            });
        }
        Ok(Self { items, guide_rows })
    }
}

impl EmailItemCatalog0104 for EmailProductionCatalog0104 {
    fn resolve(&self, item: ItemBase0104) -> Option<EmailItemCatalogMetadata0104> {
        let mut metadata = self.items.get(&(item.item_type, item.item_id))?.clone();
        if let Some(look_id) = UserEquipItemIds::from_item(item).combined_look_id {
            let look_id = i16::try_from(look_id).ok()?;
            metadata.icon_path = self
                .items
                .get(&(item.item_type, look_id))
                .and_then(|look| look.icon_path.clone());
        }
        Some(metadata)
    }
}

pub(super) const fn email_mentor_email_index_0104(mentor: GuideMentor) -> usize {
    // MissionData.m_iMentorEmailID is serialized in Edd/Dexter/Mojo/Ben/
    // Computress order. That is deliberately different from both the wire
    // IDs and GuideMentor::slot().
    match mentor {
        GuideMentor::Edd => 0,
        GuideMentor::Dexter => 1,
        GuideMentor::MojoJojo => 2,
        GuideMentor::BenTennyson => 3,
    }
}

pub(super) const fn email_mentor_npc_type_0104(mentor: GuideMentor) -> i32 {
    match mentor {
        GuideMentor::Edd => 707,
        GuideMentor::Dexter => 728,
        GuideMentor::MojoJojo => 731,
        GuideMentor::BenTennyson => 732,
    }
}

pub(super) fn email_guide_messages_0104(
    catalog: &EmailProductionCatalog0104,
    content: &TutorialMissionContent,
    guide: &GuideRuntime,
    active_task_ids: impl IntoIterator<Item = i32>,
    available_task_ids: impl IntoIterator<Item = i32>,
) -> Result<Vec<EmailGuideMessage>, String> {
    // A missing/unselected mentor affects invitations only, never NPC or player mail.
    let mentor = guide
        .authoritative()
        .and_then(|state| ffone_client::guide_runtime::GuideRawMentor::from_raw(state.raw_mentor()))
        .map(|mentor| match mentor {
            ffone_client::guide_runtime::GuideRawMentor::Selectable(mentor) => (
                email_mentor_email_index_0104(mentor),
                email_mentor_npc_type_0104(mentor),
            ),
            ffone_client::guide_runtime::GuideRawMentor::ComputressFuture => (4, 1171),
        });
    let active = active_task_ids.into_iter().collect::<BTreeSet<_>>();
    let available = available_task_ids.into_iter().collect::<BTreeSet<_>>();
    let mut messages = Vec::new();
    // Mission mail precedes invitations. A task type is not an email mode:
    // active tasks use their NPC/start copy only for message types 4 or 6.
    for mode in [1, 2] {
        for row in &catalog.guide_rows {
            let (sender_npc_id, copy, content_string_id) = if mode == 1 {
                if !active.contains(&row.task_id) || !matches!(row.start_message_type, 4 | 6) {
                    continue;
                }
                (row.start_sender, &row.start_copy, row.start_string_id)
            } else {
                if active.contains(&row.task_id) || !available.contains(&row.task_id) {
                    continue;
                }
                let Some((mentor_index, mentor_npc_id)) = mentor else {
                    continue;
                };
                let Some(copy) = row.mentor_copy[mentor_index].as_ref() else {
                    continue;
                };
                (mentor_npc_id, copy, row.mentor_string_ids[mentor_index])
            };
            let sender = content
                .gameplay_npc(sender_npc_id)
                .ok_or_else(|| format!("Email projection is missing NPC {sender_npc_id}"))?;
            let sender_icon_path = UserEquipCatalogQuery::from_non_empty_item(ItemBase0104 {
                item_type: 30,
                item_id: i16::try_from(sender_npc_id)
                    .map_err(|_| format!("Email NPC {sender_npc_id} does not fit protocol i16"))?,
                option: 0,
                time_limit: 0,
            })
            .ok()
            .and_then(|query| UserEquipItemCatalog::resolve_icon(content, query))
            .map(|icon| icon.runtime_path().to_owned());
            messages.push(EmailGuideMessage {
                mode,
                mission_task_id: row.task_id,
                sender_npc_id,
                sender_name: sender.name.clone(),
                subject: row.subject.clone(),
                subject_string_id: row.subject_string_id,
                content: copy.clone(),
                content_string_id,
                sender_icon_path,
                auto_delete_note: false,
            });
        }
    }
    Ok(messages)
}

/// First eligible invitation of each mission category, in table order.
/// Use the same authority as mission availability and the login mail notice.
#[allow(clippy::too_many_arguments)]
pub(super) fn available_email_task_ids_0104(
    catalog: &EmailProductionCatalog0104,
    content: &TutorialMissionContent,
    mission: &WorldMissionRuntime,
    guide: &GuideRuntime,
    nano_bank: &NanoFreeTuningBank0104,
    inventory: &LocalInventoryRuntime,
    player_level: i32,
) -> Vec<i32> {
    let Some(authority) = guide.authoritative() else {
        return Vec::new();
    };
    let raw_mentor = authority.raw_mentor();
    let Ok(mentor_index) = usize::try_from(raw_mentor - 1) else {
        return Vec::new();
    };
    let owned_nanos = nano_bank
        .entries()
        .iter()
        .filter_map(|nano| (nano.id > 0).then_some(i32::from(nano.id)))
        .collect::<BTreeSet<_>>();
    let quest_inventory = inventory
        .quest_inventory
        .as_ref()
        .map(|items| items.as_slice())
        .unwrap_or(&[]);
    let mut selected = [false; 2];
    let mut tasks = Vec::new();
    for row in &catalog.guide_rows {
        let Ok(candidate) = content.mission(row.task_id) else {
            continue;
        };
        let category = match candidate.mission_type {
            TutorialMissionType::Guide => 0,
            TutorialMissionType::Nano => 1,
            _ => continue,
        };
        if selected[category]
            || !row
                .mentor_copy
                .get(mentor_index)
                .is_some_and(Option::is_some)
            || mission
                .completed_mission_ids()
                .contains(&candidate.provenance.mission_id)
            || !mission.can_start_task(
                candidate,
                player_level,
                i32::from(raw_mentor),
                &owned_nanos,
                quest_inventory,
                content,
            )
        {
            continue;
        }
        selected[category] = true;
        tasks.push(row.task_id);
    }
    tasks
}

/// Clean `cnMissionManager.ReceiveStartGames(null)` login producer: publish
/// the current Guide's mail/no-mail TableData line as a passive type-9 notice.
pub(super) fn enqueue_login_guide_nanocom_0104(
    catalog: &EmailProductionCatalog0104,
    content: &TutorialMissionContent,
    mission: &WorldMissionRuntime,
    guide: &GuideRuntime,
    nano_bank: &NanoFreeTuningBank0104,
    inventory: &LocalInventoryRuntime,
    player_level: i32,
    messages: &mut NanocomMessageUiModel,
) -> bool {
    let Some(authoritative) = guide.authoritative() else {
        return false;
    };
    let raw_mentor = authoritative.raw_mentor();
    let Some(definition) = content.gameplay_guide_nanocom(raw_mentor) else {
        return false;
    };
    let has_guide_mail = !available_email_task_ids_0104(
        catalog,
        content,
        mission,
        guide,
        nano_bank,
        inventory,
        player_level,
    )
    .is_empty();
    let (string_id, fallback) = if has_guide_mail {
        (
            definition.login_mail_string_id,
            definition.login_mail_text.as_str(),
        )
    } else {
        (
            definition.login_no_mail_string_id,
            definition.login_no_mail_text.as_str(),
        )
    };
    guide_nanocom::enqueue(content, raw_mentor, string_id, fallback, messages)
}
