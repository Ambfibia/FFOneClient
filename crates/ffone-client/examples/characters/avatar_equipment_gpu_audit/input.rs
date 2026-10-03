use super::*;

pub(super) fn resolve_job_look(
    data: &AuditData,
    job: &Job,
) -> Result<(NativePlayerLook, NativePlayerPartKind), String> {
    let base = match job.gender {
        AuditGender::Male => &data.male,
        AuditGender::Female => &data.female,
    };
    let mut equipment = [ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    }; 9];
    equipment[CharacterEquipSlot0104::UpperBody as usize].item_id = base.equipped.upper_body_id;
    equipment[CharacterEquipSlot0104::LowerBody as usize].item_id = base.equipped.lower_body_id;
    equipment[CharacterEquipSlot0104::Foot as usize].item_id = base.equipped.foot_id;
    let (slot, kind) = category_slot(job.category)
        .ok_or_else(|| format!("unsupported audit category {:?}", job.category))?;
    equipment[slot as usize].item_id = i16::try_from(job.item_number)
        .map_err(|_| format!("item {} is outside protocol i16", job.item_number))?;
    let look = data
        .data
        .resolve_pc_appearance(&PcAppearance0104 {
            id: i32::try_from(job.ordinal + 1).unwrap_or(i32::MAX),
            style: base.style.clone(),
            condition_bit_flag: 0,
            pc_state: 1,
            special_state: 0,
            level: 1,
            hp: 1_000,
            map_number: 0,
            position: [0, 0, 0],
            angle: 0,
            equipment,
            nano: Nano0104 {
                id: 0,
                skill_id: 0,
                stamina: 0,
            },
            render_type: 0,
        })
        .map_err(|error| error.to_string())?;
    Ok((look, kind))
}

pub(super) fn parse_category(value: &str) -> Result<AvatarItemCategory, String> {
    match value {
        "shirt" => Ok(AvatarItemCategory::Shirt),
        "pants" => Ok(AvatarItemCategory::Pants),
        "shoes" => Ok(AvatarItemCategory::Shoes),
        "hat" => Ok(AvatarItemCategory::Hat),
        "glasses" => Ok(AvatarItemCategory::Glasses),
        "back" => Ok(AvatarItemCategory::Back),
        other => Err(format!("unsupported wearable category {other:?}")),
    }
}
