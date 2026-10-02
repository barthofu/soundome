//! Declarative tool catalog. Every tool maps onto one Soundome API route
//! (except `get_status`, which aggregates a few cheap GETs).

use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::client::ApiClient;

#[derive(Clone, Copy, PartialEq)]
pub enum Loc {
    Path,
    Query,
    Body,
}

#[derive(Clone, Copy)]
pub enum Ty {
    Int,
    Str,
    Bool,
    IntList,
    StrList,
}

pub struct Param {
    pub name: &'static str,
    pub ty: Ty,
    pub desc: &'static str,
    pub loc: Loc,
    pub required: bool,
}

impl Param {
    fn req(mut self) -> Self {
        self.required = true;
        self
    }
}

pub enum Kind {
    Http { method: Method, path: &'static str },
    Status,
}

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: Kind,
    pub params: Vec<Param>,
    pub read_only: bool,
    pub destructive: bool,
}

fn pid(name: &'static str, desc: &'static str) -> Param {
    Param {
        name,
        ty: Ty::Int,
        desc,
        loc: Loc::Path,
        required: true,
    }
}
fn pstr(name: &'static str, desc: &'static str) -> Param {
    Param { name, ty: Ty::Str, desc, loc: Loc::Path, required: true }
}
fn qp(name: &'static str, ty: Ty, desc: &'static str) -> Param {
    Param {
        name,
        ty,
        desc,
        loc: Loc::Query,
        required: false,
    }
}
fn bp(name: &'static str, ty: Ty, desc: &'static str) -> Param {
    Param {
        name,
        ty,
        desc,
        loc: Loc::Body,
        required: false,
    }
}

fn http(
    name: &'static str,
    description: &'static str,
    method: Method,
    path: &'static str,
    params: Vec<Param>,
    destructive: bool,
) -> Tool {
    Tool {
        name,
        description,
        read_only: method == Method::GET,
        destructive,
        kind: Kind::Http { method, path },
        params,
    }
}

fn paging(extra_sort: &'static str) -> Vec<Param> {
    vec![
        qp("page", Ty::Int, "1-based page number (default 1)"),
        qp("page_size", Ty::Int, "Items per page (default 60, max 500)"),
        qp("q", Ty::Str, "Free-text search"),
        qp("sort_by", Ty::Str, extra_sort),
        qp("sort_dir", Ty::Str, "`asc` (default) or `desc`"),
    ]
}

fn merge_params() -> Vec<Param> {
    vec![
        bp(
            "source_ids",
            Ty::IntList,
            "IDs merged into the target, then deleted",
        )
        .req(),
        bp("target_id", Ty::Int, "ID of the entity to keep").req(),
    ]
}

pub fn catalog() -> Vec<Tool> {
    use Method as M;
    let mut track_list = paging("Sort column (e.g. title, date)");
    track_list.push(qp(
        "filter",
        Ty::Str,
        "`ok` (validated), `pending` (needs validation) or omit",
    ));

    vec![
        Tool {
            name: "get_status",
            description: "Server version, available providers, pending validation count and active task count.",
            kind: Kind::Status,
            params: vec![],
            read_only: true,
            destructive: false,
        },
        // ---- Tracks ----
        http("list_tracks", "List/search tracks (paginated).", M::GET, "/tracks", track_list, false),
        http("get_recent_tracks", "Most recently added tracks.", M::GET, "/tracks/recent", vec![qp("limit", Ty::Int, "Max tracks to return")], false),
        http("get_track", "Get one track with its artists, album and references.", M::GET, "/tracks/{id}", vec![pid("id", "Track ID")], false),
        http("get_track_references", "List a track's references (Source/Provider/Metadata).", M::GET, "/tracks/{id}/references", vec![pid("id", "Track ID")], false),
        http(
            "add_track_reference",
            "Add a reference to a track. platform/external_id are inferred from external_url when omitted.",
            M::POST, "/tracks/{id}/references",
            vec![
                pid("id", "Track ID"),
                bp("ref_type", Ty::Str, "One of: Source, Provider, Metadata, Reference").req(),
                bp("platform", Ty::Str, "Platform name (optional, inferred from URL)"),
                bp("external_id", Ty::Str, "External ID (optional)"),
                bp("external_url", Ty::Str, "External URL"),
            ],
            false,
        ),
        http("remove_track_reference", "Remove a reference from a track.", M::DELETE, "/tracks/{id}/references/{ref_id}", vec![pid("id", "Track ID"), pid("ref_id", "Reference ID")], true),
        http(
            "update_track",
            "Update track metadata (only provided fields change).",
            M::PATCH, "/tracks/{id}",
            vec![
                pid("id", "Track ID"),
                bp("title", Ty::Str, "Title"),
                bp("artists", Ty::StrList, "Replace artists by name"),
                bp("album_title", Ty::Str, "Album title"),
                bp("genre", Ty::Str, "Genre"),
                bp("date", Ty::Str, "Release date"),
                bp("track_number", Ty::Int, "Track number"),
                bp("disc_number", Ty::Int, "Disc number"),
                bp("label", Ty::Str, "Label"),
                bp("cover", Ty::Str, "Cover URL"),
            ],
            false,
        ),
        http("merge_tracks", "Merge duplicate tracks into a target track. Sources are deleted.", M::POST, "/tracks/merge", merge_params(), true),
        http("delete_track", "Delete a track.", M::DELETE, "/tracks/{id}", vec![pid("id", "Track ID")], true),
        // ---- Albums ----
        http("list_albums", "List/search albums (paginated).", M::GET, "/albums", paging("Sort column (e.g. title, date)"), false),
        http("get_album", "Get one album.", M::GET, "/albums/{id}", vec![pid("id", "Album ID")], false),
        http("get_album_tracks", "List tracks of an album.", M::GET, "/albums/{id}/tracks", vec![pid("id", "Album ID")], false),
        http(
            "update_album", "Update album metadata.", M::PATCH, "/albums/{id}",
            vec![
                pid("id", "Album ID"),
                bp("title", Ty::Str, "Title"),
                bp("date", Ty::Str, "Release date"),
                bp("cover", Ty::Str, "Cover URL"),
            ],
            false,
        ),
        http("merge_albums", "Merge duplicate albums into a target album. Sources are deleted.", M::POST, "/albums/merge", merge_params(), true),
        http("delete_album", "Delete an album.", M::DELETE, "/albums/{id}", vec![pid("id", "Album ID")], true),
        // ---- Artists ----
        http("list_artists", "List/search artists (paginated).", M::GET, "/artists", paging("Sort column (e.g. name)"), false),
        http("get_artist", "Get one artist.", M::GET, "/artists/{id}", vec![pid("id", "Artist ID")], false),
        http("get_artist_tracks", "List tracks of an artist.", M::GET, "/artists/{id}/tracks", vec![pid("id", "Artist ID")], false),
        http("get_artist_albums", "List albums of an artist.", M::GET, "/artists/{id}/albums", vec![pid("id", "Artist ID")], false),
        http(
            "update_artist", "Update artist metadata.", M::PATCH, "/artists/{id}",
            vec![pid("id", "Artist ID"), bp("name", Ty::Str, "Name"), bp("icon", Ty::Str, "Icon URL")],
            false,
        ),
        http("merge_artists", "Merge duplicate artists into a target artist. Sources are deleted.", M::POST, "/artists/merge", merge_params(), true),
        http("delete_artist", "Delete an artist.", M::DELETE, "/artists/{id}", vec![pid("id", "Artist ID")], true),
        // ---- Playlists ----
        http(
            "list_playlists", "List/search playlists (paginated).", M::GET, "/playlists",
            vec![
                qp("page", Ty::Int, "1-based page number"),
                qp("page_size", Ty::Int, "Items per page"),
                qp("q", Ty::Str, "Free-text search"),
            ],
            false,
        ),
        http("get_playlist_tracks", "List tracks of a playlist.", M::GET, "/playlists/{id}/tracks", vec![pid("id", "Playlist ID")], false),
        http("export_playlist", "Export a playlist as an M3U8 file on the server.", M::POST, "/playlists/{id}/export", vec![pid("id", "Playlist ID")], false),
        http(
            "delete_playlist", "Delete a playlist, optionally with its tracks.", M::DELETE, "/playlists/{id}",
            vec![pid("id", "Playlist ID"), qp("delete_tracks", Ty::Bool, "Also delete the playlist's tracks (default false)")],
            true,
        ),
        // ---- Validations ----
        http("count_pending_validations", "Number of tracks awaiting manual validation.", M::GET, "/validations/count", vec![], false),
        http("list_pending_validations", "List tracks awaiting manual validation (staged).", M::GET, "/validations", vec![], false),
        http("get_validation_matches", "MusicBrainz match candidates for a pending track.", M::GET, "/validations/{id}/matches", vec![pid("id", "Pending track ID")], false),
        http("get_validation_youtube_candidates", "YouTube candidates usable as provider_url for a pending track.", M::GET, "/validations/{id}/youtube-candidates", vec![pid("id", "Pending track ID")], false),
        http(
            "approve_validation",
            "Approve a pending track: optionally override metadata, then tag, move and finalize it. Provide provider_url (YouTube/YT Music) when the track has no staged file (e.g. DRM-protected SoundCloud).",
            M::PATCH, "/validations/{id}",
            vec![
                pid("id", "Pending track ID"),
                bp("title", Ty::Str, "Title override"),
                bp("artists", Ty::StrList, "Artists override (names)"),
                bp("album_title", Ty::Str, "Album title override"),
                bp("genre", Ty::Str, "Genre override"),
                bp("date", Ty::Str, "Date override"),
                bp("track_number", Ty::Int, "Track number override"),
                bp("disc_number", Ty::Int, "Disc number override"),
                bp("label", Ty::Str, "Label override"),
                bp("provider_url", Ty::Str, "YouTube/YT Music URL to download from"),
            ],
            false,
        ),
        http("reject_validation", "Reject a pending track and delete it with its staged file.", M::DELETE, "/validations/{id}", vec![pid("id", "Pending track ID")], true),
        // ---- Data quality ----
        http("data_quality_get_structural_findings", "Structural data-quality findings (inconsistent or malformed library entities).", M::GET, "/data-quality/audit/structural", vec![], false),
        http("data_quality_get_duplicate_groups", "Groups of likely duplicate entities, excluding pairs marked as not-duplicate. Use merge_* to resolve.", M::GET, "/data-quality/duplicates/{entity_type}", vec![pstr("entity_type", "`artists`, `albums` or `tracks`")], false),
        http("data_quality_get_ignored_duplicates", "Pairs previously marked as not-duplicate.", M::GET, "/data-quality/duplicates/{entity_type}/ignored", vec![pstr("entity_type", "`artists`, `albums` or `tracks`")], false),
        http(
            "data_quality_ignore_duplicate", "Mark a pair of entities as confirmed not-duplicate.", M::POST, "/data-quality/duplicates/{entity_type}/ignore",
            vec![
                pstr("entity_type", "`artists`, `albums` or `tracks`"),
                bp("id_a", Ty::Int, "First entity ID").req(),
                bp("id_b", Ty::Int, "Second entity ID (must differ from id_a)").req(),
            ],
            false,
        ),
        http(
            "data_quality_restore_ignored_duplicate", "Undo a not-duplicate decision for a pair.", M::DELETE, "/data-quality/duplicates/{entity_type}/ignore/{id_a}/{id_b}",
            vec![pstr("entity_type", "`artists`, `albums` or `tracks`"), pid("id_a", "First entity ID"), pid("id_b", "Second entity ID")],
            false,
        ),
        http("data_quality_start_remote_audit", "Start the manual remote-reference audit as a background task (returns a task to follow with get_task).", M::POST, "/data-quality/audit/remote/run", vec![], false),
        http("data_quality_get_remote_audit", "Remote-reference audit results (local vs provider names).", M::GET, "/data-quality/audit/remote", vec![], false),
        http("data_quality_apply_remote_audit_name", "Apply the provider's remote name/title to the local entity.", M::POST, "/data-quality/audit/remote/{audit_id}/apply", vec![pid("audit_id", "Audit entry ID")], false),
        http("data_quality_dismiss_remote_audit", "Dismiss an audit result (kept in history).", M::POST, "/data-quality/audit/remote/{audit_id}/dismiss", vec![pid("audit_id", "Audit entry ID")], false),
        http("data_quality_delete_audited_reference", "Delete the audited reference from the library and drop its cached audit result.", M::POST, "/data-quality/audit/remote/{audit_id}/delete-reference", vec![pid("audit_id", "Audit entry ID")], true),
        http("data_quality_get_orphans", "Artists and albums not linked to any track.", M::GET, "/data-quality/orphans", vec![], false),
        http("data_quality_cleanup_orphans", "Delete all currently orphaned artists and albums.", M::POST, "/data-quality/orphans/cleanup", vec![], true),
        http("data_quality_get_playlist_issues", "Playlists with missing or duplicate track positions.", M::GET, "/data-quality/playlists/issues", vec![], false),
        http("data_quality_renumber_playlist", "Restore a deterministic zero-based position sequence for a playlist.", M::POST, "/data-quality/playlists/{playlist_id}/renumber", vec![pid("playlist_id", "Playlist ID")], false),
        http("data_quality_get_ai_cleanup_log", "Most recent SoundCloud AI metadata cleanup changes.", M::GET, "/data-quality/ai-cleanup-log", vec![qp("limit", Ty::Int, "Max entries (default 50, max 200)")], false),
        // ---- Import & tasks ----
        http(
            "import_url",
            "Import a track/playlist/album/artist URL. Single tracks run synchronously; collections return a task_id to follow with get_task.",
            M::POST, "/download",
            vec![bp("url", Ty::Str, "Source URL (Spotify, SoundCloud, YouTube, YouTube Music)").req()],
            false,
        ),
        http("list_tasks", "List background tasks.", M::GET, "/tasks", vec![], false),
        http("get_task", "Get a background task (status, progress, stats, errors).", M::GET, "/tasks/{id}", vec![pid("id", "Task ID")], false),
        http("cancel_task", "Cancel a running task.", M::POST, "/tasks/{id}/cancel", vec![pid("id", "Task ID")], true),
        http("retry_task", "Retry a failed/stale task.", M::POST, "/tasks/{id}/retry", vec![pid("id", "Task ID")], false),
    ]
}

impl Tool {
    pub fn input_schema(&self) -> Value {
        let mut props = Map::new();
        let mut required = Vec::new();
        for p in &self.params {
            let mut schema = match p.ty {
                Ty::Int => json!({"type": "integer"}),
                Ty::Str => json!({"type": "string"}),
                Ty::Bool => json!({"type": "boolean"}),
                Ty::IntList => json!({"type": "array", "items": {"type": "integer"}}),
                Ty::StrList => json!({"type": "array", "items": {"type": "string"}}),
            };
            schema["description"] = Value::String(p.desc.to_string());
            props.insert(p.name.to_string(), schema);
            if p.required {
                required.push(Value::String(p.name.to_string()));
            }
        }
        json!({"type": "object", "properties": props, "required": required})
    }

    pub fn descriptor(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema(),
            "annotations": {
                "readOnlyHint": self.read_only,
                "destructiveHint": !self.read_only && self.destructive,
                "openWorldHint": false,
            }
        })
    }

    /// Runs the tool; returns pretty JSON text, or an error message.
    pub async fn execute(&self, client: &ApiClient, args: &Value) -> Result<String, String> {
        let (method, path_tpl) = match &self.kind {
            Kind::Status => return status(client).await,
            Kind::Http { method, path } => (method.clone(), *path),
        };

        let mut path = path_tpl.to_string();
        let mut query: Vec<(String, String)> = Vec::new();
        let mut body = Map::new();
        let mut has_body_params = false;

        for p in &self.params {
            let value = args.get(p.name).filter(|v| !v.is_null());
            if p.loc == Loc::Body {
                has_body_params = true;
            }
            let Some(value) = value else {
                if p.required {
                    return Err(format!("missing required argument `{}`", p.name));
                }
                continue;
            };
            check_type(p, value)?;
            match p.loc {
                Loc::Path => {
                    // Path segments are either integers or short slugs (no URL injection).
                    let segment = scalar_to_string(value);
                    if !segment.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                        return Err(format!("argument `{}` contains invalid characters", p.name));
                    }
                    path = path.replace(&format!("{{{}}}", p.name), &segment);
                }
                Loc::Query => query.push((p.name.to_string(), scalar_to_string(value))),
                Loc::Body => {
                    body.insert(p.name.to_string(), value.clone());
                }
            }
        }

        let body = has_body_params.then(|| Value::Object(body));
        let result = client.request(method, &path, &query, body.as_ref()).await?;
        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }
}

fn check_type(p: &Param, v: &Value) -> Result<(), String> {
    let ok = match p.ty {
        Ty::Int => v.is_i64() || v.is_u64(),
        Ty::Str => v.is_string(),
        Ty::Bool => v.is_boolean(),
        Ty::IntList => v
            .as_array()
            .is_some_and(|a| a.iter().all(|x| x.is_i64() || x.is_u64())),
        Ty::StrList => v.as_array().is_some_and(|a| a.iter().all(Value::is_string)),
    };
    if ok {
        Ok(())
    } else {
        Err(format!("argument `{}` has the wrong type", p.name))
    }
}

fn scalar_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

async fn status(client: &ApiClient) -> Result<String, String> {
    let (version, providers, pending, active) = tokio::join!(
        client.get("/version"),
        client.get("/providers"),
        client.get("/validations/count"),
        client.get("/tasks/active-count"),
    );
    let wrap = |r: Result<Value, String>| r.unwrap_or_else(|e| json!({ "error": e }));
    let out = json!({
        "version": wrap(version),
        "providers": wrap(providers),
        "pending_validations": wrap(pending),
        "active_tasks": wrap(active),
    });
    serde_json::to_string_pretty(&out).map_err(|e| e.to_string())
}
