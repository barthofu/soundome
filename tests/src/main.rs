use std::time::Duration;

use ai::{AIBackend, AIClient};
use config::model::AppConfig;
use openrouter_api::{
    ChatCompletionRequest, JsonSchema, Message, OpenRouterClient, ProviderPreferences,
    ResponseFormat, Unconfigured,
};
use rsoundcloud::{PlaylistsApi, ResourceId, SoundCloudClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::errors::Error;

async fn openrouter(prompt: &str, api_key: &str) -> Result<String, Error> {
    let client = OpenRouterClient::<Unconfigured>::new()
        .with_base_url("https://openrouter.ai/api/v1/")
        .map_err(|err| Error::Custom(err.to_string()))?
        .with_timeout(Duration::from_secs(60))
        .with_http_referer("https://github.com/barthofu/soundome")
        .with_site_title("Soudome")
        .with_api_key(api_key)
        .map_err(|err| Error::Custom(err.to_string()))?;

    let messages = vec![Message {
        role: "user".to_string(),
        content: prompt.to_string(),
        name: None,
        tool_calls: None,
    }];

    // let provider_preferences = to_string(&json!({
    //     "order": ["Mistral"],
    //     "allowFallbacks": true
    // })).unwrap();

    let provider = ProviderPreferences {
        allow_fallbacks: Some(true),
        order: Some(vec!["Google AI Studio".to_string()]),
        data_collection: None,
        ignore: None,
        quantizations: None,
        require_parameters: Some(true),
        sort: Some("throughput".to_string()),
    };

    let response_format = ResponseFormat {
        format_type: "json_schema".to_string(),
        json_schema: Some(JsonSchema {
            name: "tracks".to_string(),
            strict: true,
            schema: json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string"
                        },
                        "artists": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["title", "artists"],
                    "additionalProperties": false
                }
            }),
        }),
    };

    // Build the chat completion request.
    let request = ChatCompletionRequest {
        // model: "mistralai/mistral-small-3.1-24b-instruct".to_string(),
        model: "google/gemini-flash-1.5-8b".to_string(),
        messages,
        stream: None,
        response_format: Some(response_format),
        tools: None,
        provider: Some(provider),
        models: None,
        transforms: None,
    };

    let chat_api = client
        .chat()
        .map_err(|err| Error::Custom(err.to_string()))?;
    chat_api
        .chat_completion(request)
        .await
        .map(|response| {
            // println!("{:#?}", response);
            // println!("Assistant says: {}", response.choices[0].message.content);
            response.choices[0].message.content.clone()
        })
        .map_err(|err| Error::Custom(err.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct Track {
    title: String,
    artists: Vec<String>,
}

#[dotenvy::load(path = "./.env", required = true)]
#[tokio::main]
async fn main() {
    let config = AppConfig::new().unwrap();

    // let orchestrator = orchestrator::Orchestrator::new(config);

    // let track_id = ResourceId::Url(.to_string());

    // let schema = generate(vec![Track {
    //     title: "Test".to_string(),
    //     artists: vec!["Test".to_string()]
    // }]);
    // println!("{}", schema);

    let client = SoundCloudClient::default().await.unwrap();
    let tracks = client
        .get_playlist_tracks(ResourceId::Url(
            "https://soundcloud.com/bartho-az/sets/euphoria-part-4".to_string(),
        ))
        .await
        .unwrap();
    let tracks = tracks
        .iter()
        .map(|t| Track {
            title: t.track.title.clone(),
            artists: vec![t.user.username.clone()],
        })
        .collect::<Vec<Track>>();

    // let openrouter = ai::backends::openrouter::OpenRouterAI::new(config.openrouter.unwrap()).unwrap();
    let ai_client = ai::AIClient::new(config).unwrap();
    //     let prompt = format!("
    // Clean and enhance track metadata from a JSON array of objects with this format:
    // ```json
    // {{
    //     \"title\": \"<raw track title>\",
    //     \"artists\": [\"<original uploader username>\"]
    // }}
    // ```
    // Tasks:
    // 1. Clean title → Remove catalog numbers, platform tags, redundant info (Original Mix, Remastered), and artist names.
    // 2. Extract artist → Detect names in title, extract featured artists (Ft., vs, &, b2b), and deduplicate.

    // Input:
    // ```json
    // {}
    // ```
    // ", serde_json::to_string(&tracks).unwrap());
    //     let processed_tracks = openrouter.generate_with_data(&prompt, tracks).await.unwrap();
    let processed_tracks = ai_client
        .generate_with_data(ai::prompts::CLEAN_TRACK_TITLE_AND_ARTIST_NAME, tracks)
        .await
        .unwrap();
    println!("{:#?}", processed_tracks);

    //     let openrouter_api_key = config.openrouter.as_ref().unwrap().api_key.clone();
    //     let prompt = format!("
    // Clean and enhance track metadata from a JSON array of objects with this format:
    // ```json
    // {{
    //     \"title\": \"<raw track title>\",
    //     \"artists\": [\"<original uploader username>\"]
    // }}
    // ```
    // Tasks:

    // 1. Clean title → Remove catalog numbers, platform tags, redundant info (Original Mix, Remastered), and artist names.
    // 2. Extract artist → Detect names in title, extract featured artists (Ft., vs, &, b2b), and deduplicate.

    // Input:

    // ```json
    // {}
    // ```
    // ", tracks_serialized);
    //     // println!("{}", prompt);
    //     let result = openrouter(&prompt, &openrouter_api_key).await.unwrap();
    //     let tracks = serde_json::from_str::<Vec<Track>>(&result).unwrap();
    //     println!("{:#?}", tracks);

    // println!("{:#?}", tracks.iter().map(|t| format_metadata(&t.track.title, &t.user.username)).collect::<Vec<(String, Vec<String>)>>());

    // let playlist_url = "https://open.spotify.com/playlist/22HjWHbry4q3DzVMOhRqBU?si=ca4f7ddb9afd4ed7";
    // let _ = orchestrator.download_playlist_from_url(playlist_url).await.map_err(|e| {
    //     eprintln!("Error: {:?}", e);
    //     std::process::exit(1);
    // });

    // let track_url = "https://soundcloud.com/jeannindamix/mamakkat-jeannine-synaptic-highway";
    // let _ = orchestrator.download_track_from_url(track_url).await.map_err(|e| {
    //     eprintln!("Error: {:?}", e);
    //     std::process::exit(1);
    // });
}
