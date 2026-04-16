use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

pub struct Recorder {
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    sample_rate: u32,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            sample_rate: 16000,
        }
    }

    pub fn start(&self) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let supported_config = device
            .default_input_config()
            .map_err(|e| format!("No default input config: {}", e))?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        self.samples.lock().unwrap().clear();
        self.is_recording.store(true, Ordering::SeqCst);

        let samples = self.samples.clone();
        let is_recording = self.is_recording.clone();

        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if is_recording.load(Ordering::SeqCst) {
                        samples.lock().unwrap().extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => {
                let samples = self.samples.clone();
                let is_recording = self.is_recording.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if is_recording.load(Ordering::SeqCst) {
                            let floats: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            samples.lock().unwrap().extend(floats);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            format => return Err(format!("Unsupported sample format: {:?}", format)),
        }
        .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {}", e))?;

        Ok(stream)
    }

    pub fn stop(&self) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        self.samples.lock().unwrap().clone()
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn duration_secs(samples: &[f32], sample_rate: u32) -> f32 {
        samples.len() as f32 / sample_rate as f32
    }

    pub const MIN_DURATION: f32 = 0.3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_calculation() {
        let samples = vec![0.0f32; 16000];
        assert!((Recorder::duration_secs(&samples, 16000) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_short_recording_detection() {
        let samples = vec![0.0f32; 4000];
        let duration = Recorder::duration_secs(&samples, 16000);
        assert!(duration < Recorder::MIN_DURATION);
    }

    #[test]
    fn test_recorder_initial_state() {
        let recorder = Recorder::new();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.sample_rate(), 16000);
    }

    #[test]
    fn test_stop_returns_empty_when_no_recording() {
        let recorder = Recorder::new();
        let samples = recorder.stop();
        assert!(samples.is_empty());
    }

    #[test]
    fn test_min_duration_is_0_3() {
        assert!((Recorder::MIN_DURATION - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_duration_at_various_sample_rates() {
        // 48000 Hz, 48000 samples = 1 second
        let samples = vec![0.0f32; 48000];
        assert!((Recorder::duration_secs(&samples, 48000) - 1.0).abs() < 0.001);

        // 44100 Hz, 22050 samples = 0.5 seconds
        let samples = vec![0.0f32; 22050];
        assert!((Recorder::duration_secs(&samples, 44100) - 0.5).abs() < 0.001);

        // 8000 Hz, 24000 samples = 3 seconds
        let samples = vec![0.0f32; 24000];
        assert!((Recorder::duration_secs(&samples, 8000) - 3.0).abs() < 0.001);

        // Empty samples = 0 seconds
        let samples: Vec<f32> = vec![];
        assert!((Recorder::duration_secs(&samples, 16000) - 0.0).abs() < 0.001);
    }
}
