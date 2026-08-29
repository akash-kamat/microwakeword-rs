use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc::SyncSender, mpsc::TrySendError};

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

use crate::{AUDIO_BLOCK_SAMPLES, Error, Result, SAMPLE_RATE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDevice {
    pub index: usize,
    pub name: String,
}

pub fn available_input_devices() -> Result<Vec<AudioDevice>> {
    cpal::default_host()
        .input_devices()
        .map_err(|e| Error::Audio(e.to_string()))?
        .enumerate()
        .map(|(index, device)| {
            Ok(AudioDevice {
                index,
                name: device.name().map_err(|e| Error::Audio(e.to_string()))?,
            })
        })
        .collect()
}

// Keeping samples inline avoids one heap allocation for every 10 ms audio block.
#[allow(clippy::large_enum_variant)]
#[derive(Default)]
pub(crate) struct AudioStatus {
    dropped: AtomicUsize,
    error: Mutex<Option<String>>,
}

impl AudioStatus {
    pub(crate) fn dropped(&self) -> usize {
        self.dropped.swap(0, Ordering::Relaxed)
    }

    pub(crate) fn report_error(&self, error: String) {
        let mut pending = self.error.lock().unwrap_or_else(|lock| lock.into_inner());
        if pending.is_none() {
            *pending = Some(error);
        }
    }

    pub(crate) fn take_error(&self) -> Option<String> {
        self.error
            .lock()
            .unwrap_or_else(|lock| lock.into_inner())
            .take()
    }
}

pub(crate) fn open_input(
    selector: Option<&str>,
    sender: SyncSender<[i16; AUDIO_BLOCK_SAMPLES]>,
    status: Arc<AudioStatus>,
) -> Result<Stream> {
    let host = cpal::default_host();
    let device = if let Some(selector) = selector {
        let devices: Vec<_> = host
            .input_devices()
            .map_err(|e| Error::Audio(e.to_string()))?
            .collect();
        if let Ok(index) = selector.parse::<usize>() {
            devices
                .into_iter()
                .nth(index)
                .ok_or_else(|| Error::Audio("microphone index is out of range".into()))?
        } else {
            let needle = selector.to_lowercase();
            devices
                .into_iter()
                .find(|d| d.name().is_ok_and(|n| n.to_lowercase().contains(&needle)))
                .ok_or_else(|| Error::Audio(format!("no microphone name contains {selector:?}")))?
        }
    } else {
        host.default_input_device()
            .ok_or_else(|| Error::Audio("there is no default microphone".into()))?
    };
    let supported = device
        .default_input_config()
        .map_err(|e| Error::Audio(e.to_string()))?;
    let format = supported.sample_format();
    let config = supported.config();
    match format {
        SampleFormat::I16 => build_stream(&device, &config, sender, status, |s: i16| {
            s as f32 / 32768.0
        }),
        SampleFormat::U16 => build_stream(&device, &config, sender, status, |s: u16| {
            (s as f32 - 32768.0) / 32768.0
        }),
        SampleFormat::F32 => build_stream(&device, &config, sender, status, |s: f32| s),
        other => Err(Error::Audio(format!(
            "unsupported microphone sample format: {other:?}"
        ))),
    }
}

struct Pipeline {
    channels: usize,
    input_frames: usize,
    pending_input: Vec<f32>,
    pending_output: Vec<f32>,
    resampler: Option<Fft<f32>>,
    sender: SyncSender<[i16; AUDIO_BLOCK_SAMPLES]>,
    status: Arc<AudioStatus>,
}

impl Pipeline {
    fn new(
        rate: u32,
        channels: usize,
        sender: SyncSender<[i16; AUDIO_BLOCK_SAMPLES]>,
        status: Arc<AudioStatus>,
    ) -> Result<Self> {
        if channels == 0 || rate % 100 != 0 {
            return Err(Error::Audio(format!(
                "unsupported microphone format: {rate} Hz, {channels} channels"
            )));
        }
        let input_frames = rate as usize / 100;
        let resampler = if rate == SAMPLE_RATE {
            None
        } else {
            Some(
                Fft::new(
                    rate as usize,
                    SAMPLE_RATE as usize,
                    input_frames,
                    1,
                    1,
                    FixedSync::Input,
                )
                .map_err(|e| Error::Audio(e.to_string()))?,
            )
        };
        Ok(Self {
            channels,
            input_frames,
            pending_input: Vec::new(),
            pending_output: Vec::new(),
            resampler,
            sender,
            status,
        })
    }

    fn push<T: Copy>(&mut self, data: &[T], convert: impl Fn(T) -> f32) -> Result<()> {
        for frame in data.chunks_exact(self.channels) {
            self.pending_input
                .push(frame.iter().copied().map(&convert).sum::<f32>() / self.channels as f32);
        }
        while self.pending_input.len() >= self.input_frames {
            if let Some(resampler) = &mut self.resampler {
                let input = InterleavedSlice::new(
                    &self.pending_input[..self.input_frames],
                    1,
                    self.input_frames,
                )
                .map_err(|e| Error::Audio(e.to_string()))?;
                self.pending_output.extend(
                    resampler
                        .process(&input, 0, None)
                        .map_err(|e| Error::Audio(e.to_string()))?
                        .take_data(),
                );
            } else {
                self.pending_output
                    .extend_from_slice(&self.pending_input[..self.input_frames]);
            }
            self.pending_input.drain(..self.input_frames);
            while self.pending_output.len() >= AUDIO_BLOCK_SAMPLES {
                let mut block = [0_i16; AUDIO_BLOCK_SAMPLES];
                for (out, sample) in block.iter_mut().zip(&self.pending_output) {
                    *out = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                }
                self.pending_output.drain(..AUDIO_BLOCK_SAMPLES);
                match self.sender.try_send(block) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => {
                        self.status.dropped.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(TrySendError::Disconnected(_)) => return Err(Error::AudioStreamEnded),
                }
            }
        }
        Ok(())
    }
}

fn build_stream<T, F>(
    device: &Device,
    config: &StreamConfig,
    sender: SyncSender<[i16; AUDIO_BLOCK_SAMPLES]>,
    status: Arc<AudioStatus>,
    convert: F,
) -> Result<Stream>
where
    T: cpal::SizedSample + Copy,
    F: Fn(T) -> f32 + Send + 'static,
{
    let mut pipeline = Pipeline::new(
        config.sample_rate.0,
        config.channels as usize,
        sender.clone(),
        status.clone(),
    )?;
    let stream_status = status.clone();
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                if let Err(error) = pipeline.push(data, &convert) {
                    pipeline.status.report_error(error.to_string());
                }
            },
            move |error| {
                stream_status.report_error(error.to_string());
            },
            None,
        )
        .map_err(|e| Error::Audio(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downmixes_stereo_into_one_ten_millisecond_block() {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let status = Arc::new(AudioStatus::default());
        let mut pipeline = Pipeline::new(SAMPLE_RATE, 2, sender, status).unwrap();
        pipeline
            .push(&[0.5_f32; AUDIO_BLOCK_SAMPLES * 2], |sample| sample)
            .unwrap();
        let samples = receiver.try_recv().unwrap();
        assert!(samples.iter().all(|sample| *sample == 16_383));
    }

    #[test]
    fn reports_queue_overflow() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(0);
        let status = Arc::new(AudioStatus::default());
        let mut pipeline = Pipeline::new(SAMPLE_RATE, 1, sender, status.clone()).unwrap();
        pipeline
            .push(&[0.0_f32; AUDIO_BLOCK_SAMPLES], |sample| sample)
            .unwrap();
        assert_eq!(status.dropped(), 1);
        assert_eq!(status.dropped(), 0);
    }

    #[test]
    fn microphone_errors_are_not_lost_when_the_sample_queue_is_full() {
        let (sender, _receiver) = std::sync::mpsc::sync_channel(1);
        let status = Arc::new(AudioStatus::default());
        let mut pipeline = Pipeline::new(SAMPLE_RATE, 1, sender, status.clone()).unwrap();
        pipeline
            .push(&[0.0_f32; AUDIO_BLOCK_SAMPLES], |sample| sample)
            .unwrap();

        status.report_error("device disconnected".into());

        assert_eq!(status.take_error().as_deref(), Some("device disconnected"));
        assert!(status.take_error().is_none());
    }
}
