use std::sync::Arc;

use crate::{
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

    /// Add an struct that implements AudioResource to the system.
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
        println!(
            "Starting stream at host {:?} with device: {}",
            self.host.id(),
            self.device.name().unwrap_or("Unknown Device".into())
        );

        let info = StreamInfo::from(&self.stream_config);
        let resources = self.resources.clone();

        let stream = self.device.build_output_stream(
            &self.stream_config,
            move |data: &mut [S], output_callback_info: &OutputCallbackInfo| {
                let stream_buffer = &mut StreamBuffer { data, info: &info };
                Self::stream_callback(resources.clone(), stream_buffer, output_callback_info)
            },
            |err| eprintln!("Stream callback error: {}", err),
        )?;

        stream.play()?;

        // Wait for shutdown signal
        self.shutdown_rx.recv()?;

        println!(
            "Stopping stream at host {:?} with device: {}",
            self.host.id(),
            self.device.name().unwrap_or("Unknown Device".into())
        );

        Ok(())
    }

    /// Send data to our audio stream
    fn stream_callback<S>(
        resources: Resources,
        stream_buffer: &mut StreamBuffer<S>,
        _: &OutputCallbackInfo,
    ) where
        S: cpal::Sample,
    {
        let mut resources = resources.lock();
        let info = stream_buffer.info;
        // For each frame, a sample per channel...
        for frame in stream_buffer.into_frames() {
            // Process each sample...
            for sample in frame.iter_mut() {
                let mut mix = 0.0;
                // Iterate each resource
                for resource in resources.iter_mut() {
                    // Add sample to the mix value
                    mix += resource.tick(info);
                }
                // Normalize back to -1, 1
                mix /= resources.len() as f32;
                *sample = S::from(&mix);
            }
        }
    }
}
