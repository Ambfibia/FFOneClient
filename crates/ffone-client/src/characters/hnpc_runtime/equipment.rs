use super::*;
use crate::character_creation_data::LegacyHatPolicy;

pub(super) fn apply(
    look: &mut NativePlayerLook,
    items: &CharacterCreationAvatarItems,
    declared: Option<u8>,
) -> Result<(), String> {
    let hat = look
        .parts
        .iter()
        .find(|p| p.kind == NativePlayerPartKind::Hat);
    let policy = if let Some(hat) = hat {
        let types: std::collections::BTreeSet<_> = items
            .items
            .iter()
            .filter(|i| i.category == AvatarItemCategory::Hat)
            .filter(|i| {
                [&i.male, &i.female]
                    .iter()
                    .any(|v| v.models.iter().any(|m| m.native_asset.path == hat.glb))
            })
            .map(|i| i.equip_type)
            .collect();
        let equip_type = match declared {
            Some(t) if types.is_empty() || types.contains(&t) => t,
            Some(t) => {
                return Err(format!(
                    "HNPC hat {:?} cannot use equipType {t}",
                    hat.exact_route
                ));
            }
            None if types.len() == 1 => *types.first().unwrap(),
            None if types.is_empty() => return Ok(()), // Existing NPC-only accessory.
            // Some accepted NPC hats share a player model across several item
            // policies. Their authored body parts remain authoritative until
            // an inventory selection supplies the exact equipType.
            None => return Ok(()),
        };
        LegacyHatPolicy::from_equip_type(equip_type).map_err(|e| e.to_string())?
    } else {
        return Ok(());
    };
    look.parts.retain(|p| {
        !(p.kind == NativePlayerPartKind::Hair && policy.hair_variant.is_none()
            || p.kind == NativePlayerPartKind::Glasses && !policy.glasses_visible)
    });
    for part in &mut look.parts {
        let (category, variant) = match part.kind {
            NativePlayerPartKind::Face => (AvatarItemCategory::Face, policy.face_variant),
            NativePlayerPartKind::Hair => (AvatarItemCategory::Head, policy.hair_variant.unwrap()),
            _ => continue,
        };
        let visual = items
            .items
            .iter()
            .filter(|i| i.category == category)
            .map(|i| {
                if look.gender == PlayerRigGender::Male {
                    &i.male
                } else {
                    &i.female
                }
            })
            .find(|v| v.models.iter().any(|m| m.native_asset.path == part.glb));
        if let Some(visual) = visual {
            let suffix = format!("_type{variant:02}");
            let model = visual
                .models
                .iter()
                .find(|m| m.true_name.to_ascii_lowercase().ends_with(&suffix))
                .ok_or_else(|| format!("HNPC {:?} has no {suffix} model", part.kind))?;
            part.exact_route = model.exact_route.clone();
            part.glb = model.native_asset.path.clone();
        }
    }
    Ok(())
}
