pub mod resource;
pub mod stream;
pub mod system;

use std::{fs::File, io::BufReader, path::PathBuf};

use color_eyre::eyre::Result;
use hound::{WavReader, WavSpec};
use rand::{thread_rng, Rng};
use resource::AudioResource;

use crate::system::AudioSystem;

struct WhiteNoise;

impl AudioResource for WhiteNoise {
    fn tick(&mut self, _stream_info: &stream::StreamInfo) -> f32 {
        ((thread_rng().gen::<f32>() * 2.0) - 1.0) * 0.03
    }
}

struct WavFile {
    reader: WavReader<BufReader<File>>,
}

impl WavFile {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let reader = WavReader::open(path)?;

        Ok(Self { reader })
    }

    fn next_sample(&mut self) -> f32 {
        let WavSpec {
            sample_format,
            bits_per_sample,
            ..
        } = self.reader.spec();

        match sample_format {
            // If it's a float the sample is already f32, just unwrap it
            hound::SampleFormat::Float => self.reader.samples::<f32>().next().unwrap().unwrap(),
            // Handle PCM encoded samples
            hound::SampleFormat::Int => {
                let next_pcm_sample = self.reader.samples::<i16>().next().unwrap().unwrap();
                // Normalize the sample based on the pow(2, bits_per_sample).
                next_pcm_sample as f32 / f32::powi(2.0, bits_per_sample as i32)
            }
        }
    }
}

impl AudioResource for WavFile {
    fn tick(&mut self, _stream_info: &stream::StreamInfo) -> f32 {
        self.next_sample()
    }
}

fn main() -> Result<()> {
    let wav_file = WavFile::from_path("./data/lighter.wav".into())?;

    let mut audio_sys = AudioSystem::new()?;
    audio_sys.add_resource(wav_file);
    audio_sys.add_resource(WhiteNoise);
    audio_sys.run()?;

    Ok(())
}
