use super::*;

#[test]
fn semantic_assets_match_clean_export_hashes_and_dimensions() {
    let assets: &[(&str, &[u8], &str, u32, u32)] = &[
        (
            "panel.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/panel.png"),
            "8f240d561955f4e9246790b7dc0fcaa1b1e79ff455075d086298b611f0d66441",
            584,
            650,
        ),
        (
            "black-shade.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/black-shade.png"),
            "e086474f16f17a51652240d0c4fe50087370f16a2962af83c211074354c85bdc",
            1024,
            762,
        ),
        (
            "look-item-bg.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/look-item-bg.png"),
            "cdd8c717ad3369a2177a54a002be98eaa45fe258738dfa7adb8b3dd505da73d9",
            305,
            133,
        ),
        (
            "look-error.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/look-error.png"),
            "0713408abb2adb16b4840380a2ca657a8d27ff2a1e458189c3838d4813414e5a",
            303,
            131,
        ),
        (
            "stat-item-bg.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/stat-item-bg.png"),
            "273bff6061c921dc4ad4e00d03ef7e43f3d156cd5b439c87dab08ed2567fc588",
            305,
            230,
        ),
        (
            "stat-error.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/stat-error.png"),
            "94f059224b5932c2f33cfae5a348dd7efbb4f883654a1463c793a80d99dae117",
            304,
            228,
        ),
        (
            "combined.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/combined.png"),
            "1050e1efd1afe851aab9784a54ab524a771786cec96c9dd6611fd66c3c66394c",
            25,
            25,
        ),
        (
            "taros-icon.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/taros-icon.png"),
            "fa45d3ff5b8a5452fbea5b12004446946c65ea45360bb8c09503162bb63027e4",
            23,
            24,
        ),
        (
            "success.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/success.png"),
            "1f8cfec2aae41b45bb32171614a547d30447979a9a40cb9e1c256a3ad5e515f5",
            354,
            552,
        ),
        (
            "npc-icon.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/npc-icon.png"),
            "89ba1fd265689f807a95c83f5a213bc3fbfef7b3fe9f17826085ed89bb6b3e46",
            64,
            63,
        ),
        (
            "waiting.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/waiting.png"),
            "63f90f06c7b45171b7ceff9fc598d3538fbfd5dee0c460b8997c9e12bb9673d3",
            357,
            384,
        ),
        (
            "button-normal.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/button-normal.png"),
            "6a841912fb35eb3c6eeceaf24176ecd3e1258157f96ed59b481b83f7c022bef3",
            20,
            25,
        ),
        (
            "button-hover.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/button-hover.png"),
            "e23a8bb2dbb1a90e785a21f776693e94beb6ae542d55996231172f4ef3dd8c73",
            20,
            25,
        ),
        (
            "restricted-item-frame.png",
            include_bytes!("../../../../../../assets/game/ui/en/combi/restricted-item-frame.png"),
            "e37646994f0fcddf24357f04aa0feaa01c5ac57c8e9a9083c1d6b915cc325835",
            62,
            62,
        ),
    ];
    for (name, bytes, expected_hash, width, height) in assets {
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            *expected_hash,
            "{name}"
        );
        assert_eq!(png_dimensions(bytes), (*width, *height), "{name}");
    }
}

#[test]
fn dynamic_and_external_copy_is_template_and_argument_driven() {
    let passthrough = combi_passthrough_text("Server-authored item name");
    assert_eq!(passthrough.key, "ui.content.passthrough");
    assert_eq!(passthrough.fallback, "{text}");
    assert_eq!(passthrough.args["text"], "Server-authored item name");

    let cost = combi_cost_text(12_345);
    assert_eq!(cost.key, "ui.combi.cost");
    assert_eq!(cost.fallback, "{cost}");
    assert_eq!(cost.args["cost"], "12345");

    let level = combi_item_level_text(36);
    assert_eq!(level.fallback, "LEVEL {level}");
    assert_eq!(level.args["level"], "36");

    let wire_error = localized_combi_wire_error(42);
    assert_eq!(
        wire_error.fallback,
        "Item Combination error. ({error_code})"
    );
    assert_eq!(wire_error.args["error_code"], "42");
    assert_eq!(
        combi_fallback_text(&wire_error),
        "Item Combination error. (42)"
    );

    for (modal, expected_key) in [
        (
            CombiSystemModal0104::AttemptConfirmation,
            "ui.combi.modal.attempt_confirmation",
        ),
        (
            CombiSystemModal0104::CombinationFailed,
            "ui.combi.modal.combination_failed",
        ),
        (
            CombiSystemModal0104::NotEnoughTaros,
            "ui.combi.modal.not_enough_taros",
        ),
    ] {
        let localized = modal.localized_text();
        assert_eq!(localized.key, expected_key);
        assert_eq!(localized.fallback, modal.exact_text());
    }
    assert_eq!(
        localized_combi_equipped_item_message().key,
        "ui.combi.modal.equipped_item"
    );
    assert_eq!(
        CombiLookError0104::GenderMismatch
            .localized_text()
            .unwrap()
            .key,
        "ui.combi.error.look.gender_mismatch"
    );
    assert_eq!(
        CombiStatsError0104::StyleTypeMismatch
            .localized_text()
            .unwrap()
            .key,
        "ui.combi.error.stats.style_type_mismatch"
    );
    assert_eq!(combi_range_text("Short").key, "ui.combi.range.short");
    assert_eq!(combi_range_text("12.50").key, "ui.content.passthrough");
    assert_eq!(combi_type_text(0).unwrap().key, "ui.combi.type.weapon");
}
