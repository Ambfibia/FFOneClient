use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TUTORIAL_EFFECT_ROOT: &str = "map/shared/effects";
pub const TUTORIAL_PROJECTILE_ROOT: &str = "map/shared/projectiles";
pub const TUTORIAL_EFFECT_CATALOG_PATH: &str = "map/shared/effects/catalog.json";
pub const TUTORIAL_PROJECTILE_CATALOG_PATH: &str = "map/shared/projectiles/catalog.json";
pub const TUTORIAL_EFFECT_CATALOG_SCHEMA: &str = "ffone.tutorial-effect-catalog.v1";
pub const TUTORIAL_PROJECTILE_CATALOG_SCHEMA: &str = "ffone.tutorial-projectile-catalog.v1";
pub const TUTORIAL_EFFECT_CLOSURE_SCHEMA: &str = "ffone.tutorial-effect-closure.v1";
pub const TUTORIAL_BULLET_ROW_SCHEMA: &str = "ffone.tutorial-bullet-row.v1";
pub const RETROBUTION_TUTORIAL_BUILD_ID: &str = "retrobution-20260821";

/// Exact tutorial/gameplay closure, including Buttercup skill-1's `tag_p`
/// ES60 event and all three `NanoMoveController.Call` style payloads.
pub const RETROBUTION_TUTORIAL_EFFECT_IDS: [i32; 33] = [
    4, 10, 38, 60, 366, 372, 527, 528, 529, 653, 668, 705, 723, 734, 736, 739, 740, 741, 742, 750,
    751, 757, 767, 771, 772, 774, 778, 809, 812, 813, 817, 865, 866,
];
/// Exact EffectPackage indices referenced by `particle` and `tag_p` events in
/// the clean-primary Fusion character AnimationClips currently published in
/// `assets/game/characters/fusions`. These are a distinct preload group from
/// the tutorial scene effects even though they share the same exact source
/// `Effects.resourceFile` and native renderer.
pub const RETROBUTION_FUSION_ACTOR_EFFECT_IDS: [i32; 95] = [
    530, 537, 538, 542, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553, 554, 555, 556, 557,
    558, 559, 560, 561, 562, 563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575, 576,
    577, 578, 579, 580, 581, 582, 583, 584, 592, 593, 600, 601, 602, 603, 604, 605, 606, 607, 608,
    609, 610, 611, 612, 613, 614, 615, 616, 617, 618, 619, 620, 621, 622, 623, 624, 625, 626, 627,
    628, 629, 630, 631, 633, 634, 635, 636, 637, 638, 639, 640, 641, 642, 643, 644, 645, 647, 650,
];
/// Additional exact EffectPackage indices referenced by `particle`/`tag_p`
/// events in the clean-primary non-Fusion character clips published by the
/// production registry. This also includes the local player's three exact
/// inventory-computer callbacks (ES425/833/834). IDs already owned by the
/// tutorial or Fusion groups remain excluded from this disjoint preload set.
pub const RETROBUTION_CHARACTER_ACTOR_EFFECT_IDS: [i32; 26] = [
    3, 21, 50, 425, 503, 688, 689, 690, 691, 692, 693, 694, 695, 697, 698, 699, 700, 726, 737, 763,
    764, 765, 768, 769, 833, 834,
];
/// Exact clean-primary status effects owned by `cnOwnAvatarStatus`: the
/// persistent infection buff, the poison damage tick, and Nano protection.
/// This group is disjoint from animation-event and world-map effects.
pub const RETROBUTION_PLAYER_STATUS_EFFECT_IDS: [i32; 3] = [376, 385, 804];
/// Exact clean-primary `NpcIconMode.LoadWarpEffect` payload. Warp owns this
/// player-position effect directly rather than through an AnimationEvent.
pub const RETROBUTION_NPC_WARP_EFFECT_IDS: [i32; 1] = [394];
/// Exact EffectPackage indices referenced by world-map `EPElementController`
/// records. They share Effects.resourceFile and the native particle renderer
/// with tutorial effects, but are kept separate from the tutorial preload set.
pub const RETROBUTION_WORLD_EP_EFFECT_IDS: [i32; 7] = [461, 464, 465, 466, 533, 534, 594];
/// Every EffectPackage reachable through clean-primary
/// `NpcMoveController.MakeGameIcon`, including race and Recall Point states.
pub const RETROBUTION_NPC_GAME_ICON_EFFECT_IDS: [i32; 20] = [
    66, 395, 446, 672, 673, 674, 675, 676, 677, 678, 679, 680, 681, 682, 683, 685, 811, 824, 825,
    826,
];
/// Disjoint EffectPackage IDs reached through the fire/success fields of the
/// clean-primary weapon BulletTable rows. IDs already owned by a broader
/// tutorial/character preload group are deliberately excluded.
pub const RETROBUTION_WEAPON_EFFECT_IDS: [i32; 23] = [
    55, 405, 504, 667, 720, 724, 752, 753, 756, 770, 773, 775, 776, 777, 779, 780, 781, 782, 783,
    784, 810, 818, 819,
];
/// Every clean-primary BulletTable row used by an obtainable XDT weapon, plus
/// the four non-weapon rows already required by tutorial/NPC choreography.
/// XDT references 170..174 too, but the primary BulletTable ends at 169; those
/// five source gaps are intentionally not fabricated here.
pub const RETROBUTION_TUTORIAL_BULLET_TYPES: [i32; 39] = [
    5, 6, 9, 13, 17, 51, 66, 68, 69, 72, 76, 77, 106, 113, 115, 118, 121, 131, 133, 134, 135, 136,
    145, 146, 147, 148, 149, 150, 151, 152, 155, 156, 158, 160, 161, 162, 163, 164, 165,
];
/// Exact particle carriers referenced by the published BulletTable rows.
pub const RETROBUTION_TUTORIAL_PROJECTILE_EFFECT_IDS: [i32; 28] = [
    15, 17, 31, 100, 321, 378, 379, 391, 718, 721, 729, 747, 749, 754, 755, 787, 788, 789, 790,
    791, 792, 793, 794, 795, 796, 797, 798, 808,
];
/// Canonical serialized-row BLAKE3 proofs from the exact primary
/// `bullettable.asset` PathID 14153. Keeping the proof beside the accepted ID
/// set prevents a catalog and row file from being edited together to redefine
/// clean weapon behavior.
pub const RETROBUTION_BULLET_ROW_PROOFS: [(i32, &str); 39] = [
    (
        5,
        "8ab0d4590b6a37ddcd50f44a1604ef284eda931d233e76381d097cddb7d37a03",
    ),
    (
        6,
        "63d7f23dc0d1193b5898ff4618cc2cf9ef7492ee9cf5ce2036e4ce1171b086c8",
    ),
    (
        9,
        "f2113eba70d6cf3696290a1d56ec8c2694e2241f7f8c830d9af8eb4c541e8324",
    ),
    (
        13,
        "aa0e49d90b56cd18a2c805252866a7f34e732f19c1da9f5e3e0047a23c5753d4",
    ),
    (
        17,
        "7339cedd7f17c03c68b48ed5b1b7b5f118bbd18742da0c09c1bd04c61de5583c",
    ),
    (
        51,
        "2cc2e41e7f21ce63e12a3b45c57238ffb9ef5b75e015a9507a3cceb492f9d93b",
    ),
    (
        66,
        "325220ace06cce3cec8fb251ce4c615d67aa923a8a58bc03c7e639021de6cb28",
    ),
    (
        68,
        "db65ec21ebe007b7fe205c5f194035b476227347cb96655f0c8daafcf36ba8cd",
    ),
    (
        69,
        "36a3ff567a1715cedcb48a58976105b92eac3782388380b019ec3b6e68af14cb",
    ),
    (
        72,
        "00a4cb4a8daaf999281bed45021383119789be7281d3f8739838419d7eefe5d1",
    ),
    (
        76,
        "30b84900990ca6826bba338b3aea9b843df51b9d04ba5344be77d3c9b41bf12a",
    ),
    (
        77,
        "2da4f11faf156aa1419e6824d0ec833e946b2b87aea14c8682252d8c1ecc690d",
    ),
    (
        106,
        "818a90220156ca01098481f5445a5e4b6f003e6b0bf5e7d7632c0b1b541bac94",
    ),
    (
        113,
        "0550c3573f47d6213ae860541bfa30bff1a13f42ee30025894f39d78954dbb5c",
    ),
    (
        115,
        "29b1aca86341a892852c190e541def0bd64e3231a163433ad0f20ffcf29ad4ba",
    ),
    (
        118,
        "857975ea341ff08e70359b14ef2a8db1a2be7bb8315303fe19522149287bfa11",
    ),
    (
        121,
        "a180e12df6d234ff917e17b6df3d6cab345916f1347db8931711bf43df4831a5",
    ),
    (
        131,
        "bb19b457b632c199941bfb305fa96d98ee53e2d0a656f90f10e7116d019e0ef4",
    ),
    (
        133,
        "09fd6ad2240d8ba1f5e120b34caf9f6956ff066983a3001c7fc8c387916f4e28",
    ),
    (
        134,
        "d7fe30b14e84bc617497e20311eab5ecfdaae0cbdf925d281acf64d1b914da1c",
    ),
    (
        135,
        "9e991b0b77e8f47cafe525ced2deab1dbb8bedf2ac302bbdc79e91b496d2d44b",
    ),
    (
        136,
        "3f2c93b6193fb345188e15a4c320e7a530d4f95d130df551bdf51c5a909eb6b0",
    ),
    (
        145,
        "deb6a0a5800de2eeba4e46c494827d42b60d8f4bf67d15e1136ec5346e1abf81",
    ),
    (
        146,
        "1dc60eea579efc83e8657d870fb17266b08b99bde05b97efd35ce40ab3dccf7b",
    ),
    (
        147,
        "783371d9c8800ce3ca4b7ff5dedc074e0cc5cc20212c0b8e04694b87f789a6a5",
    ),
    (
        148,
        "22457a0ef6d783c4c6cc5ae4b7909f50907d8a3442245123e6195ec16cd41a0d",
    ),
    (
        149,
        "dc18c3a974313963bff9672f7b886036a42ed2854d8346041f794fd8ec3d417d",
    ),
    (
        150,
        "602f8633d236912a85ae38e31e3dfbaded8237f1ecd726932908421bff728d8b",
    ),
    (
        151,
        "0e5eeb28a639c80bc8b8b2bbab6a62387b890badff258dd4b4dc422fcb5d1e9a",
    ),
    (
        152,
        "a30a23a510027b59b96593fcc0260a79e702006f811b5c015531716b0a92ff8c",
    ),
    (
        155,
        "7b893c1c92c02c12bdb41f9fd64701fbf586f70c79b2d0b6d9f595f2d1a809ac",
    ),
    (
        156,
        "9d49322e12fd70a6f572976d92f04cbacfce0d700776abfb0afefc0987ce508d",
    ),
    (
        158,
        "44e944e17a9fb789ef10ba9dadb04f9cc40491b6684179e90817bb1784289ecf",
    ),
    (
        160,
        "7900c63ec067b4195cb478469b719c418945d8bc168c54ca5c4220ced53ceb2a",
    ),
    (
        161,
        "852dc862ff70e37bd9a9863910808d6732e082dc282c802ac817a09427a5a8ab",
    ),
    (
        162,
        "f6260b0ac07f022b89b9ffb3259ccc0291f69a53cbe9dc967c0ad88e655f127b",
    ),
    (
        163,
        "a4712f5cb5dec68d61dce4eeab508b17acabcc72db80de6221f42586f7484a6a",
    ),
    (
        164,
        "5cf197eeef7efee77c518403aaeba9320f9bc32bb0c0297be44e520a56609b91",
    ),
    (
        165,
        "2172fdda27e12e1e5d412729dc08a324abade8878a2aa2e26aec76e41fa31223",
    ),
];

#[must_use]
pub fn retrobution_bullet_row_proof(bullet_type: i32) -> Option<&'static str> {
    RETROBUTION_BULLET_ROW_PROOFS
        .iter()
        .find_map(|(candidate, proof)| (*candidate == bullet_type).then_some(*proof))
}
pub const RETROBUTION_TUTORIAL_SOURCE_ASSET_PROOFS: [(&str, u64, &str, u64, &str); 3] = [
    (
        "CustomAssetBundle-fa9dbcf4604f64024b06ff1d5e375918",
        33_024_451,
        "9a1918eef69dce97e99b093f2de1ea590e00c8d6051ba559c4e24b1a789ab988",
        88_850_403,
        "de91e3f957cb7eb7039544625bc04f1e9abfffa3d0fa656388c18ed2b954a23d",
    ),
    (
        "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a",
        134_712_750,
        "80a80c9bae83b2a67263ce87c033fe173cc26dd2944e3edc25f751d5c42bc678",
        394_346_191,
        "b4b05dfa6095ff8381ca8ddc865842ef74388adde132adb3a7b60a31b10befb8",
    ),
    (
        "CustomAssetBundle-bd5f53480423447d7bcaed95cb2a96c8",
        39_189_500,
        "8293c7b0cd3d193d4c464ff7eff1dd70aaa930d27f2dd5eb64cc437cbcabb073",
        203_122_345,
        "48c0121573a834c089c1e0407bfa834456e212aad5173d0018f3ab92d04e571d",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialSourceFileProof {
    pub logical_name: String,
    pub bytes: u64,
    pub blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialSourceAssetProof {
    pub asset: String,
    pub serialized_asset: TutorialSourceFileProof,
    pub object_dump: TutorialSourceFileProof,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialEffectCatalog {
    pub schema: String,
    pub source_build: String,
    pub source_bundle: TutorialSourceFileProof,
    pub source_dump: TutorialSourceFileProof,
    pub source_assets: Vec<TutorialSourceAssetProof>,
    pub renderer_status: String,
    pub effects: Vec<TutorialEffectCatalogEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialEffectCatalogEntry {
    pub effect_id: i32,
    pub container_route: String,
    pub root_asset: String,
    pub root_path_id: i64,
    pub closure_path: String,
    pub closure_bytes: u64,
    pub closure_blake3: String,
    pub object_count: u64,
    pub object_types: Vec<String>,
    pub component_types: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialProjectileCatalog {
    pub schema: String,
    pub source_build: String,
    pub source_bundle: TutorialSourceFileProof,
    pub source_dump: TutorialSourceFileProof,
    pub source_assets: Vec<TutorialSourceAssetProof>,
    pub renderer_status: String,
    pub bullet_table_route: String,
    pub bullet_table_root_path_id: i64,
    pub bullet_table_closure_path: String,
    pub bullet_table_closure_bytes: u64,
    pub bullet_table_closure_blake3: String,
    pub particle_effects: Vec<TutorialEffectCatalogEntry>,
    pub rows: Vec<TutorialBulletCatalogEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialBulletCatalogEntry {
    pub bullet_type: i32,
    pub row_path: String,
    pub row_bytes: u64,
    pub row_blake3: String,
    pub serialized_row_blake3: String,
    pub parameters: TutorialBulletParameters,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialBulletRowFile {
    pub schema: String,
    pub bullet_type: i32,
    pub source_route: String,
    pub source_root_path_id: i64,
    pub serialized_row_blake3: String,
    pub parameters: TutorialBulletParameters,
    pub serialized_row: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialBulletParameters {
    pub cancel_script: i32,
    pub fire_script: i32,
    pub particle_script: i32,
    pub success_script: i32,
    pub cancel_model_scale: f64,
    pub curve_height: f64,
    pub fire_model_scale: f64,
    pub bullet_model_scale: f64,
    pub success_model_scale: f64,
    pub hide_time_seconds: f64,
    pub maximum_time_seconds: f64,
    pub fire_link: String,
    pub success_link: String,
    pub success_sound: String,
}

impl Eq for TutorialBulletParameters {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialEffectClosureFile {
    pub schema: String,
    pub effect_id: Option<i32>,
    pub container_route: String,
    pub root_asset: String,
    pub root_path_id: i64,
    pub source_bundle_blake3: String,
    pub source_dump_blake3: String,
    pub source_assets: Vec<TutorialSourceAssetProof>,
    pub objects: Vec<TutorialUnityObjectProof>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TutorialUnityObjectProof {
    pub asset: String,
    pub path_id: i64,
    pub type_id: i64,
    pub class_id: i64,
    pub object_type: String,
    pub name: String,
    pub canonical_blake3: String,
    pub value: Value,
}
