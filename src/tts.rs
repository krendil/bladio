use pyo3::prelude::*;
use pyo3::py_run;

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