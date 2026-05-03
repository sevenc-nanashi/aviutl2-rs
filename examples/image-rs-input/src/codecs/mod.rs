use ordered_float::OrderedFloat;

pub mod apng;
pub mod gif;
pub mod jpeg_xl;
pub mod webp;

#[derive(Debug, Clone)]
pub struct AnimationInfo {
    pub width: u32,
    pub height: u32,
    pub frame_timings: std::collections::BTreeMap<OrderedFloat<f32>, usize>,
    pub length_in_seconds: f32,
}
