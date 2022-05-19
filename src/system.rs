use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    resource::AudioResource,
    stream::{StreamBuffer, StreamInfo},
};
use color_eyre::eyre::{eyre, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, OutputCallbackInfo, StreamConfig, SupportedStreamConfig,
};

type Resources<T> = Arc<Mutex<Vec<T>>>;

pub struct AudioSystem<T> {
    _host: Host,
    device: Device,
    supported_stream_config: SupportedStreamConfig,
    stream_config: StreamConfig,
    resources: Resources<T>,
}

impl<T> AudioSystem<T>
where
    T: AudioResource,
    T: Send + 'static,
{
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| eyre!("Failed to load default output device"))?;

        let supported_stream_config = device.default_output_config()?;
        let stream_config = supported_stream_config.clone().into();

        Ok(Self {
            _host: host,
            device,
            supported_stream_config,
            stream_config,
            resources: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn add_resource(&mut self, resource: T) {
        self.resources.lock().unwrap().push(resource);
    }

    pub fn run(&self) -> Result<()> {
        match self.supported_stream_config.sample_format() {
            cpal::SampleFormat::I16 => self.stream::<i16>(),
            cpal::SampleFormat::U16 => self.stream::<u16>(),
            cpal::SampleFormat::F32 => self.stream::<f32>(),
        }
    }

    fn stream<S>(&self) -> Result<()>
    where
        S: cpal::Sample,
    {
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

        std::thread::sleep(Duration::from_millis(10000));

        Ok(())
    }

    fn stream_callback<S>(
        resources: Resources<T>,
        stream_buffer: &mut StreamBuffer<S>,
        _: &OutputCallbackInfo,
    ) where
        S: cpal::Sample,
    {
        let mut resources = resources.lock().unwrap();
        let info = stream_buffer.info;
        for frame in stream_buffer.into_frames() {
            for sample in frame.iter_mut() {
                let mut x = 0.0;
                for resource in resources.iter_mut() {
                    x += resource.tick(info);
                }
                x /= resources.len() as f32;
                *sample = S::from(&x);
            }
        }
    }
}
