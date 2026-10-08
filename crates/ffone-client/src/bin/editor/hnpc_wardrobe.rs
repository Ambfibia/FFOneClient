//! The same published inventory models and textures used by player equipment.
use super::*;
use ffone_client::player_preview::{NativePlayerPartAssembly, NativePlayerTexture};
use ffone_runtime_contracts::{
    AvatarItemCategory as Category, CharacterCreationAvatarItems, CharacterCreationRuntimeTextures,
};

impl HnpcEditor {
    pub(super) fn add_player_wardrobe(&mut self) -> Result<(), String> {
        let items: CharacterCreationAvatarItems = serde_json::from_slice(
            &fs::read(
                self.root
                    .join(ffone_runtime_contracts::CHARACTER_CREATION_AVATAR_ITEMS_PATH),
            )
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let textures: CharacterCreationRuntimeTextures = serde_json::from_slice(
            &fs::read(
                self.root
                    .join(ffone_runtime_contracts::CHARACTER_CREATION_RUNTIME_TEXTURES_PATH),
            )
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let ownership: Value = serde_json::from_slice(
            &fs::read(self.root.join("characters/player/items/catalog.json"))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        for item in items.items {
            let (kind, native, index) = match item.category {
                Category::Shirt => ("shirt", NativePlayerPartKind::Shirt, Some(2)),
                Category::Pants => ("pants", NativePlayerPartKind::Pants, Some(1)),
                Category::Shoes => ("shoes", NativePlayerPartKind::Shoes, Some(0)),
                Category::Hat => ("hat", NativePlayerPartKind::Hat, None),
                Category::Glasses => ("glasses", NativePlayerPartKind::Glasses, None),
                Category::Back => (
                    "back",
                    NativePlayerPartKind::Back,
                    (item.equip_type == 1).then_some(5),
                ),
                Category::Weapon => ("rightWeapon", NativePlayerPartKind::Weapon, None),
                _ => continue,
            };
            for (gender, visual) in [("male", &item.male), ("female", &item.female)] {
                let resolve=|reference:&ffone_runtime_contracts::AvatarTextureReference|->Option<NativePlayerTexture> {
                    let candidate=reference.candidates.first()?;
                    let contract=textures.textures.iter().find(|t|t.native_asset==*candidate)?;
                    Some(NativePlayerTexture{path:contract.native_asset.path.clone(),contract:contract.clone()})
                };
                let primary = visual.primary_texture.as_ref().and_then(resolve);
                let secondary = visual.secondary_texture.as_ref().and_then(resolve);
                if visual.primary_texture.is_some() && primary.is_none() {
                    continue;
                }
                if visual.secondary_texture.is_some() && secondary.is_none() {
                    continue;
                }
                for model in &visual.models {
                    let owner = ownership["models"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|r| r["model"]["path"].as_str() == Some(&model.native_asset.path));
                    let Some(owner) = owner else {
                        continue;
                    };
                    let names: Vec<_> = [primary.as_ref(), secondary.as_ref()]
                        .into_iter()
                        .flatten()
                        .map(|t| t.contract.true_name.clone())
                        .collect();
                    let mut value = serde_json::json!({"kind":kind,"exactRoute":model.exact_route,"sourceRoute":owner["sourceRoute"],"resourceSet":owner["resourceSet"],"trueName":model.true_name,"nativeAsset":model.native_asset,"textures":names});
                    if let Some(index) = index {
                        value["actorSkinCombinerClothesIndex"] = Value::from(index);
                    }
                    if kind == "back" && index.is_some() {
                        value["sharedSkin"] = Value::Bool(true);
                    }
                    if kind == "hat" { value["equipType"] = Value::from(item.equip_type); }
                    if self
                        .variants
                        .iter()
                        .any(|v| v.gender == gender && v.value == value)
                    {
                        continue;
                    }
                    let profile=(native==NativePlayerPartKind::Weapon).then(||ffone_client::tutorial_player_presentation::PlayerWeaponAnimationProfile::from_equip_type(i32::from(item.equip_type))).flatten();
                    self.variants.push(Variant {
                        value,
                        part: NativePlayerPartLook {
                            kind: native,
                            assembly: if index.is_some() {
                                NativePlayerPartAssembly::SharedSkin
                            } else {
                                NativePlayerPartAssembly::RigidAttachment
                            },
                            exact_route: model.exact_route.clone(),
                            glb: model.native_asset.path.clone(),
                            primary_texture: primary.clone(),
                            secondary_texture: secondary.clone(),
                        },
                        profile,
                        gender: gender.into(),
                        caption: format!(
                            "{} · ID {} · {}",
                            item.name, item.item_number, model.true_name
                        ),
                    });
                }
            }
        }
        Ok(())
    }
}
