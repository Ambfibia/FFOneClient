//! Typed administration commands. Parsing never mutates gameplay state.
use ffone_protocol::{FixedUtf16, RegisteredGameplayRequest0104, WirePayload, wire_0104::*};

pub const NAMES: &[&str] = &[
    "/rule",
    "/motd",
    "/announce",
    "/bcast",
    "/nano_equip",
    "/nano_unequip",
    "/nano_active",
    "/warptopc",
    "/nanoArr",
    "/summon",
    "/groupsummon",
    "/summonshiny",
    "/unsummon",
    "/nanoskill",
    "/mission",
    "/task",
    "/unstick_n",
    "/unstick_i",
    "/unstick_ui",
    "/unstick",
    "/locate_i",
    "/locate_ui",
    "/locate_n",
    "/teleport2me_n",
    "/teleport2me_i",
    "/teleport2me_ui",
    "/teleportXYZ_i",
    "/teleportXYZ_ui",
    "/teleportXYZ_n",
    "/teleportMapXYZ_i",
    "/teleportMapXYZ_n",
    "/teleportMapXYZ_ui",
    "/teleport_i_i",
    "/teleport_ui_ui",
    "/teleport_i_n",
    "/teleport_n_n",
    "/kick_i",
    "/kick_ui",
    "/kick_n",
    "/invisible",
    "/invulnerable",
    "/gmmarker",
    "/equipitem",
    "/viewloc",
    "/viweloc",
    "/viewnetinfo",
    "/mute_i_on",
    "/mute_i_off",
    "/mute_ui_on",
    "/mute_ui_off",
    "/mute_n_on",
    "/mute_n_off",
    "/hideui",
    "/viewcol",
    "/qinven",
    "/chnum",
    "/chinfo",
    "/chwarp",
    "/shwarp",
    "/viewid",
    "/Store",
    "/rateT",
    "/rateF",
    "/tasklog",
    "/shardwarp",
];

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Packet(RegisteredGameplayRequest0104),
    Local { name: String, value: Option<i32> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Usage(String),
    Denied(i16),
    PlayerMissing,
    TargetMissing,
}

fn bad(usage: &str) -> Error {
    Error::Usage(usage.to_owned())
}
fn number<T: std::str::FromStr>(s: &str, usage: &str) -> Result<T, Error> {
    s.parse().map_err(|_| bad(usage))
}
fn zero<T: WirePayload>() -> T {
    T::decode(&vec![0; T::SIZE]).expect("zero-initialized fixed wire request")
}
fn packet<T: WirePayload>(id: u32, p: T) -> Result<Action, Error> {
    RegisteredGameplayRequest0104::new(id, p.encode())
        .map(Action::Packet)
        .map_err(|_| bad("registered GM packet"))
}

#[derive(Default)]
struct Target {
    by: i32,
    id: i32,
    uid: i64,
    first: FixedUtf16<10>,
    last: FixedUtf16<18>,
}
fn target(kind: &str, args: &[&str]) -> Result<Target, Error> {
    let usage = "<PC-ID> | <PC-UID> | <FirstName> <LastName>";
    let mut t = Target::default();
    match (kind, args) {
        ("i", [id]) => {
            t.id = number(id, usage)?;
            if t.id <= 0 {
                return Err(bad(usage));
            }
        }
        ("ui", [id]) => {
            t.by = 2;
            t.uid = number(id, usage)?;
            if t.uid <= 0 {
                return Err(bad(usage));
            }
        }
        ("n", [first, rest @ ..]) if (1..=2).contains(&rest.len()) => {
            t.by = 1;
            t.first = FixedUtf16::from_str(first).map_err(|_| bad(usage))?;
            t.last = FixedUtf16::from_str(&rest.join(" ")).map_err(|_| bad(usage))?;
        }
        _ => return Err(bad(usage)),
    }
    Ok(t)
}

pub fn parse(
    message: &str,
    level: i16,
    pc: Option<i32>,
    position: [i32; 3],
    selected_npc: Option<i32>,
) -> Result<Action, Error> {
    let words: Vec<_> = message.split_whitespace().collect();
    let Some((&name, args)) = words.split_first() else {
        return Err(bad("/command"));
    };
    if !NAMES.contains(&name) {
        return Err(bad("/command"));
    }
    let strict = name.starts_with("/teleport")
        || name.starts_with("/unstick")
        || name.starts_with("/locate_")
        || name.starts_with("/kick_")
        || name.starts_with("/mute_")
        || matches!(
            name,
            "/announce"
                | "/bcast"
                | "/motd"
                | "/warptopc"
                | "/invisible"
                | "/invulnerable"
                | "/gmmarker"
                | "/rateT"
                | "/rateF"
                | "/summon"
                | "/unsummon"
                | "/groupsummon"
                | "/summonshiny"
        );
    let max = if strict { 30 } else { 50 };
    if level > max {
        return Err(Error::Denied(max));
    }
    let pc = pc.ok_or(Error::PlayerMissing)?;
    let one = |usage: &str| -> Result<i32, Error> {
        if let [s] = args {
            number(s, usage)
        } else {
            Err(bad(usage))
        }
    };
    match name {
        "/rule" | "/nanoArr" | "/equipitem" => Ok(Action::Local {
            name: name.to_owned(),
            value: Some(one(&format!("{name} <value>"))?),
        }),
        "/Store" => {
            let target = match args {
                [] => 0,
                [id] => number::<i32>(id, "/Store [target-PC-ID]")?,
                _ => return Err(bad("/Store [target-PC-ID]")),
            };
            if target < 0 {
                return Err(bad("/Store [target-PC-ID]"));
            }
            Ok(Action::Local {
                name: name.to_owned(),
                value: Some(target),
            })
        }
        "/viewloc" | "/viweloc" | "/viewnetinfo" | "/hideui" | "/viewcol" | "/qinven" | "/viewid"
        | "/tasklog" => {
            if !args.is_empty() && !(name == "/viewnetinfo" && args == ["history"]) {
                return Err(bad(name));
            }
            Ok(Action::Local {
                name: name.to_owned(),
                value: if args.is_empty() { None } else { Some(1) },
            })
        }
        "/announce" | "/bcast" | "/motd" => {
            let (header, text) = message.split_once(" -").ok_or_else(|| {
                bad(if name == "/motd" {
                    "/motd <type> -message"
                } else {
                    "/announce <area|shard|world|global> <type> <seconds> -message"
                })
            })?;
            let h: Vec<_> = header.split_whitespace().collect();
            if text.is_empty() {
                return Err(bad("non-empty message"));
            }
            let msg = FixedUtf16::from_str(text).map_err(|_| bad("message <= 511 UTF-16 units"))?;
            if name == "/motd" {
                if h.len() != 2 {
                    return Err(bad("/motd <type> -message"));
                }
                packet(
                    0x13000072,
                    GmPcMotdRegisterRequest0104 {
                        type_: number(h[1], "/motd <type> -message")?,
                        system_msg: msg,
                    },
                )
            } else {
                if h.len() != 4 {
                    return Err(bad("/announce <area> <type> <seconds> -message"));
                }
                let area = match h[1] {
                    "area" => 0,
                    "shard" => 1,
                    "world" => 2,
                    "global" => 3,
                    n => number(n, "area 0..3")?,
                };
                let seconds = number(h[3], "seconds >= 0")?;
                if !(0..=3).contains(&area) || seconds < 0 {
                    return Err(bad("area 0..3, seconds >= 0"));
                }
                packet(
                    0x1300006f,
                    GmPcAnnounceRequest0104 {
                        area_type: area,
                        announce_type: number(h[2], "announce type")?,
                        during_time: seconds,
                        announce_msg: msg,
                    },
                )
            }
        }
        "/nano_equip" | "/nanoskill" => {
            let [a, b] = args else {
                return Err(bad(&format!("{name} <nano-id> <slot|skill>")));
            };
            let id: i16 = number(a, "nano ID")?;
            let value: i16 = number(b, "slot or skill")?;
            if id <= 0 || value < 0 {
                return Err(bad("positive Nano ID and nonnegative slot/skill"));
            }
            if name == "/nano_equip" {
                if value > 2 {
                    return Err(bad("/nano_equip <nano-id> <slot:0..2>"));
                }
                packet(
                    0x1300000d,
                    NanoEquipRequest0104 {
                        nano_id: id,
                        nano_slot_num: value,
                    },
                )
            } else {
                packet(
                    0x13000048,
                    PcGiveNanoSkillRequest0104 {
                        nano_id: id,
                        nano_skill_id: value,
                    },
                )
            }
        }
        "/nano_unequip" | "/nano_active" => {
            let slot = one("<slot:0..2>, /nano_active -1 to dismiss")?;
            if slot > 2 || slot < if name == "/nano_active" { -1 } else { 0 } {
                return Err(bad("Nano slot"));
            }
            if name == "/nano_active" {
                packet(
                    0x1300000f,
                    NanoActiveRequest0104 {
                        nano_slot_num: slot as i16,
                    },
                )
            } else {
                packet(
                    0x1300000e,
                    NanoUnequipRequest0104 {
                        nano_slot_num: slot as i16,
                    },
                )
            }
        }
        "/warptopc" => packet(
            0x13000057,
            PcWarpToPcRequest0104 {
                pc_id: one("/warptopc <pc-id>")?,
                pcuid: 0,
            },
        ),
        "/summon" => {
            let (id, count) = match args {
                [id] => (number(id, "NPC type")?, 1),
                [id, count] => (number(id, "NPC type")?, number(count, "count 1..100")?),
                _ => return Err(bad("/summon <type> [count]")),
            };
            if id <= 0 || !(1..=100).contains(&count) {
                return Err(bad("/summon <type> [count:1..100]"));
            }
            packet(
                0x13000045,
                NpcSummonRequest0104 {
                    npc_type: id,
                    npc_cnt: count,
                },
            )
        }
        "/unsummon" => {
            if !args.is_empty() {
                return Err(bad("/unsummon (select an NPC)"));
            }
            packet(
                0x13000046,
                NpcUnsummonRequest0104 {
                    npc_id: selected_npc.ok_or(Error::TargetMissing)?,
                },
            )
        }
        "/groupsummon" => packet(
            0x13000056,
            NpcGroupSummonRequest0104 {
                npc_group_type: one("/groupsummon <group-type>")?,
            },
        ),
        "/summonshiny" => packet(
            0x13000061,
            ShinySummonRequest0104 {
                shiny_type: one("/summonshiny <type>")?,
                x: position[0]
                    .checked_add(200)
                    .ok_or_else(|| bad("position"))?,
                y: position[1]
                    .checked_add(200)
                    .ok_or_else(|| bad("position"))?,
                z: position[2],
            },
        ),
        "/mission" => packet(
            0x13000076,
            PcMissionCompleteRequest0104 {
                mission_num: one("/mission <mission-id>")?,
            },
        ),
        "/task" => packet(
            0x13000077,
            PcTaskCompleteRequest0104 {
                task_num: one("/task <task-id>")?,
            },
        ),
        "/invisible" | "/invulnerable" | "/gmmarker" => {
            if !args.is_empty() {
                return Err(bad(name));
            }
            packet(
                0x1300006a,
                GmPcSpecialStateSwitchRequest0104 {
                    pc_id: pc,
                    special_state_flag: match name {
                        "/invisible" => 2,
                        "/invulnerable" => 4,
                        _ => 1,
                    },
                },
            )
        }
        "/rateT" | "/rateF" => {
            let (get_set, index, value) = match args {
                [] => (0, 0, 0),
                [i, v] => (1, number(i, "rate index 0..4")?, number(v, "rate 0..1000")?),
                _ => return Err(bad("/rateT|/rateF [index percent]")),
            };
            if !(0..=4).contains(&index) || !(0..=1000).contains(&value) {
                return Err(bad("index 0..4, percent 0..1000"));
            }
            packet(
                0x130000a3,
                GmRewardRateRequest0104 {
                    get_set,
                    reward_type: if name == "/rateT" { 0 } else { 1 },
                    reward_rate_index: index,
                    set_rate_value: value,
                },
            )
        }
        "/chnum" | "/chinfo" => {
            if !args.is_empty() {
                return Err(bad(name));
            }
            if name == "/chnum" {
                packet(0x1300008b, PcChannelNumRequest0104)
            } else {
                packet(0x1300008a, ChannelInfoRequest0104)
            }
        }
        "/chwarp" | "/shwarp" | "/shardwarp" => {
            let channel = one(&format!("{name} <number>"))?;
            if channel <= 0 {
                return Err(bad("channel/shard >= 1"));
            }
            packet(
                0x1300008c,
                PcWarpChannelRequest0104 {
                    channel_num: channel,
                    warp_type: if name == "/chwarp" { 0 } else { 1 },
                },
            )
        }
        _ if name.starts_with("/locate_")
            || name.starts_with("/kick_")
            || name.starts_with("/mute_") =>
        {
            let bits: Vec<_> = name.split('_').collect();
            let t = target(bits[1], args)?;
            if bits[0] == "/locate" {
                packet(
                    0x1300006e,
                    GmPcLocationRequest0104 {
                        target_search_by: t.by,
                        target_pc_id: t.id,
                        target_pc_uid: t.uid,
                        target_pc_first_name: t.first,
                        target_pc_last_name: t.last,
                    },
                )
            } else if bits[0] == "/kick" {
                packet(
                    0x1300006c,
                    GmKickPlayerRequest0104 {
                        pc_id: pc,
                        target_search_by: t.by,
                        target_pc_id: t.id,
                        target_pc_uid: t.uid,
                        target_pc_first_name: t.first,
                        target_pc_last_name: t.last,
                    },
                )
            } else {
                packet(
                    0x13000082,
                    GmTargetPcSpecialStateOnoffRequest0104 {
                        target_search_by: t.by,
                        target_pc_id: t.id,
                        target_pc_uid: t.uid,
                        target_pc_first_name: t.first,
                        target_pc_last_name: t.last,
                        onoff: if bits[2] == "on" { 1 } else { 0 },
                        special_state_flag: 64,
                    },
                )
            }
        }
        _ => {
            let mut p: GmTargetPcTeleportRequest0104 = zero();
            p.pc_id = pc;
            let bits: Vec<_> = name.split('_').collect();
            let t;
            if bits[0] == "/teleport" {
                p.teleport_type = 3;
                let (a, b): (Vec<&str>, Vec<&str>) = if name == "/teleport_n_n" {
                    let (_, tail) = message.split_once(' ').ok_or_else(|| bad(name))?;
                    let (a, b) = tail
                        .split_once(';')
                        .ok_or_else(|| bad("/teleport_n_n First Last;First Last"))?;
                    (
                        a.split_whitespace().collect(),
                        b.split_whitespace().collect(),
                    )
                } else {
                    if args.len() < 2 {
                        return Err(bad(name));
                    }
                    (args[..1].to_vec(), args[1..].to_vec())
                };
                t = target(bits[1], &a)?;
                let goal = target(bits[2], &b)?;
                p.goal_pc_search_by = goal.by;
                p.goal_pc_id = goal.id;
                p.goal_pc_uid = goal.uid;
                p.goal_pc_first_name = goal.first;
                p.goal_pc_last_name = goal.last;
            } else if bits[0] == "/unstick" {
                p.teleport_type = 4;
                t = if bits.len() == 1 {
                    if !args.is_empty() {
                        return Err(bad("/unstick"));
                    }
                    Target {
                        id: pc,
                        ..Default::default()
                    }
                } else {
                    target(bits[1], args)?
                };
            } else if bits[0] == "/teleport2me" {
                p.teleport_type = 2;
                t = target(bits[1], args)?;
            } else {
                let map = bits[0] == "/teleportMapXYZ";
                let n = if map { 4 } else { 3 };
                if args.len() <= n {
                    return Err(bad("<target> <[map] x y z>, names follow coordinates"));
                }
                let (coords, who) = if bits[1] == "n" {
                    (&args[..n], &args[n..])
                } else {
                    (&args[1..], &args[..1])
                };
                if coords.len() != n {
                    return Err(bad(name));
                }
                let start = if map {
                    p.to_map = number(coords[0], "map ID")?;
                    1
                } else {
                    0
                };
                let coord = |s: &str| {
                    number::<i32>(s, "coordinate")?
                        .checked_mul(100)
                        .ok_or_else(|| bad("coordinate overflow"))
                };
                p.to_x = coord(coords[start])?;
                p.to_y = coord(coords[start + 1])?;
                p.to_z = coord(coords[start + 2])?;
                p.teleport_type = if map { 1 } else { 0 };
                t = target(bits[1], who)?;
            }
            p.target_pc_search_by = t.by;
            p.target_pc_id = t.id;
            p.target_pc_uid = t.uid;
            p.target_pc_first_name = t.first;
            p.target_pc_last_name = t.last;
            packet(0x1300006d, p)
        }
    }
}
