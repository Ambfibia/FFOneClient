//! User-authored instance IDs; empty slots remain explicit and are never renumbered.
use super::*;

impl WorldEditor {
    pub(super) fn register_instance(&mut self, id: u32) -> Result<(), String> {
        let folder = self
            .folder
            .as_ref()
            .ok_or("Select the server tabledata folder first")?;
        let paths = [
            self.root.join("data/tables/xdt.json"),
            folder.join("xdt.json"),
        ];
        let mut patches = Vec::new();
        for path in paths {
            let index = if let Some(i) = self.sources.iter().position(|s| s.path == path) {
                i
            } else {
                let base = terrain::read_source(&path)?;
                let i = self.sources.len();
                self.sources.push(model::Source {
                    path,
                    base: base.clone(),
                    draft: base,
                });
                i
            };
            let pointer = "/m_pInstanceTable/m_pInstanceData";
            let before = self.sources[index]
                .draft
                .pointer(pointer)
                .cloned()
                .ok_or("Missing instance table")?;
            let mut after = before.clone();
            let rows = after.as_array_mut().ok_or("Invalid instance table")?;
            if rows
                .iter()
                .any(|r| r["m_iInstanceNameID"].as_u64() == Some(u64::from(id)))
            {
                return Err("Instance ID already exists in the game tables".into());
            }
            rows.push(serde_json::json!({"m_ScoreMax":0,"m_SortIndex":0,"m_iInstanceNameID":id,"m_iIsEP":0,"m_iZoneX":0,"m_iZoneY":0}));
            patches.push(model::Patch {
                source: index,
                pointer: pointer.into(),
                before: Some(before),
                after: Some(after),
            });
        }
        self.commit(patches)
    }
}

pub(super) fn names() -> BTreeMap<u32, String> {
    const NAMES: &str = "World
Empty
Empty
Fusion Mega Echo
Empty
Empty
Sweet Revenge
Mandark's House
Empty
Empty
Fusion Eddy's Lair
Fusion Eduardo's Lair
Fusion Numbuh Two's Lair
Empty
Time Machine
KND Training Area
Pokey Oaks Junior High (Future)
Mandark's House (Future)
Empty
Delightful Developments (Future)
Megas' Last Stand (Future)
Pokey Oaks Junior High
Delightful Developments
Megas' Last Stand
The Boneyard
Reactor Works
Charles Darwin Middle School
Fusion Princess' Secret Lair
Fusion Him's Lair
Fusion Utonium's Lair
Fusion Wilt's Lair
Fusion Dee Dee's Lair
Fusion Coop's Lair
Fusion Numbuh Five's Lair
Fusion Edd's Lair
Fusion Billy's Lair
Fusion Bubbles' Secret Lair
Fusion Him's Secret Lair
Fusion Eddy's Secret Lair
Fusion Numbuh Five's Secret Lair
Fusion Utonium's Secret Lair
Fusion Father's Secret Lair
Fusion Numbuh Two's Secret Lair
Fusion Numbuh Three's Secret Lair
Fusion Fuzzy's Secret Lair
Fusion Wilt's Secret Lair
Fusion Frankie's Secret Lair
Fusion Mayor's Secret Lair
Fusion Blossom's Secret Lair
Fusion Blossom's Lair
Fusion Scotsman's Lair
Fusion Vilgax's Lair
Fusion Bubbles' Lair
Fusion Numbuh One's Lair
Fusion Grim's Lair
Fusion Max's Lair
Fusion Mojo Jojo's Lair
Fusion Dexter's Lair
Fusion Bloo's Lair
Fusion Mandark's Lair
Fusion Gwen's Lair
Fusion Ace's Secret Lair
Fusion Buttercup's Secret Lair
Fusion Vilgax's Secret Lair
Fusion Eduardo's Secret Lair
Fusion Mandy's Secret Lair
Fusion Tunnel to Lab
Fusion Billy's Secret Lair
Dizzy World
Sunny Bridges Auditorium
Cutts and Bruises Skate Park
Sand Castle
Construction Site
Tyrannical Gardens
Skypad Space Port
The Fissure
Jungle Training Area
Loch Mess
Crystalline Caverns
Hani-Baba Temple
The Canopy
Monkey Summit
Fusion Mandy's Lair
Fusion Juniper Lee's Lair
Fusion Demongo's Lair
Fusion Coco's Lair
Fusion Ed's Lair
Fusion Tetrax's Lair
Fusion Mac's Lair
Fusion Numbuh Four's Lair
Fusion Control Center 1
Fusion Numbuh Three's Lair
Fusion Hex's Lair
Fusion Courage's Lair
Fusion Grim's Secret Lair
Fusion Edd's Secret Lair
Fusion Dee Dee's Secret Lair
Fusion Max's Secret Lair
Fusion Hoss's Secret Lair
Ammunition Depot
Fusion Ben's Secret Lair
Fusion Mac's Secret Lair
Fusion Hex's Secret Lair
Fusion Stickybeard's Secret Lair
Fusion Numbuh Four's Secret Lair
Foster's (Kevin Fight)
Fusion Toiletnator's Secret Lair
Fusion Mojo Jojo's Secret Lair
Fusion Courage's Secret Lair
Heart of the Dark Tree
Fusion Control Center 2
Fusion Control Center 3
Fusion Control Center 4
Fusion Kevin's Secret Lair
Fusion Scotsman's Secret Lair
Fusion Bloo's Secret Lair
Dinosaur Graveyard
Inferno Fields
Dark Tree Clearing
Green Gullet
Nowhere Triangle
Empty
Fusion Gwen's Secret Lair
Lair of Fusion Edds
Fusion Juniper Lee's Secret Lair
Fusion Coop's Secret Lair
Fusion Numbuh One's Secret Lair
Fuse's Hall
Fuse's Labyrinth
Fuse's Throne
Empty - Spring Chicken's Cave
Empty - Spring Chicken's Cave
Empty - Spring Chicken's Cave
Empty - Spring Chicken's Cave
Empty - Spring Chicken's Cave
Fusion Cheese's Lair
Empty - Blowfish Lair
Empty - Blowfish Lair
Fusion Ampfibian's Secret Lair
Fusion Spidermonkey's Secret Lair
Fusion Ben's Epic Lair
Fusion Dracula's Secret Lair
Fusion Ice King's Secret Lair
Fusion Gunter's Secret Lair
The Spire
Harada-Bridges Records
Recording Studio Hall (1)
Recording Studio
Recording Studio Hall (2)
Mandroid Digitizer
Fuse's Backdoor
Unknown - Crazy Metal Boy Scout
Unknown - Cylomander
Unknown - Pack Arachnid
Unknown - Wild Stinger
Unknown - Motorolla
Unknown - Ramcycle
Unknown - Fire Hydra
Unknown - Freakshow Fred";
    NAMES
        .lines()
        .enumerate()
        .map(|(id, name)| (id as u32, name.into()))
        .collect()
}
