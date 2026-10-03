//! Authoring constraints shared by the inspector, drafts, JSON and CSV.
use super::*;

pub(super) fn choices(field: &str) -> &'static [(i64, &'static str)] {
    match field {
        "m_iHDifficultyType" => &[(0, "Easy"), (1, "Normal"), (2, "Hard")],
        "m_iHMissionType" => &[(1, "GuideMission"), (2, "NanoMission"), (3, "WorldMission")],
        "m_iHTaskType" => &[
            (1, "Talk"),
            (2, "GotoLocation"),
            (3, "UseItems"),
            (4, "Delivery"),
            (5, "Defeat"),
            (6, "EscortDefence"),
        ],
        "m_iCSUDEPNPCFollow" => &[(0, "Disabled"), (1, "Enabled")],
        _ => &[],
    }
}

pub(super) fn help(field: &str) -> Option<&'static str> {
    Some(match field {
        "m_iSTDialogBubble" | "m_iSUDialogBubble" | "m_iFDialogBubble" => {
            "Text spoken above the linked NPC at this event. Search by text or ID, edit the linked text or create a new one. Zero disables the line; set the speaking NPC separately."
        }
        "m_iSTJournalIDAdd" | "m_iSUJournaliDAdd" | "m_iFJournalIDAdd" => {
            "Journal entry used at this stage event. Search by its text or ID, or create a new entry here. Zero means no journal update."
        }
        "m_iMissionSummary" => "Short mission summary in the journal. Select or create a mission text.",
        "m_iDetaileMissionDesc" => "Detailed mission description shown when the mission is offered.",
        "m_iTaskSummary" => "Short description of the current task in this journal entry.",
        "m_iDetailedTaskDesc" => "Detailed instructions for the active task in the mission journal.",
        "m_iMissionCompleteSummary" => "Short journal summary after completing the mission.",
        "m_iDetaileMissionCompleteSummary" => "Detailed journal description after completing the mission.",
        "m_iHDifficultyType" => {
            "Difficulty label in the mission journal: 0 Easy, 1 Normal, 2 Hard. Does not set enemy health or rewards."
        }
        "m_iHMissionType" => {
            "Mission category: 1 Guide, 2 Nano, 3 World. Nano rewards and level progression must be configured separately."
        }
        "m_iHTaskType" => {
            "Stage kind: 1 Talk, 2 Go to a location, 3 Use items, 4 Delivery, 5 Defeat enemies, 6 Escort or defend. Configure the actual objectives below."
        }
        "m_iHJournalNPCID" => {
            "NPC shown in the mission journal. Select an NPC type, not a spawned NPC instance."
        }
        "m_iHCurrentObjective" => {
            "Text describing the current objective to the player. A reference to a mission text row, not a numeric gameplay condition."
        }
        "m_iCTRReqLvMin" => {
            "Minimum player level needed to start this stage. Zero means no minimum."
        }
        "m_iCTRReqLvMax" => {
            "Maximum level recorded by mission content. Zero means no configured maximum; do not assume the server enforces this field."
        }
        "m_iCSTRReqNano" => {
            "Nano IDs the player must already own. Each zero leaves a prerequisite slot unused."
        }
        "m_iCSTItemID" => {
            "Quest items required to start this stage. Each slot pairs with the starting item count in the same position."
        }
        "m_iCSTItemNumNeeded" => {
            "Quest item counts required at the start, paired with the required item IDs. These are not reward counts."
        }
        "m_iCSTTrigger" => {
            "Reference to a task used by the mission trigger. Zero disables this reference; it is a task ID, not a mission ID."
        }
        "m_iCSUDEFNPCID" => {
            "NPC type to escort or defend. Creating a new NPC type does not place it in the world."
        }
        "m_iCSUDEPNPCFollow" => {
            "Whether the escort NPC follows the player. 0 Disabled, 1 Enabled. The server also derives escort behavior from the destination NPC."
        }
        "m_iSTItemID" => {
            "Quest items given or dropped during this stage. Counts and drop chances refer to the same slots."
        }
        "m_iSTItemNumNeeded" => {
            "Quest item counts granted at stage start, paired with start item IDs. Zero grants none."
        }
        "m_iSTItemDropRate" => {
            "Drop chance for the corresponding quest item, in percent from 0 to 100."
        }
        "m_iDelItemID" => {
            "Quest items removed when this mission is abandoned. Zero leaves the slot unused."
        }
        "m_iSUItem" => {
            "Quest item IDs changed on success. Pair each with its success item quantity."
        }
        "m_iSUInstancename" => {
            "Signed quest item quantity change on success: positive adds items, negative removes them, zero changes nothing."
        }
        "m_iFItemID" => {
            "Quest item IDs changed on failure. Pair each with its failure item quantity."
        }
        "m_iFItemNumNeeded" => {
            "Signed quest item quantity change on failure: positive adds items, negative removes them."
        }
        "m_iSTMessageTextID" => {
            "Text of the NanoCom message at stage start. Select or create a mission text; its row index is stored."
        }
        "m_iSUMessagetextID" => {
            "Text of the NanoCom message on success. Select or create a mission text; its row index is stored."
        }
        "m_iFMessageTextID" => {
            "Text of the NanoCom message on failure. Select or create a mission text; its row index is stored."
        }
        "m_iSTMessageSendNPC" | "m_iSUMessageSendNPC" | "m_iFMessageSendNPC" => {
            "NPC type speaking the linked NanoCom message. Zero means no configured speaker."
        }
        "m_iSTGrantWayPoint" => {
            "NPC type used as a mission destination marker. This is not an arbitrary map coordinate."
        }
        "m_iRequireInstanceID" => {
            "Required map or instance identifier. Leaving the required instance may fail the mission. Zero leaves it unset."
        }
        "m_iHMissionName" => {
            "Shared mission title stored in a text row. All stages of a mission normally share this title; creating a private text changes only this reference."
        }
        _ => return None,
    })
}

pub(super) fn invalid(field: &str, value: &Value) -> Option<&'static str> {
    let options = choices(field);
    if !options.is_empty() {
        return (!options.iter().any(|(n, _)| value.as_i64() == Some(*n)))
            .then_some("mission_choice");
    }
    let length = match field {
        "m_iCSTReqMission" => Some(2),
        "m_iCSTRReqNano" => Some(5),
        "m_iHBarkerTextID" | "m_iDelItemID" => Some(4),
        "m_iCSUEnemyID"
        | "m_iCSUNumToKill"
        | "m_iCSUItemID"
        | "m_iCSUItemNumNeeded"
        | "m_iCSTItemID"
        | "m_iCSTItemNumNeeded"
        | "m_iSTItemID"
        | "m_iSTItemNumNeeded"
        | "m_iSTItemDropRate"
        | "m_iSUItem"
        | "m_iSUInstancename"
        | "m_iFItemID"
        | "m_iFItemNumNeeded" => Some(3),
        _ => None,
    };
    if let Some(length) = length {
        let Some(values) = value.as_array().filter(|v| v.len() == length) else {
            return Some("mission_slots");
        };
        let signed = matches!(
            field,
            "m_iSUInstancename" | "m_iFItemNumNeeded" | "m_iSTItemNumNeeded"
        );
        let max = match field {
            "m_iCSTReqMission" => 2048, // RustyFusion: 32 * 64 mission completion flags.
            "m_iCSTRReqNano" | "m_iCSUItemID" | "m_iCSTItemID" | "m_iSTItemID" | "m_iSUItem"
            | "m_iFItemID" | "m_iDelItemID" => i16::MAX as i64,
            "m_iSTItemDropRate" => 100,
            _ => i32::MAX as i64,
        };
        return values
            .iter()
            .any(|v| {
                v.as_i64()
                    .is_none_or(|n| n < if signed { i32::MIN as i64 } else { 0 } || n > max)
            })
            .then_some("mission_range");
    }
    let range = match field {
        "m_iHMissionID" => Some((1, 2048)),
        "m_iHTaskID" => Some((1, i32::MAX as i64)),
        "m_iCTRReqLvMin" | "m_iCTRReqLvMax" | "m_iSTNanoID" => Some((0, i16::MAX as i64)),
        "m_iHNPCID"
        | "m_iHTerminatorNPCID"
        | "m_iHJournalNPCID"
        | "m_iCSUDEFNPCID"
        | "m_iSUOutgoingTask"
        | "m_iFOutgoingTask"
        | "m_iSUReward"
        | "m_iCSUCheckTimer"
        | "m_iSTGrantTimer"
        | "m_iHMissionName"
        | "m_iHCurrentObjective" => Some((0, i32::MAX as i64)),
        _ => None,
    };
    range.and_then(|(min, max)| {
        value
            .as_i64()
            .is_none_or(|n| n < min || n > max)
            .then_some("mission_range")
    })
}

pub(super) fn neutral(value: &Value) -> Value {
    match value {
        Value::Array(a) => Value::Array(a.iter().map(neutral).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), neutral(v))).collect()),
        Value::Number(n) if n.is_f64() => Value::from(0.0),
        Value::Number(_) => Value::from(0),
        Value::String(_) => Value::String(String::new()),
        Value::Bool(_) => Value::Bool(false),
        Value::Null => Value::Null,
    }
}

pub(super) fn defaults(template: &Value) -> Value {
    let mut value = template.clone();
    let table = "default/m_pMissionTable/m_pMissionData";
    for (field, entry) in value.as_object_mut().into_iter().flatten() {
        if super::schema::required(table).contains(&field.as_str())
            || super::authoring::group(table, field).is_some()
        {
            *entry = neutral(entry);
        }
    }
    value["m_iHMissionType"] = Value::from(3);
    value["m_iHTaskType"] = Value::from(1);
    value["m_iHDifficultyType"] = Value::from(0);
    if value.get("m_pstrSTScript").is_some() {
        value["m_pstrSTScript"] = Value::from("0");
    }
    value
}
