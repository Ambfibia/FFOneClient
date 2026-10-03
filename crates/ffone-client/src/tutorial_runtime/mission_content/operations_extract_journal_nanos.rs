use super::*;

pub(super) fn extract_gameplay_nanos(
    rows: &[Value],
    icons: &[Value],
    strings: &[Value],
    installed_icon_paths: &BTreeSet<String>,
) -> TutorialMissionContentResult<BTreeMap<i16, GameplayNanoUiDefinition>> {
    let mut definitions = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pNanoData[{row_index}]");
        let row = value_object(row, &context)?;
        let nano_number = required_i32(row, "m_iNanoNumber", &context)?;
        if nano_number == 0 {
            continue;
        }
        let nano_id = i16::try_from(nano_number)
            .map_err(|_| invalid(format!("{context}.m_iNanoNumber is outside i16")))?;
        if nano_id < 0 {
            return Err(invalid(format!("{context}.m_iNanoNumber is negative")));
        }
        let sort_number = required_i32(row, "m_iNanoSet", &context)?;
        let icon_index = required_i32(row, "m_iIcon1", &context)?;
        let icon_context = format!("{context}.m_iIcon1");
        let icon = indexed_table_object(icons, icon_index, "icon", &icon_context)?;
        let raw_icon_number = required_i32(icon, "m_iIconNumber", &icon_context)?;
        let icon_number = u16::try_from(raw_icon_number).map_err(|_| {
            invalid(format!(
                "{icon_context} resolves outside u16: {raw_icon_number}"
            ))
        })?;
        let raw_icon_type = required_i32(icon, "m_iIconType", &icon_context)?;
        let icon_type = u8::try_from(raw_icon_type).map_err(|_| {
            invalid(format!(
                "{icon_context}.m_iIconType is outside u8: {raw_icon_type}"
            ))
        })?;
        let name_index = required_i32(row, "m_iNanoName", &context)?;
        let name = indexed_string(
            strings,
            name_index,
            "m_strName",
            &format!("{context}.m_iNanoName"),
        )?;
        let named_slug = match nano_id {
            // The restored quest identity and retained Academy alias have the
            // same display name but own different models and portraits.
            41 => Some("holo-nano"),
            52 => Some("van-kleiss"),
            69 => Some("ghostfreak"),
            70 => Some("upgrade"),
            _ => merged_nano_named_icon_slug(&name),
        };
        let icon_path = named_slug
            .map(|slug| format!("icons/entities/nanos/nanoicon_{slug}.png"))
            .filter(|path| installed_icon_paths.contains(path.as_str()))
            .or_else(|| {
                avatar_util_semantic_icon_path(icon_type, u32::from(icon_number))
                    .filter(|path| installed_icon_paths.contains(path.as_str()))
            });
        let ready_icon_path = named_slug
            .map(|slug| format!("icons/entities/nanos/ready/nanoready_{slug}.png"))
            .filter(|path| installed_icon_paths.contains(path.as_str()))
            .or_else(|| {
                nano_ready_semantic_icon_path(u32::from(icon_number))
                    .filter(|path| installed_icon_paths.contains(path.as_str()))
            });
        let attribute = indexed_string(
            strings,
            name_index,
            "m_strComment1",
            &format!("{context}.m_iNanoName"),
        )?;
        let style = required_i32(row, "m_iStyle", &context)?;
        let style = u8::try_from(style)
            .ok()
            .filter(|style| *style <= 2)
            .ok_or_else(|| invalid(format!("{context}.m_iStyle must be 0, 1, or 2")))?;
        let max_stamina = required_i32(row, "m_iNanoBattery1", &context)?;
        let max_stamina = i16::try_from(max_stamina)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| invalid(format!("{context}.m_iNanoBattery1 must be a positive i16")))?;
        let definition = GameplayNanoUiDefinition {
            nano_id,
            sort_number,
            name,
            attribute,
            icon_number,
            icon_path,
            ready_icon_path,
            style,
            max_stamina,
        };
        if let Some(previous) = definitions.insert(nano_id, definition.clone())
            && previous != definition
        {
            return Err(invalid(format!(
                "contradictory duplicate Nano ID {nano_id} in m_pNanoData"
            )));
        }
    }
    Ok(definitions)
}

pub(super) fn merged_nano_named_icon_slug(name: &str) -> Option<&'static str> {
    Some(match name {
        "Buttercup" => "buttercup",
        "Numbuh Two" => "numbuh-two",
        "Eddy" => "eddy",
        "Eduardo" => "eduardo",
        "Blossom" => "blossom",
        "Wilt" => "wilt",
        "Dee Dee" => "dee-dee",
        "Numbuh Five" => "numbuh-five",
        "Edd" => "edd",
        "Megas" => "megas",
        "Billy" => "billy",
        "Prof. Utonium" => "prof-utonium",
        "Him" => "him",
        "Bloo" => "bloo",
        "Bubbles" => "bubbles",
        "Demongo" => "demongo",
        "Fourarms" => "fourarms",
        "Numbuh One" => "numbuh-one",
        "Mojo Jojo" => "mojo-jojo",
        "Mandark" => "mandark",
        "Grim" => "grim",
        "Dexter" => "dexter",
        "Vilgax" => "vilgax",
        "Swampfire" => "swampfire",
        "Mandy" => "mandy",
        "Coco" => "coco",
        "Mac" => "mac",
        "Numbuh Three" => "numbuh-three",
        "Hex" => "hex",
        "Juniper Lee" => "juniper-lee",
        "Numbuh Four" => "numbuh-four",
        "Ed" => "ed",
        "Courage" => "courage",
        "Samurai Jack" => "samurai-jack",
        "Aku" => "aku",
        "Humongosaur" => "humongosaur",
        "Coop" => "coop",
        "Belladonna" => "belladonna",
        "Computress" => "computress",
        "Runty" => "runty",
        "Panini" => "panini",
        "Jack O'Lantern" => "jack-olantern",
        "Ben Tennyson" => "ben-10",
        "Finn" => "finn",
        "Rex" => "rex",
        "Alien X" => "alien-x",
        "Johnny Bravo" => "johnny-bravo",
        "Cheese" => "cheese",
        "Unstable Nano" => "van-kleiss",
        "Rigby" => "rigby",
        "Flapjack" => "flapjack",
        "Jake" => "jake",
        "Chowder" => "chowder",
        "Johnny Test" => "johnny-test",
        "Titan" => "titan",
        "Zak Saturday" => "zak-saturday",
        "Ice King" => "ice-king",
        "Gumball" => "gumball",
        "Ampfibian" => "ampfibian",
        "Rath" => "rath",
        "Darwin" => "darwin",
        "Mordecai" => "mordecai",
        "P.Bubblegum" => "p-bubblegum",
        "Van Kleiss" => "van-kleiss",
        _ => return None,
    })
}

pub(super) fn extract_gameplay_nano_tune_fusion_matter(
    table_root: &Map<String, Value>,
) -> TutorialMissionContentResult<BTreeMap<i32, i32>> {
    let Some(avatar_table) = table_root.get("m_pAvatarTable") else {
        return Ok(BTreeMap::new());
    };
    let avatar_table = value_object(avatar_table, "m_pAvatarTable")?;
    let rows = required_array(avatar_table, "m_pAvatarGrowData", "m_pAvatarTable")?;
    let mut costs = BTreeMap::new();
    for (level, row) in rows.iter().enumerate() {
        let context = format!("m_pAvatarGrowData[{level}]");
        let row = value_object(row, &context)?;
        let Some(cost) = optional_i32(row, "m_iReqBlob_NanoTune", &context)? else {
            continue;
        };
        let level =
            i32::try_from(level).map_err(|_| invalid("m_pAvatarGrowData index is outside i32"))?;
        costs.insert(level, cost);
    }
    Ok(costs)
}

pub(super) fn extract_gameplay_skills(
    rows: &[Value],
    icons: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i16, GameplaySkillUiDefinition>> {
    let mut definitions = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pSkillData[{row_index}]");
        let row = value_object(row, &context)?;
        let skill_number = required_i32(row, "m_iSkillNumber", &context)?;
        if skill_number == 0 {
            continue;
        }
        let skill_id = i16::try_from(skill_number)
            .map_err(|_| invalid(format!("{context}.m_iSkillNumber is outside i16")))?;
        if skill_id < 0 {
            return Err(invalid(format!("{context}.m_iSkillNumber is negative")));
        }
        let icon_index = required_i32(row, "m_iIcon", &context)?;
        let icon_number = indexed_icon_number(icons, icon_index, &format!("{context}.m_iIcon"))?;
        let definition = GameplaySkillUiDefinition {
            skill_id,
            skill_type: required_i32(row, "m_iSkillType", &context)?,
            values_a: required_i32_array4(row, "m_iValueA", &context)?,
            icon_number,
            active: required_i32(row, "m_iBatteryDrainType", &context)? == 1,
            effect_type: required_i32(row, "m_iEffectType", &context)?,
            target_effect: required_i32(row, "m_iTargetEffect", &context)?,
            target: required_i32(row, "m_iEffectTarget", &context)?,
            target_type: required_i32(row, "m_iTargetType", &context)?,
            range: required_i32(row, "m_iEffectRange", &context)?,
            area: required_i32(row, "m_iEffectArea", &context)?,
            angle: required_i32(row, "m_iEffectAngle", &context)?,
            target_number: required_i32(row, "m_iTargetNumber", &context)?,
            cooldown: required_i32(row, "m_iCoolTime", &context)?,
            cool_type: required_i32(row, "m_iCoolType", &context)?,
        };
        if let Some(previous) = definitions.insert(skill_id, definition)
            && previous != definition
        {
            return Err(invalid(format!(
                "contradictory duplicate Skill ID {skill_id} in m_pSkillData"
            )));
        }
    }
    Ok(definitions)
}

pub(super) fn extract_gameplay_skill_buffs(
    rows: &[Value],
    icons: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, GameplaySkillBuffUiDefinition>> {
    let mut definitions = BTreeMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        let context = format!("m_pSkillBuffData[{row_index}]");
        let row = value_object(row, &context)?;
        let buff_id = required_i32(row, "m_iBuffNumber", &context)?;
        if buff_id == 0 {
            continue;
        }
        if buff_id < 0 {
            return Err(invalid(format!("{context}.m_iBuffNumber is negative")));
        }
        let icon_index = required_i32(row, "m_iBuffIcon", &context)?;
        let cash_icon_index = required_i32(row, "m_iBuffCashIcon", &context)?;
        let definition = GameplaySkillBuffUiDefinition {
            buff_id,
            effect_id: required_i32(row, "m_iBuffEffect", &context)?,
            instant_effect_id: required_i32(row, "m_iBuffEffectInstant", &context)?,
            icon_number: indexed_icon_number(icons, icon_index, &format!("{context}.m_iBuffIcon"))?,
            cash_icon_number: indexed_icon_number(
                icons,
                cash_icon_index,
                &format!("{context}.m_iBuffCashIcon"),
            )?,
        };
        if let Some(previous) = definitions.insert(buff_id, definition)
            && previous != definition
        {
            return Err(invalid(format!(
                "contradictory duplicate buff ID {buff_id} in m_pSkillBuffData"
            )));
        }
    }
    Ok(definitions)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn extract_journal_nanos(
    missions: &BTreeMap<i32, TutorialMissionDefinition>,
    nano_rows: &[Value],
    nano_icons: &[Value],
    nano_strings: &[Value],
    nano_tunes: &[Value],
    nano_tune_strings: &[Value],
    skill_rows: &[Value],
    skill_icons: &[Value],
) -> TutorialMissionContentResult<BTreeMap<i32, TutorialNanoJournalUi>> {
    let mut required_nano_ids = BTreeSet::new();
    for mission in missions
        .values()
        .filter(|mission| mission.mission_type == TutorialMissionType::Nano)
    {
        let nano_id = mission.provenance.nano_id;
        if nano_id <= 0 {
            return Err(invalid(format!(
                "tutorial Nano mission task {} has invalid m_iSTNanoID {nano_id}",
                mission.provenance.task_id
            )));
        }
        required_nano_ids.insert(nano_id);
    }
    let mission_nano_ids = required_nano_ids.clone();

    // `CnGuiNanoFreeTuning.SetNanoInfo` is a gameplay owner, not a tutorial-only
    // journal projection. Consider every positive Nano row, but omit optional
    // gameplay rows whose legacy tune slots are incomplete. Mission-owned Nano
    // rows remain fail-closed because the journal must be complete for them.
    for (row_index, row) in nano_rows.iter().enumerate() {
        let context = format!("m_pNanoData[{row_index}]");
        let row = value_object(row, &context)?;
        let nano_id = required_i32(row, "m_iNanoNumber", &context)?;
        if nano_id > 0 {
            required_nano_ids.insert(nano_id);
        } else if nano_id < 0 {
            return Err(invalid(format!("{context}.m_iNanoNumber is negative")));
        }
    }

    let mut definitions = BTreeMap::new();
    for nano_id in required_nano_ids {
        let context = format!("tutorial m_iSTNanoID {nano_id}");
        let definition: TutorialMissionContentResult<TutorialNanoJournalUi> = (|| {
            let nano = indexed_table_object(nano_rows, nano_id, "m_pNanoData", &context)?;
            let row_nano_id = required_i32(nano, "m_iNanoNumber", &context)?;
            if row_nano_id != nano_id {
                return Err(invalid(format!(
                    "{context} resolves to m_iNanoNumber {row_nano_id}"
                )));
            }

            let name_id = required_i32(nano, "m_iNanoName", &context)?;
            let icon_id = required_i32(nano, "m_iIcon1", &context)?;
            let [first_tune, second_tune, third_tune] =
                required_i32_array3(nano, "m_iTune", &context)?;
            Ok(TutorialNanoJournalUi {
                nano_id,
                icon_number: indexed_icon_number(
                    nano_icons,
                    icon_id,
                    &format!("{context}.m_iIcon1"),
                )?,
                name: indexed_string(
                    nano_strings,
                    name_id,
                    "m_strName",
                    &format!("{context}.m_iNanoName"),
                )?,
                attribute: indexed_string(
                    nano_strings,
                    name_id,
                    "m_strComment1",
                    &format!("{context}.m_iNanoName"),
                )?,
                description: indexed_string(
                    nano_strings,
                    name_id,
                    "m_strComment",
                    &format!("{context}.m_iNanoName"),
                )?,
                skills: [
                    extract_journal_nano_skill(
                        first_tune,
                        nano_tunes,
                        nano_tune_strings,
                        skill_rows,
                        skill_icons,
                        &context,
                    )?,
                    extract_journal_nano_skill(
                        second_tune,
                        nano_tunes,
                        nano_tune_strings,
                        skill_rows,
                        skill_icons,
                        &context,
                    )?,
                    extract_journal_nano_skill(
                        third_tune,
                        nano_tunes,
                        nano_tune_strings,
                        skill_rows,
                        skill_icons,
                        &context,
                    )?,
                ],
            })
        })();
        let definition = match definition {
            Ok(definition) => definition,
            Err(error) if mission_nano_ids.contains(&nano_id) => return Err(error),
            Err(_) => continue,
        };
        definitions.insert(nano_id, definition);
    }
    Ok(definitions)
}

pub(super) fn extract_journal_nano_skill(
    tune_slot: i32,
    nano_tunes: &[Value],
    nano_tune_strings: &[Value],
    skill_rows: &[Value],
    skill_icons: &[Value],
    nano_context: &str,
) -> TutorialMissionContentResult<TutorialNanoJournalSkillUi> {
    if tune_slot <= 0 {
        return Err(invalid(format!(
            "{nano_context}.m_iTune contains invalid tune slot {tune_slot}"
        )));
    }
    let context = format!("{nano_context}.m_iTune -> m_pNanoTuneData[{tune_slot}]");
    let tune = indexed_table_object(nano_tunes, tune_slot, "m_pNanoTuneData", &context)?;
    let row_tune_id = required_i32(tune, "m_iTuneNumber", &context)?;
    if row_tune_id <= 0 {
        return Err(invalid(format!(
            "{context}.m_iTuneNumber must be positive, got {row_tune_id}"
        )));
    }
    let skill_id = required_i32(tune, "m_iSkillID", &context)?;
    if skill_id <= 0 {
        return Err(invalid(format!(
            "{context}.m_iSkillID must be positive, got {skill_id}"
        )));
    }
    let skill_context = format!("{context}.m_iSkillID -> m_pSkillData[{skill_id}]");
    let skill = indexed_table_object(skill_rows, skill_id, "m_pSkillData", &skill_context)?;
    let row_skill_id = required_i32(skill, "m_iSkillNumber", &skill_context)?;
    if row_skill_id != skill_id {
        return Err(invalid(format!(
            "{skill_context} has m_iSkillNumber {row_skill_id}"
        )));
    }
    let skill_icon_id = required_i32(skill, "m_iIcon", &skill_context)?;
    let tune_name_id = required_i32(tune, "m_iTuneName", &context)?;

    Ok(TutorialNanoJournalSkillUi {
        tune_id: row_tune_id,
        skill_id,
        icon_number: indexed_icon_number(
            skill_icons,
            skill_icon_id,
            &format!("{skill_context}.m_iIcon"),
        )?,
        name: indexed_string(
            nano_tune_strings,
            tune_name_id,
            "m_strName",
            &format!("{context}.m_iTuneName"),
        )?,
        type_label: indexed_string(
            nano_tune_strings,
            tune_name_id,
            "m_strComment1",
            &format!("{context}.m_iTuneName"),
        )?,
        description: indexed_string(
            nano_tune_strings,
            tune_name_id,
            "m_strComment",
            &format!("{context}.m_iTuneName"),
        )?,
        required_item_id: optional_i32(tune, "m_iReqItemID", &context)?.unwrap_or_default(),
        required_item_count: optional_i32(tune, "m_iReqItemCount", &context)?.unwrap_or_default(),
    })
}

pub(super) fn indexed_icon_number(
    rows: &[Value],
    source_id: i32,
    context: &str,
) -> TutorialMissionContentResult<u16> {
    let index = usize::try_from(source_id)
        .map_err(|_| invalid(format!("{context} has negative icon ID {source_id}")))?;
    let row = rows
        .get(index)
        .ok_or_else(|| invalid(format!("{context} references missing icon row {source_id}")))?;
    let row = value_object(row, &format!("{context} -> icon[{source_id}]"))?;
    let icon_number = required_i32(row, "m_iIconNumber", context)?;
    u16::try_from(icon_number)
        .map_err(|_| invalid(format!("{context} resolves outside u16: {icon_number}")))
}

pub(super) fn extract_mission_dialogue(
    row: &Map<String, Value>,
    strings: &[Value],
    text_field: &str,
    npc_field: &str,
    context: &str,
) -> TutorialMissionContentResult<Option<TutorialMissionDialogue>> {
    let string_id = optional_i32(row, text_field, context)?.unwrap_or(0);
    let npc_type = optional_i32(row, npc_field, context)?.unwrap_or(0);
    if npc_type <= 0 || string_id <= 0 || string_id as usize >= strings.len() {
        return Ok(None);
    }
    let text = indexed_string(strings, string_id, "m_pstrNameString", context)?;
    Ok(
        (text.chars().count() > 1).then_some(TutorialMissionDialogue {
            npc_type,
            string_id,
            text,
        }),
    )
}

pub(super) fn extract_mission_nanocom_message(
    strings: &[Value],
    npc_type: i32,
    message_type: i32,
    string_id: i32,
    context: &str,
) -> TutorialMissionContentResult<Option<TutorialMissionNanocomMessage>> {
    // `SetMissionMessage` returns before touching TableData unless all three
    // source gates select the passive NanoCom branch.
    if npc_type <= 0 || string_id <= 0 || message_type & 2 == 0 {
        return Ok(None);
    }
    let text = indexed_string(strings, string_id, "m_pstrNameString", context)?;
    if text.chars().count() <= 1 {
        return Ok(None);
    }
    Ok(Some(TutorialMissionNanocomMessage {
        npc_type,
        message_type,
        string_id,
        text,
    }))
}

pub(super) fn extract_gameplay_guide_nanocom(
    table: &Map<String, Value>,
) -> TutorialMissionContentResult<BTreeMap<i16, GameplayGuideNanocomDefinition>> {
    let rows = required_array(table, "m_pGuideData", "m_pGuideTable")?;
    let strings = required_array(table, "m_pGuideStringData", "m_pGuideTable")?;
    // Clean `GuideNpcNum[guide - 1]` order recovered from primary Assembly-CSharp.
    const GUIDE_NPC_TYPES: [i32; 5] = [707, 728, 731, 732, 730];
    let mut definitions = BTreeMap::new();
    for (raw_mentor, npc_type) in (1_i16..=5).zip(GUIDE_NPC_TYPES) {
        let row_index = usize::try_from(raw_mentor).expect("positive mentor index");
        let context = format!("m_pGuideTable.m_pGuideData[{row_index}]");
        let row = rows
            .get(row_index)
            .ok_or_else(|| invalid(format!("{context} is missing")))?;
        let row = value_object(row, &context)?;
        let login_mail_string_id = required_i32(row, "m_iLoginMail", &context)?;
        let login_no_mail_string_id = required_i32(row, "m_iLoginNomail", &context)?;
        let level_up_string_id = required_i32(row, "m_iLevelUp", &context)?;
        let definition = GameplayGuideNanocomDefinition {
            raw_mentor,
            npc_type,
            login_mail_string_id,
            login_mail_text: indexed_string(
                strings,
                login_mail_string_id,
                "m_pszString",
                &format!("{context}.m_iLoginMail"),
            )?,
            login_no_mail_string_id,
            login_no_mail_text: indexed_string(
                strings,
                login_no_mail_string_id,
                "m_pszString",
                &format!("{context}.m_iLoginNomail"),
            )?,
            level_up_string_id,
            level_up_text: indexed_string(
                strings,
                level_up_string_id,
                "m_pszString",
                &format!("{context}.m_iLevelUp"),
            )?,
        };
        definitions.insert(raw_mentor, definition);
    }
    Ok(definitions)
}
