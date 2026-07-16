mod model;
mod narration;
mod text;

pub use model::{download_model, model_dir, model_ready, TtsModelEvent};
pub use narration::{spawn_narration, NarrationEvent, NarrationHandle, NarrationUiState};
pub use text::{build_narration_units, NarrationTextError, NarrationUnit};

use std::fmt;

use sherpa_onnx::{OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig};

pub struct TTS {
    pub engine: OfflineTts,
}

impl fmt::Debug for TTS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TTS")
    }
}

impl TTS {
    pub fn load() -> Result<Self, String> {
        if !model_ready() {
            return Err(format!("TTS model missing at {}", model_dir().display()));
        }

        let dir = model_dir();
        let config = OfflineTtsConfig {
            model: sherpa_onnx::OfflineTtsModelConfig {
                kokoro: OfflineTtsKokoroModelConfig {
                    model: Some(dir.join("model.onnx").to_string_lossy().into_owned()),
                    voices: Some(dir.join("voices.bin").to_string_lossy().into_owned()),
                    tokens: Some(dir.join("tokens.txt").to_string_lossy().into_owned()),
                    data_dir: Some(dir.join("espeak-ng-data").to_string_lossy().into_owned()),
                    length_scale: 1.0,
                    ..Default::default()
                },
                num_threads: 2,
                debug: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let engine = OfflineTts::create(&config)
            .ok_or_else(|| format!("failed to load TTS from {}", dir.display()))?;

        Ok(Self { engine })
    }
}
