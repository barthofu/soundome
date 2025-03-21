use std::collections::HashMap;

use serde::Serialize;
use shared::{types::SoundomeResult, utils::string::render_template};

pub fn clean_track_title_and_artist_name(single_track: bool) -> SoundomeResult<String> {
    let template: &str = r#"
        Clean and enhance track metadata from a JSON {{ if single_track }}object{{ else }}array{{ endif }} of objects with this format:
        ```json
        \{
            "id": "<track id>",
            "title": "<raw track title>",
            "artists": ["<original uploader username>"]
        \}
        ```
        Tasks:
        1. Clean title → Remove catalog numbers, platform tags, redundant info (Original Mix, Remastered), and artist names.
        2. Extract artist → Detect names in title, extract featured artists (Ft., vs, &, b2b), and deduplicate.
        "#;

    let context = HashMap::from([("single_track", if single_track { "true" } else { "false" })]);
    render_template(template, &context)
}

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
