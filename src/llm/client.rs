use crate::config::settings::LLMSettings;
use anyhow::Result;
use ollama_rs::{
    Ollama,
    generation::chat::{ChatMessage, request::ChatMessageRequest},
};
use tracing::{debug, info};
use url::Url;

#[allow(dead_code)]
pub struct LLMClient {
    client: Ollama,
    model: String,
}

impl LLMClient {
    #[allow(dead_code)]
    pub fn new(settings: &LLMSettings, model: &str) -> Self {
        let url = Url::parse(&settings.host).unwrap_or_else(|_| {
            eprintln!(
                "Invalid LLM host URL: {}, defaulting to http://localhost:11434",
                settings.host
            );
            Url::parse("http://localhost:11434").unwrap()
        });

        let host = format!(
            "{}://{}",
            url.scheme(),
            url.host_str().unwrap_or("localhost")
        );
        let port = url.port().unwrap_or(11434);

        let client = Ollama::new(host, port);

        Self {
            client,
            model: model.to_string(),
        }
    }

    #[allow(dead_code)]
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        info!(
            "Sending chat request to model: {} with {} messages",
            self.model,
            messages.len()
        );
        debug!("Messages: {:?}", messages);

        let request = ChatMessageRequest::new(self.model.clone(), messages);
        let response = self.client.send_chat_messages(request).await?;

        info!(
            "Received response from LLM ({} chars)",
            response.message.content.len()
        );
        debug!("Response content: {}", response.message.content);

        Ok(response.message.content)
    }

    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<bool> {
        // Simple check, maybe list models
        let _models = self.client.list_local_models().await?;
        Ok(true)
    }
}
