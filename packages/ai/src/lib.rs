pub mod backends;
pub mod prompts;

use async_trait::async_trait;
use backends::openrouter::OpenRouterAI;
use config::model::AiConfig;
use serde::{Deserialize, Serialize};
use shared::{errors::Error, types::SoundomeResult};

#[async_trait]
pub trait AIBackend {
    async fn generate(&self, prompt: &str) -> SoundomeResult<String>;
    async fn generate_with_data<
        T: schemars::JsonSchema + for<'de> Deserialize<'de> + Serialize + Send,
    >(
        &self,
        prompt: &str,
        data: T,
    ) -> SoundomeResult<T>;
}

pub enum AIBackendInstance {
    OpenRouter(OpenRouterAI),
}

#[async_trait]
impl AIBackend for AIBackendInstance {
    async fn generate(&self, prompt: &str) -> SoundomeResult<String> {
        match self {
            AIBackendInstance::OpenRouter(open_router) => open_router.generate(prompt).await,
        }
    }

    async fn generate_with_data<
        T: schemars::JsonSchema + for<'de> Deserialize<'de> + Serialize + Send,
    >(
        &self,
        prompt: &str,
        data: T,
    ) -> SoundomeResult<T> {
        match self {
            AIBackendInstance::OpenRouter(open_router) => {
                open_router.generate_with_data(prompt, data).await
            }
        }
    }
}

pub struct AIClient;

impl AIClient {
    pub fn new(config: &AiConfig) -> SoundomeResult<AIBackendInstance> {
        if !config.enabled {
            return Err(Error::Config("AI is not enabled".to_string()));
        }
        match config {
            _ if config.openrouter.is_some() => {
                let openrouter = OpenRouterAI::new(config.openrouter.as_ref().unwrap())?; // safe unwrap
                Ok(AIBackendInstance::OpenRouter(openrouter))
            }
            _ => Err(Error::NoAIBackend),
        }
    }
}
