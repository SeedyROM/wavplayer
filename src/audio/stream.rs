use std::slice::ChunksMut;

pub struct StreamInfo {
    pub sample_rate: f32,
    pub channels: usize,
}

impl From<&cpal::StreamConfig> for StreamInfo {
    fn from(stream_config: &cpal::StreamConfig) -> Self {
        Self {
            sample_rate: stream_config.sample_rate.0 as f32,
            channels: stream_config.channels as usize,
        }
    }
}

pub struct StreamBuffer<'a> {
    pub info: &'a StreamInfo,
    pub data: &'a mut Vec<f32>,
}

impl<'a> StreamBuffer<'a> {
    pub fn into_frames(&mut self) -> ChunksMut<'_, f32> {
        self.data.chunks_mut(self.info.channels)
    }
}
