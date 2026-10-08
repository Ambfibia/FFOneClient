//! Read-only stage summaries. References retain their original XDT domains.
use super::*;

pub(super) const WIDTH: f32 = 460.;
pub(super) const EVENT_HEIGHT: f32 = 94.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Channel {
    Journal,
    Message,
    Bubble,
    Email,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Start,
    Complete,
    Failure,
    Available,
}
#[derive(Clone, Debug)]
pub(super) struct Event {
    pub field: &'static str,
    pub channel: Channel,
    pub phase: Phase,
    pub id: i64,
    pub npc: i64,
    pub slot:Option<usize>,
}
pub(super) fn events(row: &Value) -> Vec<Event> {
    let mut result = Vec::new();
    for (phase, journal, message, speaker, bubble, actor) in [
        (
            Phase::Start,
            "m_iSTJournalIDAdd",
            "m_iSTMessageTextID",
            "m_iSTMessageSendNPC",
            "m_iSTDialogBubble",
            "m_iSTDialogBubbleNPCID",
        ),
        (
            Phase::Complete,
            "m_iSUJournaliDAdd",
            "m_iSUMessagetextID",
            "m_iSUMessageSendNPC",
            "m_iSUDialogBubble",
            "m_iSUDialogBubbleNPCID",
        ),
        (
            Phase::Failure,
            "m_iFJournalIDAdd",
            "m_iFMessageTextID",
            "m_iFMessageSendNPC",
            "m_iFDialogBubble",
            "m_iFDialogBubbleNPCID",
        ),
    ] {
        for (channel, field, npc_field) in [
            (Channel::Journal, journal, "m_iHJournalNPCID"),
            (Channel::Message, message, speaker),
            (Channel::Bubble, bubble, actor),
        ] {
            if let Some(id) = row[field].as_i64().filter(|id| *id > 0) {
                result.push(Event {
                    field,
                    channel,
                    phase,
                    id,
                    npc: row[npc_field].as_i64().unwrap_or(0),
                    slot:None,
                });
            }
        }
    }
    for (phase,field,route,speaker) in [(Phase::Start,"m_iSTMessageTextID","m_iSTMessageType","m_iSTMessageSendNPC"),(Phase::Complete,"m_iSUMessagetextID","m_iSUMessageType","m_iSUMessageSendNPC"),(Phase::Failure,"m_iFMessageTextID","m_iFMessageType","m_iFMessageSendNPC")] {
        let flags=row[route].as_i64().unwrap_or(0);
        if row[route].as_i64().is_some() && flags & 2 == 0 {result.retain(|e|!(e.field==field && e.channel==Channel::Message));}
        if flags & 4 != 0 {
            let separate=mail_fields::email_field(row,field);
            let field=if row[separate].as_i64().is_some_and(|id|id>0){separate}else{field};
            let npc=mission_events::speaker(field).and_then(|key|row[key].as_i64()).filter(|id|*id>0).unwrap_or_else(||row[speaker].as_i64().unwrap_or(0));
            if let Some(id)=row[field].as_i64().filter(|id|*id>0) {result.push(Event{field,channel:Channel::Email,phase,id,npc,slot:None});}
        }
    }
    for (slot,npc) in [707,728,731,732,730].into_iter().enumerate() {
        if let Some(id)=row["m_iMentorEmailID"].get(slot).and_then(Value::as_i64).filter(|id|*id>0) {
            result.push(Event{field:"m_iMentorEmailID",channel:Channel::Email,phase:Phase::Available,id,npc,slot:Some(slot)});
        }
    }
    result
}

#[derive(Debug)]
pub(super) struct Goal {
    pub kind: &'static str,
    pub id: i64,
    pub count: Option<i64>,
    pub drop_rate: Option<i64>,
    pub sources: Vec<i64>,
}
fn positive(row: &Value, field: &str) -> i64 {
    row[field].as_i64().filter(|id| *id > 0).unwrap_or(0)
}
fn slot(row: &Value, field: &str, index: usize) -> i64 {
    row[field].get(index).and_then(Value::as_i64).unwrap_or(0)
}
pub(super) fn goals(row: &Value) -> Vec<Goal> {
    let mut result = Vec::new();
    let escort = positive(row, "m_iCSUDEFNPCID");
    if escort > 0 {
        let follows = positive(row, "m_iCSUDEPNPCFollow") > 0
            || row["m_iHTaskType"] == 6 && positive(row, "m_iHTerminatorNPCID") > 0;
        result.push(Goal {
            kind: if follows { "escort" } else { "defend" },
            id: escort,
            count: None,
            drop_rate: None,
            sources: Vec::new(),
        });
    }
    for i in 0..3 {
        let id = slot(row, "m_iCSUItemID", i);
        let count = slot(row, "m_iCSUItemNumNeeded", i);
        if id <= 0 || count <= 0 {
            continue;
        }
        // The drop slot belongs to STItemID; it need not have the same position as CSUItemID.
        let drop_rate = (0..3)
            .find(|j| slot(row, "m_iSTItemID", *j) == id)
            .map(|j| slot(row, "m_iSTItemDropRate", j))
            .filter(|rate| *rate > 0);
        let sources = if drop_rate.is_some() {
            (0..3)
                .map(|j| slot(row, "m_iCSUEnemyID", j))
                .filter(|id| *id > 0)
                .collect()
        } else {
            Vec::new()
        };
        result.push(Goal {
            kind: if row["m_iHTaskType"] == 4 {
                "deliver"
            } else {
                "collect"
            },
            id,
            count: Some(count),
            drop_rate,
            sources,
        });
    }
    for i in 0..3 {
        let id = slot(row, "m_iCSUEnemyID", i);
        let count = slot(row, "m_iCSUNumToKill", i);
        if id > 0 && count > 0 {
            result.push(Goal {
                kind: "defeat",
                id,
                count: Some(count),
                drop_rate: None,
                sources: Vec::new(),
            });
        }
    }
    // Empty slots and unrelated objective templates never become visible rows.
    if result.is_empty() {
        let target = positive(row, "m_iHTerminatorNPCID");
        let waypoint = positive(row, "m_iSTGrantWayPoint");
        let id = if target > 0 { target } else { waypoint };
        if id > 0 {
            result.push(Goal {
                kind: if row["m_iHTaskType"] == 2 {
                    "goto"
                } else if row["m_iHTaskType"] == 3 {
                    "use"
                } else {
                    "talk"
                },
                id,
                count: None,
                drop_rate: None,
                sources: Vec::new(),
            });
        }
    }
    result
}
pub(super) fn size(row: &Value) -> Vec2 {
    let goals = goals(row);
    Vec2::new(
        WIDTH,
        154. + if goals.is_empty() {
            42.
        } else {
            goals.iter().map(|g| goal_height(g) + 4.).sum()
        } + events(row).len() as f32 * (EVENT_HEIGHT + 8.),
    )
}
pub(super) fn goal_height(goal: &Goal) -> f32 {
    38. + goal.sources.len() as f32 * 38.
}

fn table_rows<'a>(e: &'a XdtEditor, suffix: &str) -> Option<&'a Vec<Value>> {
    let table = e
        .tables
        .iter()
        .find(|table| table.label.ends_with(suffix))?;
    e.document.pointer(&table.pointer)?.as_array()
}
pub(super) fn phrase(
    l: &Localization,
    lang: &Language,
    key: &str,
    fallback: &str,
    args: &[(&str, String)],
) -> String {
    let mut text = LocalizedText::new(format!("ui.editor.xdt.preview.{key}"), fallback);
    for (key, value) in args {
        text = text.with_arg(*key, value.clone());
    }
    l.text(lang, &text)
}
fn localized(
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    key: String,
    fallback: &str,
) -> String {
    let locale = usize::from(lang.effective == "ru");
    e.workspace
        .locale_drafts
        .get(&key)
        .and_then(|values| values[locale].as_ref())
        .filter(|text| !text.trim().is_empty())
        .cloned()
        .or_else(|| {
            e.workspace.locale_base[locale]
                .get(&key)
                .filter(|text| !text.trim().is_empty())
                .cloned()
        })
        .unwrap_or_else(|| l.text(lang, &LocalizedText::new(key, fallback)))
}
pub(super) fn string(e: &XdtEditor, l: &Localization, lang: &Language, id: i64) -> String {
    let raw = usize::try_from(id)
        .ok()
        .and_then(|id| table_rows(e, "/m_pMissionTable/m_pMissionStringData")?.get(id))
        .and_then(|row| row["m_pstrNameString"].as_str());
    raw.map(|raw| {
        localized(
            e,
            l,
            lang,
            format!("content.tabledata.mission.mission_string.{id}.str_name_string"),
            raw,
        )
    })
    .unwrap_or_else(|| {
        phrase(
            l,
            lang,
            "missing_text",
            "Missing text (ID: {id})",
            &[("id", id.to_string())],
        )
    })
}
pub(super) fn npc_name(e: &XdtEditor, l: &Localization, lang: &Language, id: i64) -> String {
    if id <= 0 {
        return phrase(l, lang, "no_speaker", "NPC not assigned", &[]);
    }
    let name = table_rows(e, "/m_pNpcTable/m_pNpcData")
        .and_then(|rows| rows.iter().find(|v| v["m_iNpcNumber"] == id))
        .and_then(|v| v["m_iNpcName"].as_u64())
        .and_then(|i| table_rows(e, "/m_pNpcTable/m_pNpcStringData")?.get(i as usize))
        .and_then(|v| v["m_strName"].as_str())
        .unwrap_or("NPC");
    format!(
        "{} (ID: {id})",
        excerpt(
            &localized(e, l, lang, format!("content.npc.{id}.name"), name),
            32
        )
    )
}
pub(super) fn goal_text(e: &XdtEditor, l: &Localization, lang: &Language, goal: &Goal) -> String {
    let (key, fallback) = match goal.kind {
        "collect" => ("collect", "Collect"),
        "deliver" => ("deliver", "Deliver"),
        "defeat" => ("defeat", "Defeat"),
        "escort" => ("escort", "Escort"),
        "defend" => ("defend", "Defend"),
        "goto" => ("goto", "Reach"),
        "use" => ("use", "Use at"),
        _ => ("talk", "Talk to"),
    };
    let target = if matches!(goal.kind, "collect" | "deliver") {
        let raw = table_rows(e, "/m_pQuestItemTable/m_pItemData")
            .and_then(|rows| rows.iter().find(|v| v["m_iItemNumber"] == goal.id))
            .and_then(|v| v["m_iItemName"].as_u64())
            .and_then(|i| table_rows(e, "/m_pQuestItemTable/m_pItemStringData")?.get(i as usize))
            .and_then(|v| v["m_strName"].as_str())
            .unwrap_or("");
        format!(
            "{} (ID: {})",
            excerpt(
                &localized(
                    e,
                    l,
                    lang,
                    format!("content.quest_item.{}.name", goal.id),
                    raw
                ),
                32
            ),
            goal.id
        )
    } else {
        npc_name(e, l, lang, goal.id)
    };
    let mut text = format!("{}: {target}", phrase(l, lang, key, fallback, &[]));
    if let Some(count) = goal.count {
        text.push_str(&format!(" ×{count}"));
    }
    if let Some(rate) = goal.drop_rate {
        text.push_str(&format!(
            " · {}",
            phrase(
                l,
                lang,
                "drop",
                "drop {rate}%",
                &[("rate", rate.to_string())]
            )
        ));
    }
    if !goal.sources.is_empty() {
        for id in &goal.sources {
            let names = npc_name(e, l, lang, *id);
            text.push_str(&format!(
                "\n{}",
                phrase(l, lang, "from", "From: {npcs}", &[("npcs", names)])
            ));
        }
    }
    text
}
pub(super) fn event_text(
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    event: &Event,
) -> String {
    if event.channel != Channel::Journal {
        return string(e, l, lang, event.id);
    }
    let Some(row) = table_rows(e, "/m_pMissionTable/m_pJournalData")
        .and_then(|rows| rows.get(event.id as usize))
    else {
        return phrase(
            l,
            lang,
            "missing_journal",
            "Missing journal entry (ID: {id})",
            &[("id", event.id.to_string())],
        );
    };
    let fields = if event.phase == Phase::Complete {
        [
            "m_iDetaileMissionCompleteSummary",
            "m_iMissionCompleteSummary",
            "m_iDetailedTaskDesc",
            "m_iDetaileMissionDesc",
            "m_iTaskSummary",
            "m_iMissionSummary",
        ]
    } else {
        [
            "m_iDetailedTaskDesc",
            "m_iDetaileMissionDesc",
            "m_iTaskSummary",
            "m_iMissionSummary",
            "m_iDetaileMissionCompleteSummary",
            "m_iMissionCompleteSummary",
        ]
    };
    fields
        .into_iter()
        .filter_map(|f| row[f].as_i64().filter(|id| *id > 0))
        .map(|id| string(e, l, lang, id))
        .find(|text| !text.trim().is_empty())
        .unwrap_or_else(|| phrase(l, lang, "empty_journal", "Journal text is empty", &[]))
}
pub(super) fn event_heading(l: &Localization, lang: &Language, event: &Event) -> String {
    let (kind, caption) = match event.channel {
        Channel::Journal => ("journal", "Journal"),
        Channel::Message => ("message", "NanoCom"),
        Channel::Email => ("email", "Email"),
        Channel::Bubble => ("bubble", "Overhead speech"),
    };
    let (phase, when) = match event.phase {
        Phase::Start => ("start", "At start"),
        Phase::Available => ("available", "When available"),
        Phase::Complete => ("complete", "After completion"),
        Phase::Failure => ("failure", "On failure"),
    };
    format!(
        "{} · {} · ID {}",
        phrase(l, lang, kind, caption, &[]),
        phrase(l, lang, phase, when, &[]),
        event.id
    )
}
pub(super) fn excerpt(text: &str, limit: usize) -> String {
    let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.chars().count() <= limit {
        clean
    } else {
        format!(
            "{}…",
            clean
                .chars()
                .take(limit.saturating_sub(1))
                .collect::<String>()
                .trim_end()
        )
    }
}
