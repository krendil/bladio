use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::{PyIterator,IntoPyDict};

use crate::types::Samp;

pub struct Speaker {
    utter_result: Option<PyObject>,
    buf: Vec<Samp>,
}

impl Speaker {

    pub fn new() -> Self {
        return Speaker {
            utter_result: None,
            buf: Vec::new()
        };
    }

    pub fn is_speaking(&self) -> bool {
        return !self.buf.is_empty() || self.utter_result.is_some();
    }

    pub fn say(&mut self, message: &str) -> PyResult<()> {
        return Python::with_gil(|py| {
            let mimic3 = py.import("mimic3_tts")?;
            let tts_class = mimic3.getattr("Mimic3TextToSpeechSystem")?;
            let opts_class = mimic3.getattr("Mimic3Settings")?;

            let kwargs = [("length_scale",1.into_py(py)),("use_cuda",false.into_py(py))].into_py_dict(py);
            let opts = opts_class.call((), Some(kwargs))?;

            let tts = tts_class.call1((opts,))?;
            tts.call_method0("begin_utterance")?;
            tts.call_method1("speak_text", (message,))?;
            let results = tts.call_method0("end_utterance")?;
            self.utter_result = Some(results.into());

            return Ok(());
        });
    }

    pub fn next(&mut self, buf: &mut[Samp]) -> usize {
        if self.buf.is_empty() {
            // No half-filled buffer
            return self.copy_from_tts(buf);
        } else {
            // Half-filled buffer
            let mut samples_filled = self.partial_copy_from_stored_buf(buf);
            if samples_filled < buf.len() {
                samples_filled += self.copy_from_tts(&mut buf[samples_filled..]);
            }
            return samples_filled
        }
    }

    fn copy_from_tts(&mut self, buf: &mut [Samp]) -> usize {
        let mut result_exhausted = false;
        let res: PyResult<usize> =  Python::with_gil(|py| {
            let samples_written = match &self.utter_result {
                None => 0,
                Some(results) => {
                    let mut iter = PyIterator::from_object(py, results.as_ref(py))?;

                    let mut samples_filled: usize = 0;

                    while samples_filled < buf.len() {
                        let n = iter.next();
                        if n.is_some() {
                            unsafe {
                                let bytes = n.unwrap()?.getattr("bytes")?.extract::<&[u8]>()?;
                                let (_, shorts, _) = bytes.align_to::<Samp>();
                                
                                samples_filled += self.partial_copy(shorts, &mut buf[samples_filled..]);
                            }
                        } else {
                            result_exhausted = true;
                            break;
                        }
                    }
                    samples_filled
                }
            };
            return Ok(samples_written);
            // TODO: Error checking
            // return res.unwrap_or(0);
        });
        if result_exhausted {
            self.utter_result = None;
        }
        return res.unwrap();
    }

    // Copy what we can and stash the rest in self.buf
    fn partial_copy_from_stored_buf(&mut self, to: &mut [Samp]) -> usize {
        let mut buf_exhausted = false;
        let mut samples_written = 0;
        if !self.buf.is_empty() {
            
            if to.len() >= self.buf.len() {
                to[..self.buf.len()].copy_from_slice(&self.buf);
                // self.buf = None;
                buf_exhausted = true;
                samples_written = self.buf.len()
            } else {
                to.copy_from_slice(&self.buf[..to.len()]);
                self.buf.drain(..to.len());
                samples_written = to.len();
            }
        }
        if buf_exhausted {
            self.buf.truncate(0);
        }
        return samples_written;
    }
    // Copy what we can and stash the rest in self.buf
    fn partial_copy(&mut self, from: &[Samp], to: &mut [Samp]) -> usize {
        if to.len() >= from.len() {
            to[..from.len()].copy_from_slice(from);
            self.buf.truncate(0);
            return from.len();
        } else {
            to.copy_from_slice(&from[..to.len()]);
            self.buf.copy_from_slice(&from[to.len()..]);
            return to.len();
        }
    }

}

pub fn mimic_test() {
    Python::with_gil(|py| {
        let foo = 3;
        // let mimic3 = Pymodule::Import(py, "mimic3_tts");
        py_run!(py, foo, r#"
from mimic3_tts import Mimic3Settings, Mimic3TextToSpeechSystem
import sys

tts = Mimic3TextToSpeechSystem(
    Mimic3Settings(
        length_scale=1,
        noise_scale=0.667,
        noise_w=0.8,
        voices_directories=None,
        use_cuda=False,
        use_deterministic_compute=True,
    )
)

tts.begin_utterance()
tts.speak_text("This is a test.")
results = tts.end_utterance()

for result in results:
    wav_bytes = result.to_wav_bytes()
    sys.stdout.buffer.write(wav_bytes)
    sys.stdout.buffer.flush()
        "#);
    });
}