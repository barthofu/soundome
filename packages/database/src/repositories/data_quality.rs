use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use domain::ports::repositories::DataQualityRepository;
use shared::{
    models::{
        AiCleanupLogEntry, DataQualityEntityType, DedupIgnoreEntry, Platform, ReferenceAuditResult,
        ReferenceAuditStatus, StructuralFinding, StructuralFindingEntity, StructuralFindingKind,
    },
    types::SoundomeResult,
};

use crate::{
    entities::{
        AlbumEntity, AlbumRefEntity, ArtistEntity, ArtistRefEntity, NewAiCleanupLogEntity,
        NewDedupIgnoreEntity, NewReferenceAuditCacheEntity, TrackEntity, TrackRefEntity,
    },
    mappers::map_error,
    schema,
};

#[derive(Default)]
pub struct DieselDataQualityRepository {}

impl DieselDataQualityRepository {
    pub fn new() -> Self {
        Self {}
    }
}

/// Flattened view of a single reference row, tagged with the entity it belongs
/// to, regardless of which `_ref` table it came from. Building this once lets
/// every structural check below share the same in-memory dataset instead of
/// re-querying per check.
struct RefRow {
    ref_id: i32,
    entity_type: DataQualityEntityType,
    entity_id: i32,
    ref_type: String,
    platform: String,
    external_id: Option<String>,
    external_url: Option<String>,
}

impl DataQualityRepository for DieselDataQualityRepository {
    fn find_structural_findings(
        &self,
        conn: &mut SqliteConnection,
    ) -> SoundomeResult<Vec<StructuralFinding>> {
        // 1) Load display names for every entity kind, so findings can carry a
        // human-readable name instead of a bare id.
        let artists: Vec<ArtistEntity> = schema::artist::table.load(conn).map_err(map_error)?;
        let albums: Vec<AlbumEntity> = schema::album::table.load(conn).map_err(map_error)?;
        let tracks: Vec<TrackEntity> = schema::track::table.load(conn).map_err(map_error)?;

        let artist_names: HashMap<i32, String> =
            artists.iter().map(|a| (a.id, a.name.clone())).collect();
        let album_names: HashMap<i32, String> =
            albums.iter().map(|a| (a.id, a.title.clone())).collect();
        let track_names: HashMap<i32, String> =
            tracks.iter().map(|t| (t.id, t.title.clone())).collect();

        let name_for = |entity_type: DataQualityEntityType, id: i32| -> String {
            match entity_type {
                DataQualityEntityType::Artist => artist_names
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| format!("#{id}")),
                DataQualityEntityType::Album => album_names
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| format!("#{id}")),
                DataQualityEntityType::Track => track_names
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| format!("#{id}")),
            }
        };

        let entity = |entity_type: DataQualityEntityType, id: i32| StructuralFindingEntity {
            entity_type,
            id,
            name: name_for(entity_type, id),
        };

        // 2) Load every reference row across the three `_ref` tables into one
        // flat, entity-tagged list.
        let artist_refs: Vec<ArtistRefEntity> =
            schema::artist_ref::table.load(conn).map_err(map_error)?;
        let album_refs: Vec<AlbumRefEntity> =
            schema::album_ref::table.load(conn).map_err(map_error)?;
        let track_refs: Vec<TrackRefEntity> =
            schema::track_ref::table.load(conn).map_err(map_error)?;

        let mut refs: Vec<RefRow> =
            Vec::with_capacity(artist_refs.len() + album_refs.len() + track_refs.len());
        refs.extend(artist_refs.into_iter().map(|r| RefRow {
            ref_id: r.id,
            entity_type: DataQualityEntityType::Artist,
            entity_id: r.artist_id,
            ref_type: r.ref_type,
            platform: r.platform,
            external_id: r.external_id,
            external_url: r.external_url,
        }));
        refs.extend(album_refs.into_iter().map(|r| RefRow {
            ref_id: r.id,
            entity_type: DataQualityEntityType::Album,
            entity_id: r.album_id,
            ref_type: r.ref_type,
            platform: r.platform,
            external_id: r.external_id,
            external_url: r.external_url,
        }));
        refs.extend(track_refs.into_iter().map(|r| RefRow {
            ref_id: r.id,
            entity_type: DataQualityEntityType::Track,
            entity_id: r.track_id,
            ref_type: r.ref_type,
            platform: r.platform,
            external_id: r.external_id,
            external_url: r.external_url,
        }));

        let mut findings = Vec::new();

        // (A) Conflicting reference: the same (entity_type, platform, id/url) is
        // attached to more than one distinct entity. This is the exact
        // fingerprint of the SoundCloud artist-mapping bug: an uploader's
        // SoundCloud reference ending up on the wrong artist row.
        {
            let mut by_external_id: HashMap<(DataQualityEntityType, String, String), Vec<&RefRow>> =
                HashMap::new();
            let mut by_external_url: HashMap<
                (DataQualityEntityType, String, String),
                Vec<&RefRow>,
            > = HashMap::new();
            for r in &refs {
                if let Some(id) = &r.external_id {
                    by_external_id
                        .entry((r.entity_type, r.platform.clone(), id.clone()))
                        .or_default()
                        .push(r);
                }
                if let Some(url) = &r.external_url {
                    by_external_url
                        .entry((r.entity_type, r.platform.clone(), url.clone()))
                        .or_default()
                        .push(r);
                }
            }
            for group in by_external_id.values().chain(by_external_url.values()) {
                let distinct_entities: Vec<i32> = {
                    let mut ids: Vec<i32> = group.iter().map(|r| r.entity_id).collect();
                    ids.sort_unstable();
                    ids.dedup();
                    ids
                };
                if distinct_entities.len() < 2 {
                    continue;
                }
                let entity_type = group[0].entity_type;
                let platform = Platform::from_str(&group[0].platform);
                let entities: Vec<StructuralFindingEntity> = distinct_entities
                    .iter()
                    .map(|id| entity(entity_type, *id))
                    .collect();
                let names = entities
                    .iter()
                    .map(|e| e.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                findings.push(StructuralFinding {
                    kind: StructuralFindingKind::ConflictingReference,
                    message: format!(
                        "The same {} reference is attached to {} different {}s: {}",
                        platform,
                        distinct_entities.len(),
                        entity_type.as_str(),
                        names
                    ),
                    entities,
                    platform: Some(platform),
                    reference_id: None,
                });
            }
        }

        // (B) Multiple platform references on the same entity with different
        // external_ids for the same (platform, ref_type) — usually a botched
        // merge or a reference attached to the wrong entity.
        {
            let mut by_entity: HashMap<(DataQualityEntityType, i32, String, String), Vec<&RefRow>> =
                HashMap::new();
            for r in &refs {
                if r.external_id.is_none() {
                    continue;
                }
                by_entity
                    .entry((
                        r.entity_type,
                        r.entity_id,
                        r.platform.clone(),
                        r.ref_type.clone(),
                    ))
                    .or_default()
                    .push(r);
            }
            for ((entity_type, entity_id, platform, _ref_type), group) in by_entity {
                let distinct_ids: Vec<&String> = {
                    let mut ids: Vec<&String> = group
                        .iter()
                        .filter_map(|r| r.external_id.as_ref())
                        .collect();
                    ids.sort();
                    ids.dedup();
                    ids
                };
                if distinct_ids.len() < 2 {
                    continue;
                }
                let platform_enum = Platform::from_str(&platform);
                findings.push(StructuralFinding {
                    kind: StructuralFindingKind::MultiplePlatformReferences,
                    message: format!(
                        "{} \"{}\" has {} different {} reference ids: {}",
                        entity_type.as_str(),
                        name_for(entity_type, entity_id),
                        distinct_ids.len(),
                        platform_enum,
                        distinct_ids
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    entities: vec![entity(entity_type, entity_id)],
                    platform: Some(platform_enum),
                    reference_id: None,
                });
            }
        }

        // (C) Platform/URL mismatch: the reference's own URL resolves (by host)
        // to a different platform than the one stored on the reference.
        for r in &refs {
            let Some(url) = &r.external_url else {
                continue;
            };
            let declared = Platform::from_str(&r.platform);
            let detected = Platform::from_url(url);
            if detected == Platform::Unknown || detected == declared {
                continue;
            }
            findings.push(StructuralFinding {
                kind: StructuralFindingKind::PlatformUrlMismatch,
                message: format!(
                    "{} \"{}\" has a reference declared as {} but its URL looks like {}: {}",
                    r.entity_type.as_str(),
                    name_for(r.entity_type, r.entity_id),
                    declared,
                    detected,
                    url
                ),
                entities: vec![entity(r.entity_type, r.entity_id)],
                platform: Some(declared),
                reference_id: Some(r.ref_id),
            });
        }

        // (D) Track missing a Source reference entirely — should never happen
        // through the normal download pipeline.
        {
            let has_source: std::collections::HashSet<i32> = refs
                .iter()
                .filter(|r| {
                    r.entity_type == DataQualityEntityType::Track
                        && r.ref_type.eq_ignore_ascii_case("source")
                })
                .map(|r| r.entity_id)
                .collect();
            for track in &tracks {
                if has_source.contains(&track.id) {
                    continue;
                }
                findings.push(StructuralFinding {
                    kind: StructuralFindingKind::TrackMissingSourceReference,
                    message: format!("Track \"{}\" has no Source reference", track.title),
                    entities: vec![entity(DataQualityEntityType::Track, track.id)],
                    platform: None,
                    reference_id: None,
                });
            }
        }

        Ok(findings)
    }

    fn add_dedup_ignore(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
        id_a: i32,
        id_b: i32,
    ) -> SoundomeResult<()> {
        let (id_a, id_b) = if id_a <= id_b {
            (id_a, id_b)
        } else {
            (id_b, id_a)
        };
        let new_entry = NewDedupIgnoreEntity {
            entity_type: entity_type.as_str().to_string(),
            id_a,
            id_b,
        };
        diesel::insert_or_ignore_into(schema::dedup_ignore::table)
            .values(&new_entry)
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn list_dedup_ignored(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
    ) -> SoundomeResult<Vec<DedupIgnoreEntry>> {
        use crate::entities::DedupIgnoreEntity;

        let rows: Vec<DedupIgnoreEntity> = schema::dedup_ignore::table
            .filter(schema::dedup_ignore::entity_type.eq(entity_type.as_str()))
            .load(conn)
            .map_err(map_error)?;

        Ok(rows
            .into_iter()
            .map(|r| DedupIgnoreEntry {
                id: Some(r.id),
                entity_type: DataQualityEntityType::from_str(&r.entity_type),
                id_a: r.id_a,
                id_b: r.id_b,
                created_at: Some(r.created_at),
            })
            .collect())
    }

    fn remove_dedup_ignore(
        &self,
        conn: &mut SqliteConnection,
        entity_type: DataQualityEntityType,
        id_a: i32,
        id_b: i32,
    ) -> SoundomeResult<()> {
        let (id_a, id_b) = if id_a <= id_b {
            (id_a, id_b)
        } else {
            (id_b, id_a)
        };
        diesel::delete(
            schema::dedup_ignore::table
                .filter(schema::dedup_ignore::entity_type.eq(entity_type.as_str()))
                .filter(schema::dedup_ignore::id_a.eq(id_a))
                .filter(schema::dedup_ignore::id_b.eq(id_b)),
        )
        .execute(conn)
        .map_err(map_error)?;
        Ok(())
    }

    fn upsert_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        result: &ReferenceAuditResult,
    ) -> SoundomeResult<()> {
        let new_entry = NewReferenceAuditCacheEntity {
            entity_type: result.entity_type.as_str().to_string(),
            entity_id: result.entity_id,
            reference_id: result.reference_id,
            remote_name: result.remote_name.clone(),
            similarity_score: result.similarity_score.map(|score| score as f32),
            status: result.status.as_str().to_string(),
        };
        // `reference_id` is uniquely indexed: REPLACE INTO re-runs of the same
        // reference simply refresh the cached row instead of accumulating stale
        // duplicates.
        diesel::replace_into(schema::reference_audit_cache::table)
            .values(&new_entry)
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn list_reference_audit(
        &self,
        conn: &mut SqliteConnection,
        entity_type: Option<DataQualityEntityType>,
    ) -> SoundomeResult<Vec<ReferenceAuditResult>> {
        use crate::entities::ReferenceAuditCacheEntity;

        let rows: Vec<ReferenceAuditCacheEntity> = match entity_type {
            Some(t) => schema::reference_audit_cache::table
                .filter(schema::reference_audit_cache::entity_type.eq(t.as_str()))
                .load(conn)
                .map_err(map_error)?,
            None => schema::reference_audit_cache::table
                .load(conn)
                .map_err(map_error)?,
        };

        Ok(rows
            .into_iter()
            .map(|r| ReferenceAuditResult {
                id: Some(r.id),
                entity_type: DataQualityEntityType::from_str(&r.entity_type),
                entity_id: r.entity_id,
                reference_id: r.reference_id,
                remote_name: r.remote_name,
                similarity_score: r.similarity_score.map(f64::from),
                status: ReferenceAuditStatus::from_str(&r.status),
                checked_at: Some(r.checked_at),
            })
            .collect())
    }

    fn create_ai_cleanup_log(
        &self,
        conn: &mut SqliteConnection,
        entry: &AiCleanupLogEntry,
    ) -> SoundomeResult<()> {
        let new_entry = NewAiCleanupLogEntity {
            track_id: entry.track_id,
            platform: entry.platform.as_ref().to_string(),
            source_external_id: entry.source_external_id.clone(),
            before_title: entry.before_title.clone(),
            before_artists: serde_json::to_string(&entry.before_artists).unwrap_or_default(),
            after_title: entry.after_title.clone(),
            after_artists: serde_json::to_string(&entry.after_artists).unwrap_or_default(),
            rejected_artists: if entry.rejected_artists.is_empty() {
                None
            } else {
                serde_json::to_string(&entry.rejected_artists).ok()
            },
        };
        diesel::insert_into(schema::ai_cleanup_log::table)
            .values(&new_entry)
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn list_ai_cleanup_log(
        &self,
        conn: &mut SqliteConnection,
        limit: i64,
    ) -> SoundomeResult<Vec<AiCleanupLogEntry>> {
        use crate::entities::AiCleanupLogEntity;

        let rows: Vec<AiCleanupLogEntity> = schema::ai_cleanup_log::table
            .order(schema::ai_cleanup_log::created_at.desc())
            .limit(limit)
            .load(conn)
            .map_err(map_error)?;

        Ok(rows
            .into_iter()
            .map(|r| AiCleanupLogEntry {
                id: Some(r.id),
                track_id: r.track_id,
                platform: Platform::from_str(&r.platform),
                source_external_id: r.source_external_id,
                before_title: r.before_title,
                before_artists: serde_json::from_str(&r.before_artists).unwrap_or_default(),
                after_title: r.after_title,
                after_artists: serde_json::from_str(&r.after_artists).unwrap_or_default(),
                rejected_artists: r
                    .rejected_artists
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                created_at: Some(r.created_at),
            })
            .collect())
    }
}
