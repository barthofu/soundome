use std::time::Duration;

use async_trait::async_trait;
use config::{models::OpenRouterConfig, Config};
use openrouter_api::{
    models::{
        provider_preferences::{DataCollection, ProviderPreferences, ProviderSort},
        structured::{JsonSchemaConfig, JsonSchemaDefinition},
    },
    ChatCompletionRequest, Message, MessageContent, OpenRouterClient, Ready, Unconfigured,
};
use serde::{Deserialize, Serialize};
use shared::{
    errors::Error,
    types::SoundomeResult,
    utils::{json::generate_json_schema, with_default},
};

use crate::{prompts::prompt_with_data, AIBackend};

pub struct OpenRouterAI {
    client: OpenRouterClient<Ready>,
    model: String,
    #[allow(dead_code)]
    provider: Option<String>,
}

impl OpenRouterAI {
    const DEFAULT_MODEL: &str = "google/gemini-2.5-flash";

    pub fn new(openrouter_config: &OpenRouterConfig) -> SoundomeResult<Self> {
        let base_url = with_default(
            openrouter_config.base_url.clone(),
            "https://openrouter.ai/api/v1/".to_string(),
        );

        let client_builder = OpenRouterClient::<Unconfigured>::new()
            .with_base_url(base_url.clone())
            .map_err(|_| {
                Error::Network(format!(
                    "Failed to configure OpenRouter base URL: {}",
                    base_url
                ))
            })?
            .with_timeout(Duration::from_secs(with_default(
                openrouter_config.timeout,
                60,
            )))
            .with_http_referer("")
            .with_site_title("Soudome");

        // TODO: If the openrouter_api library supports proxy configuration,
        // it should be added here. For now, we document this limitation.
        if let Some(proxy) = Config::get().proxy.as_ref() {
            if proxy.enabled {
                tracing::warn!("Proxy configuration for OpenRouter is not yet supported by the openrouter_api library");
            }
        }

        let client = client_builder
            .with_api_key(&openrouter_config.api_key)
            .map_err(|err| {
                Error::Config(format!("Failed to configure OpenRouter API key: {}", err))
            })?;

        Ok(Self {
            client,
            model: with_default(
                openrouter_config.model.clone(),
                Self::DEFAULT_MODEL.to_string(),
            ),
            provider: openrouter_config.provider.clone(),
        })
    }

    // Utils

    fn get_message(&self, prompt: &str) -> Message {
        Message::text(openrouter_api::ChatRole::User, prompt.to_string())
    }

    fn get_provider(
        &self,
        require_parameters: Option<bool>,
        sort: Option<ProviderSort>,
    ) -> ProviderPreferences {
        ProviderPreferences::new()
            .with_allow_fallbacks(true)
            .with_data_collection(DataCollection::Deny) // Enable Zero Data Retention (ZDR)
            .with_require_parameters(require_parameters.unwrap_or(false))
            .with_sort(sort.unwrap_or(ProviderSort::Throughput))

        //     .with_order(self.provider.clone().map(|p| vec![p.clone()]).unwrap
        //         .into_iter()
        //         .chain(vec!["azure".to_string(), "anthropic".to_string(), "google".to_string()])
        //         .collect())
        //     .with_require_parameters(require_parameters.unwrap_or(true))
        //     .with_sort(sort.unwrap_or_else(|| "throughput".to_string()));

        // ProviderPreferences {
        //     allow_fallbacks: Some(true),
        //     allow: self.provider.clone().map(|p| vec![p]),
        //     order: self.provider.clone().map(|p| vec![p]),
        //     data_collection: None,
        //     ignore: None,
        //     quantizations: None,
        //     require_parameters,
        //     sort,
        //     ..Default::default()
        // }
    }
}

#[async_trait]
impl AIBackend for OpenRouterAI {
    async fn generate(&self, prompt: &str) -> SoundomeResult<String> {
        let messages = vec![self.get_message(prompt)];

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            provider: Some(self.get_provider(None, None)),
            response_format: None,
            stream: None,
            tools: None,
            models: None,
            transforms: None,
            ..Default::default()
        };

        let content = self
            .client
            .chat()
            .map_err(|err| {
                Error::Network(format!(
                    "Failed to initiate OpenRouter chat on model {}: {}",
                    self.model, err
                ))
            })?
            .chat_completion(request)
            .await
            .map_err(|err| {
                Error::Network(format!(
                    "Failed to get OpenRouter chat response on model {}: {}",
                    self.model, err
                ))
            })?
            .choices[0]
            .message
            .content
            .clone();

        match content {
            MessageContent::Text(text) => Ok(text),
            _ => Err(Error::Network(
                "Unexpected non-text response from OpenRouter".to_string(),
            )),
        }
    }

    async fn generate_with_data<
        T: schemars::JsonSchema + for<'de> Deserialize<'de> + Serialize + Send,
    >(
        &self,
        prompt: &str,
        data: T,
    ) -> SoundomeResult<T> {
        let prompt_with_data = prompt_with_data(prompt, &data)?;
        let messages = vec![self.get_message(&prompt_with_data)];

        let schema = JsonSchemaConfig {
            name: "tracks".to_string(),
            strict: true,
            schema: serde_json::from_value::<JsonSchemaDefinition>(generate_json_schema(data))
                .map_err(Error::Json)?,
        };
        // let response_format = ResponseFormat {
        //     format_type: "json_schema".to_string(),
        //     json_schema: Some(JsonSchema {
        //         name: "tracks".to_string(),
        //         strict: true,
        //         schema: generate_json_schema(data),
        //     }),
        // };

        self.client
            .structured()
            .map_err(|err| {
                Error::Network(format!(
                    "Failed to initiate OpenRouter structured generation on model {}: {}",
                    self.model, err
                ))
            })?
            .generate(&self.model, messages, schema)
            .await
            .map_err(|err| {
                Error::Network(format!(
                    "Failed to get OpenRouter structured response on model {}: {}",
                    self.model, err
                ))
            })
    }
}
