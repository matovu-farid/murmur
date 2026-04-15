use super::traits::TranscribeError;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub fn transcribe_local(audio: &[f32], model_path: &str) -> Result<String, TranscribeError> {
    if !Path::new(model_path).exists() {
        return Err(TranscribeError::ModelError(format!(
            "Model not found: {}. Download it from settings.", model_path
        )));
    }

    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| TranscribeError::ModelError(format!("Failed to load model: {}", e)))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_no_context(true);

    let mut state = ctx.create_state()
        .map_err(|e| TranscribeError::ModelError(format!("Failed to create state: {}", e)))?;

    state.full(params, audio)
        .map_err(|e| TranscribeError::ModelError(format!("Transcription failed: {}", e)))?;

    let n_segments = state.full_n_segments();

    let mut text = String::new();
    for i in 0..n_segments {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(s) = segment.to_str_lossy() {
                text.push_str(&s);
            }
        }
    }

    Ok(text.trim().to_string())
}

pub fn model_download_url(model_name: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_name
    )
}

pub async fn download_model(
    model_name: &str,
    models_dir: &Path,
    on_progress: impl Fn(u64, u64),
) -> Result<String, TranscribeError> {
    std::fs::create_dir_all(models_dir).map_err(TranscribeError::IoError)?;

    let dest = models_dir.join(format!("ggml-{}.bin", model_name));
    if dest.exists() {
        return Ok(dest.to_string_lossy().to_string());
    }

    let url = model_download_url(model_name);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await
        .map_err(|e| TranscribeError::ApiError(format!("Download failed: {}", e)))?;

    let total = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = std::fs::File::create(&dest).map_err(TranscribeError::IoError)?;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| TranscribeError::ApiError(format!("Download error: {}", e)))?;
        std::io::Write::write_all(&mut file, &chunk).map_err(TranscribeError::IoError)?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    Ok(dest.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_download_url() {
        let url = model_download_url("medium.en");
        assert_eq!(url, "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin");
    }

    #[test]
    fn test_transcribe_missing_model() {
        let result = transcribe_local(&[0.0; 16000], "/nonexistent/model.bin");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Model not found"));
    }
}
