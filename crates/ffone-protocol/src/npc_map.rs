//! FFOne server placement snapshot, sent after PRESENT_NPC_TYPES.
//! All integers are little endian. Header: flags (1=start, 2=end), count.
//! Entries: runtime NPC id, table type, protocol x/y/z. Packets are ordered;
//! consumers publish only complete snapshots and never merge old placements.
pub const NPC_MAP_SNAPSHOT: u32 = 0x3100013b;
pub const MAX_ENTRIES_PER_PACKET: usize = 203;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcMapEntry {
    pub id: i32,
    pub npc_type: i32,
    pub position: [i32; 3],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpcMapChunk {
    pub start: bool,
    pub end: bool,
    pub entries: Vec<NpcMapEntry>,
}
impl NpcMapChunk {
    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 8 {
            return Err("short map snapshot header");
        }
        let int = |offset| i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let flags = int(0);
        let count = usize::try_from(int(4)).map_err(|_| "negative map count")?;
        if !(0..=3).contains(&flags)
            || count > MAX_ENTRIES_PER_PACKET
            || bytes.len() != 8 + 20 * count
        {
            return Err("invalid map snapshot size or flags");
        }
        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let at = 8 + i * 20;
            let entry = NpcMapEntry {
                id: int(at),
                npc_type: int(at + 4),
                position: [int(at + 8), int(at + 12), int(at + 16)],
            };
            if entry.id <= 0 || entry.npc_type <= 0 {
                return Err("invalid map NPC identity");
            }
            entries.push(entry);
        }
        Ok(Self {
            start: flags & 1 != 0,
            end: flags & 2 != 0,
            entries,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct NpcMapSnapshot {
    pub entries: Option<std::collections::BTreeMap<i32, NpcMapEntry>>,
    pending: Option<std::collections::BTreeMap<i32, NpcMapEntry>>,
}
impl NpcMapSnapshot {
    pub fn receive(&mut self, bytes: &[u8]) -> Result<bool, &'static str> {
        let chunk = match NpcMapChunk::decode(bytes) {
            Ok(c) => c,
            Err(e) => {
                self.pending = None;
                return Err(e);
            }
        };
        if chunk.start {
            self.pending = Some(Default::default());
        }
        let pending = self
            .pending
            .as_mut()
            .ok_or("map continuation without start")?;
        for entry in chunk.entries {
            if pending.len() >= 100_000 || pending.insert(entry.id, entry).is_some() {
                self.pending = None;
                return Err("duplicate or excessive map NPCs");
            }
        }
        if chunk.end {
            self.entries = self.pending.take();
            return Ok(true);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests;
