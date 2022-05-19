use std::sync::Arc;

use parking_lot::Mutex;

use super::stream::StreamBuffer;

pub type Resources = Arc<Mutex<Vec<Box<dyn AudioResource>>>>;

/// A trait all things audio in the system needs to implment.
pub trait AudioResource: Send + Sync {
    /// Process audio into the buffer
    fn process(&mut self, stream_buffer: &mut StreamBuffer);
}
