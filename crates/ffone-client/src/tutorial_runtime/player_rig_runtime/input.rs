use super::*;

pub(super) fn resolve_required_tutorial_player_clips(
    capabilities: &TutorialPlayerRigCapabilities,
    gender: PlayerRigGender,
) -> Result<BTreeMap<TutorialPlayerClip, u32>, String> {
    let mut indices = BTreeMap::new();
    let mut unique_indices = BTreeSet::new();
    for clip in TutorialPlayerClip::ALL {
        let request = match clip {
            TutorialPlayerClip::Stand1 => TutorialPlayerAnimationRequest::stand_force(gender),
            TutorialPlayerClip::Staying
            | TutorialPlayerClip::Standup
            | TutorialPlayerClip::Run
            | TutorialPlayerClip::FfrDance02
            | TutorialPlayerClip::FfrDance04
            | TutorialPlayerClip::FfrDance07
            | TutorialPlayerClip::FfrDance08
            | TutorialPlayerClip::FfrDance10
            | TutorialPlayerClip::FfrDance13
            | TutorialPlayerClip::FfrDance15
            | TutorialPlayerClip::FfrDance18
            | TutorialPlayerClip::FfrDance19
            | TutorialPlayerClip::FfrDance20
            | TutorialPlayerClip::FfrDanceBully
            | TutorialPlayerClip::FfrDanceTellMe
            | TutorialPlayerClip::FfrEmoteCatPose
            | TutorialPlayerClip::FfrEmoteIdolPose => {
                TutorialPlayerAnimationRequest::avatar_emote(gender, clip.name())
                    .map_err(|error| error.to_string())?
            }
            clip if clip.is_avatar_emote_code_clip() => {
                TutorialPlayerAnimationRequest::avatar_emote_clip(gender, clip).ok_or_else(
                    || {
                        format!(
                            "missing AvatarEmote(int) request mapping for {:?}",
                            clip.name()
                        )
                    },
                )?
            }
            TutorialPlayerClip::RunBack
            | TutorialPlayerClip::JumpStart
            | TutorialPlayerClip::Jump
            | TutorialPlayerClip::JumpEnd
            | TutorialPlayerClip::JumpLandRun
            | TutorialPlayerClip::Slide
            | TutorialPlayerClip::RopeDown
            | TutorialPlayerClip::RopeDrop
            | TutorialPlayerClip::RopeLeft
            | TutorialPlayerClip::RopeRight
            | TutorialPlayerClip::RopeStand1
            | TutorialPlayerClip::RopeStand2
            | TutorialPlayerClip::RopeTurn
            | TutorialPlayerClip::RopeUp
            | TutorialPlayerClip::Mount1
            | TutorialPlayerClip::Mount2
            | TutorialPlayerClip::Inventory
            | TutorialPlayerClip::BoardStand1
            | TutorialPlayerClip::BoardRun
            | TutorialPlayerClip::BoardRunBack
            | TutorialPlayerClip::BoardJumpStart
            | TutorialPlayerClip::BoardJump
            | TutorialPlayerClip::BoardJumpEnd
            | TutorialPlayerClip::BoardJumpLandRun
            | TutorialPlayerClip::ScooterStand1
            | TutorialPlayerClip::ScooterRun
            | TutorialPlayerClip::ScooterRunBack
            | TutorialPlayerClip::ScooterJumpStart
            | TutorialPlayerClip::ScooterJump
            | TutorialPlayerClip::ScooterJumpEnd
            | TutorialPlayerClip::ScooterJumpLandRun
            | TutorialPlayerClip::BoardInventory
            | TutorialPlayerClip::ScooterInventory
            | TutorialPlayerClip::StickStand1
            | TutorialPlayerClip::StickReady
            | TutorialPlayerClip::StickRun
            | TutorialPlayerClip::StickRunBack
            | TutorialPlayerClip::StickJumpStart
            | TutorialPlayerClip::StickJump
            | TutorialPlayerClip::StickJumpEnd
            | TutorialPlayerClip::StickJumpLandRun
            | TutorialPlayerClip::PistolStand1
            | TutorialPlayerClip::PistolReady
            | TutorialPlayerClip::PistolRun
            | TutorialPlayerClip::PistolRunBack
            | TutorialPlayerClip::PistolJumpStart
            | TutorialPlayerClip::PistolJump
            | TutorialPlayerClip::PistolJumpEnd
            | TutorialPlayerClip::PistolJumpLandRun
            | TutorialPlayerClip::RifleStand1
            | TutorialPlayerClip::RifleReady
            | TutorialPlayerClip::RifleRun
            | TutorialPlayerClip::RifleRunBack
            | TutorialPlayerClip::RifleJumpStart
            | TutorialPlayerClip::RifleJump
            | TutorialPlayerClip::RifleJumpEnd
            | TutorialPlayerClip::RifleJumpLandRun
            | TutorialPlayerClip::BombStand1
            | TutorialPlayerClip::BombReady
            | TutorialPlayerClip::BombRun
            | TutorialPlayerClip::BombRunBack
            | TutorialPlayerClip::BombJumpStart
            | TutorialPlayerClip::BombJump
            | TutorialPlayerClip::BombJumpEnd
            | TutorialPlayerClip::BombJumpLandRun
            | TutorialPlayerClip::RocketStand1
            | TutorialPlayerClip::RocketReady
            | TutorialPlayerClip::RocketRun
            | TutorialPlayerClip::RocketRunBack
            | TutorialPlayerClip::RocketJumpStart
            | TutorialPlayerClip::RocketJump
            | TutorialPlayerClip::RocketJumpEnd
            | TutorialPlayerClip::RocketJumpLandRun
            | TutorialPlayerClip::Swim
            | TutorialPlayerClip::SwimBack
            | TutorialPlayerClip::SwimIdle
            | TutorialPlayerClip::SwimLeft
            | TutorialPlayerClip::SwimRight => {
                TutorialPlayerAnimationRequest::locomotion(gender, clip).ok_or_else(|| {
                    format!("missing locomotion request mapping for {:?}", clip.name())
                })?
            }
            TutorialPlayerClip::WoundUpper
            | TutorialPlayerClip::Stun
            | TutorialPlayerClip::StickDash
            | TutorialPlayerClip::RifleDash
            | TutorialPlayerClip::RifleTumbling
            | TutorialPlayerClip::RocketSomersault
            | TutorialPlayerClip::StickDodgeUpper
            | TutorialPlayerClip::RifleDodgeUpper
            | TutorialPlayerClip::Die
            | TutorialPlayerClip::Death
            | TutorialPlayerClip::Attack1
            | TutorialPlayerClip::Attack1Upper
            | TutorialPlayerClip::StickAttack1
            | TutorialPlayerClip::StickAttack1Upper
            | TutorialPlayerClip::PistolAttack1
            | TutorialPlayerClip::PistolAttack1Upper
            | TutorialPlayerClip::RifleAttack1
            | TutorialPlayerClip::RifleAttack1Upper
            | TutorialPlayerClip::BombAttack1
            | TutorialPlayerClip::BombAttack1Upper
            | TutorialPlayerClip::RocketAttack1
            | TutorialPlayerClip::RocketAttack1Upper => {
                TutorialPlayerAnimationRequest::runtime_cross_fade(gender, clip)
            }
            _ => unreachable!("AvatarEmote(int) clips are handled by the guarded arm"),
        };
        let resolved = capabilities.resolve(request).map_err(|error| {
            format!(
                "selected tutorial player capability {:?}: {error}",
                clip.name()
            )
        })?;
        if !unique_indices.insert(resolved.gltf_animation_index) {
            return Err(format!(
                "selected tutorial player contract repeats GLB animation index {}",
                resolved.gltf_animation_index
            ));
        }
        indices.insert(clip, resolved.gltf_animation_index);
    }
    Ok(indices)
}
