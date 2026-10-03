use super::*;

impl CombiMachine0104 {
    pub fn open(&mut self) {
        self.phase = CombiPhase0104::Ready;
        self.selection = CombiSelectionOverlay0104::default();
        self.pending = None;
    }

    pub fn request_close(&mut self) -> Result<(), CombiStateError0104> {
        if self.phase != CombiPhase0104::Ready {
            return Err(CombiStateError0104::InputBlocked {
                phase: self.phase.clone(),
            });
        }
        self.phase = CombiPhase0104::Hidden;
        self.selection = CombiSelectionOverlay0104::default();
        self.pending = None;
        Ok(())
    }

    pub fn attach(
        &mut self,
        snapshot: &CombiAuthoritativeSnapshot0104,
        source_location: CombiSourceLocation0104,
        source_index: usize,
        destination: CombiSelectionSlot0104,
    ) -> Result<Vec<CombiSelectionChange0104>, CombiStateError0104> {
        self.require_ready()?;
        self.selection
            .attach(snapshot, source_location, source_index, destination)
            .map_err(CombiStateError0104::Selection)
    }

    pub fn detach(
        &mut self,
        slot: CombiSelectionSlot0104,
    ) -> Result<Vec<CombiSelectionChange0104>, CombiStateError0104> {
        self.require_ready()?;
        Ok(self.selection.detach(slot))
    }

    pub fn clear_all(&mut self) -> Result<Vec<CombiSelectionChange0104>, CombiStateError0104> {
        self.require_ready()?;
        Ok(self.selection.clear_all())
    }

    pub fn begin_combine(
        &mut self,
        snapshot: &CombiAuthoritativeSnapshot0104,
        projection: &CombiModeProjection0104,
    ) -> Result<(), CombiStateError0104> {
        self.require_ready()?;
        if projection.selection != self.selection || projection.owner_pc_id != snapshot.owner_pc_id
        {
            return Err(CombiStateError0104::StaleProjection);
        }
        if !projection.combine_enabled || projection.cost <= 0 {
            return Err(CombiStateError0104::CombinationNotReady);
        }
        let style_slot = self
            .selection
            .style_inventory_index()
            .ok_or(CombiStateError0104::CombinationNotReady)?;
        let stats_slot = self
            .selection
            .stats_inventory_index()
            .ok_or(CombiStateError0104::CombinationNotReady)?;
        if snapshot.taros <= projection.cost {
            // Exact clean comparison: equal Taros is insufficient (`>` only).
            self.phase = CombiPhase0104::Modal(CombiSystemModal0104::NotEnoughTaros);
            return Ok(());
        }
        self.pending = Some(CombiPendingAttempt0104 {
            style_slot,
            stats_slot,
            style_before: snapshot.inventory[style_slot],
            stats_before: snapshot.inventory[stats_slot],
            cost: projection.cost,
        });
        self.phase = CombiPhase0104::Modal(CombiSystemModal0104::AttemptConfirmation);
        Ok(())
    }

    pub fn resolve_modal(
        &mut self,
        choice: CombiModalChoice0104,
    ) -> Result<Vec<CombiSelectionChange0104>, CombiStateError0104> {
        match (self.phase.clone(), choice) {
            (
                CombiPhase0104::Modal(CombiSystemModal0104::AttemptConfirmation),
                CombiModalChoice0104::Continue,
            ) => {
                if self.pending.is_none() {
                    return Err(CombiStateError0104::MissingPendingAttempt);
                }
                self.phase = CombiPhase0104::Waiting {
                    elapsed_seconds: 0.0,
                };
                Ok(Vec::new())
            }
            (
                CombiPhase0104::Modal(CombiSystemModal0104::AttemptConfirmation),
                CombiModalChoice0104::Cancel,
            ) => {
                self.phase = CombiPhase0104::Ready;
                self.pending = None;
                Ok(Vec::new())
            }
            (
                CombiPhase0104::Modal(CombiSystemModal0104::NotEnoughTaros),
                CombiModalChoice0104::Ok,
            ) => {
                self.phase = CombiPhase0104::Ready;
                self.pending = None;
                Ok(Vec::new())
            }
            (
                CombiPhase0104::Modal(CombiSystemModal0104::CombinationFailed),
                CombiModalChoice0104::Ok,
            ) => {
                self.phase = CombiPhase0104::Ready;
                self.pending = None;
                Ok(self.selection.clear_all())
            }
            _ => Err(CombiStateError0104::InvalidModalChoice {
                phase: self.phase.clone(),
                choice,
            }),
        }
    }

    /// Advances clean `CombiWaiting`. A request is emitted only once, and only
    /// after accumulated time is strictly greater than 4.0 seconds.
    pub fn tick_waiting(
        &mut self,
        delta_seconds: f32,
    ) -> Result<Option<CombiRequest0104>, CombiStateError0104> {
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err(CombiStateError0104::InvalidDelta { delta_seconds });
        }
        let CombiPhase0104::Waiting { elapsed_seconds } = &mut self.phase else {
            return Ok(None);
        };
        *elapsed_seconds += delta_seconds;
        if *elapsed_seconds <= COMBI_WAIT_SECONDS_0104 {
            return Ok(None);
        }
        let pending = self
            .pending
            .ok_or(CombiStateError0104::MissingPendingAttempt)?;
        self.phase = CombiPhase0104::AwaitingAuthoritativeReply;
        Ok(Some(CombiRequest0104::new(
            pending.style_slot,
            pending.stats_slot,
        )))
    }

    pub fn receive_authoritative_reply(
        &mut self,
        snapshot: &CombiAuthoritativeSnapshot0104,
        projection: &CombiModeProjection0104,
        reply: CombiSuccessReply0104,
    ) -> Result<CombiAuthorityReceipt0104, CombiReplyError0104> {
        if self.phase != CombiPhase0104::AwaitingAuthoritativeReply {
            return Err(CombiReplyError0104::NotAwaiting {
                phase: self.phase.clone(),
            });
        }
        let pending = self.pending.ok_or(CombiReplyError0104::NoPendingAttempt)?;
        if projection.owner_pc_id != snapshot.owner_pc_id
            || projection.selection != self.selection
            || projection.look.as_ref().map_or(true, |look| {
                look.inventory_index != pending.style_slot || look.item != pending.style_before
            })
            || projection.stats.as_ref().map_or(true, |stats| {
                stats.inventory_index != pending.stats_slot || stats.item != pending.stats_before
            })
        {
            return Err(CombiReplyError0104::StalePresentationProjection);
        }
        validate_reply_envelope(pending, reply)?;
        if snapshot.inventory[pending.style_slot] != pending.style_before
            || snapshot.inventory[pending.stats_slot] != pending.stats_before
        {
            return Err(CombiReplyError0104::SnapshotChangedWhileAwaiting);
        }
        if reply.taros_after < 0 {
            return Err(CombiReplyError0104::NegativeAuthoritativeTaros {
                taros_after: reply.taros_after,
            });
        }
        match reply.success_flag {
            1 => {
                let style_after =
                    expected_success_style_item(pending.style_before, pending.stats_before);
                if reply.new_item != style_after {
                    return Err(CombiReplyError0104::UnexpectedSuccessItem {
                        expected: style_after,
                        actual: reply.new_item,
                    });
                }
                let look = projection
                    .look
                    .clone()
                    .ok_or(CombiReplyError0104::MissingSuccessPresentation)?;
                let stats = projection
                    .stats
                    .clone()
                    .ok_or(CombiReplyError0104::MissingSuccessPresentation)?;
                self.phase = CombiPhase0104::Success(CombiSuccessPresentation0104 {
                    look,
                    stats,
                    combined_item_cannot_equip: projection.combined_item_cannot_equip,
                });
                Ok(CombiAuthorityReceipt0104::Success {
                    owner_pc_id: snapshot.owner_pc_id,
                    style_slot: pending.style_slot,
                    stats_slot: pending.stats_slot,
                    style_before: pending.style_before,
                    stats_before: pending.stats_before,
                    style_after,
                    stats_after: empty_item_0104(),
                    taros_after: reply.taros_after,
                })
            }
            0 => {
                if reply.new_item != pending.style_before {
                    return Err(CombiReplyError0104::UnexpectedFailureItem {
                        expected: pending.style_before,
                        actual: reply.new_item,
                    });
                }
                self.phase = CombiPhase0104::Modal(CombiSystemModal0104::CombinationFailed);
                Ok(CombiAuthorityReceipt0104::Failure {
                    owner_pc_id: snapshot.owner_pc_id,
                    style_slot: pending.style_slot,
                    stats_slot: pending.stats_slot,
                    style_unchanged: pending.style_before,
                    stats_unchanged: pending.stats_before,
                    taros_after: reply.taros_after,
                })
            }
            success_flag => Err(CombiReplyError0104::UnsupportedSuccessFlag { success_flag }),
        }
    }

    /// Exact clean fail-packet behavior: show the error and remain send-locked.
    pub fn receive_wire_failure(
        &mut self,
        reply: CombiFailureReply0104,
    ) -> Result<(), CombiReplyError0104> {
        if self.phase != CombiPhase0104::AwaitingAuthoritativeReply {
            return Err(CombiReplyError0104::NotAwaiting {
                phase: self.phase.clone(),
            });
        }
        let pending = self.pending.ok_or(CombiReplyError0104::NoPendingAttempt)?;
        if reply.costume_item_slot != pending.style_slot as i32
            || reply.stat_item_slot != pending.stats_slot as i32
            || reply.cash_item_slot_1 != 0
            || reply.cash_item_slot_2 != 0
        {
            return Err(CombiReplyError0104::MismatchedFailureEnvelope {
                reply,
                expected_style_slot: pending.style_slot,
                expected_stats_slot: pending.stats_slot,
            });
        }
        // A matching rejection did not mutate inventory. Release the send lock;
        // the shell's keyed error popup owns input until dismissed.
        self.phase = CombiPhase0104::Ready;
        self.pending = None;
        Ok(())
    }

    pub fn combine_more_items(&mut self) -> Result<(), CombiStateError0104> {
        if !matches!(self.phase, CombiPhase0104::Success(_)) {
            return Err(CombiStateError0104::InputBlocked {
                phase: self.phase.clone(),
            });
        }
        self.phase = CombiPhase0104::Ready;
        self.selection = CombiSelectionOverlay0104::default();
        self.pending = None;
        Ok(())
    }

    pub fn go_to_my_stuff(&mut self) -> Result<(), CombiStateError0104> {
        if !matches!(self.phase, CombiPhase0104::Success(_)) {
            return Err(CombiStateError0104::InputBlocked {
                phase: self.phase.clone(),
            });
        }
        self.phase = CombiPhase0104::Hidden;
        self.selection = CombiSelectionOverlay0104::default();
        self.pending = None;
        Ok(())
    }

    pub(super) fn require_ready(&self) -> Result<(), CombiStateError0104> {
        if self.phase == CombiPhase0104::Ready {
            Ok(())
        } else {
            Err(CombiStateError0104::InputBlocked {
                phase: self.phase.clone(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CombiExternalModalState0104 {
    pub system_popup: bool,
    pub help: bool,
    pub inventory_popup_modal: bool,
    pub popup_controller: bool,
}

impl CombiExternalModalState0104 {
    #[must_use]
    pub const fn blocks_main_ui(self) -> bool {
        self.system_popup || self.help || self.inventory_popup_modal || self.popup_controller
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct CombiUiState0104 {
    pub phase: CombiPhase0104,
    pub external_modal: CombiExternalModalState0104,
    /// External native camera ownership is not implemented in this slice.
    /// These stay false until a real camera producer explicitly binds them.
    pub primary_npc_camera_bound: bool,
    pub waiting_npc_camera_bound: bool,
}

impl Default for CombiUiState0104 {
    fn default() -> Self {
        Self {
            phase: CombiPhase0104::Hidden,
            external_modal: CombiExternalModalState0104::default(),
            primary_npc_camera_bound: false,
            waiting_npc_camera_bound: false,
        }
    }
}

impl CombiUiState0104 {
    #[must_use]
    pub fn input_capabilities(
        &self,
        projection: &CombiModeProjection0104,
    ) -> CombiInputCapabilities0104 {
        let draw = !matches!(self.phase, CombiPhase0104::Hidden);
        let ready = matches!(self.phase, CombiPhase0104::Ready);
        let main_controls = draw && ready && !self.external_modal.blocks_main_ui();
        let success_controls = matches!(self.phase, CombiPhase0104::Success(_));
        CombiInputCapabilities0104 {
            draw,
            main_controls,
            selection_controls: main_controls,
            clear: main_controls && projection.clear_enabled,
            combine: main_controls && projection.combine_enabled,
            close: main_controls,
            // Clean DoCombiSucc forces GUI.enabled=true while drawing.
            success_controls,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombiUiCommand0104 {
    BeginInventoryDrag {
        inventory_index: usize,
    },
    /// The carried item was released away from both selection slots.
    CancelInventoryDrag,
    RejectEquippedItem {
        equipment_index: usize,
        message_id: i32,
    },
    DropOnSelection {
        slot: CombiSelectionSlot0104,
    },
    DetachSelection {
        slot: CombiSelectionSlot0104,
    },
    ClearAll,
    Combine,
    Close,
    Help,
    CombineMoreItems,
    GoToMyStuff,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct CombiUiOutbox0104 {
    pub(super) commands: VecDeque<CombiUiCommand0104>,
}

impl CombiUiOutbox0104 {
    pub fn push(&mut self, command: CombiUiCommand0104) {
        self.commands.push_back(command);
    }

    pub fn pop_front(&mut self) -> Option<CombiUiCommand0104> {
        self.commands.pop_front()
    }

    #[must_use]
    pub fn as_slices(&self) -> (&[CombiUiCommand0104], &[CombiUiCommand0104]) {
        self.commands.as_slices()
    }
}

/// The bag item on the pointer, from the press that picks it up until it lands
/// on a selection slot or goes back. A plain click (press and release on the
/// same cell) keeps it on the pointer until the next click.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiCarriedItem0104 {
    pub inventory_index: usize,
    /// The left button is still down from the press that picked it up.
    pub held: bool,
}

#[derive(Clone, Resource)]
pub(super) struct CombiUiAssets {
    pub(super) images: [Handle<Image>; CombiStaticAssetRole::COUNT],
    pub(super) jeffe_font: Handle<Font>,
    pub(super) chalet_font: Handle<Font>,
}

impl CombiUiAssets {
    pub(super) fn load(asset_server: &AssetServer, contract: &CombiUiAssetContract) -> Self {
        Self {
            images: array::from_fn(|index| asset_server.load(contract.image_paths[index].clone())),
            jeffe_font: asset_server.load(contract.jeffe_font_path.clone()),
            chalet_font: asset_server.load(contract.chalet_font_path.clone()),
        }
    }

    pub(super) fn image(&self, role: CombiStaticAssetRole) -> Handle<Image> {
        self.images[role.index()].clone()
    }

    pub(super) fn readiness(&self, asset_server: &AssetServer) -> CombiStaticAssetReadiness0104 {
        if self
            .images
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
            || matches!(
                asset_server.load_state(self.jeffe_font.id()),
                LoadState::Failed(_)
            )
            || matches!(
                asset_server.load_state(self.chalet_font.id()),
                LoadState::Failed(_)
            )
        {
            CombiStaticAssetReadiness0104::Failed
        } else if self
            .images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(
                asset_server.load_state(self.jeffe_font.id()),
                LoadState::Loaded
            )
            && matches!(
                asset_server.load_state(self.chalet_font.id()),
                LoadState::Loaded
            )
        {
            CombiStaticAssetReadiness0104::Ready
        } else {
            CombiStaticAssetReadiness0104::Loading
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum CombiInteractiveControl0104 {
    InventorySlot(usize),
    EquipmentSlot(usize),
    LookDrop,
    StatsDrop,
    LookSelection,
    StatsSelection,
    ClearAll,
    Combine,
    Close,
    Help,
    CombineMore,
    GoToStuff,
}

impl CombiInteractiveControl0104 {
    /// The selection slot that takes the carried item when it lands here: the
    /// framed slot or the detail area of the same STYLE or STATS column.
    #[must_use]
    pub const fn drop_slot(self) -> Option<CombiSelectionSlot0104> {
        match self {
            Self::LookDrop | Self::LookSelection => Some(CombiSelectionSlot0104::Style),
            Self::StatsDrop | Self::StatsSelection => Some(CombiSelectionSlot0104::Stats),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum CombiUiElement0104 {
    Backdrop,
    Panel,
    RightBackplate,
    MainGroup,
    PrimaryNpcBoundary,
    Title,
    Intro,
    StyleTitle,
    StatsTitle,
    Cost,
    TarosLabel,
    TarosIcon,
    ChanceTitle,
    Chance,
    ChanceLevel,
    NewItemTitle,
    LookDrop,
    LookBackground,
    LookSelectionFrame,
    LookSelectionIcon,
    LookSelectionBadge,
    LookSelectionHover,
    LookDetailRestricted,
    LookDetailIcon,
    LookDetailBadge,
    LookName,
    LookLevel,
    LookDescription,
    LookErrorBackground,
    LookErrorText,
    LookEmptyText,
    StatDrop,
    StatBackground,
    StatSelectionFrame,
    StatSelectionIcon,
    StatSelectionBadge,
    StatSelectionHover,
    StatLevel,
    StatSection,
    StatSingle,
    StatMulti,
    StatDefense,
    InfoSection,
    InfoTypeLabel,
    InfoRangeLabel,
    InfoRarityLabel,
    InfoTradeLabel,
    InfoTypeValue,
    InfoRangeValue,
    InfoRarityValue,
    InfoTradeValue,
    StatErrorBackground,
    StatErrorText,
    StatEmptyText,
    LookDropHighlight,
    StatDropHighlight,
    ClearAll,
    Combine,
    PcStuffPanel,
    ItemTabLabel,
    InventoryViewport,
    InventoryContent,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotBadge(usize),
    InventorySlotHover(usize),
    EquipmentPanel,
    EquipmentTitle,
    EquipmentSlotFrame(usize),
    EquipmentSlotIcon(usize),
    EquipmentSlotBadge(usize),
    EquipmentSlotHover(usize),
    Close,
    Trash,
    Help,
    Shade,
    SuccessGroup,
    SuccessNpcIcon,
    SuccessHooray,
    SuccessMessage,
    SuccessIcon,
    SuccessRestricted,
    SuccessBadge,
    SuccessName,
    SuccessLevel,
    SuccessDescription,
    SuccessSingle,
    SuccessMulti,
    SuccessDefense,
    SuccessTypeLabel,
    SuccessRangeLabel,
    SuccessRarityLabel,
    SuccessTradeLabel,
    SuccessTypeValue,
    SuccessRangeValue,
    SuccessRarityValue,
    SuccessTradeValue,
    CombineMore,
    GoToStuff,
    WaitingGroup,
    WaitingNpcBoundary,
    DraggedItem,
}

#[derive(Component)]
pub struct CombiUiRoot0104;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum CombiUiSet0104 {
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct CombiUiPlugin;

impl Plugin for CombiUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<CombiUiState0104>()
            .init_resource::<CombiModeProjection0104>()
            .init_resource::<CombiUiOutbox0104>()
            .init_resource::<CombiPointerState0104>()
            .init_resource::<CombiUiAssetContract>()
            .init_resource::<CombiUiAssetStatus0104>()
            .configure_sets(
                Update,
                (CombiUiSet0104::Interaction, CombiUiSet0104::Bind).chain(),
            )
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_combi_ui)
            .add_systems(
                Update,
                (collect_combi_ui_input.in_set(CombiUiSet0104::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (bind_combi_ui
                    .in_set(CombiUiSet0104::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
