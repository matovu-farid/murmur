use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranscribeError {
    #[error("Audio too short: {0:.1}s (minimum {1:.1}s)")]
    AudioTooShort(f32, f32),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Model error: {0}")]
    ModelError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl serde::Serialize for TranscribeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
