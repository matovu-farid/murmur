use super::traits::TranscribeError;
use reqwest::multipart;

pub async fn transcribe_api(
    audio: &[f32],
    sample_rate: u32,
    api_key: &str,
) -> Result<String, TranscribeError> {
    let tmp_path = std::env::temp_dir().join("wipr_recording.wav");
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
        .map_err(|e| TranscribeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    for &sample in samples {
        let s = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        writer.write_sample(s).map_err(|e| {
            TranscribeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;
    }
    writer.finalize().map_err(|e| {
        TranscribeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_wav_creates_file() {
        let samples = vec![0.0f32; 16000];
        let path = std::env::temp_dir().join("wipr_test.wav");
        write_wav(&path, &samples, 16000).unwrap();
        assert!(path.exists());
        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().sample_rate, 16000);
        assert_eq!(reader.spec().channels, 1);
        std::fs::remove_file(&path).ok();
    }
}
