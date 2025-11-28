use anyhow::Result;
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use crate::config::settings::LLMSettings;

#[allow(dead_code)]
pub struct LLMClient {
    client: Ollama,
    model: String,
}

impl LLMClient {
    #[allow(dead_code)]
    pub fn new(settings: &LLMSettings, model: &str) -> Self {
        let _host = &settings.host;
        // Parse host to get port if needed, but ollama-rs defaults to localhost:11434
        // For now, we assume the host string is sufficient or we might need to parse it.
        // ollama-rs constructor takes host and port.
        // Let's assume standard default for now or parse from settings.
        // A simple implementation:
        let client = Ollama::default(); 
        // TODO: Configure client with host from settings if it differs from default
        
        Self {
            client,
            model: model.to_string(),
        }
    }

    #[allow(dead_code)]
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());
        let response = self.client.generate(request).await?;
        Ok(response.response)
    }
    
    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<bool> {
        // Simple check, maybe list models
        let _models = self.client.list_local_models().await?;
        Ok(true)
    }
}
