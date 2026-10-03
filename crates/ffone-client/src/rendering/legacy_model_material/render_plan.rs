//! Blend/cull/write/alpha state and the per-shader multi-pass render plan.

use super::shader_kind::LegacyShaderKind;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyBlendMode {
    Replace,
    SrcAlphaOneMinusSrcAlpha,
    SrcAlphaOneMinusSrcColor,
    SrcColorOneMinusSrcAlpha,
    SrcAlphaZero,
    SrcAlphaOne,
    OneMinusSrcAlphaOne,
    OneMinusDstColorOne,
    OneOne,
}

impl LegacyBlendMode {
    pub(super) const fn uses_additive_fog_color(self) -> bool {
        matches!(
            self,
            Self::SrcAlphaOne
                | Self::OneMinusSrcAlphaOne
                | Self::OneMinusDstColorOne
                | Self::OneOne
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyCullMode {
    Off,
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyColorWriteMask {
    Rgb,
    Rgba,
}

/// Exact source state. `source_queue` is metadata for deterministic scene
/// ordering because Bevy's `Material` API has no Unity-style integer queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegacyRenderMode {
    pub blend: LegacyBlendMode,
    pub cull: LegacyCullMode,
    pub depth_write: bool,
    pub color_write: LegacyColorWriteMask,
    pub alpha_cutout: bool,
    pub source_queue: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyPassKind {
    Surface,
    CutoutDepth,
    TransparentColor,
    Outline,
}

/// The two audited alpha-test commands are deliberately distinct. Unity's
/// `Greater` rejects equality while `GEqual` accepts it, and the cutout glass
/// uses a literal 0.9 rather than its saved `_Cutoff` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyAlphaTest {
    Disabled,
    GreaterMaterialCutoff,
    GreaterLiteralOneHundredth,
    GreaterEqualLiteralNineTenths,
}

impl LegacyAlphaTest {
    pub(super) const fn enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    pub(super) const fn cutoff(self, material_cutoff: f32) -> f32 {
        match self {
            Self::Disabled => 0.0,
            Self::GreaterMaterialCutoff => material_cutoff,
            Self::GreaterLiteralOneHundredth => 0.01,
            Self::GreaterEqualLiteralNineTenths => 0.9,
        }
    }

    pub(super) const fn rejects_equality(self) -> bool {
        matches!(
            self,
            Self::GreaterMaterialCutoff | Self::GreaterLiteralOneHundredth
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegacyPassPlan {
    pub kind: LegacyPassKind,
    pub render_mode: LegacyRenderMode,
    pub alpha_test: LegacyAlphaTest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyModelRenderPlan {
    pub passes: Vec<LegacyPassPlan>,
}

impl LegacyModelRenderPlan {
    pub fn for_shader(shader: LegacyShaderKind) -> Self {
        let toon_surface = LegacyRenderMode {
            blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
            cull: if shader == LegacyShaderKind::SkinnedToonCullOffCategory {
                LegacyCullMode::Off
            } else {
                LegacyCullMode::Back
            },
            depth_write: true,
            color_write: LegacyColorWriteMask::Rgba,
            alpha_cutout: false,
            source_queue: 2900,
        };
        let passes = match shader {
            LegacyShaderKind::SkinnedToon
            | LegacyShaderKind::SkinnedToonCullOffFallback
            | LegacyShaderKind::SkinnedToonCullOffCategory
            | LegacyShaderKind::SkinnedToonRim
            | LegacyShaderKind::SkinnedToonRimTransparent
            | LegacyShaderKind::SkinnedToonRimMatcap
            | LegacyShaderKind::SkinnedToonFlipped
            | LegacyShaderKind::RimEmissiveColoredTransparent
            | LegacyShaderKind::Toon => vec![
                LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        source_queue: if shader == LegacyShaderKind::RimEmissiveColoredTransparent {
                            3000
                        } else if shader == LegacyShaderKind::SkinnedToonRimTransparent {
                            3003
                        } else {
                            toon_surface.source_queue
                        },
                        ..toon_surface
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                },
                LegacyPassPlan {
                    kind: LegacyPassKind::Outline,
                    render_mode: LegacyRenderMode {
                        cull: if shader == LegacyShaderKind::SkinnedToonFlipped {
                            LegacyCullMode::Back
                        } else {
                            LegacyCullMode::Front
                        },
                        source_queue: if shader == LegacyShaderKind::RimEmissiveColoredTransparent {
                            3000
                        } else if shader == LegacyShaderKind::SkinnedToonRimTransparent {
                            3003
                        } else {
                            toon_surface.source_queue
                        },
                        ..toon_surface
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                },
            ],
            LegacyShaderKind::OpaqueNormal => vec![LegacyPassPlan {
                kind: LegacyPassKind::Surface,
                render_mode: LegacyRenderMode {
                    blend: LegacyBlendMode::Replace,
                    cull: LegacyCullMode::Back,
                    depth_write: true,
                    color_write: LegacyColorWriteMask::Rgba,
                    alpha_cutout: false,
                    source_queue: 2000,
                },
                alpha_test: LegacyAlphaTest::Disabled,
            }],
            LegacyShaderKind::AlphaBlendNormal
            | LegacyShaderKind::AlphaBlendNormalVertexColorAd
            | LegacyShaderKind::AlphaBlendNormalCullOff => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                        cull: if shader == LegacyShaderKind::AlphaBlendNormalCullOff {
                            LegacyCullMode::Off
                        } else {
                            LegacyCullMode::Back
                        },
                        depth_write: true,
                        color_write: LegacyColorWriteMask::Rgba,
                        alpha_cutout: false,
                        source_queue: 3000,
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                }]
            }
            LegacyShaderKind::AlphaBlendNormalGlow => vec![LegacyPassPlan {
                kind: LegacyPassKind::Surface,
                render_mode: LegacyRenderMode {
                    blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                    cull: LegacyCullMode::Back,
                    depth_write: true,
                    color_write: LegacyColorWriteMask::Rgb,
                    alpha_cutout: false,
                    source_queue: 3000,
                },
                alpha_test: LegacyAlphaTest::Disabled,
            }],
            LegacyShaderKind::SkinDirectionalAlphaBlend => vec![LegacyPassPlan {
                kind: LegacyPassKind::Surface,
                render_mode: LegacyRenderMode {
                    blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                    cull: LegacyCullMode::Back,
                    depth_write: true,
                    color_write: LegacyColorWriteMask::Rgba,
                    alpha_cutout: false,
                    source_queue: 2900,
                },
                alpha_test: LegacyAlphaTest::Disabled,
            }],
            LegacyShaderKind::TransparentNormal | LegacyShaderKind::TransparentNormalCullOff => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                        cull: if shader == LegacyShaderKind::TransparentNormalCullOff {
                            LegacyCullMode::Off
                        } else {
                            LegacyCullMode::Back
                        },
                        depth_write: false,
                        color_write: if shader == LegacyShaderKind::TransparentNormalCullOff {
                            LegacyColorWriteMask::Rgb
                        } else {
                            LegacyColorWriteMask::Rgba
                        },
                        alpha_cutout: false,
                        source_queue: 3010,
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                }]
            }
            LegacyShaderKind::SrcAlphaZeroBackfaceDepthWrite => vec![LegacyPassPlan {
                kind: LegacyPassKind::Surface,
                render_mode: LegacyRenderMode {
                    blend: LegacyBlendMode::SrcAlphaZero,
                    cull: LegacyCullMode::Back,
                    depth_write: true,
                    color_write: LegacyColorWriteMask::Rgba,
                    alpha_cutout: false,
                    source_queue: 3000,
                },
                alpha_test: LegacyAlphaTest::Disabled,
            }],
            LegacyShaderKind::SrcAlphaAdditiveBackface
            | LegacyShaderKind::SrcAlphaAdditiveBackfaceDepthWrite
            | LegacyShaderKind::SrcAlphaAdditiveTwoSided
            | LegacyShaderKind::SrcAlphaAdditiveTwoSidedVertexColorAd => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: LegacyBlendMode::SrcAlphaOne,
                        cull: if matches!(
                            shader,
                            LegacyShaderKind::SrcAlphaAdditiveTwoSided
                                | LegacyShaderKind::SrcAlphaAdditiveTwoSidedVertexColorAd
                        ) {
                            LegacyCullMode::Off
                        } else {
                            LegacyCullMode::Back
                        },
                        depth_write: shader == LegacyShaderKind::SrcAlphaAdditiveBackfaceDepthWrite,
                        color_write: LegacyColorWriteMask::Rgb,
                        alpha_cutout: false,
                        source_queue: if shader
                            == LegacyShaderKind::SrcAlphaAdditiveBackfaceDepthWrite
                        {
                            3000
                        } else {
                            3010
                        },
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                }]
            }
            LegacyShaderKind::ParticleSrcAlphaAdditiveTwoSided => vec![LegacyPassPlan {
                kind: LegacyPassKind::Surface,
                render_mode: LegacyRenderMode {
                    blend: LegacyBlendMode::SrcAlphaOne,
                    cull: LegacyCullMode::Off,
                    depth_write: false,
                    color_write: LegacyColorWriteMask::Rgb,
                    alpha_cutout: true,
                    source_queue: 3003,
                },
                alpha_test: LegacyAlphaTest::GreaterLiteralOneHundredth,
            }],
            LegacyShaderKind::ParticleOneMinusSrcAlphaAdditiveTwoSided
            | LegacyShaderKind::ParticleOneMinusDstColorAdditiveTwoSided
            | LegacyShaderKind::ParticleOneMinusDstColorAdditiveBackface => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: if shader
                            == LegacyShaderKind::ParticleOneMinusSrcAlphaAdditiveTwoSided
                        {
                            LegacyBlendMode::OneMinusSrcAlphaOne
                        } else {
                            LegacyBlendMode::OneMinusDstColorOne
                        },
                        cull: if shader
                            == LegacyShaderKind::ParticleOneMinusDstColorAdditiveBackface
                        {
                            LegacyCullMode::Back
                        } else {
                            LegacyCullMode::Off
                        },
                        depth_write: false,
                        color_write: LegacyColorWriteMask::Rgb,
                        alpha_cutout: true,
                        source_queue: 3003,
                    },
                    alpha_test: LegacyAlphaTest::GreaterLiteralOneHundredth,
                }]
            }
            LegacyShaderKind::RotatingFlipbook | LegacyShaderKind::ScrollDistortAdditive => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: if shader == LegacyShaderKind::RotatingFlipbook {
                            LegacyBlendMode::SrcAlphaOneMinusSrcAlpha
                        } else {
                            LegacyBlendMode::SrcAlphaOne
                        },
                        cull: LegacyCullMode::Off,
                        depth_write: false,
                        color_write: LegacyColorWriteMask::Rgba,
                        alpha_cutout: false,
                        source_queue: if shader == LegacyShaderKind::RotatingFlipbook {
                            3003
                        } else {
                            3000
                        },
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                }]
            }
            LegacyShaderKind::HologramSolidAdditive => vec![
                LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                        cull: LegacyCullMode::Off,
                        depth_write: true,
                        color_write: LegacyColorWriteMask::Rgba,
                        alpha_cutout: false,
                        source_queue: 2900,
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                },
                LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: LegacyBlendMode::SrcAlphaOne,
                        cull: LegacyCullMode::Off,
                        depth_write: true,
                        color_write: LegacyColorWriteMask::Rgba,
                        alpha_cutout: false,
                        source_queue: 2900,
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                },
            ],
            LegacyShaderKind::AdditiveTwoSided
            | LegacyShaderKind::AdditiveTestTwoSidedQueue3011
            | LegacyShaderKind::SrcAlphaAdditiveTestTwoSided => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: if shader == LegacyShaderKind::SrcAlphaAdditiveTestTwoSided {
                            LegacyBlendMode::SrcAlphaOne
                        } else {
                            LegacyBlendMode::OneOne
                        },
                        cull: LegacyCullMode::Off,
                        depth_write: shader == LegacyShaderKind::SrcAlphaAdditiveTestTwoSided,
                        color_write: LegacyColorWriteMask::Rgb,
                        // Exact ShaderLab: `Alphatest Greater [_Cutoff]`. The
                        // Fusion eye material resolves `_Cutoff` to 0.0, but the
                        // command must still survive alongside additive blending.
                        alpha_cutout: true,
                        source_queue: if shader == LegacyShaderKind::AdditiveTestTwoSidedQueue3011 {
                            3011
                        } else {
                            3000
                        },
                    },
                    alpha_test: LegacyAlphaTest::GreaterMaterialCutoff,
                }]
            }
            LegacyShaderKind::AdditiveOneOneTwoSided
            | LegacyShaderKind::AdditiveOneOneTwoSidedVertexColorAd
            | LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite
            | LegacyShaderKind::AdditiveOneOneBackface
            | LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: LegacyBlendMode::OneOne,
                        cull: if matches!(
                            shader,
                            LegacyShaderKind::AdditiveOneOneBackface
                                | LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite
                        ) {
                            LegacyCullMode::Back
                        } else {
                            LegacyCullMode::Off
                        },
                        depth_write: matches!(
                            shader,
                            LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite
                                | LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite
                        ),
                        color_write: LegacyColorWriteMask::Rgb,
                        alpha_cutout: false,
                        source_queue: if shader
                            == LegacyShaderKind::AdditiveOneOneTwoSidedDepthWrite
                            || shader == LegacyShaderKind::AdditiveOneOneBackfaceDepthWrite
                        {
                            3000
                        } else {
                            3011
                        },
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                }]
            }
            LegacyShaderKind::TransparentCutoutDefaultCulling
            | LegacyShaderKind::TransparentCutoutTwoSided => {
                let cull = if shader == LegacyShaderKind::TransparentCutoutTwoSided {
                    LegacyCullMode::Off
                } else {
                    LegacyCullMode::Back
                };
                vec![
                    LegacyPassPlan {
                        kind: LegacyPassKind::CutoutDepth,
                        render_mode: LegacyRenderMode {
                            blend: LegacyBlendMode::Replace,
                            cull,
                            depth_write: true,
                            color_write: LegacyColorWriteMask::Rgba,
                            alpha_cutout: true,
                            source_queue: 2900,
                        },
                        alpha_test: LegacyAlphaTest::GreaterEqualLiteralNineTenths,
                    },
                    LegacyPassPlan {
                        kind: LegacyPassKind::TransparentColor,
                        render_mode: LegacyRenderMode {
                            blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                            cull,
                            depth_write: false,
                            color_write: LegacyColorWriteMask::Rgb,
                            alpha_cutout: false,
                            source_queue: 2900,
                        },
                        alpha_test: LegacyAlphaTest::Disabled,
                    },
                ]
            }
            LegacyShaderKind::TransparentCutoutZWriteOffDefaultCulling => {
                vec![LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: LegacyRenderMode {
                        blend: LegacyBlendMode::Replace,
                        cull: LegacyCullMode::Back,
                        depth_write: false,
                        color_write: LegacyColorWriteMask::Rgba,
                        alpha_cutout: true,
                        source_queue: 2900,
                    },
                    alpha_test: LegacyAlphaTest::GreaterMaterialCutoff,
                }]
            }
            LegacyShaderKind::FusionEffect => vec![LegacyPassPlan {
                kind: LegacyPassKind::Surface,
                render_mode: LegacyRenderMode {
                    color_write: LegacyColorWriteMask::Rgb,
                    ..toon_surface
                },
                alpha_test: LegacyAlphaTest::Disabled,
            }],
            LegacyShaderKind::FusionMatterLightDir => vec![
                LegacyPassPlan {
                    kind: LegacyPassKind::Surface,
                    render_mode: toon_surface,
                    alpha_test: LegacyAlphaTest::Disabled,
                },
                LegacyPassPlan {
                    kind: LegacyPassKind::Outline,
                    render_mode: LegacyRenderMode {
                        cull: LegacyCullMode::Front,
                        ..toon_surface
                    },
                    alpha_test: LegacyAlphaTest::Disabled,
                },
            ],
            LegacyShaderKind::DiffuseFade => vec![LegacyPassPlan {
                kind: LegacyPassKind::Surface,
                render_mode: LegacyRenderMode {
                    blend: LegacyBlendMode::SrcAlphaOneMinusSrcAlpha,
                    cull: LegacyCullMode::Back,
                    depth_write: true,
                    color_write: LegacyColorWriteMask::Rgb,
                    alpha_cutout: false,
                    source_queue: 2900,
                },
                alpha_test: LegacyAlphaTest::Disabled,
            }],
        };
        Self { passes }
    }

    pub fn outline(&self) -> Option<LegacyPassPlan> {
        self.passes
            .iter()
            .copied()
            .find(|pass| pass.kind == LegacyPassKind::Outline)
    }
}
