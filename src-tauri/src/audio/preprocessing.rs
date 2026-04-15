/// Trim leading and trailing silence from audio samples.
pub fn trim_silence(samples: &[f32], threshold: f32) -> &[f32] {
    let start = samples.iter().position(|&s| s.abs() > threshold).unwrap_or(0);
    let end = samples.iter().rposition(|&s| s.abs() > threshold).map(|p| p + 1).unwrap_or(0);
    if start >= end { return &[]; }
    &samples[start..end]
}

/// Normalize audio gain to a target peak amplitude.
pub fn normalize_gain(samples: &[f32], target_peak: f32) -> Vec<f32> {
    let max_amplitude = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_amplitude < 1e-6 { return samples.to_vec(); }
    let gain = target_peak / max_amplitude;
    samples.iter().map(|&s| (s * gain).clamp(-1.0, 1.0)).collect()
}

/// Basic noise reduction using spectral subtraction.
pub fn reduce_noise(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let noise_sample_len = (sample_rate as f32 * 0.2) as usize;
    if samples.len() <= noise_sample_len { return samples.to_vec(); }
    let noise_floor: f32 = samples[..noise_sample_len].iter().map(|s| s.abs()).sum::<f32>() / noise_sample_len as f32;
    samples.iter().map(|&s| {
        if s.abs() > noise_floor * 2.0 { s }
        else { s * (s.abs() / (noise_floor * 2.0)).clamp(0.0, 1.0) }
    }).collect()
}

/// Full preprocessing pipeline: noise reduction -> silence trimming -> gain normalization
pub fn preprocess(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let denoised = reduce_noise(samples, sample_rate);
    let trimmed = trim_silence(&denoised, 0.01);
    if trimmed.is_empty() { return Vec::new(); }
    normalize_gain(trimmed, 0.9)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_silence_removes_leading_trailing() {
        let samples = vec![0.0, 0.0, 0.0, 0.5, 0.3, 0.0, 0.0];
        let trimmed = trim_silence(&samples, 0.01);
        assert_eq!(trimmed, &[0.5, 0.3]);
    }

    #[test]
    fn test_trim_silence_all_silent() {
        let samples = vec![0.0, 0.001, 0.0];
        let trimmed = trim_silence(&samples, 0.01);
        assert!(trimmed.is_empty());
    }

    #[test]
    fn test_normalize_gain() {
        let samples = vec![0.0, 0.25, -0.5, 0.1];
        let normalized = normalize_gain(&samples, 0.9);
        assert!((normalized[2] - (-0.9)).abs() < 0.001);
    }

    #[test]
    fn test_normalize_gain_silence() {
        let samples = vec![0.0, 0.0, 0.0];
        let normalized = normalize_gain(&samples, 0.9);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_reduce_noise_preserves_loud_signals() {
        let mut samples = vec![0.001f32; 3200];
        samples.extend(vec![0.5f32; 1600]);
        let result = reduce_noise(&samples, 16000);
        assert!(result[3200] > 0.4);
    }

    #[test]
    fn test_preprocess_pipeline() {
        let mut samples = vec![0.001f32; 3200];
        samples.extend(vec![0.0; 1600]);
        samples.extend(vec![0.5, -0.3, 0.4]);
        samples.extend(vec![0.0; 1600]);
        let result = preprocess(&samples, 16000);
        assert!(!result.is_empty());
        let max = result.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!((max - 0.9).abs() < 0.1);
    }

    #[test]
    fn test_trim_silence_no_silence() {
        let samples = vec![0.5, -0.3, 0.8, -0.6, 0.2];
        let trimmed = trim_silence(&samples, 0.01);
        assert_eq!(trimmed, &[0.5, -0.3, 0.8, -0.6, 0.2]);
    }

    #[test]
    fn test_trim_silence_empty_input() {
        let samples: Vec<f32> = vec![];
        let trimmed = trim_silence(&samples, 0.01);
        assert!(trimmed.is_empty());
    }

    #[test]
    fn test_normalize_gain_clamps_to_range() {
        // With target_peak > 1.0, values should still be clamped to [-1, 1]
        let samples = vec![0.5, -0.8, 0.3];
        let normalized = normalize_gain(&samples, 2.0);
        for &s in &normalized {
            assert!(s >= -1.0 && s <= 1.0, "Sample {} out of [-1, 1] range", s);
        }
    }

    #[test]
    fn test_reduce_noise_very_short_input() {
        // Input shorter than noise sample window should be returned as-is
        let samples = vec![0.5, -0.3, 0.1];
        let result = reduce_noise(&samples, 16000);
        assert_eq!(result, samples);
    }

    #[test]
    fn test_preprocess_all_silence_returns_empty() {
        // All-silence input: noise floor region + silent signal
        let samples = vec![0.0f32; 16000];
        let result = preprocess(&samples, 16000);
        assert!(result.is_empty());
    }
}
