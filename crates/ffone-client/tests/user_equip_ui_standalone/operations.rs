use super::*;

pub(super) fn semantic_primary_nano_slug(number: u8) -> &'static str {
    match number {
        0 => "samurai-jack",
        1 => "eduardo",
        2 => "mojo-jojo",
        3 => "numbuh-five",
        4 => "mandark",
        5 => "fourarms",
        6 => "billy",
        7 => "blossom",
        8 => "dexter",
        9 => "ed",
        10 => "grim",
        11 => "juniper-lee",
        12 => "mandy",
        13 => "aku",
        14 => "bloo",
        15 => "coco",
        16 => "courage",
        17 => "dee-dee",
        18 => "demongo",
        19 => "edd",
        20 => "eddy",
        21 => "humongosaur",
        22 => "hex",
        23 => "him",
        24 => "mac",
        25 => "megas",
        26 => "numbuh-one",
        27 => "numbuh-four",
        28 => "numbuh-two",
        29 => "numbuh-three",
        30 => "swampfire",
        31 => "prof-utonium",
        32 => "vilgax",
        33 => "wilt",
        34 => "buttercup",
        35 => "bubbles",
        36 => "missing",
        37 => "ben-10",
        38 => "johnny-bravo",
        39 => "cheese",
        40 | 46 => "finn",
        41 => "flapjack",
        42 => "holo-nano",
        43 => "belladonna",
        44 => "computress",
        45 => "runty",
        47 => "coop",
        48 => "panini",
        49 => "jack-olantern",
        _ => panic!("missing semantic primary Nano slug {number}"),
    }
}

pub(super) fn placeholders(value: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut remainder = value;
    while let Some(open) = remainder.find('{') {
        remainder = &remainder[open + 1..];
        let close = remainder
            .find('}')
            .unwrap_or_else(|| panic!("unclosed localization placeholder in {value:?}"));
        let name = &remainder[..close];
        assert!(
            !name.is_empty(),
            "empty localization placeholder in {value:?}"
        );
        result.insert(name.to_owned());
        remainder = &remainder[close + 1..];
    }
    result
}

pub(super) fn sha256_lower(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn merged_named_nano_icons_are_exact_recorded_donor_publications() {
    assert_eq!(ALTERNATE_ICONS_ARCHIVE_BYTES, 6_054_910);
    assert_eq!(
        ALTERNATE_ICONS_ARCHIVE_SHA256,
        "b3995181bc9e917d0bf2b35e38361e0e18eed6f14c95aab52eb5cfd81f056cea"
    );
    let asset_root = workspace_root().join("assets/game/icons/entities/nanos");
    for (icon_number, path_id, png_bytes, png_sha256) in EXTENDED_NANO_ICONS {
        let Some(slug) = (match icon_number {
            39 => Some("cheese"),
            46 => Some("jake"),
            50 => Some("titan"),
            51 => Some("gumball"),
            52 => Some("chowder"),
            53 => Some("zak-saturday"),
            54 => Some("rath"),
            57 => Some("van-kleiss"),
            58 => Some("mordecai"),
            59 => Some("darwin"),
            60 => Some("p-bubblegum"),
            _ => None,
        }) else {
            continue;
        };
        let path = asset_root.join(format!("nanoicon_{slug}.png"));
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "read alternate PathID {path_id} {}: {error}",
                path.display()
            )
        });
        assert_eq!(bytes.len() as u64, png_bytes, "PathID {path_id}");
        assert_eq!(sha256_lower(&bytes), png_sha256, "PathID {path_id}");
    }
    let rex = fs::read(asset_root.join("nanoicon_rex.png")).unwrap();
    assert_eq!(rex.len(), 5_115, "Academy Rex PathID 289558199");
    assert_eq!(
        sha256_lower(&rex),
        "bf8417ff8cf512c2e8846b7233ebf3bca230fc4c880319ce75943bae25c07bc4",
        "Academy Rex PathID 289558199"
    );
}

#[test]
fn retrobution_overrides_use_semantic_names_and_exact_primary_bytes() {
    let nano_root = workspace_root().join("assets/game/icons/entities/nanos");
    for (slug, path_id, bytes, sha256) in [
        (
            "ben-10",
            24_i64,
            3_651_u64,
            "08e64ee4b5cd251364a2acc30ef5d73dfa8af6aa1e2311c439215e54185a3602",
        ),
        (
            "johnny-bravo",
            2_640,
            5_444,
            "cd52e6761ebe1d709155be2c456ac75d04965d4c5451df0b3c6f96f063004135",
        ),
        (
            "flapjack",
            2_774,
            5_652,
            "fbc70777c4f1f54eebda9e2bc8339d1ab02c6fce8c9ff7980a963a7db3030972",
        ),
        (
            "finn",
            2_868,
            5_782,
            "37aeaf6085aaf827ece7cc92fc94e23fafb60785a281af5c925018ea997cfb5f",
        ),
    ] {
        let path = nano_root.join(format!("nanoicon_{slug}.png"));
        let payload = fs::read(&path)
            .unwrap_or_else(|error| panic!("read primary PathID {path_id}: {error}"));
        assert_eq!(payload.len() as u64, bytes, "primary PathID {path_id}");
        assert_eq!(sha256_lower(&payload), sha256, "primary PathID {path_id}");
    }
}

#[test]
fn nano_ready_gallery_is_exact_primary_icons_publication() {
    assert_eq!(PRIMARY_ICONS_ARCHIVE_BYTES, 5_800_411);
    assert_eq!(
        PRIMARY_ICONS_ARCHIVE_SHA256,
        "a05602d6e96e2e74ecad207f42e519605259434210de8da19e422b30d642e544"
    );
    assert_eq!(PRIMARY_NANO_READY_TEXTURES.len(), 37);
    assert_eq!(
        PRIMARY_NANO_READY_TEXTURES
            .iter()
            .map(|(path_id, _, _, _)| *path_id)
            .collect::<BTreeSet<_>>()
            .len(),
        37
    );
    let asset_root = workspace_root().join("assets/game/icons/entities/nanos/ready");
    for (path_id, number, png_bytes, png_sha256) in PRIMARY_NANO_READY_TEXTURES {
        let slug = semantic_primary_nano_slug(number);
        let path = asset_root.join(format!("nanoready_{slug}.png"));
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("read PathID {path_id} {}: {error}", path.display()));
        assert_eq!(bytes.len() as u64, png_bytes, "PathID {path_id}");
        assert_eq!(sha256_lower(&bytes), png_sha256, "PathID {path_id}");
        assert_eq!(
            image::image_dimensions(&path).unwrap(),
            (64, 64),
            "PathID {path_id}"
        );
    }
}

#[test]
fn valid_item_icons_are_not_misreported_while_loading() {
    assert!(USER_EQUIP_SOURCE.contains("LoadState::Failed(_)"));
    assert!(USER_EQUIP_SOURCE.contains("NotLoaded`/`Loading` as a catalog"));
    assert!(USER_EQUIP_SOURCE.contains("UserEquipPresentationIcon::MissingChecker"));
}
