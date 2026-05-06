use async_trait::async_trait;
use config::models::OllamaConfig;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        parameters::{FormatType, JsonStructure},
    },
    Ollama,
};
use serde::{Deserialize, Serialize};
use shared::{errors::Error, types::SoundomeResult, utils::json::generate_json_schema};

use crate::{prompts::prompt_with_data, AIBackend};

pub struct OllamaAI {
    client: Ollama,
    model: String,
}

impl OllamaAI {
    const DEFAULT_MODEL: &'static str = "llama3.2";
    const DEFAULT_HOST: &'static str = "http://localhost";
    const DEFAULT_PORT: u16 = 11434;

    pub fn new(config: &OllamaConfig) -> SoundomeResult<Self> {
        let host = config
            .host
            .clone()
            .unwrap_or_else(|| Self::DEFAULT_HOST.to_string());
        let port = config.port.unwrap_or(Self::DEFAULT_PORT);
        let model = config
            .model
            .clone()
            .unwrap_or_else(|| Self::DEFAULT_MODEL.to_string());

        let client = Ollama::new(host, port);

        Ok(Self { client, model })
    }
}

#[async_trait]
impl AIBackend for OllamaAI {
    async fn generate(&self, prompt: &str) -> SoundomeResult<String> {
        let request = ChatMessageRequest::new(
            self.model.clone(),
            vec![ChatMessage::user(prompt.to_string())],
        );

        let response = self
            .client
            .send_chat_messages(request)
            .await
            .map_err(|err| {
                Error::Network(format!(
                    "Ollama chat request failed for model {}: {}",
                    self.model, err
                ))
            })?;

        Ok(response.message.content)
    }

    async fn generate_with_data<
        T: schemars::JsonSchema + for<'de> Deserialize<'de> + Serialize + Send,
    >(
        &self,
        prompt: &str,
        data: T,
    ) -> SoundomeResult<T> {
        let full_prompt = prompt_with_data(prompt, &data)?;

        // Generate the JSON schema using schemars 1.x (required by ollama-rs).
        // We convert from a serde_json::Value produced by our schemars 0.8 utility.
        let schema_value = generate_json_schema(data);
        let schema: schemars_v1::Schema =
            serde_json::from_value(schema_value).map_err(Error::Json)?;
        let format = FormatType::StructuredJson(Box::new(JsonStructure::new_for_schema(schema)));

        let request =
            ChatMessageRequest::new(self.model.clone(), vec![ChatMessage::user(full_prompt)])
                .format(format);

        let response = self
            .client
            .send_chat_messages(request)
            .await
            .map_err(|err| {
                Error::Network(format!(
                    "Ollama structured chat request failed for model {}: {}",
                    self.model, err
                ))
            })?;

        serde_json::from_str::<T>(&response.message.content).map_err(Error::Json)
    }
}
