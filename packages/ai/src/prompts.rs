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
        1. Extract artist → Detect names in title if present, extract featured artists (Ft., vs, &, b2b), and deduplicate.
        2. Clean title → Remove catalog numbers, platform tags, redundant info (Original Mix, Remastered), and artist names.
        {{ if single_track }}{{ else }}3. Keep the tracks in the same order and preserve every `id` exactly as given.{{ endif }}

        Precisions:
        - If the track seems to be a remix or cover, keep the original remixed artist in the title (and don't put it in the artists field!), as well as the remix mention.
        - Always prioritize artist names found in the title over the uploader username.
        - If no artist name can be found in the title, keep the original uploader username as the only artist.
        - Capitalize artist names and titles properly.
        - Return the cleaned data in the same format as the input, with the same `id` values.

        Disambiguating "A - B" titles:
        - SoundCloud titles frequently follow the pattern "<part A> - <part B>" where either side can be the artist and the other side the title.
        - Use the input `artists` field (the uploader username) as the tie-breaker: whichever side matches the uploader username (allowing spaces vs underscores, casing differences) is the artist, and the OTHER side is the title.
        - Example: input `\{ "title": "GRÄV - Habits Sales & VYRAX", "artists": ["Habits_Sales"] \}` must become `\{ "title": "GRÄV", "artists": ["Habits Sales", "VYRAX"] \}` — because "Habits Sales" matches the uploader "Habits_Sales", so it is the artist, and "GRÄV" is the title.
        - When neither side matches the uploader, fall back to the classic "<artist> - <title>" convention.

        Strict rules:
        - Never invent, guess, or hallucinate an artist or title fragment. Every artist name in the output MUST already appear (up to spacing, casing, and punctuation) in the input `title` or `artists` of that same `id`.
        - Never rename an artist to a similar-looking one from a different track.

        {{ if single_track }}{{ else }}CROSS-TRACK ISOLATION (critical): Each object in the array is a fully independent track. Every `id` is its own isolated context: never reuse, borrow, or infer an artist name from another track's `title` or `artists` field, even if the names look similar, share letters, or repeat across the batch. Base every extracted artist strictly on the `title` and `artists` of that same `id` only. If artist "Zorven" appears in track A alongside "ZadernaS", and track B has a completely different uploader like "Mylacid", the output for track A must still contain "ZadernaS" (never "Mylacid"). Cross-track artist leakage is a critical error.{{ endif }}

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
