use std::sync::Arc;

use parking_lot::Mutex;

use crate::stream::StreamInfo;

pub type Resources = Arc<Mutex<Vec<Box<dyn AudioResource + 'static>>>>;

pub trait AudioResource: Send + Sync {
    fn tick(&mut self, stream_info: &StreamInfo) -> f32;
}
