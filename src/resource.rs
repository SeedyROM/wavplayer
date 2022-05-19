use crate::stream::StreamInfo;

pub trait AudioResource {
    fn tick(&mut self, stream_info: &StreamInfo) -> f32;
}
