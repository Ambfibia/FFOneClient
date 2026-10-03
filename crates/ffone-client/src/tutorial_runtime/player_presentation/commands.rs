use super::*;

/// Exact local equip event emitted by tutorial skip/live choreography.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialPlayerEquipmentRequest {
    pub slot: CharacterEquipSlot0104,
    pub item: EquippedItem0104,
}

/// Reproduce `localized.local == 0` and the non-English class branches.
///
/// English always equips item 328. A non-English client equips 43 for class 2,
/// 328 for class 3, and 197 for class 4. Other non-English classes emit no
/// equip event. Item 134 is not part of the original tutorial routing.
#[must_use]
pub const fn tutorial_weapon_request(
    original_english_locale: bool,
    avatar_class: i32,
) -> Option<TutorialPlayerEquipmentRequest> {
    let item_id = if original_english_locale {
        328
    } else {
        match avatar_class {
            2 => 43,
            3 => 328,
            4 => 197,
            _ => return None,
        }
    };
    Some(TutorialPlayerEquipmentRequest {
        slot: TUTORIAL_WEAPON_SLOT,
        item: EquippedItem0104 {
            item_type: TUTORIAL_WEAPON_ITEM_TYPE,
            item_id,
            option: 0,
            time_limit: 0,
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialPlayerPresentationCommand {
    Animation(TutorialPlayerAnimationRequest),
    AnimationDelay(Duration),
    Equipment(TutorialPlayerEquipmentRequest),
}

/// Renderer-neutral bridge from tutorial choreography to selected-player
/// presentation. It preserves ordering without claiming that a command has
/// been rendered.
#[derive(Debug, Default, Resource)]
pub struct TutorialPlayerPresentationCommandQueue {
    pub(super) pending: VecDeque<TutorialPlayerPresentationCommand>,
    pub(super) emote_continuation_echoes: VecDeque<(i32, bool)>,
}

impl TutorialPlayerPresentationCommandQueue {
    /// Discard a shot already bridged into the renderer FIFO before a higher
    /// priority damage or traversal pose is consumed.
    pub fn cancel_pending_runtime_attacks(&mut self) {
        self.pending.retain(|command| {
            !matches!(command, TutorialPlayerPresentationCommand::Animation(request)
                if request.clip.is_upper_attack() || matches!(request.clip,
                    TutorialPlayerClip::Attack1 | TutorialPlayerClip::StickAttack1
                    | TutorialPlayerClip::PistolAttack1 | TutorialPlayerClip::RifleAttack1
                    | TutorialPlayerClip::BombAttack1 | TutorialPlayerClip::RocketAttack1))
        });
    }

    pub fn cancel_pending_upper_layers(&mut self) {
        self.pending.retain(|command| {
            !matches!(command, TutorialPlayerPresentationCommand::Animation(request)
                if request.clip.is_upper_layer())
        });
    }
    pub fn push_animation(&mut self, request: TutorialPlayerAnimationRequest) {
        self.pending
            .push_back(TutorialPlayerPresentationCommand::Animation(request));
    }

    pub fn push_animation_delay(&mut self, duration: Duration) {
        self.pending
            .push_back(TutorialPlayerPresentationCommand::AnimationDelay(duration));
    }

    pub fn avatar_emote(
        &mut self,
        gender: PlayerRigGender,
        name: &str,
    ) -> Result<TutorialPlayerAnimationRequest, TutorialPlayerPresentationError> {
        let request = TutorialPlayerAnimationRequest::avatar_emote(gender, name)?;
        self.push_animation(request);
        Ok(request)
    }

    pub fn avatar_emote_code(
        &mut self,
        gender: PlayerRigGender,
        code: i32,
    ) -> Option<TutorialPlayerAnimationRequest> {
        let request = TutorialPlayerAnimationRequest::avatar_emote_code(gender, code)?;
        self.push_animation(request);
        Some(request)
    }
    pub fn stand_force(&mut self, gender: PlayerRigGender) -> TutorialPlayerAnimationRequest {
        let request = TutorialPlayerAnimationRequest::stand_force(gender);
        self.push_animation(request);
        request
    }

    pub fn tutorial_weapon(
        &mut self,
        original_english_locale: bool,
        avatar_class: i32,
    ) -> Option<TutorialPlayerEquipmentRequest> {
        let request = tutorial_weapon_request(original_english_locale, avatar_class)?;
        self.pending
            .push_back(TutorialPlayerPresentationCommand::Equipment(request));
        Some(request)
    }

    /// Queue one authoritative hand/equipment update. Ordinary-world entry
    /// uses this for the item retained in the shard PC load record; tutorial
    /// choreography continues to use [`Self::tutorial_weapon`].
    pub fn push_equipment(&mut self, request: TutorialPlayerEquipmentRequest) {
        self.pending
            .push_back(TutorialPlayerPresentationCommand::Equipment(request));
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// A choreography emote/forced stand owns the whole pose and intentionally
    /// suppresses ordinary action-state requests queued in the same frame.
    /// Equipment commands do not: a tutorial rifle can be attached and start
    /// its requested animation in one renderer batch.
    #[must_use]
    pub(crate) fn has_pending_direct_pose_override(&self) -> bool {
        self.pending.iter().any(|command| {
            matches!(
                command,
                TutorialPlayerPresentationCommand::Animation(request)
                    if request.sets_emote_state || request.reset_move_direction
            )
        })
    }

    /// `cnAvatarAnimation.AvatarDead` calls `EndEmote` before starting `die`.
    /// Retain unrelated equipment/delay commands while discarding only queued
    /// choreography poses which would otherwise overwrite that death edge.
    pub(crate) fn cancel_pending_direct_pose_overrides(&mut self) {
        self.pending.retain(|command| {
            !matches!(
                command,
                TutorialPlayerPresentationCommand::Animation(request)
                    if request.sets_emote_state || request.reset_move_direction
            )
        });
    }

    /// Accepted attacks cancel player-selected emotes, including a repeat echo
    /// queued this frame, while preserving scripted poses and equipment.
    pub(crate) fn cancel_pending_avatar_emotes(&mut self) {
        for (_, cancelled) in &mut self.emote_continuation_echoes {
            *cancelled = true;
        }
        self.pending.retain(|command| {
            !matches!(command,
                TutorialPlayerPresentationCommand::Animation(request)
                    if request.clip.is_avatar_emote_code_clip()
            )
        });
    }

    /// Register only successfully sent local continuation requests. The wire
    /// protocol has no request token; matching replies arrive in stream order.
    pub fn record_emote_continuation_sent(&mut self, code: i32) {
        self.emote_continuation_echoes.push_back((code, false));
    }

    /// A late reply to a cancelled repeat must not resurrect the old emote.
    /// Unsolicited/new menu emotes continue through the ordinary path.
    pub fn accept_avatar_emote_echo(&mut self, code: i32) -> bool {
        let Some(index) = self
            .emote_continuation_echoes
            .iter()
            .position(|(pending, _)| *pending == code)
        else {
            return true;
        };
        !self
            .emote_continuation_echoes
            .remove(index)
            .expect("matched echo")
            .1
    }

    /// Last pending hand item wins because the renderer drains this FIFO in
    /// order and replaces any previously attached weapon for every request.
    #[must_use]
    pub(crate) fn pending_weapon_item_id(&self) -> Option<i16> {
        self.pending.iter().rev().find_map(|command| match command {
            TutorialPlayerPresentationCommand::Equipment(request)
                if request.slot == TUTORIAL_WEAPON_SLOT =>
            {
                Some(request.item.item_id)
            }
            _ => None,
        })
    }

    pub fn pop_front(&mut self) -> Option<TutorialPlayerPresentationCommand> {
        self.pending.pop_front()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
        self.emote_continuation_echoes.clear();
    }
}
