use super::*;

pub(super) fn require_legacy_height(sample: NativeTerrainLegacyInterpolatedHeightSample) -> f32 {
    match sample {
        NativeTerrainLegacyInterpolatedHeightSample::Height(height) => height,
        NativeTerrainLegacyInterpolatedHeightSample::Rejected(rejection) => {
            panic!("expected a legacy height, got {rejection:?}")
        }
    }
}
