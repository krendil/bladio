use std::num::NonZeroU32;

use nonzero_ext::nonzero;

pub type Samp = f32;
pub const SAMPLE_RATE: NonZeroU32 = nonzero!(22050u32);