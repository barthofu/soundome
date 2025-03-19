use std::time::Duration;

use async_trait::async_trait;
use config::model::OpenRouterConfig;
use openrouter_api::{ChatCompletionRequest, JsonSchema, Message, OpenRouterClient, ProviderPreferences, Ready, ResponseFormat, Unconfigured};
use serde::{Deserialize, Serialize};
use shared::{errors::Error, types::SoundomeResult, utils::{json::generate_json_schema, with_default}};

use crate::{prompts::prompt_with_data, AIBackend};


pub struct OpenRouterAI {
    client: OpenRouterClient<Ready>,
    model: String,
    provider: Option<String>,
}

impl OpenRouterAI {
    const DEFAULT_MODEL: &str = "google/gemini-flash-1.5-8b";

    pub fn new(config: OpenRouterConfig) -> SoundomeResult<Self> {

        let base_url = with_default(config.base_url, "https://openrouter.ai/api/v1/".to_string());
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url(base_url.clone())
                .map_err(|_| Error::Network(format!("Failed to configure OpenRouter base URL: {}", base_url)))?
            .with_timeout(Duration::from_secs(with_default(config.timeout, 60)))
            .with_http_referer("")
            .with_site_title("Soudome")
            .with_api_key(&config.api_key)
                .map_err(|err| Error::Config(format!("Failed to configure OpenRouter API key: {}", err)))?;

        Ok(Self {
            client: client,
            model: with_default(config.model, Self::DEFAULT_MODEL.to_string()),
            provider: config.provider,
        })
    }

    // Utils

    fn get_message(&self, prompt: &str) -> Message {
        Message {
            role: "user".to_string(),
            content: prompt.to_string(),
            name: None,
            tool_calls: None,
        }
    }

    fn get_provider(&self, require_parameters: Option<bool>, sort: Option<String>) -> ProviderPreferences {
        ProviderPreferences {
            allow_fallbacks: Some(true),
            order: self.provider.clone().map(|p| vec![p]),
            data_collection: None,
            ignore: None,
            quantizations: None,
            require_parameters,
            sort,
        }
    }
}

#[async_trait]
impl AIBackend for OpenRouterAI {
    async fn generate(&self, prompt: &str) -> SoundomeResult<String> {

        let messages = vec![self.get_message(prompt)];

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            provider: Some(self.get_provider(None, Some("throughput".to_string()))),
            response_format: None,
            stream: None,
            tools: None,
            models: None,
            transforms: None,
        };

        self.client.chat()
            .map_err(|err| Error::Network(format!("Failed to initiate OpenRouter chat on model {}: {}", self.model, err)))?
            .chat_completion(request)
            .await
            .map(|response| response.choices[0].message.content.clone())
            .map_err(|err| Error::Network(format!("Failed to get OpenRouter chat response on model {}: {}", self.model, err)))
    }

    async fn generate_with_data<T: schemars::JsonSchema + for<'de> Deserialize<'de> + Serialize + Send>(&self, prompt: &str, data: T) -> SoundomeResult<T> {

        let prompt_with_data = prompt_with_data(prompt, &data)?;
        let messages = vec![
            self.get_message(&prompt_with_data)
        ];

        let response_format1 = ResponseFormat {
            format_type: "json_schema".to_string(),
            json_schema: Some(JsonSchema {
                name: "tracks".to_string(),
                strict: true,
                schema: generate_json_schema(data),
            })
        };

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            provider: Some(self.get_provider(Some(true), Some("throughput".to_string()))),
            response_format: Some(response_format1),
            stream: None,
            tools: None,
            models: None,
            transforms: None,
        };

        self.client.chat()
            .map_err(|err| Error::Network(format!("Failed to initiate OpenRouter chat on model {}: {}", self.model, err)))?
            .chat_completion(request)
            .await
            .map(|response|
                serde_json::from_str::<T>(&response.choices[0].message.content)
                    .map_err(Error::Json)
                )
            .map_err(|err| Error::Network(format!("Failed to get OpenRouter chat response on model {}: {}", self.model, err)))?
    }

}
