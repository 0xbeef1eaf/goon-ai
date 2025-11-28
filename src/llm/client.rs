use anyhow::Result;
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use crate::config::settings::LLMSettings;
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
            eprintln!("Invalid LLM host URL: {}, defaulting to http://localhost:11434", settings.host);
            Url::parse("http://localhost:11434").unwrap()
        });

        let host = format!("{}://{}", url.scheme(), url.host_str().unwrap_or("localhost"));
        let port = url.port().unwrap_or(11434);

        let client = Ollama::new(host, port);
        
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
