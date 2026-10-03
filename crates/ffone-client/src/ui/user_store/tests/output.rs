use super::*;

#[test]
fn dynamic_user_store_copy_is_template_and_argument_driven() {
    let passthrough = user_store_passthrough_text("Player item");
    assert_eq!(passthrough.key, "ui.content.passthrough");
    assert_eq!(passthrough.fallback, "{text}");
    assert_eq!(passthrough.args["text"], "Player item");

    let level = user_store_level_text(12);
    assert_eq!(level.key, "ui.user_store.item.level");
    assert_eq!(level.fallback, "LEVEL {level}");
    assert_eq!(level.args["level"], "12");

    let price = user_store_taros_text("12,500");
    assert_eq!(price.key, "ui.user_store.price_taros");
    assert_eq!(price.fallback, "{amount} TAROS");
    assert_eq!(price.args["amount"], "12,500");

    let error = user_store_error_text(13);
    assert_eq!(error.key, "ui.user_store.error");
    assert_eq!(error.fallback, "STORE ERROR {code}");
    assert_eq!(error.args["code"], "13");
    assert_eq!(user_store_resolved_fallback(&error), "STORE ERROR 13");

    let cost = user_store_popup_cost_text(18);
    assert_eq!(cost.key, "ui.user_store.popup.cost_level");
    assert_eq!(cost.fallback, "COST {level}");
    assert_eq!(cost.args["level"], "18");

    let value = user_store_popup_value_text(999);
    assert_eq!(value.key, "ui.user_store.popup.value");
    assert_eq!(value.fallback, "{value}");
    assert_eq!(value.args["value"], "999");
}

#[test]
fn production_bundles_publish_every_gum_popup_key_with_exact_placeholder_parity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut bundles = BTreeMap::new();
    for locale in ["en", "ru"] {
        let runtime = fs::read(
            root.join("assets/game/localization")
                .join(format!("{locale}.json")),
        )
        .expect("read runtime localization bundle");
        let value: serde_json::Value =
            serde_json::from_slice(&runtime).expect("parse production localization bundle");
        let entries = value["entries"]
            .as_object()
            .expect("production bundle entries")
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    value
                        .as_str()
                        .expect("string localization value")
                        .to_owned(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        bundles.insert(locale, entries);
    }
    assert_eq!(
        bundles["en"].keys().collect::<BTreeSet<_>>(),
        bundles["ru"].keys().collect::<BTreeSet<_>>(),
        "production EN/RU key sets differ"
    );
    let exact = [
        (
            "ui.user_store.popup.cost_level",
            "COST {level}",
            "УРОВЕНЬ {level}",
        ),
        ("ui.user_store.popup.amount", "Amount", "Количество"),
        (
            "ui.user_store.popup.amount_taros",
            "Amount Taros",
            "Сумма в таро",
        ),
        ("ui.user_store.popup.value", "{value}", "{value}"),
        ("ui.user_store.popup.digit", "{digit}", "{digit}"),
        ("ui.user_store.popup.clear", "C", "С"),
        ("ui.user_store.popup.add", "ADD", "ДОБАВИТЬ"),
        ("ui.user_store.popup.taros", "Taros", "Таро"),
        (
            "ui.user_store.popup.remove",
            "REMOVE FROM STORE",
            "УБРАТЬ ИЗ МАГАЗИНА",
        ),
        ("ui.user_store.popup.buy", "BUY", "КУПИТЬ"),
    ];
    for (key, en, ru) in exact {
        assert_eq!(bundles["en"].get(key).map(String::as_str), Some(en));
        assert_eq!(bundles["ru"].get(key).map(String::as_str), Some(ru));
        assert_eq!(
            user_store_template_args(en),
            user_store_template_args(ru),
            "placeholder mismatch for {key}"
        );
    }
}
