use super::*;

impl TutorialPlayerRigCapabilities {
    pub fn from_contract(
        gender: PlayerRigGender,
        clips: &[PlayerRigClipContract],
    ) -> Result<Self, TutorialPlayerRigCapabilityError> {
        let mut exact = BTreeMap::new();
        for contract in clips {
            let Some(clip) = TutorialPlayerClip::from_exact_name(&contract.name) else {
                continue;
            };
            if exact.insert(clip, contract.clone()).is_some() {
                return Err(TutorialPlayerRigCapabilityError::DuplicateExactClip { clip });
            }
        }
        Ok(Self {
            gender,
            clips: exact,
        })
    }

    #[must_use]
    pub const fn gender(&self) -> PlayerRigGender {
        self.gender
    }

    #[must_use]
    pub fn exact_names(&self) -> impl ExactSizeIterator<Item = &'static str> + '_ {
        self.clips.keys().copied().map(TutorialPlayerClip::name)
    }

    pub fn resolve(
        &self,
        request: TutorialPlayerAnimationRequest,
    ) -> Result<ContractResolvedTutorialPlayerAnimation, TutorialPlayerRigCapabilityError> {
        let expected_path_id = request.clip.source_path_id(self.gender);
        if request.source_path_id != expected_path_id {
            return Err(
                TutorialPlayerRigCapabilityError::RequestSourcePathIdMismatch {
                    clip: request.clip,
                    expected: expected_path_id,
                    actual: request.source_path_id,
                },
            );
        }
        let Some(contract) = self.clips.get(&request.clip) else {
            return Err(TutorialPlayerRigCapabilityError::MissingExactClip {
                clip: request.clip,
                available_exact_names: self.exact_names().map(str::to_owned).collect(),
            });
        };
        if contract.source_path_id != expected_path_id {
            return Err(
                TutorialPlayerRigCapabilityError::ContractSourcePathIdMismatch {
                    clip: request.clip,
                    expected: expected_path_id,
                    actual: contract.source_path_id,
                },
            );
        }
        let playback = request.clip.playback();
        if contract.playback != playback.contract_value() {
            return Err(TutorialPlayerRigCapabilityError::PlaybackMismatch {
                clip: request.clip,
                expected: playback,
                actual: contract.playback.clone(),
            });
        }
        if contract.runtime_status != TUTORIAL_PLAYER_RUNTIME_READY_STATUS {
            return Err(TutorialPlayerRigCapabilityError::RuntimeStatusNotReady {
                clip: request.clip,
                actual: contract.runtime_status.clone(),
            });
        }
        Ok(ContractResolvedTutorialPlayerAnimation {
            request,
            gltf_animation_index: contract.gltf_animation_index,
            playback,
        })
    }
}

/// Contract validation succeeded. This is deliberately not evidence that a
/// Bevy `AnimationPlayer` applied the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContractResolvedTutorialPlayerAnimation {
    pub request: TutorialPlayerAnimationRequest,
    pub gltf_animation_index: u32,
    pub playback: TutorialPlayerClipPlayback,
}
