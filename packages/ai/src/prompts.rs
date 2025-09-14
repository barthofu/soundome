use std::collections::HashMap;

use serde::Serialize;
use shared::{types::SoundomeResult, utils::string::render_template};

pub fn clean_track_title_and_artist_name(single_track: bool) -> SoundomeResult<String> {
    let template: &str = r#"
        Refine and standardize track metadata from a JSON {{ if single_track }}object{{ else }}array{{ endif }} of objects with this format:
        ```json
        \{
            "id": "<track id>",
            "title": "<raw track title>",
            "artists": ["<original uploader username>"]
        \}
        ```
        Tasks:
        1. Extract artist → Detect names in title if it is present, extract featured artists (Ft., vs, &, b2b), and deduplicate.
        2. Clean title → Remove catalog numbers, platform tags, redundant info (Original Mix, Remastered), and artist names.
        {{ if single_track }}{{ else }}3. Keep the tracks in the same order.{{ endif }}

        Precisions:
        - If the track seems to be a remix or cover, keep the original remixed artist in the title (and don't put it in the artists field!), as well as the remix mention. 
        - Always prioritize artist names found in the title over the uploader username.
        - If no artist name can be found, keep the original uploader username as the only artist.
        - Capitalize artist names and titles properly.
        - Return the cleaned data in the same format as the input, with the same `id` values.

        For some context, this data is sourced from SoundCloud, where titles often include extra information and the artist field may not always reflect the actual track artist.
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
