pub mod audio;
pub mod logging;

use std::{fs::File, io::BufReader, path::PathBuf};

use audio::{resource::AudioResource, stream::StreamInfo};
use color_eyre::eyre::Result;
use hound::{WavReader, WavSpec};
use rand::{thread_rng, Rng};

use audio::system::AudioSystem;
use tracing::{error, info};

/// Example white noise AudioResource
struct WhiteNoise;

impl AudioResource for WhiteNoise {
    fn tick(&mut self, _stream_info: &StreamInfo) -> f32 {
        ((thread_rng().gen::<f32>() * 2.0) - 1.0) * 0.03
    }
}

/// WAV file player
struct WavFile {
    reader: WavReader<BufReader<File>>,
}

impl WavFile {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let reader = WavReader::open(path)?;

        Ok(Self { reader })
    }

    fn next_sample(&mut self) -> Result<f32> {
        let WavSpec {
            sample_format,
            bits_per_sample,
            ..
        } = self.reader.spec();

        match sample_format {
            // If it's a float the sample is already f32, just unwrap it
            hound::SampleFormat::Float => {
                Ok(self.reader.samples::<f32>().next().unwrap_or(Ok(0.0))?)
            }
            // Handle PCM encoded samples
            hound::SampleFormat::Int => {
                let next_pcm_sample = self.reader.samples::<i32>().next().unwrap_or(Ok(0))?;
                // Normalize the sample based on the pow(2, bits_per_sample).
                let normalized_sample =
                    next_pcm_sample as f32 / f32::powi(2.0, bits_per_sample as i32);
                Ok(normalized_sample)
            }
        }
    }
}

impl AudioResource for WavFile {
    fn tick(&mut self, _stream_info: &StreamInfo) -> f32 {
        // TODO: If there is no next sample, return 0.0 for now.
        self.next_sample().unwrap_or_else(|err| {
            error!("Failed to process sample: {}", err);
            0.0
        })
    }
}

fn main() -> Result<()> {
    logging::setup()?;

    // Setup an audio file to stream
    let wav_file = WavFile::from_path("./data/lighter.wav".into())?;

    // Handle shutdown
    let (shutdown_tx, shutdown_rx) = crossbeam::channel::bounded::<()>(1);
    ctrlc::set_handler(move || {
        info!("Shutting down...");
        shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal...");
    })?;

    // Create the audio system and add our wave file resource
    let mut audio_sys = AudioSystem::new(shutdown_rx.clone())?;
    audio_sys.add_resource(wav_file);

    // 🎶 Make some NOISE! 🎶
    audio_sys.run()?;

    Ok(())
}
