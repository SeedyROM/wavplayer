use std::sync::Arc;

use parking_lot::Mutex;

use crate::stream::StreamInfo;

pub type Resources = Arc<Mutex<Vec<Box<dyn AudioResource>>>>;

/// A trait all things audio in the system needs to implment.
pub trait AudioResource: Send + Sync {
    /// Process get an audio sample for the current tick
    fn tick(&mut self, stream_info: &StreamInfo) -> f32;
}
