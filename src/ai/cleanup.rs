use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

pub async fn cleanup_text(
    raw_text: &str,
    context: Option<&str>,
    custom_instructions: Option<&str>,
    api_key: &str,
) -> Result<String, String> {
    let mut system_prompt = String::from(
        "You are a dictation cleanup assistant. Clean up the transcribed speech while \
         preserving the speaker's intent. Fix filler words, grammar, and punctuation. \
         Match the tone of the surrounding context if provided.",
    );

    if let Some(instructions) = custom_instructions {
        if !instructions.is_empty() {
            system_prompt.push_str(&format!("\n\nAdditional instructions: {}", instructions));
        }
    }

    let mut user_content = String::new();
    if let Some(ctx) = context {
        if !ctx.is_empty() {
            user_content.push_str(&format!("Context (text before cursor): {}\n", ctx));
        }
    }
    user_content.push_str(&format!(
        "Raw transcription: {}\n\nReturn only the cleaned text, nothing else.",
        raw_text
    ));

    let request = ChatRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_content,
            },
        ],
        temperature: 0.3,
        max_tokens: 2048,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Cleanup API error: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Cleanup API error: {}", error_text));
    }

    let result: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse cleanup response: {}", e))?;

    result
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "No response from cleanup API".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "test system".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "test user".to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 2048,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4o-mini"));
        assert!(json.contains("test system"));
    }

    #[test]
    fn test_system_prompt_includes_custom_instructions() {
        // Simulate what cleanup_text builds for the system prompt
        let mut system_prompt = String::from(
            "You are a dictation cleanup assistant. Clean up the transcribed speech while \
             preserving the speaker's intent. Fix filler words, grammar, and punctuation. \
             Match the tone of the surrounding context if provided.",
        );
        let custom_instructions = Some("Always use formal English");
        if let Some(instructions) = custom_instructions {
            if !instructions.is_empty() {
                system_prompt.push_str(&format!("\n\nAdditional instructions: {}", instructions));
            }
        }
        assert!(system_prompt.contains("Additional instructions: Always use formal English"));
    }

    #[test]
    fn test_user_content_includes_context_when_provided() {
        let mut user_content = String::new();
        let context = Some("Dear Sir,");
        if let Some(ctx) = context {
            if !ctx.is_empty() {
                user_content.push_str(&format!("Context (text before cursor): {}\n", ctx));
            }
        }
        user_content.push_str(&format!(
            "Raw transcription: {}\n\nReturn only the cleaned text, nothing else.",
            "hello world"
        ));
        assert!(user_content.contains("Context (text before cursor): Dear Sir,"));
        assert!(user_content.contains("Raw transcription: hello world"));
    }

    #[test]
    fn test_user_content_excludes_context_when_none() {
        let mut user_content = String::new();
        let context: Option<&str> = None;
        if let Some(ctx) = context {
            if !ctx.is_empty() {
                user_content.push_str(&format!("Context (text before cursor): {}\n", ctx));
            }
        }
        user_content.push_str(&format!(
            "Raw transcription: {}\n\nReturn only the cleaned text, nothing else.",
            "hello world"
        ));
        assert!(!user_content.contains("Context"));
        assert!(user_content.starts_with("Raw transcription:"));
    }
}
