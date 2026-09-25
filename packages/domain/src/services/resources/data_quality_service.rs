use std::{collections::HashSet, sync::Arc};

use diesel::SqliteConnection;
use shared::{
    models::{
        AiCleanupLogEntry, Album, Artist, DataQualityEntityType, DedupIgnoreEntry,
        DuplicateCandidate, DuplicateGroup, ReferenceAuditResult, StructuralFinding, Track,
    },
    types::SoundomeResult,
};

use crate::ports::repositories::{
    AlbumRepository, ArtistRepository, DataQualityRepository, TrackRepository,
};

const ARTIST_DUPLICATE_THRESHOLD: f64 = 0.8;
const ALBUM_DUPLICATE_THRESHOLD: f64 = 0.8;
// This deliberately matches TrackService's existing automatic dedup threshold.
const TRACK_DUPLICATE_THRESHOLD: f64 = 0.8;

/// Orchestrates the "Data Quality" area: duplicate suggestions and their
/// persistent ignore-list, structural reference audits, the remote audit
/// cache, and the AI cleanup change log.
pub struct DataQualityService {
    repo: Arc<dyn DataQualityRepository + Send + Sync>,
    artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
    album_repo: Arc<dyn AlbumRepository + Send + Sync>,
    track_repo: Arc<dyn TrackRepository + Send + Sync>,
}

impl DataQualityService {
    pub fn new(
        repo: Arc<dyn DataQualityRepository + Send + Sync>,
        artist_repo: Arc<dyn ArtistRepository + Send + Sync>,
        album_repo: Arc<dyn AlbumRepository + Send + Sync>,
        track_repo: Arc<dyn TrackRepository + Send + Sync>,
    ) -> Self {
        Self {
            repo,
            artist_repo,
            album_repo,
            track_repo,
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
