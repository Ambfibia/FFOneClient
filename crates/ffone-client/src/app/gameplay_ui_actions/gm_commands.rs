use ffone_protocol::{RegisteredGameplayRequest0104, WirePayload, packet, wire_0104::*};

pub(super) fn usage(name: &str) -> Option<&'static str> {
    Some(match name {
        "/taro" | "/taros" => "/taro <value>",
        "/fm" | "/fusionmatter" => "/fm <value>",
        "/health" | "/hp" => "/health <value>",
        "/batteryW" => "/batteryW <value>",
        "/batteryN" => "/batteryN <value>",
        "/jump" => "/jump <value>",
        "/warp" => "/warp <tile-x> <tile-y>",
        "/goto" => "/goto <x> <y> [z]",
        "/item" | "/itemN" => "/item <type> <id> <count> [time-left]",
        "/itemQ" => "/itemQ <id> <count>",
        _ => return None,
    })
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Error {
    Usage,
    Denied,
    MissingPlayer,
    Full,
}

pub(super) fn request(
    message: &str,
    level: i16,
    player: Option<i32>,
    free_slot: impl FnOnce(bool, i16) -> Option<usize>,
) -> Result<RegisteredGameplayRequest0104, Error> {
    let mut words = message.split_whitespace();
    let name = words.next().ok_or(Error::Usage)?;
    if usage(name).is_none() {
        return Err(Error::Usage);
    }
    if level > 50 {
        return Err(Error::Denied);
    }
    let player = player.ok_or(Error::MissingPlayer)?;
    let args: Vec<i32> = words
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| Error::Usage)?;
    let value_type = match name {
        "/health" | "/hp" => 1,
        "/batteryW" => 2,
        "/batteryN" => 3,
        "/fm" | "/fusionmatter" => 4,
        "/taro" | "/taros" => 5,
        "/jump" => 7,
        _ => 0,
    };
    let (id, bytes) = if value_type != 0 {
        let [value] = args.as_slice() else {
            return Err(Error::Usage);
        };
        (
            packet::P_CL2FE_GM_REQ_PC_SET_VALUE,
            ffone_protocol::GmSetValueRequest0104 {
                pc_id: player,
                value_type,
                value: *value,
            }
            .encode(),
        )
    } else if name == "/warp" || name == "/goto" {
        let (x, y, z) = match args.as_slice() {
            [x, y] => (*x, *y, 10_000),
            [x, y, z] if name == "/goto" => (*x, *y, *z),
            _ => return Err(Error::Usage),
        };
        let coordinate = |v: i32| {
            if name == "/warp" {
                v.checked_mul(512)?.checked_add(256)?.checked_mul(100)
            } else {
                // /goto accepts raw server coordinates, including height.
                Some(v)
            }
        };
        (
            packet::P_CL2FE_REQ_PC_GOTO,
            PcGotoRequest0104 {
                to_x: coordinate(x).ok_or(Error::Usage)?,
                to_y: coordinate(y).ok_or(Error::Usage)?,
                to_z: z,
            }
            .encode(),
        )
    } else {
        let quest = name == "/itemQ";
        let (kind, id, count, time_left) = match args.as_slice() {
            [id, count] if quest => (8, *id, *count, 0),
            [kind, id, count] if !quest => (*kind, *id, *count, 0),
            [kind, id, count, time] if !quest => (*kind, *id, *count, *time),
            _ => return Err(Error::Usage),
        };
        if !(0..=10).contains(&kind) || id <= 0 || count <= 0 || time_left < 0 {
            return Err(Error::Usage);
        }
        let id = i16::try_from(id).map_err(|_| Error::Usage)?;
        let slot = free_slot(quest, id).ok_or(Error::Full)?;
        (
            packet::P_CL2FE_REQ_PC_GIVE_ITEM,
            PcGiveItemRequest0104 {
                e_il: if quest { 2 } else { 1 },
                slot_num: i32::try_from(slot).map_err(|_| Error::Full)?,
                item: ItemBase0104 {
                    type_: kind as i16,
                    id,
                    opt: count,
                    time_limit: 0,
                },
                time_left,
            }
            .encode(),
        )
    };
    RegisteredGameplayRequest0104::new(id, bytes).map_err(|_| Error::Usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn destination(command: &str) -> PcGotoRequest0104 {
        let request = request(command, 1, Some(42), |_, _| None).unwrap();
        assert_eq!(request.packet_type(), packet::P_CL2FE_REQ_PC_GOTO);
        PcGotoRequest0104::decode(request.payload()).unwrap()
    }

    #[test]
    fn goto_preserves_protocol_coordinates_including_negative_height() {
        let target = destination("/goto 648846 194661 -5667");
        assert_eq!(
            [target.to_x, target.to_y, target.to_z],
            [648846, 194661, -5667]
        );
        let target = destination("/goto -2147483648 2147483647 2147483647");
        assert_eq!(
            [target.to_x, target.to_y, target.to_z],
            [i32::MIN, i32::MAX, i32::MAX]
        );
    }

    #[test]
    fn warp_keeps_tile_centers_and_optional_goto_keeps_default_height() {
        let target = destination("/warp 12 3");
        assert_eq!(
            [target.to_x, target.to_y, target.to_z],
            [640000, 179200, 10000]
        );
        let target = destination("/goto 648846 194661");
        assert_eq!(
            [target.to_x, target.to_y, target.to_z],
            [648846, 194661, 10000]
        );
        assert_eq!(
            request("/warp 2147483647 3", 1, Some(42), |_, _| None),
            Err(Error::Usage)
        );
    }
}
