use super::*;

#[must_use]
pub fn transportation_region_for_point(point: TransportationUiPoint) -> &'static str {
    RETROBUTION_WORLD_NAME_REGIONS
        .iter()
        .copied()
        .find(|region| region.contains(point))
        .map_or("unknown", |region| region.zone_name)
}

pub(super) fn extract_locations(
    rows: &[Value],
    strings: &[Value],
    icon_numbers: &[i32],
    table_name: &str,
) -> Result<Vec<TransportationLocation>, TransportationCatalogError> {
    if rows.len() != strings.len() {
        return Err(invalid(format!(
            "{table_name} has {} rows but its string array has {}",
            rows.len(),
            strings.len()
        )));
    }
    let mut locations = Vec::with_capacity(rows.len());
    for (row_index, (row, string_row)) in rows.iter().zip(strings).enumerate() {
        let context = format!("{table_name}[{row_index}]");
        let row = value_object(row, &context)?;
        let string_context = format!("{table_name}String[{row_index}]");
        let string_row = value_object(string_row, &string_context)?;
        let location_id = required_i32(row, "m_iLocationID", &context)?;
        if location_id != row_index as i32 {
            return Err(invalid(format!(
                "{context}.m_iLocationID={location_id}; clean cnTrans indexes this array by location ID"
            )));
        }
        let icon_index = required_i32(row, "m_iIcon", &context)?;
        let icon_number = usize::try_from(icon_index)
            .ok()
            .and_then(|index| icon_numbers.get(index))
            .copied()
            .ok_or_else(|| {
                invalid(format!(
                    "{context}.m_iIcon={icon_index} is outside m_pTransIcon"
                ))
            })?;
        let server_x = required_i32(row, "m_iXpos", &context)?;
        let server_y = required_i32(row, "m_iYpos", &context)?;
        let server_z = required_i32(row, "m_iZpos", &context)?;
        let position = TransportationUiPoint::new(server_x as f32 * 0.01, server_y as f32 * 0.01);
        locations.push(TransportationLocation {
            row_index,
            location_id,
            npc_id: required_i32(row, "m_iNPCID", &context)?,
            server_x,
            server_y,
            server_z,
            position,
            icon_index,
            icon_number,
            table_zone: required_i32(row, "m_iZone", &context)?,
            name: required_string(string_row, "m_pstrLocationName", &string_context)?.to_owned(),
            information: required_string(string_row, "m_pstrLocationInfo", &string_context)?
                .to_owned(),
            region: transportation_region_for_point(position).to_owned(),
        });
    }
    Ok(locations)
}

pub(super) fn indexed_location<'a>(
    locations: &'a [TransportationLocation],
    location_index: i32,
    route_index: usize,
    endpoint: &str,
) -> Result<&'a TransportationLocation, TransportationModelError> {
    usize::try_from(location_index)
        .ok()
        .and_then(|index| locations.get(index))
        .ok_or(TransportationModelError::InvalidCatalog(format!(
            "route {route_index} {endpoint} location {location_index} is outside its legacy location array"
        )))
}

pub(super) fn invalid(detail: impl Into<String>) -> TransportationCatalogError {
    TransportationCatalogError::Invalid(detail.into())
}

pub(super) fn required_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<&'a [Value], TransportationCatalogError> {
    object
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid(format!("{context}.{field} must be an array")))
}

pub(super) fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<&'a str, TransportationCatalogError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(format!("{context}.{field} must be a string")))
}

pub(super) fn required_i32(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<i32, TransportationCatalogError> {
    object
        .get(field)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| invalid(format!("{context}.{field} must be an i32")))
}

pub(super) fn transportation_target_view(
    service: TransportationService,
    future: bool,
    point: TransportationUiPoint,
) -> TransportationUiRect {
    let (width, height) = if service == TransportationService::Wyvern || future {
        (0.288_054_3, 0.307_583_4)
    } else {
        (0.576_108_6, 0.615_166_8)
    };
    let mut view = TransportationUiRect::new(
        point.x / TRANSPORTATION_WORLD_EXTENT - width * 0.5,
        point.y / TRANSPORTATION_WORLD_EXTENT - height * 0.5,
        width,
        height,
    );
    if future {
        let min_x = TRANSPORTATION_FUTURE_ZONE_RECT.x / TRANSPORTATION_WORLD_EXTENT;
        let min_y = TRANSPORTATION_FUTURE_ZONE_RECT.y / TRANSPORTATION_WORLD_EXTENT;
        let max_x = (TRANSPORTATION_FUTURE_ZONE_RECT.x + TRANSPORTATION_FUTURE_ZONE_RECT.width)
            / TRANSPORTATION_WORLD_EXTENT;
        let max_y = (TRANSPORTATION_FUTURE_ZONE_RECT.y + TRANSPORTATION_FUTURE_ZONE_RECT.height)
            / TRANSPORTATION_WORLD_EXTENT;
        // Preserve cnTrans's sequential clamps. The legacy view is wider and
        // taller than FutureZoneRect, so a conventional `clamp(min, max)`
        // would have an inverted range and would not match the final branch.
        if view.x < min_x {
            view.x = min_x;
        }
        if view.y < min_y {
            view.y = min_y;
        }
        if view.x + width > max_x {
            view.x = max_x - width;
        }
        if view.y + height > max_y {
            view.y = max_y - height;
        }
    } else {
        view.x = view.x.clamp(0.0, 1.0 - width);
        view.y = view.y.clamp(0.0, 1.0 - height);
    }
    view
}

pub(super) fn transportation_node(rect: TransportationUiRect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.x),
        top: px(rect.y),
        width: px(rect.width),
        height: px(rect.height),
        ..default()
    }
}

pub(super) fn transportation_text_node(rect: TransportationUiRect, style: TransportationUiTextStyle) -> Node {
    let mut node = transportation_node(rect);
    style.apply_to_container(&mut node);
    node
}

pub(super) fn transportation_empty_subtitle_text() -> LocalizedText {
    LocalizedText::new("ui.transportation.subtitle.empty", "")
}

pub(super) fn transportation_subtitle_text(name: &str, region: &str) -> LocalizedText {
    LocalizedText::new("ui.transportation.subtitle", "{name} - {region}")
        .with_arg("name", name)
        .with_arg("region", region)
}

pub(super) fn transportation_cost_text(amount: i32) -> LocalizedText {
    LocalizedText::new("ui.transportation.cost", "{amount}").with_arg("amount", amount.to_string())
}

pub(super) fn queue_transportation_controls(
    model: Res<TransportationModel>,
    input: Res<TransportationPresentationInput>,
    mut outbox: ResMut<TransportationUiCommandOutbox>,
    controls: Query<
        (&Interaction, &TransportationPresentationControlNode),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if model.phase() != TransportationPhase::Browsing || input.gates.system_popup_open {
        return;
    }
    for (interaction, control) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let command = match control.0 {
            TransportationPresentationControl::Close => {
                TransportationUiCommand::CloseButton(input.gates)
            }
            TransportationPresentationControl::GoNow => TransportationUiCommand::GoNow,
            TransportationPresentationControl::Turbo => TransportationUiCommand::ToggleTurbo,
            TransportationPresentationControl::Route(index) => {
                TransportationUiCommand::SelectRoute(index)
            }
        };
        outbox.push(command);
    }
}

pub(super) fn queue_transportation_escape(
    model: Res<TransportationModel>,
    input: Res<TransportationPresentationInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut outbox: ResMut<TransportationUiCommandOutbox>,
) {
    if model.phase() == TransportationPhase::Browsing && keys.just_pressed(KeyCode::Escape) {
        outbox.push(TransportationUiCommand::Escape(input.gates));
    }
}
