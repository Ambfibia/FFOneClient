use super::*;

impl CashmallUiState0104 {
    #[must_use]
    pub const fn phase(&self) -> CashmallLifecyclePhase0104 {
        self.phase
    }

    #[must_use]
    pub const fn opening_elapsed_seconds(&self) -> f32 {
        self.opening_elapsed_seconds
    }

    #[must_use]
    pub const fn tab(&self) -> CashmallTab0104 {
        self.tab
    }

    #[must_use]
    pub const fn cashmall_scroll_y(&self) -> f32 {
        self.cashmall_scroll_y
    }

    #[must_use]
    pub const fn inventory_scroll_y(&self) -> f32 {
        self.inventory_scroll_y
    }

    #[must_use]
    pub const fn scroll_target(&self) -> CashmallScrollTarget0104 {
        self.scroll_target
    }

    #[must_use]
    pub const fn cursor_locked(&self) -> bool {
        self.cursor_locked
    }

    #[must_use]
    pub const fn ui_input_active(&self) -> bool {
        self.ui_input_active
    }

    #[must_use]
    pub const fn user_cash(&self) -> i32 {
        CASHMALL_USER_CASH_0104
    }

    #[must_use]
    pub const fn legacy_send_latch(&self) -> bool {
        false
    }

    pub fn open_from(
        &mut self,
        source: CashmallOpenSource0104,
        previous_cursor_locked: bool,
        outbox: &mut CashmallUiOutbox0104,
    ) {
        match source {
            CashmallOpenSource0104::HiddenChatCommand => {}
        }
        self.phase = CashmallLifecyclePhase0104::Opening;
        self.opening_elapsed_seconds = 0.0;
        self.previous_cursor_locked = previous_cursor_locked;
        self.cursor_locked = false;
        self.ui_input_active = true;
        // Clean deliberately retains eTabMode and both scroll vectors across
        // Exit/ShowWindow. The delegate selected by the last in-area panel is
        // sticky too; only the shared inventory tab is forced to zero.
        outbox.push(CashmallLocalEffect0104::EnterMode {
            ui_input_event: [11, 0],
            ui_input_enter_value: 1,
            force_inventory_tab: 0,
            force_cursor_unlocked: true,
        });
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        if self.phase == CashmallLifecyclePhase0104::Hidden {
            return;
        }
        // `cnCashmallMode.Update` forces this every active frame.
        self.cursor_locked = false;
        if self.phase != CashmallLifecyclePhase0104::Opening
            || delta_seconds.is_nan()
            || delta_seconds <= 0.0
        {
            return;
        }
        if !delta_seconds.is_finite() {
            self.opening_elapsed_seconds = CASHMALL_OPEN_SECONDS;
        } else {
            self.opening_elapsed_seconds =
                (self.opening_elapsed_seconds + delta_seconds).min(CASHMALL_OPEN_SECONDS);
        }
        if self.opening_elapsed_seconds >= CASHMALL_OPEN_SECONDS {
            self.phase = CashmallLifecyclePhase0104::Visible;
        }
    }

    #[must_use]
    pub const fn input_capabilities(
        &self,
        modal: CashmallModalState0104,
    ) -> CashmallInputCapabilities0104 {
        let draw = !matches!(self.phase, CashmallLifecyclePhase0104::Hidden);
        let visible = matches!(self.phase, CashmallLifecyclePhase0104::Visible);
        let base_gui = visible && !modal.help_active;
        let cashmall_controls = base_gui
            && !modal.system_popup_active
            && !modal.generic_popup_active
            && !self.legacy_send_latch();
        let pc_stuff_controls = cashmall_controls && !modal.inventory_popup_modal;
        // Update's scroll branch checks only help, system-popup, and bSend. It
        // does not repeat Panel_Cashmall's generic-popup check.
        let update_scroll =
            draw && !modal.help_active && !modal.system_popup_active && !self.legacy_send_latch();
        CashmallInputCapabilities0104 {
            draw,
            cashmall_controls,
            pc_stuff_controls,
            tabs: cashmall_controls,
            row_actions: cashmall_controls,
            go_to_stuff: cashmall_controls,
            close_button: pc_stuff_controls,
            update_scroll,
            escape_request: draw && !modal.system_popup_active && !self.legacy_send_latch(),
            cursor_forced_unlocked: draw,
        }
    }

    pub fn select_tab(
        &mut self,
        modal: CashmallModalState0104,
        tab: CashmallTab0104,
        outbox: &mut CashmallUiOutbox0104,
    ) -> Result<CashmallTabActivation0104, CashmallActionBlocked0104> {
        if !self.input_capabilities(modal).tabs {
            return Err(CashmallActionBlocked0104::ControlsDisabled);
        }
        if self.tab == tab {
            // Selected GUI.Button return value is intentionally ignored.
            return Ok(CashmallTabActivation0104::SelectedButtonReturnDiscarded);
        }
        self.tab = tab;
        self.cashmall_scroll_y = 0.0;
        outbox.push(CashmallLocalEffect0104::PlayAudio(
            CashmallAudioCue0104::TabClick01,
        ));
        Ok(CashmallTabActivation0104::Changed)
    }

    pub fn note_scroll_areas(&mut self, pc_stuff_in_area: bool, cashmall_in_area: bool) {
        // Exact precedence and sticky delegate from `cnCashmallMode.Update`.
        if pc_stuff_in_area {
            self.scroll_target = CashmallScrollTarget0104::PcStuff;
        } else if cashmall_in_area {
            self.scroll_target = CashmallScrollTarget0104::Cashmall;
        }
    }

    pub fn apply_scroll_axis(
        &mut self,
        modal: CashmallModalState0104,
        axis: f32,
    ) -> Result<(), CashmallActionBlocked0104> {
        if !self.input_capabilities(modal).update_scroll {
            return Err(CashmallActionBlocked0104::ControlsDisabled);
        }
        if !axis.is_finite() || axis == 0.0 {
            return Ok(());
        }
        // ConfigurableInput clamps the axis to [-1, 1] and multiplies by the
        // shared 200 velocity. Panel_Cashmall then clamps to +/-30.
        let shared_value = axis.clamp(-1.0, 1.0) * 200.0;
        match self.scroll_target {
            CashmallScrollTarget0104::Cashmall => {
                let value = shared_value.clamp(-30.0, 30.0);
                self.cashmall_scroll_y -= value;
                // BeginScrollView immediately clamps against the dead cached
                // zero-height content rect, so the visible result remains 0.
                self.cashmall_scroll_y = 0.0;
            }
            CashmallScrollTarget0104::PcStuff => {
                self.inventory_scroll_y = crate::user_equip_ui::clamp_user_equip_scroll(
                    self.inventory_scroll_y - shared_value,
                );
            }
        }
        Ok(())
    }

    pub fn activate_row(
        &self,
        modal: CashmallModalState0104,
        projection: &CashmallModeProjection0104,
        row_index: usize,
        button: CashmallPointerButton0104,
        outbox: &mut CashmallUiOutbox0104,
    ) -> Result<(), CashmallActionBlocked0104> {
        if !self.input_capabilities(modal).row_actions {
            return Err(CashmallActionBlocked0104::ControlsDisabled);
        }
        let rows = projection.rows_for_tab(self.tab);
        let row = rows
            .get(row_index)
            .ok_or(CashmallActionBlocked0104::MissingRow)?;
        outbox.push(CashmallLocalEffect0104::PlayAudio(
            CashmallAudioCue0104::ButtonSound,
        ));
        if button == CashmallPointerButton0104::Secondary && row.source.item.item_type < 7 {
            outbox.push(CashmallLocalEffect0104::RetainedRightClickEquipmentEvent {
                event: [2, 3, 1],
                item_id: row.source.item.item_id,
                item_type: row.source.item.item_type,
                option: 0,
            });
        } else {
            outbox.push(CashmallLocalEffect0104::VendorClickItem(
                CashmallVendorClickBoundary0104 {
                    slot_type: row.source.slot_type,
                    slot_id: row.source.slot_id,
                    item: row.source.item,
                    popup_action: CASHMALL_VENDOR_POPUP_ACTION,
                    popup_rect: CASHMALL_VENDOR_POPUP_RECT,
                },
            ));
        }
        Ok(())
    }

    pub fn request_help(
        &self,
        modal: CashmallModalState0104,
        outbox: &mut CashmallUiOutbox0104,
    ) -> Result<(), CashmallActionBlocked0104> {
        if !self.input_capabilities(modal).pc_stuff_controls {
            return Err(CashmallActionBlocked0104::ControlsDisabled);
        }
        outbox.push(CashmallLocalEffect0104::PlayAudio(
            CashmallAudioCue0104::ButtonSound,
        ));
        outbox.push(CashmallLocalEffect0104::DeadHelpSendMessage);
        Ok(())
    }

    pub fn request_nano_tab(
        &self,
        modal: CashmallModalState0104,
    ) -> Result<(), CashmallActionBlocked0104> {
        if !self.input_capabilities(modal).pc_stuff_controls {
            return Err(CashmallActionBlocked0104::ControlsDisabled);
        }
        Err(CashmallActionBlocked0104::NanoProjectionUnavailable)
    }

    pub fn request_redeem_code(
        &self,
        modal: CashmallModalState0104,
    ) -> Result<(), CashmallActionBlocked0104> {
        if !self.input_capabilities(modal).pc_stuff_controls {
            return Err(CashmallActionBlocked0104::ControlsDisabled);
        }
        Err(CashmallActionBlocked0104::RedeemRuntimeUnavailable)
    }

    pub fn request_go_to_stuff(
        &mut self,
        modal: CashmallModalState0104,
        outbox: &mut CashmallUiOutbox0104,
    ) -> Result<(), CashmallActionBlocked0104> {
        if !self.input_capabilities(modal).go_to_stuff {
            return Err(CashmallActionBlocked0104::ControlsDisabled);
        }
        outbox.push(CashmallLocalEffect0104::GoToMyStuff(
            CashmallGoToStuffBoundary0104::default(),
        ));
        self.hide_for_local_transition(false, outbox);
        Ok(())
    }

    pub fn request_close(
        &mut self,
        source: CashmallCloseSource0104,
        modal: CashmallModalState0104,
        gate: CashmallCloseGate0104,
        outbox: &mut CashmallUiOutbox0104,
    ) -> Result<(), CashmallActionBlocked0104> {
        if self.phase == CashmallLifecyclePhase0104::Hidden {
            return Err(CashmallActionBlocked0104::NotActive);
        }
        match source {
            CashmallCloseSource0104::PcStuffCloseButton => {
                if !self.input_capabilities(modal).close_button {
                    return Err(CashmallActionBlocked0104::ControlsDisabled);
                }
            }
            CashmallCloseSource0104::ConfigurableKey4 => {
                if !self.input_capabilities(modal).escape_request {
                    return Err(CashmallActionBlocked0104::ControlsDisabled);
                }
                if !gate.mode_accepts_escape {
                    return Err(CashmallActionBlocked0104::ModeRejectedEscape);
                }
            }
        }
        if !gate.exit_arbitration_clear {
            return Err(CashmallActionBlocked0104::ExitArbitrationRejected);
        }
        self.hide_for_local_transition(true, outbox);
        Ok(())
    }

    pub(super) fn hide_for_local_transition(
        &mut self,
        emit_close_boundary: bool,
        outbox: &mut CashmallUiOutbox0104,
    ) {
        self.phase = CashmallLifecyclePhase0104::Hidden;
        self.opening_elapsed_seconds = 0.0;
        self.cursor_locked = self.previous_cursor_locked;
        self.ui_input_active = false;
        if emit_close_boundary {
            outbox.push(CashmallLocalEffect0104::Close(CashmallCloseBoundary0104 {
                ui_input_event: [11, 0],
                ui_input_exit_value: 10,
                restore_cursor_locked: self.previous_cursor_locked,
                notify_game_mode_exit: [2, 1],
                stop_ui_mode_sound: true,
                request_asset_gc: true,
                loaded_textures_actually_cleared: CASHMALL_FREE_ASSETS_CLEARS_LOADED_TEXTURES,
            }));
        }
        // eTabMode and both scroll positions intentionally survive.
    }

    /// Models the complete clean `ReceivePacket`: packet type is observed and
    /// then discarded; no state, cash, catalog, or send latch changes.
    pub fn receive_packet_ignored(&mut self, packet_type: u32) -> u32 {
        packet_type
    }
}

#[derive(Clone, Resource)]
pub(super) struct CashmallUiAssets0104 {
    pub(super) images: [Handle<Image>; CashmallStaticAssetRole0104::COUNT],
    pub(super) font: Handle<Font>,
    pub(super) missing_checker: Handle<Image>,
}

impl CashmallUiAssets0104 {
    pub(super) fn load(
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        contract: &CashmallUiAssetContract0104,
    ) -> Self {
        Self {
            images: array::from_fn(|index| asset_server.load(contract.image_paths[index].clone())),
            font: asset_server.load(contract.font_path.clone()),
            missing_checker: images.add(cashmall_missing_checker_image_0104()),
        }
    }

    #[must_use]
    pub(super) fn image(&self, role: CashmallStaticAssetRole0104) -> Handle<Image> {
        self.images[role.index()].clone()
    }

    #[must_use]
    pub(super) fn readiness(&self, asset_server: &AssetServer) -> CashmallStaticAssetReadiness0104 {
        if self
            .images
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
            || matches!(
                asset_server.load_state(self.font.id()),
                LoadState::Failed(_)
            )
        {
            return CashmallStaticAssetReadiness0104::Failed;
        }
        if self
            .images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(asset_server.load_state(self.font.id()), LoadState::Loaded)
        {
            CashmallStaticAssetReadiness0104::Ready
        } else {
            CashmallStaticAssetReadiness0104::Loading
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct CashmallUiRoot0104;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum CashmallUiElement0104 {
    Backdrop,
    CashmallBackplate,
    RightBackplate,
    CashmallPanel,
    Info,
    Title,
    NpcName,
    Cash,
    CashDigit(usize),
    Dialog,
    ListBack,
    ListDivider,
    TabVisual(CashmallTab0104),
    TabLabel(CashmallTab0104),
    Table,
    ListViewport,
    ListContent,
    Row(usize),
    RowFrame(usize),
    RowIcon(usize),
    RowName(usize),
    RowLevel(usize),
    TableShadow,
    GoToStuff,
    GoToStuffLabel,
    PcStuffPanel,
    NanoTab,
    NanoTabHit,
    InventoryPanel,
    ItemTabLabel,
    NanoTabLabel,
    InventoryViewport,
    InventoryContent,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotBadge(usize),
    InventorySlotCount(usize),
    DexlabsBanner,
    TarosCounter,
    TarosDigit(usize),
    RedeemCode,
    RedeemCodeLabel,
    Close,
    Trash,
    Help,
    EquipmentPanel,
    EquipmentTitle,
    EquipmentTitleLabel,
    EquipmentSlotFrame(usize),
    EquipmentSlotIcon(usize),
    EquipmentSlotBadge(usize),
    EquipmentSlotLabel(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum CashmallInteractiveControl0104 {
    Tab(CashmallTab0104),
    Row(usize),
    GoToStuff,
    NanoTab,
    RedeemCode,
    Close,
    Help,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum CashmallUiSet0104 {
    Lifecycle,
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct CashmallUiPlugin0104;

impl Plugin for CashmallUiPlugin0104 {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<CashmallUiState0104>()
            .init_resource::<CashmallModalState0104>()
            .init_resource::<CashmallCloseGate0104>()
            .init_resource::<CashmallModeProjection0104>()
            .init_resource::<CashmallUiOutbox0104>()
            .init_resource::<CashmallUiAssetContract0104>()
            .init_resource::<CashmallUiAssetStatus0104>()
            .init_resource::<CashmallHoverState0104>()
            .configure_sets(
                Update,
                (
                    CashmallUiSet0104::Lifecycle,
                    CashmallUiSet0104::Interaction,
                    CashmallUiSet0104::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_cashmall_ui_0104,
            )
            .add_systems(
                Update,
                (advance_cashmall_lifecycle_0104.in_set(CashmallUiSet0104::Lifecycle))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (collect_cashmall_ui_input_0104.in_set(CashmallUiSet0104::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (bind_cashmall_ui_0104
                    .in_set(CashmallUiSet0104::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
