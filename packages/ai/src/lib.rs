pub mod backends;
pub mod prompts;

use async_trait::async_trait;
use backends::{ollama::OllamaAI, openrouter::OpenRouterAI};
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
    Ollama(OllamaAI),
    /// Tries each backend in order. The first successful response is returned.
    Fallback(Vec<AIBackendInstance>),
}

#[async_trait]
impl AIBackend for AIBackendInstance {
    async fn generate(&self, prompt: &str) -> SoundomeResult<String> {
        match self {
            AIBackendInstance::OpenRouter(b) => b.generate(prompt).await,
            AIBackendInstance::Ollama(b) => b.generate(prompt).await,
            AIBackendInstance::Fallback(backends) => {
                let mut last_err = Error::NoAIBackend;
                for backend in backends {
                    match backend.generate(prompt).await {
                        Ok(result) => return Ok(result),
                        Err(err) => {
                            tracing::warn!(
                                "AI backend failed, trying next in fallback chain: {}",
                                err
                            );
                            last_err = err;
                        }
                    }
                }
                Err(last_err)
            }
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
            AIBackendInstance::OpenRouter(b) => b.generate_with_data(prompt, data).await,
            AIBackendInstance::Ollama(b) => b.generate_with_data(prompt, data).await,
            AIBackendInstance::Fallback(backends) => {
                if backends.is_empty() {
                    return Err(Error::NoAIBackend);
                }

                // Pre-serialize data before it may be consumed by the first backend,
                // so that remaining backends can still receive a prompt with the data.
                let pre_built_prompt = crate::prompts::prompt_with_data(prompt, &data)?;

                let mut backends_iter = backends.iter();

                // First backend: use generate_with_data for proper structured output.
                if let Some(first) = backends_iter.next() {
                    match first.generate_with_data(prompt, data).await {
                        Ok(result) => return Ok(result),
                        Err(err) => {
                            tracing::warn!(
                                "Primary AI backend failed for structured generation, \
                                 trying fallback(s) with plain JSON mode: {}",
                                err
                            );
                            // data is consumed; fall back to plain generate for remaining backends.
                            let mut last_err = err;
                            for backend in backends_iter {
                                match backend.generate(&pre_built_prompt).await {
                                    Ok(text) => {
                                        return serde_json::from_str::<T>(&text)
                                            .map_err(Error::Json);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Fallback AI backend also failed: {}", e);
                                        last_err = e;
                                    }
                                }
                            }
                            return Err(last_err);
                        }
                    }
                }

                Err(Error::NoAIBackend)
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

        let mut backends: Vec<AIBackendInstance> = Vec::new();

        for provider_name in &ai_config.provider_order {
            match provider_name.as_str() {
                "ollama" => match &ai_config.ollama {
                    Some(cfg) => match OllamaAI::new(cfg) {
                        Ok(ollama) => backends.push(AIBackendInstance::Ollama(ollama)),
                        Err(err) => {
                            tracing::warn!("Failed to configure Ollama backend: {}", err)
                        }
                    },
                    None => tracing::warn!(
                        "'ollama' is listed in ai.provider_order but [ai.ollama] is not configured"
                    ),
                },
                "openrouter" => match &ai_config.openrouter {
                    Some(cfg) => match OpenRouterAI::new(cfg) {
                        Ok(openrouter) => {
                            backends.push(AIBackendInstance::OpenRouter(openrouter))
                        }
                        Err(err) => {
                            tracing::warn!("Failed to configure OpenRouter backend: {}", err)
                        }
                    },
                    None => tracing::warn!(
                        "'openrouter' is listed in ai.provider_order but [ai.openrouter] is not configured"
                    ),
                },
                unknown => {
                    tracing::warn!(
                        "Unknown AI provider '{}' in ai.provider_order, skipping",
                        unknown
                    );
                }
            }
        }

        if backends.is_empty() {
            return Err(Error::NoAIBackend);
        }

        if backends.len() == 1 {
            return Ok(backends.remove(0));
        }

        Ok(AIBackendInstance::Fallback(backends))
    }
}
