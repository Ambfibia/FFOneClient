use super::*;

#[test]
fn rejects_incomplete_or_unrepresentable_render_state() {
    let mut model = materialized_rex();
    model.materials[0].passes.clear();
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("no resolved render passes")
    );

    let mut model = materialized_rex();
    model.materials[0].colors[0].value[0] = f64::MAX;
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("not representable as f32")
    );

    let mut model = materialized_rex();
    model.samplers[0].mip_map_bias = f64::MAX;
    model.materials[0].texture_bindings[1]
        .sampler
        .as_mut()
        .unwrap()
        .descriptor
        .mip_map_bias = f64::MAX;
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("not representable as f32")
    );

    let mut model = materialized_rex();
    model.materials[0].passes[0].blend.enabled = false;
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("disabled blend state is contradictory")
    );

    let mut model = materialized_rex();
    model.textures[0].mip_provenance.source_chain_complete = false;
    model.materials[0].texture_bindings[1].mip_provenance =
        Some(model.textures[0].mip_provenance.clone());
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("mip chain is incomplete")
    );

    let mut model = materialized_rex();
    model.textures[0].mip_provenance.published_policy = PublishedMipPolicy::BaseLevelOnly;
    model.materials[0].texture_bindings[1].mip_provenance =
        Some(model.textures[0].mip_provenance.clone());
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("discard source mip behavior")
    );

    let mut model = materialized_rex();
    model.samplers[0].wrap_t = SamplerWrapMode::ClampToEdge;
    model.materials[0].texture_bindings[1]
        .sampler
        .as_mut()
        .unwrap()
        .descriptor
        .wrap_t = SamplerWrapMode::ClampToEdge;
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("contradicts legacy provenance")
    );

    let mut model = materialized_rex();
    model.materials[0].passes[0].alpha_test = MaterialAlphaTestState::Enabled {
        compare: MaterialCompareFunction::Disabled,
        reference: MaterialAlphaReference::Literal { value: 0.9 },
    };
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("alpha test has no compare function")
    );

    let mut model = materialized_rex();
    model.materials[0].passes[0].alpha_test = MaterialAlphaTestState::Enabled {
        compare: MaterialCompareFunction::GreaterEqual,
        reference: MaterialAlphaReference::FloatProperty {
            name: "_Cutoff".into(),
            resolved_value: 0.75,
        },
    };
    assert!(
        validate(&model)
            .unwrap_err()
            .to_string()
            .contains("contradicts material property")
    );
}
