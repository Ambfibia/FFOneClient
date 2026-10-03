use std::time::Duration;

use super::SeekError;
use crate::common::{ChannelCount, SampleRate};
use crate::source::ChannelVolume;
use crate::{Float, Source};

/// A simple spatial audio source. The underlying source is transformed to Mono
/// and then played in stereo. The left and right channel's volume are amplified
/// differently depending on the distance of the left and right ear to the source.
#[derive(Clone)]
pub struct Spatial<I>
where
    I: Source,
{
    input: ChannelVolume<I>,
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
}

impl<I> Spatial<I>
where
    I: Source,
{
    /// Builds a new `SpatialPlayer`, beginning playback on a stream.
    pub fn new(
        input: I,
        emitter_position: [f32; 3],
        left_ear: [f32; 3],
        right_ear: [f32; 3],
    ) -> Spatial<I>
    where
        I: Source,
    {
        let mut ret = Spatial {
            input: ChannelVolume::new(input, vec![0.0, 0.0]),
        };
        ret.set_positions(emitter_position, left_ear, right_ear);
        ret
    }

    /// Sets the position of the emitter and ears in the 3D world.
    pub fn set_positions(
        &mut self,
        emitter_pos: [f32; 3],
        left_ear: [f32; 3],
        right_ear: [f32; 3],
    ) {
        debug_assert!(left_ear != right_ear);
        let left_dist_sq = dist_sq(left_ear, emitter_pos);
        let right_dist_sq = dist_sq(right_ear, emitter_pos);
        let max_diff = dist_sq(left_ear, right_ear).sqrt();
        let left_dist = left_dist_sq.sqrt();
        let right_dist = right_dist_sq.sqrt();
        let left_diff_modifier = (((right_dist - left_dist) / max_diff + 1.0) / 4.0 + 0.5).min(1.0);
        let right_diff_modifier =
            (((left_dist - right_dist) / max_diff + 1.0) / 4.0 + 0.5).min(1.0);
        let left_dist_modifier = (1.0 / left_dist_sq).min(1.0);
        let right_dist_modifier = (1.0 / right_dist_sq).min(1.0);
        self.input
            .set_volume(0, (left_diff_modifier * left_dist_modifier) as Float);
        self.input
            .set_volume(1, (right_diff_modifier * right_dist_modifier) as Float);
    }
}

impl<I> Iterator for Spatial<I>
where
    I: Source,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        self.input.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I> ExactSizeIterator for Spatial<I> where I: Source + ExactSizeIterator {}

impl<I> Source for Spatial<I>
where
    I: Source,
{
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        self.input.current_span_len()
    }

    #[inline]
    fn channels(&self) -> ChannelCount {
        self.input.channels()
    }

    #[inline]
    fn sample_rate(&self) -> SampleRate {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.input.try_seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::SamplesBuffer;

    fn stereo(emitter: [f32; 3], left: [f32; 3], right: [f32; 3]) -> [f32; 2] {
        let mut source = Spatial::new(
            SamplesBuffer::new(1.try_into().unwrap(), 48000.try_into().unwrap(), vec![1.0_f32; 4]),
            emitter,
            left,
            right,
        );
        [source.next().unwrap(), source.next().unwrap()]
    }

    #[test]
    fn screen_side_matches_output_channel_at_near_and_far_distances() {
        // The gameplay listener's 0.2 ear gap with spatial scale 0.1.
        // Rotate the camera's right/forward vectors through all four headings.
        for (right, forward) in [
            ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
            ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
            ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0]),
        ] {
            let origin = [2.0, 1.0, -3.0];
            let left_ear = std::array::from_fn(|i| origin[i] - right[i] * 0.01);
            let right_ear = std::array::from_fn(|i| origin[i] + right[i] * 0.01);
            for distance in [0.1, 0.5, 1.0, 3.0, 10.0] {
                for side in [-1.0, 1.0] {
                    let emitter = std::array::from_fn(|i| {
                        origin[i] + distance * (right[i] * side + forward[i])
                    });
                    let [left, right] = stereo(emitter, left_ear, right_ear);
                    assert!(left.is_finite() && right.is_finite());
                    assert!(
                        (right - left) * side > 0.0,
                        "side={side} distance={distance}: {left}, {right}"
                    );
                }
                let centered = std::array::from_fn(|i| origin[i] + distance * forward[i]);
                let [left, right] = stereo(centered, left_ear, right_ear);
                assert!((left - right).abs() < 0.00001);
            }
        }
    }

    #[test]
    fn moving_emitter_changes_pan_without_restarting_source() {
        let left = [-0.01, 0.0, 0.0];
        let right = [0.01, 0.0, 0.0];
        let mut source = Spatial::new(
            SamplesBuffer::new(1.try_into().unwrap(), 48000.try_into().unwrap(), vec![1.0_f32; 8]),
            [-0.5, 0.0, 0.0],
            left,
            right,
        );
        assert!(source.next().unwrap() > source.next().unwrap());
        source.set_positions([0.5, 0.0, 0.0], left, right);
        assert!(source.next().unwrap() < source.next().unwrap());
    }

    #[test]
    fn centered_distance_attenuation_is_preserved() {
        let left = [-0.01, 0.0, 0.0];
        let right = [0.01, 0.0, 0.0];
        for distance in [0.0_f32, 0.5, 2.0, 10.0] {
            let channels = stereo([0.0, 0.0, -distance], left, right);
            let expected = 0.75 * (1.0 / (distance * distance + 0.0001)).min(1.0);
            for channel in channels {
                assert!((channel - expected).abs() < 0.00001);
            }
        }
    }
}
