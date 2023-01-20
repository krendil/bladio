use std::io;
use std::path::Path;
use std::fs::{self, DirEntry, File};

use nonzero_ext::nonzero;
use vorbis_rs::{VorbisError, VorbisDecoder};

use crate::types::{Samp, SAMPLE_RATE};

#[derive(Debug)]
enum SampleError {
    VorbisError(VorbisError),
    IoError(io::Error),
    BladioError(String)
}

impl From<VorbisError> for SampleError {
    fn from(value: VorbisError) -> Self {
        return SampleError::VorbisError(value);
    }
}
impl From<io::Error> for SampleError {
    fn from(value: io::Error) -> Self {
        return SampleError::IoError(value);
    }
}


pub struct SampleLibrary {
    samples: Vec<Vec<Samp>>
}

impl SampleLibrary {

    pub fn new(sfx_path: &Path) -> Self {
        let mut samples = Vec::new();
        for entry in fs::read_dir(sfx_path).unwrap() {
            match entry {
                Ok(f) => {
                    let os_name = f.file_name();
                    let clean_name = os_name.to_str();
                    let is_ogg = match clean_name {
                        Some(n) => n.ends_with(".ogg"),
                        _ => false
                    };
                    if is_ogg {
                        load_ogg(f).map_or_else( |err| {
                            eprintln!("{:?}", err)
                        },
                        |s| {
                            samples.push(s)
                        });
                    }
                },
                Err(e) => eprintln!("{}", e)
            };
        }
        return SampleLibrary {
            samples: samples,
        };
    }

    pub fn get(& self, i: usize) -> &[Samp] {
        return self.samples[i].as_slice();
    }

    pub fn len(&self) -> usize {
        return self.samples.len();
    }

}

fn load_ogg(file: DirEntry) -> Result<Vec<Samp>, SampleError> {
    let reader = File::open(file.path())?;
    let mut vdec = VorbisDecoder::new(reader)?;

    if vdec.channels() != nonzero!(1u8) {
        let err = format!("Error loading {:?}: too many channels.", file.file_name());
        return Err(SampleError::BladioError(err));
    } else if vdec.sampling_frequency() != SAMPLE_RATE {
        let err = format!("Error loading {:?}: incorrect sample rate.", file.file_name());
        return Err(SampleError::BladioError(err));
    } else {
        let mut buf: Vec<Samp> = Vec::new();
        loop {
            let res = vdec.decode_audio_block()?;
            match res {
                None => break,
                Some(block) => {
                    buf.extend_from_slice( block.samples()[0] );
                }
            }
        }
        return Ok(buf);
    }
}
