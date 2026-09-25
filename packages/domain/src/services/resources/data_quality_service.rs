use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use diesel::{Connection, SqliteConnection};
use fetcher::{Fetcher, Source};
use shared::{
    errors::Error,
    http::HttpClientBuilder,
    models::{
        AiCleanupLogEntry, Album, Artist, DataQualityEntityType, DedupIgnoreEntry,
        DuplicateCandidate, DuplicateGroup, Platform, Reference, ReferenceAuditResult,
        ReferenceAuditStatus, ReferenceAuditView, ReferenceType, StructuralFinding, Track,
    },
    types::SoundomeResult,
    utils::string::{string_similarity, SimilarityAlgorithm},
};

use crate::ports::repositories::{
    AlbumRepository, ArtistRepository, DataQualityRepository, TrackRepository,
};
use crate::services::resources::task_service::TaskService;

const ARTIST_DUPLICATE_THRESHOLD: f64 = 0.8;
const ALBUM_DUPLICATE_THRESHOLD: f64 = 0.8;
// This deliberately matches TrackService's existing automatic dedup threshold.
const TRACK_DUPLICATE_THRESHOLD: f64 = 0.8;
const REMOTE_AUDIT_MATCH_THRESHOLD: f64 = 0.8;

#[derive(Debug, Clone)]
struct RemoteAuditJob {
    entity_type: DataQualityEntityType,
    entity_id: i32,
    local_name: String,
    reference: Reference,
}

/// Orchestrates the "Data Quality" area: duplicate suggestions and their
/// persistent ignore-list, structural reference audits, the remote audit
/// cache, and the AI cleanup change log.
pub struct DataQualityService {
    repo: Arc<dyn DataQualityRepository + Send + Sync>,
    artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
    album_repo: Arc<dyn AlbumRepository + Send + Sync>,
    track_repo: Arc<dyn TrackRepository + Send + Sync>,
    task_service: Arc<TaskService>,
}

impl DataQualityService {
    pub fn new(
        repo: Arc<dyn DataQualityRepository + Send + Sync>,
        artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
        album_repo: Arc<dyn AlbumRepository + Send + Sync>,
        track_repo: Arc<dyn TrackRepository + Send + Sync>,
        task_service: Arc<TaskService>,
    ) -> Self {
        Self {
            repo,
            artist_repo,
            album_repo,
            track_repo,
            task_service,
        }
    }

    /// Runs the structural audit (A-D). Pure reads, no network calls.
    pub fn structural_findings(
        &self,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Vec<StructuralFinding>> {
        self.repo.find_structural_findings(conn)
    }

    /// Build duplicate suggestions as connected components. Explicitly ignored
    /// pairs are removed from the similarity graph before the components are
    /// calculated. The UI can choose a target within each returned group.
    pub fn duplicate_groups(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
    ) -> SoundomeResult<Vec<DuplicateGroup>> {
        let ignored = self
            .repo
            .list_dedup_ignored(conn, entity_type)?
            .into_iter()
            .map(|entry| (entry.id_a, entry.id_b))
            .collect::<HashSet<_>>();

        match entity_type {
            DataQualityEntityType::Artist => {
                let artists = self.artist_repo.get_all(conn)?;
                let albums = self.album_repo.get_all(conn)?;
                let tracks = self.track_repo.get_all(conn)?;
                Ok(build_groups(
                    DataQualityEntityType::Artist,
                    &artists,
                    &ignored,
                    ARTIST_DUPLICATE_THRESHOLD,
                    Artist::compare,
                    |artist| {
                        let id = artist.id.unwrap_or_default();
                        let track_count = tracks
                            .iter()
                            .filter(|track| track.artists.iter().any(|a| a.id == Some(id)))
                            .count();
                        let album_count = albums
                            .iter()
                            .filter(|album| album.artists.iter().any(|a| a.id == Some(id)))
                            .count();
                        DuplicateCandidate {
                            id,
                            name: artist.name.clone(),
                            artists: Vec::new(),
                            album_title: None,
                            date: None,
                            duration: None,
                            track_count,
                            album_count,
                            reference_count: artist.references.len(),
                            similarity_score: 0.0,
                            quality_value: None,
                            references: artist.references.clone(),
                        }
                    },
                ))
            }
            DataQualityEntityType::Album => {
                let albums = self.album_repo.get_all(conn)?;
                let tracks = self.track_repo.get_all(conn)?;
                Ok(build_groups(
                    DataQualityEntityType::Album,
                    &albums,
                    &ignored,
                    ALBUM_DUPLICATE_THRESHOLD,
                    Album::compare,
                    |album| {
                        let id = album.id.unwrap_or_default();
                        DuplicateCandidate {
                            id,
                            name: album.title.clone(),
                            artists: album.artists.iter().map(|a| a.name.clone()).collect(),
                            album_title: None,
                            date: album.date.clone(),
                            duration: None,
                            track_count: tracks
                                .iter()
                                .filter(|track| track.album.as_ref().and_then(|a| a.id) == Some(id))
                                .count(),
                            album_count: 0,
                            reference_count: album.references.len(),
                            similarity_score: 0.0,
                            quality_value: None,
                            references: album.references.clone(),
                        }
                    },
                ))
            }
            DataQualityEntityType::Track => {
                let tracks = self.track_repo.get_all(conn)?;
                // Validation tracks are staged/unfinalized and should not be
                // offered to the library duplicate merge (which manages files).
                let tracks: Vec<Track> = tracks
                    .into_iter()
                    .filter(|track| !track.needs_validation && track.file_path.is_some())
                    .collect();
                let mut groups = build_groups(
                    DataQualityEntityType::Track,
                    &tracks,
                    &ignored,
                    TRACK_DUPLICATE_THRESHOLD,
                    Track::compare,
                    |track| DuplicateCandidate {
                        id: track.id.unwrap_or_default(),
                        name: track.title.clone(),
                        artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                        album_title: track.album.as_ref().map(|a| a.title.clone()),
                        date: track.date.clone(),
                        duration: track.duration,
                        track_count: 1,
                        album_count: 0,
                        reference_count: track.references.len(),
                        similarity_score: 0.0,
                        // Probe audio only for actual duplicate candidates below;
                        // reading every file just to render an empty queue would
                        // make a page scan unnecessarily expensive.
                        quality_value: None,
                        references: track.references.clone(),
                    },
                );
                for group in &mut groups {
                    for candidate in &mut group.candidates {
                        if let Some(track) =
                            tracks.iter().find(|track| track.id == Some(candidate.id))
                        {
                            candidate.quality_value = track.get_bitrate();
                        }
                    }
                }
                Ok(groups)
            }
        }
    }

    pub fn ignore_duplicate(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
        id_a: i32,
        id_b: i32,
    ) -> SoundomeResult<()> {
        self.repo.add_dedup_ignore(conn, entity_type, id_a, id_b)
    }

    pub fn list_ignored_duplicates(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
    ) -> SoundomeResult<Vec<DedupIgnoreEntry>> {
        self.repo.list_dedup_ignored(conn, entity_type)
    }

    pub fn restore_ignored_duplicate(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
        id_a: i32,
        id_b: i32,
    ) -> SoundomeResult<()> {
        self.repo.remove_dedup_ignore(conn, entity_type, id_a, id_b)
    }

    pub fn record_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        result: &ReferenceAuditResult,
    ) -> SoundomeResult<()> {
        self.repo.upsert_reference_audit(conn, result)
    }

    pub fn list_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        entity_type: Option<DataQualityEntityType>,
    ) -> SoundomeResult<Vec<ReferenceAuditResult>> {
        self.repo.list_reference_audit(conn, entity_type)
    }

    pub fn list_reference_audit_views(
        &self,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Vec<ReferenceAuditView>> {
        let audits = self.repo.list_reference_audit(conn, None)?;
        if audits.is_empty() {
            return Ok(Vec::new());
        }
        let artists = self.artist_repo.get_all(conn)?;
        let albums = self.album_repo.get_all(conn)?;
        let tracks = self.track_repo.get_all(conn)?;

        Ok(audits
            .into_iter()
            .filter_map(|audit| {
                let audit_id = audit.id?;
                let (entity_name, reference) = match audit.entity_type {
                    DataQualityEntityType::Artist => artists
                        .iter()
                        .find(|artist| artist.id == Some(audit.entity_id))
                        .map(|artist| {
                            (
                                artist.name.clone(),
                                artist
                                    .references
                                    .iter()
                                    .find(|reference| reference.id == Some(audit.reference_id)),
                            )
                        })
                        .unwrap_or_else(|| (format!("Artist #{}", audit.entity_id), None)),
                    DataQualityEntityType::Album => albums
                        .iter()
                        .find(|album| album.id == Some(audit.entity_id))
                        .map(|album| {
                            (
                                album.title.clone(),
                                album
                                    .references
                                    .iter()
                                    .find(|reference| reference.id == Some(audit.reference_id)),
                            )
                        })
                        .unwrap_or_else(|| (format!("Album #{}", audit.entity_id), None)),
                    DataQualityEntityType::Track => tracks
                        .iter()
                        .find(|track| track.id == Some(audit.entity_id))
                        .map(|track| {
                            (
                                track.title.clone(),
                                track
                                    .references
                                    .iter()
                                    .find(|reference| reference.id == Some(audit.reference_id)),
                            )
                        })
                        .unwrap_or_else(|| (format!("Track #{}", audit.entity_id), None)),
                };
                Some(ReferenceAuditView {
                    id: audit_id,
                    entity_type: audit.entity_type,
                    entity_id: audit.entity_id,
                    entity_name: entity_name.clone(),
                    reference_id: audit.reference_id,
                    local_name: entity_name,
                    platform: reference.map(|reference| reference.platform.clone()),
                    ref_type: reference.map(|reference| reference.ref_type.clone()),
                    external_id: reference.and_then(|reference| reference.external_id.clone()),
                    external_url: reference.and_then(|reference| reference.external_url.clone()),
                    remote_name: audit.remote_name,
                    similarity_score: audit.similarity_score,
                    status: audit.status,
                    checked_at: audit.checked_at.map(|checked_at| checked_at.to_string()),
                })
            })
            .collect())
    }

    /// Run the remote audit only when explicitly enqueued by the user. Each
    /// reference result is persisted as soon as it is fetched, so cancellation
    /// or an individual provider error does not discard prior results.
    pub async fn run_remote_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        task_id: i32,
        cancel_flag: &AtomicBool,
    ) -> SoundomeResult<()> {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let jobs = self.collect_remote_audit_jobs(conn)?;
        let total = jobs.len();
        let total_progress = total.min(i32::MAX as usize) as i32;
        self.task_service
            .update_progress(conn, task_id, 0, total_progress)?;
        if total == 0 {
            return Ok(());
        }

        let needs_fetcher = jobs.iter().any(|job| {
            job.reference.platform != Platform::MusicBrainz && remote_audit_supported(job)
        });
        let fetcher = if needs_fetcher {
            Some(Fetcher::new().await)
        } else {
            None
        };
        let has_musicbrainz = jobs
            .iter()
            .any(|job| job.reference.platform == Platform::MusicBrainz);
        let musicbrainz_client = if has_musicbrainz {
            Some(
                HttpClientBuilder::get_reqwest_client_builder()?
                    .timeout(Duration::from_secs(20))
                    .build()
                    .map_err(|error| Error::Network(format!("MusicBrainz client: {error}")))?,
            )
        } else {
            None
        };

        let mut last_provider_request = None;
        let mut last_musicbrainz_request = None;
        let progress_step = (total / 100).max(1);
        for (index, job) in jobs.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                return Err(Error::Cancelled);
            }

            let (remote_name, similarity_score, status) = if job.reference.id == Some(0) {
                (None, None, ReferenceAuditStatus::Missing)
            } else {
                let result = fetch_remote_name(
                    fetcher.as_ref(),
                    musicbrainz_client.as_ref(),
                    &mut last_provider_request,
                    &mut last_musicbrainz_request,
                    job,
                )
                .await;
                match result {
                Ok(remote_name) if remote_name.trim().is_empty() => {
                    (None, None, ReferenceAuditStatus::Unreachable)
                }
                Ok(remote_name) => {
                    let score = string_similarity(
                        &job.local_name,
                        &remote_name,
                        SimilarityAlgorithm::Smart,
                    );
                    let status = if score >= REMOTE_AUDIT_MATCH_THRESHOLD {
                        ReferenceAuditStatus::Ok
                    } else {
                        ReferenceAuditStatus::Mismatch
                    };
                    (Some(remote_name), Some(score), status)
                }
                Err(Error::NotImplemented(_) | Error::InvalidUrl(_)) => {
                    (None, None, ReferenceAuditStatus::Unsupported)
                }
                Err(error) => {
                    tracing::warn!(
                        entity_type = job.entity_type.as_str(),
                        entity_id = job.entity_id,
                        reference_id = job.reference.id,
                        %error,
                        "Remote reference audit could not resolve a reference"
                    );
                    (None, None, ReferenceAuditStatus::Unreachable)
                }
                }
            };

            self.repo.upsert_reference_audit(
                conn,
                &ReferenceAuditResult {
                    id: None,
                    entity_type: job.entity_type,
                    entity_id: job.entity_id,
                    reference_id: job.reference.id.unwrap_or_default(),
                    remote_name,
                    similarity_score,
                    status,
                    checked_at: None,
                },
            )?;

            let processed = index + 1;
            if processed % progress_step == 0 || processed == total {
                self.task_service.update_progress(
                    conn,
                    task_id,
                    processed.min(i32::MAX as usize) as i32,
                    total_progress,
                )?;
            }
        }
        Ok(())
    }

    pub fn apply_remote_reference_name(
        &self,
        conn: &mut SqliteConnection,
        audit_id: i32,
    ) -> SoundomeResult<()> {
        let audit = self.cached_audit_by_id(conn, audit_id)?;
        let remote_name = audit
            .remote_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .ok_or_else(|| Error::Custom("Audit result has no remote name to apply".into()))?
            .to_string();

        conn.transaction(|tx| {
            match audit.entity_type {
                DataQualityEntityType::Artist => {
                    let mut artist = self.artist_repo.get_by_id(tx, audit.entity_id)?;
                    if !artist
                        .references
                        .iter()
                        .any(|reference| reference.id == Some(audit.reference_id))
                    {
                        return Err(Error::NotFound(
                            "Audited artist reference no longer exists".into(),
                        ));
                    }
                    artist.name = remote_name.clone();
                    self.artist_repo.update(tx, audit.entity_id, &artist)?;
                }
                DataQualityEntityType::Album => {
                    let mut album = self.album_repo.get_by_id(tx, audit.entity_id)?;
                    if !album
                        .references
                        .iter()
                        .any(|reference| reference.id == Some(audit.reference_id))
                    {
                        return Err(Error::NotFound(
                            "Audited album reference no longer exists".into(),
                        ));
                    }
                    album.title = remote_name.clone();
                    self.album_repo.update(tx, audit.entity_id, &album)?;
                }
                DataQualityEntityType::Track => {
                    let mut track = self.track_repo.get_by_id(tx, audit.entity_id)?;
                    if !track
                        .references
                        .iter()
                        .any(|reference| reference.id == Some(audit.reference_id))
                    {
                        return Err(Error::NotFound(
                            "Audited track reference no longer exists".into(),
                        ));
                    }
                    track.title = remote_name.clone();
                    self.track_repo.update(tx, audit.entity_id, &track)?;
                }
            }
            self.repo
                .set_reference_audit_status(tx, audit_id, ReferenceAuditStatus::Ok, Some(1.0))
        })
    }

    pub fn dismiss_remote_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        audit_id: i32,
    ) -> SoundomeResult<()> {
        let audit = self.cached_audit_by_id(conn, audit_id)?;
        self.repo.set_reference_audit_status(
            conn,
            audit_id,
            ReferenceAuditStatus::Dismissed,
            audit.similarity_score,
        )
    }

    pub fn delete_audited_reference(
        &self,
        conn: &mut SqliteConnection,
        audit_id: i32,
    ) -> SoundomeResult<()> {
        let audit = self.cached_audit_by_id(conn, audit_id)?;
        conn.transaction(|tx| {
            match audit.entity_type {
                DataQualityEntityType::Artist => {
                    self.artist_repo.delete_reference(tx, audit.reference_id)?;
                }
                DataQualityEntityType::Album => {
                    self.album_repo.delete_reference(tx, audit.reference_id)?;
                }
                DataQualityEntityType::Track => {
                    self.track_repo.delete_reference(tx, audit.reference_id)?;
                }
            }
            self.repo
                .delete_reference_audit(tx, audit.entity_type, audit.reference_id)
        })
    }

    fn cached_audit_by_id(
        &self,
        conn: &mut SqliteConnection,
        audit_id: i32,
    ) -> SoundomeResult<ReferenceAuditResult> {
        self.repo
            .list_reference_audit(conn, None)?
            .into_iter()
            .find(|result| result.id == Some(audit_id))
            .ok_or_else(|| Error::NotFound(format!("Remote audit result {audit_id}")))
    }

    fn collect_remote_audit_jobs(
        &self,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Vec<RemoteAuditJob>> {
        let mut jobs = Vec::new();
        for artist in self.artist_repo.get_all(conn)? {
            if let Some(entity_id) = artist.id {
                push_audit_jobs(
                    &mut jobs,
                    DataQualityEntityType::Artist,
                    entity_id,
                    &artist.name,
                    &artist.references,
                );
                if !has_metadata_reference(&artist.references) {
                    push_missing_audit_job(
                        &mut jobs,
                        DataQualityEntityType::Artist,
                        entity_id,
                        &artist.name,
                    );
                }
            }
        }
        for album in self.album_repo.get_all(conn)? {
            if let Some(entity_id) = album.id {
                push_audit_jobs(
                    &mut jobs,
                    DataQualityEntityType::Album,
                    entity_id,
                    &album.title,
                    &album.references,
                );
                if !has_metadata_reference(&album.references) {
                    push_missing_audit_job(
                        &mut jobs,
                        DataQualityEntityType::Album,
                        entity_id,
                        &album.title,
                    );
                }
            }
        }
        for track in self.track_repo.get_all(conn)? {
            if let Some(entity_id) = track.id {
                push_audit_jobs(
                    &mut jobs,
                    DataQualityEntityType::Track,
                    entity_id,
                    &track.title,
                    &track.references,
                );
                if !has_metadata_reference(&track.references) {
                    push_missing_audit_job(
                        &mut jobs,
                        DataQualityEntityType::Track,
                        entity_id,
                        &track.title,
                    );
                }
            }
        }
        Ok(jobs)
    }

    pub fn log_ai_cleanup(
        &self,
        conn: &mut SqliteConnection,
        entry: &AiCleanupLogEntry,
    ) -> SoundomeResult<()> {
        self.repo.create_ai_cleanup_log(conn, entry)
    }

    pub fn list_ai_cleanup_log(
        &self,
        conn: &mut SqliteConnection,
        limit: i64,
    ) -> SoundomeResult<Vec<AiCleanupLogEntry>> {
        self.repo.list_ai_cleanup_log(conn, limit)
    }
}

fn push_audit_jobs(
    jobs: &mut Vec<RemoteAuditJob>,
    entity_type: DataQualityEntityType,
    entity_id: i32,
    local_name: &str,
    references: &[Reference],
) {
    for reference in references {
        if reference.ref_type != ReferenceType::Source
            && reference.ref_type != ReferenceType::Provider
            && reference.ref_type != ReferenceType::Metadata
        {
            continue;
        }
        if reference.id.is_none() {
            continue;
        }
        jobs.push(RemoteAuditJob {
            entity_type,
            entity_id,
            local_name: local_name.to_string(),
            reference: reference.clone(),
        });
    }
}

fn has_metadata_reference(references: &[Reference]) -> bool {
    references
        .iter()
        .any(|reference| reference.ref_type == ReferenceType::Metadata)
}

fn push_missing_audit_job(
    jobs: &mut Vec<RemoteAuditJob>,
    entity_type: DataQualityEntityType,
    entity_id: i32,
    local_name: &str,
) {
    jobs.push(RemoteAuditJob {
        entity_type,
        entity_id,
        local_name: local_name.to_string(),
        // Reference id 0 is a sentinel: real reference rows are AUTOINCREMENT
        // values starting at 1. It lets the existing cache table represent a
        // missing-reference finding without inventing a new table.
        reference: Reference {
            id: Some(0),
            ref_type: ReferenceType::Metadata,
            platform: Platform::Unknown,
            external_id: None,
            external_url: None,
        },
    });
}

fn remote_audit_supported(job: &RemoteAuditJob) -> bool {
    match (job.reference.platform.clone(), job.entity_type) {
        (Platform::Spotify, _) | (Platform::YoutubeMusic, _) | (Platform::MusicBrainz, _) => true,
        (Platform::Youtube, DataQualityEntityType::Track) => true,
        (Platform::SoundCloud, DataQualityEntityType::Artist)
        | (Platform::SoundCloud, DataQualityEntityType::Track) => true,
        (Platform::Youtube, _) | (Platform::SoundCloud, _) => false,
        (Platform::Bandcamp, _) | (Platform::Unknown, _) => false,
    }
}

fn remote_audit_url(job: &RemoteAuditJob) -> Option<String> {
    if let Some(url) = job
        .reference
        .external_url
        .as_deref()
        .filter(|url| !url.trim().is_empty())
    {
        return Some(url.to_string());
    }
    let id = job.reference.external_id.as_deref()?;
    match (job.reference.platform.clone(), job.entity_type) {
        (Platform::Spotify, DataQualityEntityType::Artist) => {
            Some(format!("https://open.spotify.com/artist/{id}"))
        }
        (Platform::Spotify, DataQualityEntityType::Album) => {
            Some(format!("https://open.spotify.com/album/{id}"))
        }
        (Platform::Spotify, DataQualityEntityType::Track) => {
            Some(format!("https://open.spotify.com/track/{id}"))
        }
        (Platform::YoutubeMusic, DataQualityEntityType::Artist) => {
            Some(format!("https://music.youtube.com/channel/{id}"))
        }
        (Platform::YoutubeMusic, DataQualityEntityType::Album) => {
            Some(format!("https://music.youtube.com/playlist?list={id}"))
        }
        (Platform::YoutubeMusic, DataQualityEntityType::Track) => {
            Some(format!("https://music.youtube.com/watch?v={id}"))
        }
        (Platform::Youtube, DataQualityEntityType::Track) => {
            Some(format!("https://www.youtube.com/watch?v={id}"))
        }
        _ => None,
    }
}

async fn fetch_remote_name(
    fetcher: Option<&Fetcher>,
    musicbrainz_client: Option<&reqwest::Client>,
    last_provider_request: &mut Option<Instant>,
    last_musicbrainz_request: &mut Option<Instant>,
    job: &RemoteAuditJob,
) -> SoundomeResult<String> {
    if !remote_audit_supported(job) {
        return Err(Error::NotImplemented(format!(
            "Remote audit is not supported for {} references on {} entities",
            job.reference.platform,
            job.entity_type.as_str()
        )));
    }

    if job.reference.platform == Platform::MusicBrainz {
        let client = musicbrainz_client
            .ok_or_else(|| Error::Network("MusicBrainz HTTP client is unavailable".to_string()))?;
        return fetch_musicbrainz_name(client, last_musicbrainz_request, job).await;
    }

    if job.reference.platform == Platform::YoutubeMusic
        && job.entity_type == DataQualityEntityType::Album
    {
        if let Some(external_id) = job.reference.external_id.as_deref() {
            let fetcher = fetcher
                .ok_or_else(|| Error::ProviderUnavailable(job.reference.platform.to_string()))?;
            return fetcher
                .get_youtube_music_album_from_id(external_id)
                .await
                .map(|album| album.title);
        }
    }

    let url = remote_audit_url(job).ok_or_else(|| {
        Error::NotImplemented("Reference has no resolvable provider URL or external id".to_string())
    })?;
    let url_platform = Platform::from_url(&url);
    if url_platform != Platform::Unknown && url_platform != job.reference.platform {
        return Err(Error::InvalidUrl(format!(
            "Reference declares {} but URL resolves to {}",
            job.reference.platform, url_platform
        )));
    }

    // The remote check is sequential and manually requested. Leave a small
    // gap between provider lookups to avoid turning a full-library audit into
    // a burst of requests (MusicBrainz has its stricter one-request/second gap
    // in `fetch_musicbrainz_name`).
    if let Some(last_request) = last_provider_request {
        let wait = Duration::from_millis(250).saturating_sub(last_request.elapsed());
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
    }
    *last_provider_request = Some(Instant::now());

    let fetcher =
        fetcher.ok_or_else(|| Error::ProviderUnavailable(job.reference.platform.to_string()))?;
    match job.entity_type {
        DataQualityEntityType::Artist => fetcher
            .get_artist_from_url(&url)
            .await
            .map(|artist| artist.name),
        DataQualityEntityType::Album => fetcher
            .get_album_from_url(&url)
            .await
            .map(|album| album.title),
        DataQualityEntityType::Track => fetcher
            .get_track_from_url(&url)
            .await
            .map(|track| track.title),
    }
}

async fn fetch_musicbrainz_name(
    client: &reqwest::Client,
    last_request: &mut Option<Instant>,
    job: &RemoteAuditJob,
) -> SoundomeResult<String> {
    let resource_type = match job.entity_type {
        DataQualityEntityType::Artist => "artist",
        DataQualityEntityType::Album => "release",
        DataQualityEntityType::Track => "recording",
    };
    let external_id = musicbrainz_external_id(job).ok_or_else(|| {
        Error::NotImplemented("MusicBrainz reference has no valid MBID".to_string())
    })?;

    if let Some(last_request) = last_request {
        let wait = Duration::from_secs(1).saturating_sub(last_request.elapsed());
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
    }
    *last_request = Some(Instant::now());

    let base = format!("https://musicbrainz.org/ws/2/{resource_type}/");
    let mut url = reqwest::Url::parse(&base)
        .map_err(|error| Error::InvalidUrl(format!("MusicBrainz URL: {error}")))?;
    url.path_segments_mut()
        .map_err(|_| Error::InvalidUrl("MusicBrainz URL cannot accept a resource id".into()))?
        .push(&external_id);
    url.query_pairs_mut().append_pair("fmt", "json");

    let response = client
        .get(url)
        .header(
            reqwest::header::USER_AGENT,
            format!(
                "Soundome/{} (https://github.com/barthofu/soundome)",
                env!("CARGO_PKG_VERSION")
            ),
        )
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|error| Error::Network(format!("MusicBrainz request failed: {error}")))?;
    if !response.status().is_success() {
        return Err(Error::Network(format!(
            "MusicBrainz returned HTTP {}",
            response.status()
        )));
    }
    let body = response
        .text()
        .await
        .map_err(|error| Error::Network(format!("MusicBrainz response read failed: {error}")))?;
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|error| {
        Error::Network(format!("MusicBrainz response was invalid JSON: {error}"))
    })?;
    let field = if job.entity_type == DataQualityEntityType::Artist {
        "name"
    } else {
        "title"
    };
    json.get(field)
        .and_then(serde_json::Value::as_str)
        .map(|value| value.to_string())
        .ok_or_else(|| Error::NotFound(format!("MusicBrainz response has no {field}")))
}

/// MusicBrainz IDs are UUIDs. Older/manual references can contain a provider
/// id from another platform or put the MBID only in the URL; reject the former
/// as unsupported and recover the latter before making a request.
fn musicbrainz_external_id(job: &RemoteAuditJob) -> Option<String> {
    let from_url = job
        .reference
        .external_url
        .as_deref()
        .and_then(|url| url.split(['?', '#']).next())
        .and_then(|url| url.trim_end_matches('/').rsplit('/').next())
        .filter(|id| uuid::Uuid::parse_str(id).is_ok());
    let from_id = job
        .reference
        .external_id
        .as_deref()
        .filter(|id| uuid::Uuid::parse_str(id.trim()).is_ok())
        .map(str::trim);
    from_id.or(from_url).map(str::to_string)
}

fn build_groups<T>(
    entity_type: DataQualityEntityType,
    items: &[T],
    ignored: &HashSet<(i32, i32)>,
    threshold: f64,
    compare: impl Fn(&T, &T) -> f64,
    mut candidate_for: impl FnMut(&T) -> DuplicateCandidate,
) -> Vec<DuplicateGroup> {
    if items.len() < 2 {
        return Vec::new();
    }

    let base_candidates: Vec<DuplicateCandidate> = items.iter().map(&mut candidate_for).collect();
    let ids: Vec<i32> = base_candidates
        .iter()
        .map(|candidate| candidate.id)
        .collect();
    let mut parents: Vec<usize> = (0..items.len()).collect();
    let mut edge_scores: Vec<(usize, usize, f64)> = Vec::new();

    for left in 0..items.len() {
        for right in (left + 1)..items.len() {
            let pair = if ids[left] < ids[right] {
                (ids[left], ids[right])
            } else {
                (ids[right], ids[left])
            };
            if ignored.contains(&pair) {
                continue;
            }
            let score = compare(&items[left], &items[right]);
            if score >= threshold {
                union(&mut parents, left, right);
                edge_scores.push((left, right, score));
            }
        }
    }

    let mut components: Vec<Vec<usize>> = (0..items.len()).map(|_| Vec::new()).collect();
    for index in 0..items.len() {
        let root = find_root(&mut parents, index);
        components[root].push(index);
    }

    let mut groups = Vec::new();
    for component in components.into_iter().filter(|members| members.len() > 1) {
        let mut candidates: Vec<DuplicateCandidate> = component
            .iter()
            .map(|index| base_candidates[*index].clone())
            .collect();
        for candidate in &mut candidates {
            let index = ids.iter().position(|id| *id == candidate.id).unwrap_or(0);
            candidate.similarity_score = edge_scores
                .iter()
                .filter(|(left, right, _)| *left == index || *right == index)
                .map(|(_, _, score)| *score)
                .fold(0.0, f64::max);
        }
        candidates.sort_by(|a, b| {
            b.track_count
                .cmp(&a.track_count)
                .then_with(|| b.reference_count.cmp(&a.reference_count))
                .then_with(|| name_cleanliness(&b.name).total_cmp(&name_cleanliness(&a.name)))
                .then_with(|| a.id.cmp(&b.id))
        });
        let suggested_target_id = candidates[0].id;
        groups.push(DuplicateGroup {
            entity_type,
            suggested_target_id,
            candidates,
        });
    }
    groups.sort_by(|a, b| {
        b.candidates.len().cmp(&a.candidates.len()).then_with(|| {
            a.candidates[0]
                .name
                .to_lowercase()
                .cmp(&b.candidates[0].name.to_lowercase())
        })
    });
    groups
}

fn find_root(parents: &mut [usize], index: usize) -> usize {
    if parents[index] != index {
        parents[index] = find_root(parents, parents[index]);
    }
    parents[index]
}

fn union(parents: &mut [usize], left: usize, right: usize) {
    let left_root = find_root(parents, left);
    let right_root = find_root(parents, right);
    if left_root != right_root {
        parents[right_root] = left_root;
    }
}

fn name_cleanliness(name: &str) -> f64 {
    let total = name.chars().count();
    if total == 0 {
        return 0.0;
    }
    name.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .count() as f64
        / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NamedItem {
        id: i32,
        name: &'static str,
    }

    fn candidate(item: &NamedItem) -> DuplicateCandidate {
        DuplicateCandidate {
            id: item.id,
            name: item.name.to_string(),
            artists: Vec::new(),
            album_title: None,
            date: None,
            duration: None,
            track_count: 0,
            album_count: 0,
            reference_count: 0,
            similarity_score: 0.0,
            quality_value: None,
            references: Vec::new(),
        }
    }

    #[test]
    fn duplicate_groups_include_transitive_matches() {
        let items = [
            NamedItem { id: 1, name: "A" },
            NamedItem { id: 2, name: "B" },
            NamedItem { id: 3, name: "C" },
        ];
        let groups = build_groups(
            DataQualityEntityType::Artist,
            &items,
            &HashSet::new(),
            0.8,
            |left, right| match (left.id, right.id) {
                (1, 2) | (2, 1) => 0.9,
                (2, 3) | (3, 2) => 0.85,
                _ => 0.2,
            },
            candidate,
        );

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].candidates.len(), 3);
    }

    #[test]
    fn ignored_pair_is_removed_from_the_similarity_graph() {
        let items = [
            NamedItem { id: 1, name: "A" },
            NamedItem { id: 2, name: "B" },
            NamedItem { id: 3, name: "C" },
        ];
        let ignored = HashSet::from([(1, 2)]);
        let groups = build_groups(
            DataQualityEntityType::Artist,
            &items,
            &ignored,
            0.8,
            |left, right| match (left.id, right.id) {
                (1, 2) | (2, 1) | (2, 3) | (3, 2) => 0.9,
                _ => 0.2,
            },
            candidate,
        );

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].candidates.len(), 2);
        assert!(groups[0]
            .candidates
            .iter()
            .all(|candidate| candidate.id != 1));
    }
}
