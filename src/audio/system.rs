use std::sync::Arc;

use crate::audio::{
    resource::{AudioResource, Resources},
    stream::{StreamBuffer, StreamInfo},
};
use color_eyre::eyre::{eyre, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, OutputCallbackInfo, StreamConfig, SupportedStreamConfig,
};
use crossbeam::channel::Receiver;
use parking_lot::Mutex;
use tracing::{error, info};

/// The global audio system
pub struct AudioSystem {
    // TODO: Move me into my own struct
    host: Host,
    device: Device,
    supported_stream_config: SupportedStreamConfig,
    stream_config: StreamConfig,

    /// Audio resources to run each tick
    resources: Resources,

    /// Handle shutdown
    shutdown_rx: Receiver<()>,
}

impl AudioSystem {
    /// Create a new audio system to play some sounds.
    pub fn new(shutdown_rx: Receiver<()>) -> Result<Self> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| eyre!("Failed to load default output device"))?;

        let supported_stream_config = device.default_output_config()?;
        let stream_config = supported_stream_config.clone().into();

        Ok(Self {
            host,
            device,
            supported_stream_config,
            stream_config,
            resources: Arc::new(Mutex::new(Vec::new())),
            shutdown_rx,
        })
    }

    /// Add a struct that implements AudioResource to the system.
    pub fn add_resource(&mut self, resource: impl AudioResource + 'static) {
        self.resources.lock().push(Box::new(resource));
    }

    /// Run the audio system and start a stream with the specified sample format.
    pub fn run(&self) -> Result<()> {
        match self.supported_stream_config.sample_format() {
            cpal::SampleFormat::I16 => self.stream::<i16>(),
            cpal::SampleFormat::U16 => self.stream::<u16>(),
            cpal::SampleFormat::F32 => self.stream::<f32>(),
        }
    }

    /// Start an audio stream
    fn stream<S>(&self) -> Result<()>
    where
        S: cpal::Sample,
    {
        info!(
            "Starting stream at host {:?} with device: {}",
            self.host.id(),
            self.device.name().unwrap_or("Unknown Device".into())
        );

        let info = StreamInfo::from(&self.stream_config);
        let resources = self.resources.clone();
        let mut work_buffer = Vec::<f32>::new();

        let stream = self.device.build_output_stream(
            &self.stream_config,
            move |data: &mut [S], output_callback_info: &OutputCallbackInfo| {
                let stream_buffer = &mut StreamBuffer {
                    data: &mut work_buffer,
                    info: &info,
                };
                Self::stream_callback(resources.clone(), data, stream_buffer, output_callback_info)
            },
            |err| error!("Stream callback error: {}", err),
        )?;

        stream.play()?;

        // Wait for shutdown signal
        self.shutdown_rx.recv()?;

        info!(
            "Stopping stream at host {:?} with device: {}",
            self.host.id(),
            self.device.name().unwrap_or("Unknown Device".into())
        );

        Ok(())
    }

    /// Send data to our audio stream
    fn stream_callback<S>(
        resources: Resources,
        data: &mut [S],
        stream_buffer: &mut StreamBuffer,
        _: &OutputCallbackInfo,
    ) where
        S: cpal::Sample,
    {
        let mut resources = resources.lock();

        // Resize the working buffer based on the data slice, if not the same.
        if stream_buffer.data.len() != data.len() {
            stream_buffer.data.resize(data.len(), 0.0);
        }
        // Zero the buffer.
        stream_buffer.data.fill(0.0);

        // Write into the working buffer for each resoure
        for resource in resources.iter_mut() {
            resource.process(stream_buffer);
        }

        // Write to the output buffer
        for i in 0..data.len() {
            data[i] = S::from(&(stream_buffer.data[i] / resources.len() as f32))
        }
    }
}
