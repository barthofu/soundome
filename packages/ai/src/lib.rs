pub mod backends;
pub mod prompts;

use async_trait::async_trait;
use backends::openrouter::OpenRouterAI;
use config::Config;
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
    pub fn new() -> SoundomeResult<AIBackendInstance> {
        let ai_config = Config::get().ai.clone();

        if !ai_config.enabled {
            return Err(Error::Config("AI is not enabled".to_string()));
        }

        if let Some(ref openrouter_config) = ai_config.openrouter {
            let openrouter = OpenRouterAI::new(openrouter_config)?;
            Ok(AIBackendInstance::OpenRouter(openrouter))
        } else {
            Err(Error::NoAIBackend)
        }
    }
}
