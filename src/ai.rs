use std::path::PathBuf;

use anyhow::Context;
use openai_api_rs::v1::{
    api::OpenAIClient,
    audio::{AudioTranscriptionRequest, WHISPER_1},
    chat_completion::{self, ChatCompletionMessage, ChatCompletionRequest, MessageRole},
    common::GPT4_O_MINI,
};

const SUMMARY_INSTRUCTIONS: &str = r#"You are an expert for social media posts & working with texts in any language. Sometimes you get a text in German, English, Spanish or other languages.

You get a text from a user and you should make a social media draft out of it.
Your responses should ALWAYS be IN the language of the USERS TEXT.

Whenever you get a text you should do the following:

1. Properly format the given text, making it readable, by adding appropriate commas, breaks etc.
2. Make sure the post has a punchline.
3. Don't use hashtags.
4. Don't overdo it with emojis.
5. Make sure the post is not too long.
6. Make sure the post is not too short.
7. Make sure the post is not too boring.
8. Make sure you don't use typical AI words like: driven, motivated, inspired, delve, into the future 
"#;

const FORMAT_ONLY_INSTRUCTIONS: &str = r#"You are an expert for formatting text in any language. Sometimes you get a text in German, English, Spanish or other languages.

You get a text from a user and you should format it for social media.
Your responses should ALWAYS be IN the language of the USERS TEXT.

Whenever you get a text you should do the following:

1. Properly format the given text, making it readable, by adding appropriate commas, breaks etc.
2. Don't change the content or meaning of the text.
3. Don't add or remove any information.
4. Don't use hashtags.
5. Don't add emojis.
6. Keep the original tone and style of the text.
"#;

pub async fn transcribe_voice_note(path: PathBuf, api_key: String) -> anyhow::Result<String> {
    // TODO: error handling and keeping the client static
    let client = OpenAIClient::builder()
        .with_api_key(api_key)
        .build()
        .ok()
        .with_context(|| "")?;

    let path = format!("{}", path.display());
    dbg!(path.clone());
    let req = AudioTranscriptionRequest::new(path, WHISPER_1.to_string());

    let result = client.audio_transcription(req).await?;

    Ok(result.text)
}

pub async fn make_summary(
    from_user: String,
    text: String,
    api_key: String,
    rewrite_enabled: bool,
) -> anyhow::Result<String> {
    let client = OpenAIClient::builder()
        .with_api_key(api_key)
        .build()
        .ok()
        .with_context(|| "")?;

    let instructions = if rewrite_enabled {
        SUMMARY_INSTRUCTIONS
    } else {
        FORMAT_ONLY_INSTRUCTIONS
    };

    let msgs = vec![
        ChatCompletionMessage {
            role: MessageRole::system,
            content: chat_completion::Content::Text(instructions.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
        ChatCompletionMessage {
            role: MessageRole::user,
            content: chat_completion::Content::Text(text),
            name: Some(from_user),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let req = ChatCompletionRequest::new(GPT4_O_MINI.to_string(), msgs);

    let result = client.chat_completion(req).await?;

    let last_msg = result.choices.last();

    match last_msg {
        None => Ok("No summary available".to_string()),
        Some(last_msg) => Ok(last_msg
            .message
            .content
            .clone()
            .unwrap_or("No content".to_string())),
    }
}
