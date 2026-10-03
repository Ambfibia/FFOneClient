//! Names and constraints verified against native data consumers. Unknown legacy fields
//! remain editable, but do not receive speculative descriptions or enum meanings.
use super::*;

pub(super) fn text(l: &Localization, lang: &Language, key: &str, fallback: &str) -> String {
    l.text(
        lang,
        &LocalizedText::new(format!("ui.editor.xdt.{key}"), fallback),
    )
}
pub(super) fn status(status: &str, l: &Localization, lang: &Language) -> String {
    let key = match status {
        "Finish or cancel the new record first" => Some("finish_form"),
        "Missing empty NPC template: Location A256 (1401)" => Some("missing_placeholder_template"),
        "This parameter is required by the native client" => Some("native_required"),
        "All prerequisite slots are occupied" => Some("mission.error_full_requirements"),
        "Mission prerequisite would create a cycle" => Some("mission.error_dependency_cycle"),
        "Transitions must stay within the same mission" => Some("mission.error_stage_group"),
        "Success transitions cannot create a cycle" | "Success cannot return to the same stage" => Some("mission.error_stage_cycle"),
        _ if status.starts_with("Required parameters:") => Some("check_required"),
        _ if status.starts_with("Duplicate identity:") => Some("duplicate_id"),
        _ => None,
    };
    if let Some(key) = key {
        return text(l, lang, key, status);
    }
    if let Some((field, reason)) = status.split_once(": ") {
        if field.starts_with("m_") {
            return format!(
                "{}: {}",
                field_name(field, l, lang),
                text(l, lang, &format!("error.{reason}"), reason)
            );
        }
    }
    status.to_owned()
}
pub(super) fn words(raw: &str) -> String {
    let raw = ["m_pstr", "m_str", "m_p", "m_i", "m_f", "m_u", "m_"]
        .iter()
        .find_map(|prefix| raw.strip_prefix(prefix))
        .unwrap_or(raw);
    let chars: Vec<_> = raw.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if i > 0
            && c.is_uppercase()
            && (chars[i - 1].is_lowercase()
                || chars.get(i + 1).is_some_and(|c| c.is_lowercase())
                    && chars[i - 1].is_uppercase())
        {
            out.push(' ');
        }
        out.push(if c == '_' { ' ' } else { c });
    }
    out
}
pub(super) fn type_error(value: &Value) -> &'static str {
    match value {
        Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        _ => "value",
    }
}
pub(super) fn table_name(table: &str, l: &Localization, lang: &Language) -> String {
    let parts: Vec<_> = table.split('/').collect();
    let group = parts
        .get(parts.len().saturating_sub(2))
        .copied()
        .unwrap_or("");
    let leaf = parts.last().copied().unwrap_or("");
    let group = group.trim_start_matches("m_p").trim_end_matches("Table");
    let group = text(l, lang, &format!("group.{group}"), &words(group));
    let kind = if leaf == "m_pNanoTuneStringData" {
        "TuningTexts"
    } else if leaf == "m_pNanoTuneIconData" {
        "TuningIcons"
    } else if table.contains("/m_pAnimationTable/") && leaf == "m_pAvatarData" {
        "PlayerAnimations"
    } else if leaf.ends_with("StringData") || leaf.ends_with("String") {
        "Texts"
    } else if leaf.ends_with("IconData") {
        "Icons"
    } else if leaf.ends_with("MeshData") {
        "Models"
    } else if leaf.ends_with("SoundData") {
        "Sounds"
    } else {
        leaf.trim_start_matches("m_p").trim_end_matches("Data")
    };
    let title = text(l, lang, &format!("table.{kind}"), &words(kind));
    if leaf == "m_pItemData" && table.contains("ItemTable/") {
        return group;
    }
    if group.is_empty() || group == title || !parts.iter().any(|s| s.ends_with("Table")) {
        title
    } else {
        format!("{group} · {title}")
    }
}
pub(super) fn field_name(field: &str, l: &Localization, lang: &Language) -> String {
    if let Some((base, suffix)) = field.split_once('[') {
        return format!("{} [{suffix}", field_name(base, l, lang));
    }
    text(l, lang, &format!("field.{field}"), &words(field))
}
pub(super) fn choices(table: &str, field: &str) -> &'static [(i64, &'static str)] {
    if table.ends_with("/m_pMissionTable/m_pRewardData"){return mission_reward::choices(field);}
    if table.ends_with("/m_pMissionTable/m_pMissionData") {
        return mission_fields::choices(field);
    }
    if table.ends_with("/m_pNpcTable/m_pNpcData") && field == "m_iNpcStyle"
        || table.ends_with("/m_pNanoTable/m_pNanoData") && field == "m_iStyle"
    {
        &[(0, "Adaptium"), (1, "Blastons"), (2, "Cosmix")]
    } else {
        &[]
    }
}
pub(super) fn value_name(
    table: &str,
    field: &str,
    value: &Value,
    l: &Localization,
    lang: &Language,
) -> Option<String> {
    let n = value.as_i64()?;
    choices(table, field)
        .iter()
        .find(|(v, _)| *v == n)
        .map(|(code, name)| format!("{} ({code})", text(l, lang, &format!("attribute.{name}"), name)))
}
pub(super) fn help(field: &str, l: &Localization, lang: &Language) -> String {
    if let Some(help) = mission_fields::help(field) {
        return text(l, lang, &format!("help.{field}"), help);
    }
    let fallback = match field {
        "m_iNpcNumber" => {
            "NPC identity used by missions, spawns and the server. Changing it does not update those references automatically."
        }
        "m_iNanoNumber" => {
            "Nano identity used by missions and the server; must fit a positive 16-bit integer."
        }
        "m_iHTaskID" => "Unique task identity. Success and failure routes point to this ID.",
        "m_iHMissionID" => "Groups tasks into one mission. Several tasks may share this ID.",
        "m_iNpcName"
        | "m_iNanoName"
        | "m_iHMissionName"
        | "m_iHCurrentObjective"
        | "m_iItemName" => {
            "Select a text record. This stores its zero-based row index, not an entity ID. Editing that text affects every record using it."
        }
        "m_iMesh" => {
            "Select a model record. The row index chooses the model and texture routes; shared model changes affect all users."
        }
        "m_iIcon1" => {
            "Select an icon record. This is a zero-based row index; the target record contains the icon number and atlas type."
        }
        "m_iHP" => "Maximum NPC health. Native client accepts zero or a positive integer.",
        "m_iNpcLevel" => "NPC level shown by the native client; must be zero or positive.",
        "m_iNpcStyle" | "m_iStyle" => {
            "Combat attribute code. Native data supports 0, 1 and 2. Keep the accepted codes."
        }
        "m_iRadius" | "m_iHeight" | "m_iSightRange" => {
            "NPC geometry or visibility range in server units; must be zero or positive."
        }
        "m_iNanoBattery1" => {
            "Maximum Nano stamina shown by the native client. Allowed range: 1–32767."
        }
        "m_iNanoSet" => "Sort order in the Nano collection.",
        "m_iBarkerNumber" => "Select a set of NPC ambient speech lines. Zero disables the set.",
        "m_strName" => {
            "Display name or text. Records that point to this text row share this value."
        }
        "m_strComment" => {
            "Additional text. In NPC text records this is the greeting shown by the native client."
        }
        "m_iHNPCID" => "NPC identity that starts the mission task.",
        "m_iHTerminatorNPCID" => "NPC identity that receives the completed task.",
        "m_iSUOutgoingTask" => {
            "Task ID to activate after success. Follow the link to inspect the next task."
        }
        "m_iFOutgoingTask" => "Task ID to activate after failure.",
        "m_iSUReward" => {
            "Reward identity granted on task success; follow the link to edit money, Fusion Matter and items."
        }
        "m_iSTNanoID" => "Nano identity granted when this task starts.",
        "m_iSTGrantTimer" => "Task timer read by the native mission system.",
        "m_iCash" => "Money included in the mission reward.",
        "m_iFusionMatter" => "Fusion Matter included in the mission reward.",
        "m_iCSUEnemyID" => {
            "NPC IDs for the defeat objective. Each element pairs with the same element in Required defeat counts; zero leaves the slot unused."
        }
        "m_iCSUNumToKill" => {
            "Required defeat counts. Each element pairs with the same element in Target enemies."
        }
        "m_iCSUItemID" => {
            "Quest item IDs for the collection objective. Each element pairs with the same element in Objective item counts; zero leaves the slot unused."
        }
        "m_iCSUItemNumNeeded" => {
            "Required quest item counts, paired with Objective items by element position."
        }
        "m_iCSTReqMission" => {
            "Mission IDs that must already be completed. Zero leaves the element unused."
        }
        "m_iCSUCheckTimer" => "Objective time limit in seconds. Zero means no time limit.",
        "m_fScale" => "Visual model scale multiplier.",
        _ => {
            "Additional source parameter. Its exact gameplay effect has not been documented in this editor. The original key and value type are preserved."
        }
    };
    let key = if fallback.starts_with("Additional source") {
        "help.unknown".into()
    } else {
        format!("help.{field}")
    };
    text(l, lang, &key, fallback)
}
pub(super) fn identity(table: &str) -> Option<&'static str> {
    if table.contains("/m_pAnimationTable/") {
        return None;
    }
    match table.rsplit('/').next()? {
        "m_pNpcData" => Some("m_iNpcNumber"),
        "m_pNanoData" => Some("m_iNanoNumber"),
        "m_pMissionData" => Some("m_iHTaskID"),
        "m_pRewardData" => Some("m_iMissionRewardID"),
        "m_pNanoTuneData" => Some("m_iTuneNumber"),
        "m_pItemData" if table.contains("ItemTable/") => Some("m_iItemNumber"),
        _ => None,
    }
}
pub(super) fn name_field(table: &str) -> Option<&'static str> {
    if table.contains("AnimationTable") {
        return None;
    }
    match table.rsplit('/').next()? {
        "m_pNpcData" => Some("m_iNpcName"),
        "m_pNanoData" => Some("m_iNanoName"),
        "m_pMissionData" => Some("m_iHMissionName"),
        "m_pItemData" if table.contains("ItemTable/") => Some("m_iItemName"),
        _ => None,
    }
}
pub(super) fn section(field: &str) -> &'static str {
    match field {
        "m_iNpcNumber" | "m_iNanoNumber" | "m_iItemNumber" | "m_iHTaskID" | "m_iHMissionID"
        | "m_iMissionRewardID" | "m_iTuneNumber" | "m_iNpcName" | "m_iNanoName" | "m_iItemName"
        | "m_iHMissionName" | "m_strName" | "m_pstrNameString" => "identity",
        "m_iMesh" | "m_iIcon" | "m_iIcon1" | "m_fScale" | "m_iRadius" | "m_iHeight" => "appearance",
        "m_iHP" | "m_iNpcLevel" | "m_iNpcStyle" | "m_iStyle" | "m_iNanoBattery1" | "m_iPower"
        | "m_iProtection" | "m_iAccuracy" | "m_iDodge" => "combat",
        "m_iHCurrentObjective"
        | "m_iHNPCID"
        | "m_iHTerminatorNPCID"
        | "m_iSUOutgoingTask"
        | "m_iFOutgoingTask"
        | "m_iSUReward"
        | "m_iSTNanoID"
        | "m_iSTGrantTimer"
        | "m_iHTaskType"
        | "m_iHMissionType"
        | "m_iSTJournalIDAdd"
        | "m_iHJournalNPCID"
        | "m_iSTGrantWayPoint"
        | "m_iHDifficultyType" => "mission",
        _ => "other",
    }
}
pub(super) fn ordered_fields(fields: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|f| {
        let group = ["identity", "combat", "mission", "appearance", "other"]
            .iter()
            .position(|s| *s == section(f))
            .unwrap_or(4);
        let rank = if f.ends_with("Number") || f.ends_with("ID") {
            0
        } else if f.to_lowercase().contains("name") {
            1
        } else {
            2
        };
        (group, rank, f.clone())
    });
    fields
}
pub(super) fn required(table: &str) -> &'static [&'static str] {
    if table.contains("/m_pAnimationTable/") {
        return &[];
    }
    match table.rsplit('/').next().unwrap_or("") {
        "m_pNpcData" => &[
            "m_iNpcNumber",
            "m_iNpcName",
            "m_iHP",
            "m_iNpcLevel",
            "m_iNpcStyle",
            "m_iNpcType",
            "m_iRadius",
            "m_iHeight",
            "m_iSightRange",
        ],
        "m_pNanoData" => &[
            "m_iNanoNumber",
            "m_iNanoName",
            "m_iIcon1",
            "m_iStyle",
            "m_iNanoBattery1",
            "m_iNanoSet",
        ],
        "m_pMissionData" => &[
            "m_iHTaskID",
            "m_iHMissionID",
            "m_iHMissionName",
            "m_iHCurrentObjective",
            "m_iHMissionType",
            "m_iHTaskType",
            "m_iHNPCID",
            "m_iHTerminatorNPCID",
            "m_iSUOutgoingTask",
            "m_iFOutgoingTask",
            "m_iSUReward",
            "m_iSTGrantTimer",
            "m_iSTNanoID",
            "m_iSTJournalIDAdd",
            "m_iHDifficultyType",
            "m_iHJournalNPCID",
            "m_iSTGrantWayPoint",
        ],
        "m_pRewardData" => &["m_iMissionRewardID", "m_iCash", "m_iFusionMatter"],
        "m_pJournalData" => mission_journal::TEXT_FIELDS,
        "m_pNpcStringData"
        | "m_pNanoStringData"
        | "m_pItemStringData"
        | "m_pNanoTuneStringData" => &["m_strName"],
        "m_pMissionStringData" => &["m_pstrNameString"],
        "m_pItemData" if table.contains("ItemTable/") => {
            &["m_iItemNumber", "m_iItemName", "m_iIcon"]
        }
        _ => &[],
    }
}
pub(super) fn overview(table: &str, fields: &[String]) -> Vec<String> {
    let preferred: &[&str] = match table.rsplit('/').next().unwrap_or("") {
        "m_pNpcData" if !table.contains("AnimationTable") => {
            &["m_iNpcNumber", "m_iNpcName", "m_iNpcLevel", "m_iHP"]
        }
        "m_pNanoData" if !table.contains("AnimationTable") => &[
            "m_iNanoNumber",
            "m_iNanoName",
            "m_iNanoBattery1",
            "m_iStyle",
        ],
        "m_pMissionData" => &[
            "m_iHTaskID",
            "m_iHMissionName",
            "m_iHCurrentObjective",
            "m_iSUReward",
        ],
        "m_pRewardData" => &["m_iMissionRewardID", "m_iCash", "m_iFusionMatter"],
        "m_pItemData" if table.contains("ItemTable/") => &[
            "m_iItemNumber",
            "m_iItemName",
            "m_iMinReqLev",
            "m_iItemPrice",
        ],
        _ if table.ends_with("StringData") => &[
            "m_strName",
            "m_pstrNameString",
            "m_strComment",
            "m_strComment1",
            "m_strComment2",
        ],
        _ => &[],
    };
    if preferred.is_empty() {
        fields.iter().take(4).cloned().collect()
    } else {
        preferred
            .iter()
            .filter(|f| fields.iter().any(|s| s == **f))
            .map(|f| (*f).to_owned())
            .collect()
    }
}
pub(super) fn basic(table: &str, fields: &[String]) -> Vec<String> {
    let mut selected = required(table)
        .iter()
        .map(|f| (*f).to_owned())
        .collect::<Vec<_>>();
    selected.extend(
        [
            "m_strName",
            "m_strComment",
            "m_iMesh",
            "m_iIcon1",
            "m_iBarkerNumber",
            "m_iWalkSpeed",
            "m_iRunSpeed",
            "m_fScale",
        ]
        .iter()
        .map(|f| (*f).to_owned()),
    );
    selected.extend(
        ["m_pstrNameString", "m_strComment1", "m_strComment2"]
            .iter()
            .map(|f| (*f).to_owned()),
    );
    selected.retain(|f| fields.contains(f));
    if selected.is_empty() {
        fields.iter().take(12).cloned().collect()
    } else {
        selected
    }
}
pub(super) struct FieldLimits {
    pub min: i64,
    pub max: Option<i64>,
    pub error: &'static str,
}

pub(super) fn limits(table: &str, field: &str) -> Option<FieldLimits> {
    let npc = table.ends_with("/m_pNpcTable/m_pNpcData");
    let nano = table.ends_with("/m_pNanoTable/m_pNanoData");
    if npc && field == "m_iNpcStyle" || nano && field == "m_iStyle" {
        return Some(FieldLimits {
            min: 0,
            max: Some(2),
            error: "style",
        });
    }
    if nano && field == "m_iNanoNumber" {
        return Some(FieldLimits {
            min: 1,
            max: Some(i16::MAX as i64),
            error: "i16",
        });
    }
    if nano && field == "m_iNanoBattery1" {
        return Some(FieldLimits {
            min: 1,
            max: Some(i16::MAX as i64),
            error: "stamina",
        });
    }
    if npc
        && matches!(
            field,
            "m_iHP" | "m_iNpcLevel" | "m_iRadius" | "m_iHeight" | "m_iSightRange"
        )
    {
        return Some(FieldLimits {
            min: 0,
            max: None,
            error: "nonnegative",
        });
    }
    if identity(table) == Some(field) {
        return Some(FieldLimits {
            min: 1,
            max: None,
            error: "positive",
        });
    }
    None
}

pub(super) fn invalid(table: &str, field: &str, value: &Value) -> Option<&'static str> {
    if table.ends_with("/m_pMissionTable/m_pRewardData") {
        if let Some(error)=mission_reward::invalid(field,value){return Some(error);}
    }
    if table.ends_with("/m_pMissionTable/m_pMissionData") {
        if let Some(error) = mission_fields::invalid(field, value) { return Some(error); }
    }
    let n = value.as_i64()?;
    // Keep the existing identity error for zero/negative Nano IDs.
    if identity(table) == Some(field) && n <= 0 {
        return Some("positive");
    }
    let limits = limits(table, field)?;
    (n < limits.min || limits.max.is_some_and(|max| n > max)).then_some(limits.error)
}
