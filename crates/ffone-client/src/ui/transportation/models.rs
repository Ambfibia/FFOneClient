use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum TransportationModelError {
    NotBrowsing,
    NonFiniteInput,
    InvalidRouteIndex(usize),
    RouteUnregistered(usize),
    InvalidCatalog(String),
}

#[derive(Clone, Debug, Resource)]
pub struct TransportationModel {
    pub(super) phase: TransportationPhase,
    pub(super) service: TransportationService,
    pub(super) routes: Vec<TransportationRoute>,
    pub(super) context: Option<TransportationOpenContext>,
    pub(super) selected_route: Option<usize>,
    pub(super) turbo: bool,
    pub(super) scroll_y: f32,
    pub(super) future: bool,
    pub(super) dark_zone: bool,
    pub(super) map: TransportationMap,
    pub(super) start_position: TransportationUiPoint,
    pub(super) target_position: TransportationUiPoint,
    pub(super) target_view: TransportationUiRect,
    pub(super) display_view: TransportationUiRect,
    pub(super) pending_elapsed: f32,
    pub(super) outbox: VecDeque<TransportationOutboxEvent>,
    pub(super) travel_intents: VecDeque<TransportationTravelIntent>,
}

impl Default for TransportationModel {
    fn default() -> Self {
        let empty_view = TransportationUiRect::new(0.0, 0.0, 1.0, 1.0);
        Self {
            phase: TransportationPhase::Hidden,
            service: TransportationService::Warp,
            routes: Vec::new(),
            context: None,
            selected_route: None,
            turbo: false,
            scroll_y: 0.0,
            future: false,
            dark_zone: false,
            map: TransportationMap::PayZone,
            start_position: TransportationUiPoint::new(0.0, 0.0),
            target_position: TransportationUiPoint::new(0.0, 0.0),
            target_view: empty_view,
            display_view: empty_view,
            pending_elapsed: 0.0,
            outbox: VecDeque::new(),
            travel_intents: VecDeque::new(),
        }
    }
}

impl TransportationModel {
    pub fn open(
        &mut self,
        catalog: &TransportationCatalog,
        context: TransportationOpenContext,
    ) -> Result<(), TransportationModelError> {
        context.validate()?;
        let projection = catalog.project(&context)?;
        let player_map_point =
            TransportationUiPoint::new(context.player.position.x, context.player.position.z);
        let future = TRANSPORTATION_FUTURE_ZONE_RECT.contains(player_map_point)
            || TRANSPORTATION_TUTORIAL_RECT.contains(player_map_point);
        let dark_zone = !future && TRANSPORTATION_DARK_ZONE_RECT.contains(player_map_point);
        let map = if future {
            TransportationMap::Future
        } else if dark_zone {
            TransportationMap::Darklands
        } else {
            TransportationMap::PayZone
        };
        let target_position = projection.start_position;

        self.phase = TransportationPhase::Browsing;
        self.service = projection.service;
        self.routes = projection.routes;
        self.context = Some(context);
        self.selected_route = None;
        self.turbo = false;
        self.scroll_y = 0.0;
        self.future = future;
        self.dark_zone = dark_zone;
        self.map = map;
        self.start_position = projection.start_position;
        self.target_position = target_position;
        self.target_view = transportation_target_view(projection.service, future, target_position);
        self.display_view = self.target_view;
        self.pending_elapsed = 0.0;
        self.outbox.clear();
        self.travel_intents.clear();
        self.outbox
            .push_back(TransportationOutboxEvent::SetCursorLocked(false));
        if let TransportationTarget::Npc {
            npc_instance_id, ..
        } = context.target
        {
            self.outbox
                .push_back(TransportationOutboxEvent::SetTransportCameraTarget(
                    npc_instance_id,
                ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn phase(&self) -> TransportationPhase {
        self.phase
    }

    #[must_use]
    pub const fn service(&self) -> TransportationService {
        self.service
    }

    #[must_use]
    pub fn routes(&self) -> &[TransportationRoute] {
        &self.routes
    }

    #[must_use]
    pub const fn selected_route(&self) -> Option<usize> {
        self.selected_route
    }

    #[must_use]
    pub const fn turbo(&self) -> bool {
        self.turbo
    }

    #[must_use]
    pub const fn map(&self) -> TransportationMap {
        self.map
    }

    #[must_use]
    pub const fn is_future(&self) -> bool {
        self.future
    }

    #[must_use]
    pub const fn is_dark_zone(&self) -> bool {
        self.dark_zone
    }

    #[must_use]
    pub const fn start_position(&self) -> TransportationUiPoint {
        self.start_position
    }

    #[must_use]
    pub const fn target_view(&self) -> TransportationUiRect {
        self.target_view
    }

    #[must_use]
    pub const fn display_view(&self) -> TransportationUiRect {
        self.display_view
    }

    #[must_use]
    pub const fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    #[must_use]
    pub fn player_taros(&self) -> Option<i32> {
        self.context.map(|context| context.player.taros)
    }

    #[must_use]
    pub fn player_position(&self) -> Option<TransportationWorldPoint> {
        self.context.map(|context| context.player.position)
    }

    /// The source GameMode retains its initialization target until it exits.
    /// Production adapters use this read-only handle for the NPC sub-target
    /// camera and the matching MainGame interaction-release edge.
    #[must_use]
    pub fn target(&self) -> Option<TransportationTarget> {
        self.context.map(|context| context.target)
    }

    #[must_use]
    pub fn route_effective_cost(&self, route_index: usize) -> Option<i32> {
        let route = self.routes.get(route_index)?;
        Some(
            if self.turbo && self.service == TransportationService::Wyvern {
                route.definition.cost.wrapping_mul(2)
            } else {
                route.definition.cost
            },
        )
    }

    #[must_use]
    pub fn route_affordable(&self, route_index: usize) -> Option<bool> {
        let taros = self.player_taros()?;
        self.route_effective_cost(route_index)
            .map(|cost| taros >= cost)
    }

    #[must_use]
    pub fn selected_subtitle(&self) -> Option<String> {
        self.selected_route
            .and_then(|index| self.routes.get(index))
            .map(|route| format!("{} - {}", route.name, route.region))
    }

    pub fn select_route(
        &mut self,
        route_index: usize,
    ) -> Result<TransportationInputResult, TransportationModelError> {
        self.ensure_browsing()?;
        let route = self
            .routes
            .get(route_index)
            .ok_or(TransportationModelError::InvalidRouteIndex(route_index))?;
        if !route.registered {
            return Err(TransportationModelError::RouteUnregistered(route_index));
        }
        if self.selected_route == Some(route_index) {
            return Ok(TransportationInputResult::Ignored);
        }
        self.selected_route = Some(route_index);
        self.target_position = route.position;
        if self.service == TransportationService::ItemUse {
            self.dark_zone = TRANSPORTATION_DARK_ZONE_RECT.contains(route.position);
            self.map = if self.dark_zone {
                TransportationMap::Darklands
            } else {
                TransportationMap::PayZone
            };
        }
        self.target_view =
            transportation_target_view(self.service, self.future, self.target_position);
        Ok(TransportationInputResult::Changed)
    }

    pub fn toggle_turbo(&mut self) -> Result<TransportationInputResult, TransportationModelError> {
        self.ensure_browsing()?;
        if self.service != TransportationService::Wyvern {
            return Ok(TransportationInputResult::Ignored);
        }
        self.turbo = !self.turbo;
        self.outbox.push_back(TransportationOutboxEvent::PlaySound(
            TransportationSound::Button,
        ));
        Ok(TransportationInputResult::Changed)
    }

    pub fn apply_scroll_axis(
        &mut self,
        axis: f32,
        legacy_scroll_velocity: f32,
    ) -> Result<TransportationInputResult, TransportationModelError> {
        if !axis.is_finite() || !legacy_scroll_velocity.is_finite() {
            return Err(TransportationModelError::NonFiniteInput);
        }
        let previous = self.scroll_y;
        self.scroll_y -= axis.clamp(-1.0, 1.0) * legacy_scroll_velocity;
        self.scroll_y = self.scroll_y.clamp(0.0, self.maximum_scroll_y());
        Ok(if (self.scroll_y - previous).abs() > f32::EPSILON {
            TransportationInputResult::Changed
        } else {
            TransportationInputResult::Ignored
        })
    }

    #[must_use]
    pub fn maximum_scroll_y(&self) -> f32 {
        (self.routes.len() as f32 * TRANSPORTATION_ROUTE_STRIDE
            + TRANSPORTATION_ROUTE_CONTENT_PADDING
            - TRANSPORTATION_SELECT_RECT.height)
            .max(0.0)
    }

    pub fn press_go(&mut self) -> Result<TransportationInputResult, TransportationModelError> {
        self.ensure_browsing()?;
        let Some(route_index) = self.selected_route else {
            self.outbox
                .push_back(TransportationOutboxEvent::SystemMessage(
                    TRANSPORTATION_NO_SELECTION_MESSAGE_ID,
                ));
            self.outbox.push_back(TransportationOutboxEvent::PlaySound(
                TransportationSound::ActionFailure,
            ));
            return Ok(TransportationInputResult::Ignored);
        };

        // DoLeft plays this before calling Trans(), including validation fails.
        self.outbox.push_back(TransportationOutboxEvent::PlaySound(
            TransportationSound::TransportationWarp,
        ));
        let route = self
            .routes
            .get(route_index)
            .ok_or(TransportationModelError::InvalidRouteIndex(route_index))?;
        if !route.registered {
            self.outbox
                .push_back(TransportationOutboxEvent::SystemMessage(
                    TRANSPORTATION_UNREGISTERED_MESSAGE_ID,
                ));
            return Ok(TransportationInputResult::Ignored);
        }
        if !self.route_affordable(route_index).unwrap_or(false) {
            self.outbox
                .push_back(TransportationOutboxEvent::SystemMessage(
                    TRANSPORTATION_INSUFFICIENT_TAROS_MESSAGE_ID,
                ));
            return Ok(TransportationInputResult::Ignored);
        }

        self.outbox
            .push_back(TransportationOutboxEvent::SetGameConditionCooldown(
                TRANSPORTATION_GAME_CONDITION_COOLDOWN,
            ));
        self.outbox
            .push_back(TransportationOutboxEvent::EndCameraSubTarget);
        self.outbox
            .push_back(TransportationOutboxEvent::InstantiatePlayerEffect(
                TRANSPORTATION_WARP_EFFECT_ID,
            ));
        if let Some(context) = self.context
            && let TransportationTarget::Npc {
                npc_instance_id,
                has_move_ok_voice,
                ..
            } = context.target
        {
            if has_move_ok_voice {
                self.outbox
                    .push_back(TransportationOutboxEvent::PlayNpcMoveOkVoice(
                        npc_instance_id,
                    ));
            }
            // Trans() plays a second NPC-attached Transportation_Warp sound.
            self.outbox.push_back(TransportationOutboxEvent::PlaySound(
                TransportationSound::TransportationWarp,
            ));
        }
        self.pending_elapsed = 0.0;
        self.phase = TransportationPhase::PendingWarp;
        Ok(TransportationInputResult::BeganWarp)
    }

    pub fn advance(
        &mut self,
        delta_seconds: f32,
    ) -> Result<TransportationInputResult, TransportationModelError> {
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err(TransportationModelError::NonFiniteInput);
        }
        if self.phase != TransportationPhase::PendingWarp {
            return Ok(TransportationInputResult::Ignored);
        }
        self.pending_elapsed += delta_seconds;
        // Clean code uses `> 1.5f`, not `>=`.
        if self.pending_elapsed <= TRANSPORTATION_WARP_DELAY_SECONDS {
            return Ok(TransportationInputResult::Ignored);
        }
        let route_index = self
            .selected_route
            .ok_or(TransportationModelError::NotBrowsing)?;
        let route = self
            .routes
            .get(route_index)
            .ok_or(TransportationModelError::InvalidRouteIndex(route_index))?;
        let context = self.context.ok_or(TransportationModelError::NotBrowsing)?;
        let (npc_id, transporation_id, e_il, slot_number) = match context.target {
            TransportationTarget::Npc {
                npc_instance_id, ..
            } => (npc_instance_id, route.definition.vehicle_id, 4, 0),
            TransportationTarget::ItemUse { e_il, slot_number } => {
                (0, route.definition.end_location, e_il, slot_number)
            }
        };
        self.travel_intents.push_back(TransportationTravelIntent {
            npc_id,
            transporation_id,
            e_il,
            slot_number,
            turbo: self.turbo,
        });
        self.outbox
            .push_back(TransportationOutboxEvent::SetMovementPacketEmission(false));
        self.outbox.push_back(TransportationOutboxEvent::Fade(
            TransportationFade::WindowIn,
        ));
        self.phase = TransportationPhase::AwaitingServer;
        Ok(TransportationInputResult::Changed)
    }

    pub fn advance_map_paint(
        &mut self,
    ) -> Result<TransportationInputResult, TransportationModelError> {
        if !self.target_view.is_valid() || !self.display_view.is_valid() {
            return Err(TransportationModelError::NonFiniteInput);
        }
        let previous = self.display_view;
        self.display_view.x = self.target_view.x * 0.2 + self.display_view.x * 0.8;
        self.display_view.y = self.target_view.y * 0.2 + self.display_view.y * 0.8;
        self.display_view.width = self.target_view.width * 0.2 + self.display_view.width * 0.8;
        self.display_view.height = self.target_view.height * 0.2 + self.display_view.height * 0.8;
        Ok(if previous == self.display_view {
            TransportationInputResult::Ignored
        } else {
            TransportationInputResult::Changed
        })
    }

    pub fn receive_success(&mut self) -> TransportationInputResult {
        self.close_internal();
        self.outbox
            .push_back(TransportationOutboxEvent::MarkPacketHandled(
                TRANSPORTATION_SUCCESS_PACKET_ID,
            ));
        self.outbox.push_back(TransportationOutboxEvent::PlaySound(
            TransportationSound::ActionSuccess,
        ));
        TransportationInputResult::Closed
    }

    pub fn receive_failure(&mut self, error_code: i32) -> TransportationInputResult {
        let message_id = match error_code {
            5 => TRANSPORTATION_UNREGISTERED_MESSAGE_ID,
            7 => TRANSPORTATION_INSUFFICIENT_TAROS_MESSAGE_ID,
            8 => TRANSPORTATION_FAILURE_MESSAGE_ID_8,
            _ => TRANSPORTATION_GENERIC_FAILURE_MESSAGE_ID,
        };
        self.outbox
            .push_back(TransportationOutboxEvent::SystemMessage(message_id));
        self.outbox
            .push_back(TransportationOutboxEvent::MarkPacketHandled(
                TRANSPORTATION_FAILURE_PACKET_ID,
            ));
        // Clean cnTrans does not reset bSend on a failure reply.
        TransportationInputResult::Ignored
    }

    pub fn close_button(&mut self, gates: TransportationInputGates) -> TransportationInputResult {
        if self.phase != TransportationPhase::Browsing || gates.system_popup_open {
            return TransportationInputResult::Ignored;
        }
        self.outbox.push_back(TransportationOutboxEvent::PlaySound(
            TransportationSound::Button,
        ));
        self.close_internal();
        TransportationInputResult::Closed
    }

    pub fn escape(&mut self, gates: TransportationInputGates) -> TransportationInputResult {
        if self.phase != TransportationPhase::Browsing
            || gates.system_popup_open
            || !gates.escape_close_allowed
        {
            return TransportationInputResult::Ignored;
        }
        self.close_internal();
        TransportationInputResult::Closed
    }

    pub fn pop_outbox(&mut self) -> Option<TransportationOutboxEvent> {
        self.outbox.pop_front()
    }

    pub fn pop_travel_intent(&mut self) -> Option<TransportationTravelIntent> {
        self.travel_intents.pop_front()
    }

    #[must_use]
    pub fn marker_projections(&self) -> Vec<TransportationMarkerProjection> {
        if self.phase != TransportationPhase::Browsing {
            return Vec::new();
        }
        let mut markers = Vec::new();
        for (route_index, route) in self.routes.iter().enumerate() {
            if !self.route_visible_in_current_zone(route) {
                continue;
            }
            if let Some(rect) = transportation_marker_rect(route.position, self.display_view) {
                markers.push(TransportationMarkerProjection {
                    route_index: Some(route_index),
                    rect,
                    asset_path: route.map_marker_path,
                    kind: TransportationMarkerKind::Route,
                });
            }
        }
        let start_asset = if self.service == TransportationService::Wyvern {
            TRANSPORTATION_REGISTERED_WYVERN_PATH
        } else {
            TRANSPORTATION_REGISTERED_WARP_PATH
        };
        if let Some(rect) = transportation_marker_rect(self.start_position, self.display_view) {
            markers.push(TransportationMarkerProjection {
                route_index: None,
                rect,
                asset_path: start_asset,
                kind: TransportationMarkerKind::Start,
            });
            let label_rect =
                TransportationUiRect::new(rect.x + 26.0, rect.y, rect.width, rect.height);
            if label_rect.x + label_rect.width
                <= TRANSPORTATION_MAP_RECT.x + TRANSPORTATION_MAP_RECT.width
            {
                markers.push(TransportationMarkerProjection {
                    route_index: None,
                    rect: label_rect,
                    asset_path: TRANSPORTATION_START_LABEL_PATH,
                    kind: TransportationMarkerKind::StartLabel,
                });
            }
        }
        if let Some(route_index) = self.selected_route
            && let Some(route) = self.routes.get(route_index)
        {
            let selected_visible = if self.service == TransportationService::Wyvern {
                true
            } else if self.future {
                // Exact clean omission: non-Wyvern selected marker has no
                // future branch in DoWindow.
                false
            } else {
                self.route_visible_in_current_zone(route)
            };
            if selected_visible
                && let Some(rect) = transportation_marker_rect(route.position, self.display_view)
            {
                markers.push(TransportationMarkerProjection {
                    route_index: Some(route_index),
                    rect,
                    asset_path: if self.service == TransportationService::Wyvern {
                        TRANSPORTATION_SELECTED_WYVERN_PATH
                    } else {
                        TRANSPORTATION_SELECTED_WARP_PATH
                    },
                    kind: TransportationMarkerKind::Selected,
                });
            }
        }
        markers
    }

    pub(super) fn ensure_browsing(&self) -> Result<(), TransportationModelError> {
        if self.phase == TransportationPhase::Browsing {
            Ok(())
        } else {
            Err(TransportationModelError::NotBrowsing)
        }
    }

    pub(super) fn route_visible_in_current_zone(&self, route: &TransportationRoute) -> bool {
        self.future
            || if self.dark_zone {
                TRANSPORTATION_DARK_ZONE_RECT.contains(route.position)
            } else {
                !TRANSPORTATION_DARK_ZONE_RECT.contains(route.position)
            }
    }

    pub(super) fn close_internal(&mut self) {
        if self.phase == TransportationPhase::Hidden {
            return;
        }
        self.outbox
            .push_back(TransportationOutboxEvent::EndCameraSubTarget);
        if let Some(context) = self.context {
            self.outbox
                .push_back(TransportationOutboxEvent::RestoreCursorLocked(
                    context.player.cursor_was_locked,
                ));
        }
        let reason = if self.phase == TransportationPhase::AwaitingServer {
            TransportationCloseReason::Departure
        } else {
            TransportationCloseReason::UserClose
        };
        self.outbox
            .push_back(TransportationOutboxEvent::ExitMode { reason });
        self.phase = TransportationPhase::Hidden;
    }
}
