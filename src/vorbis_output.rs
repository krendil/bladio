use std::io::Write;

use nonzero_ext::nonzero;
use vorbis_rs::{VorbisEncoder,VorbisBitrateManagementStrategy, VorbisError};

use crate::types::{Samp, SAMPLE_RATE};

const BLOCK_SIZE: usize = 1024;

pub fn output_to_vorbis<F, W>(mut supplier: F, writer: &mut W) -> Result<(), VorbisError>
    where F: FnMut(&mut [Samp]) -> usize, W: Write
{
    let mut venc = VorbisEncoder::new(
        0,
        [("a","b")],
        SAMPLE_RATE,
        nonzero!(1u8),
        VorbisBitrateManagementStrategy::QualityVbr { target_quality: 0.5 },
        None,
        writer)?;

    let mut ibuf: [Samp; BLOCK_SIZE] = [0.0; BLOCK_SIZE];
    let mut fbuf: [f32; BLOCK_SIZE] = [0.0; BLOCK_SIZE];
    let mut empty_count = 0;
    while empty_count < 3 {
        let samples_filled = supplier(&mut ibuf);
        // for (i, s) in ibuf[..samples_filled].iter().enumerate() {
        //     fbuf[i] = (i16::from_le(*s) as f32) / 32768.0;
        // }

        venc.encode_audio_block([&ibuf[..samples_filled]])?;
        if samples_filled < BLOCK_SIZE {
            empty_count += 1;
        }
    }

    venc.finish()?;

    return Ok(());
}