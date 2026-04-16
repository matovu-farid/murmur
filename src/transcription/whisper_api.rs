use super::traits::TranscribeError;
use reqwest::multipart;

pub async fn transcribe_api(
    audio: &[f32],
    sample_rate: u32,
    api_key: &str,
) -> Result<String, TranscribeError> {
    let tmp_path = std::env::temp_dir().join("murmur_recording.wav");
    write_wav(&tmp_path, audio, sample_rate)?;

    let file_bytes = std::fs::read(&tmp_path).map_err(TranscribeError::IoError)?;

    let part = multipart::Part::bytes(file_bytes)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|e| TranscribeError::ApiError(e.to_string()))?;

    let form = multipart::Form::new()
        .text("model", "whisper-1")
        .part("file", part);

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| TranscribeError::ApiError(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(TranscribeError::ApiError(error_text));
    }

    #[derive(serde::Deserialize)]
    struct WhisperResponse {
        text: String,
    }

    let result: WhisperResponse = response
        .json()
        .await
        .map_err(|e| TranscribeError::ApiError(e.to_string()))?;

    std::fs::remove_file(&tmp_path).ok();
    Ok(result.text.trim().to_string())
}

fn write_wav(
    path: &std::path::Path,
    samples: &[f32],
    sample_rate: u32,
) -> Result<(), TranscribeError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| TranscribeError::IoError(std::io::Error::other(e)))?;
    for &sample in samples {
        let s = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        writer
            .write_sample(s)
            .map_err(|e| TranscribeError::IoError(std::io::Error::other(e)))?;
    }
    writer
        .finalize()
        .map_err(|e| TranscribeError::IoError(std::io::Error::other(e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_wav_creates_file() {
        let samples = vec![0.0f32; 16000];
        let path = std::env::temp_dir().join("murmur_test.wav");
        write_wav(&path, &samples, 16000).unwrap();
        assert!(path.exists());
        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().sample_rate, 16000);
        assert_eq!(reader.spec().channels, 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_wav_format_is_16bit_mono() {
        let samples = vec![0.1f32; 100];
        let path = std::env::temp_dir().join("murmur_test_format.wav");
        write_wav(&path, &samples, 44100).unwrap();
        let reader = hound::WavReader::open(&path).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.bits_per_sample, 16);
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_wav_contains_correct_sample_count() {
        let samples = vec![0.0f32; 500];
        let path = std::env::temp_dir().join("murmur_test_count.wav");
        write_wav(&path, &samples, 16000).unwrap();
        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.len(), 500);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_write_wav_with_nonzero_samples() {
        let samples = vec![0.5f32, -0.5, 0.25, -0.25, 1.0, -1.0];
        let path = std::env::temp_dir().join("murmur_test_nonzero.wav");
        write_wav(&path, &samples, 16000).unwrap();
        let mut reader = hound::WavReader::open(&path).unwrap();
        let read_samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(read_samples.len(), 6);
        // 0.5 * 32767 ~ 16383
        assert!(read_samples[0] > 16000);
        // -0.5 * 32767 ~ -16383
        assert!(read_samples[1] < -16000);
        std::fs::remove_file(&path).ok();
    }
}
