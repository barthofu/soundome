use rocket::serde::Serialize;
use schemars::JsonSchema;

#[derive(Serialize, JsonSchema)]
pub struct Success {
    pub success: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct Message {
    pub message: String,
}

/// Generic pagination envelope returned by every paginated library list route
/// (`/tracks`, `/albums`, `/artists`, `/playlists`).
#[derive(Serialize, JsonSchema)]
pub struct PageDto<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Simple `{ count }` envelope for cheap polling endpoints (pending
/// validations, active tasks) that only need a number, not the full payload.
#[derive(Serialize, JsonSchema)]
pub struct CountDto {
    pub count: i64,
}

// ================================================================================================
// Pagination query-param helpers
// ================================================================================================
//
// Each paginated list route (`/tracks`, `/albums`, `/artists`, `/playlists`)
// declares its own `page`/`page_size`/`q`/`sort_by`/`sort_dir`/`filter`
// `Option<...>` query parameters directly in its route signature (the same
// pattern already used by `playlists::delete`'s `delete_tracks: Option<bool>`)
// rather than a shared `FromForm` struct — a custom `FromForm` type used
// alongside `#[openapi]` would additionally need to satisfy rocket_okapi's
// query-guard schema traits, which nothing else in this codebase currently
// exercises. These free functions keep the per-route boilerplate to one line
// each while avoiding that risk.

pub const DEFAULT_PAGE_SIZE: i64 = 60;

pub fn resolve_page(page: Option<i64>) -> i64 {
    page.unwrap_or(1).max(1)
}

pub fn resolve_page_size(page_size: Option<i64>) -> i64 {
    page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, 500)
}

pub fn resolve_offset(page: i64, page_size: i64) -> i64 {
    (page - 1) * page_size
}

/// `true` when `sort_dir=desc` was requested; defaults to ascending.
pub fn is_desc(sort_dir: &Option<String>) -> bool {
    matches!(sort_dir.as_deref(), Some("desc"))
}

/// Trims a raw `q` query param and turns blank input into `None`, so an empty
/// search box doesn't add a no-op `WHERE` filter.
pub fn normalize_search(q: &Option<String>) -> Option<String> {
    q.as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
