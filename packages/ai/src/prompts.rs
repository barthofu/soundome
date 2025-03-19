use serde::Serialize;
use shared::types::SoundomeResult;

pub const CLEAN_TRACK_TITLE_AND_ARTIST_NAME: &str = r#"
Clean and enhance track metadata from a JSON array of objects with this format:
```json
{{
    "title": "<raw track title>",
    "artists": ["<original uploader username>"]
}}
```
Tasks:
1. Clean title → Remove catalog numbers, platform tags, redundant info (Original Mix, Remastered), and artist names.
2. Extract artist → Detect names in title, extract featured artists (Ft., vs, &, b2b), and deduplicate.
"#;

// Utils

const PROMPT_WITH_DATA: &str = r#"
{PROMPT}

Input:
```json
{DATA}
```
"#;

pub fn prompt_with_data<T: Serialize>(prompt: &str, data: T) -> SoundomeResult<String> {
    serde_json::to_string(&data)
        .map(|data| {
            PROMPT_WITH_DATA
                .replace("{PROMPT}", prompt)
                .replace("{DATA}", &data)
        })
        .map_err(Into::into)
}
