use bandcamp::{search, SearchResultItem};
use shared::models::{Album, Artist, Track};
use shared::models::{AlbumType, Platform, Reference, ReferenceType};
use shared::utils::enums::Match;
use shared::utils::string::{split_collab_artist_name, string_similarity, SimilarityAlgorithm};

use crate::TagProvider;

#[derive(Default)]
pub struct Bandcamp;

impl Bandcamp {
    const EXACT_MATCH_THRESHOLD: f64 = 0.8;
    const PARTIAL_MATCH_THRESHOLD: f64 = 0.5;

    pub fn new() -> Self {
        // Note: the bandcamp crate uses its own internal reqwest client and does not
        // support proxy configuration from shared::libs::http::HttpClientBuilder.
        Self
    }
}

impl TagProvider for Bandcamp {
    async fn get_best_match_from_track(&self, track: &Track) -> Match<Track> {
        let query = format!(
            "{} {}",
            track.artists.first().map(|a| a.name.as_str()).unwrap_or(""),
            track.title
        );

        let tracks = self.get_matches_from_query(&query).await;

        tracks
            .into_iter()
            .map(|candidate| {
                let score = track.compare(&candidate);
                (score, candidate)
            })
            .filter(|(score, _)| *score > 0.0)
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(Match::None, |(best_score, best_track)| {
                if best_score > Self::EXACT_MATCH_THRESHOLD {
                    Match::Exact(best_track)
                } else if best_score > Self::PARTIAL_MATCH_THRESHOLD {
                    Match::Partial(best_track)
                } else {
                    Match::None
                }
            })
    }

    async fn get_match_from_query(&self, query: &str) -> Match<Track> {
        let normalized_query = query.replace("- ", "");
        let tracks = self.get_matches_from_query(query).await;

        tracks
            .iter()
            .map(|track| {
                let match_score = string_similarity(
                    &normalized_query,
                    &format!("{} {}", track.artists[0].name, track.title),
                    SimilarityAlgorithm::SorensenDice,
                );
                (match_score, track)
            })
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(Match::None, |(best_score, best_track)| {
                if best_score > Self::EXACT_MATCH_THRESHOLD {
                    Match::Exact(best_track.clone())
                } else if best_score > Self::PARTIAL_MATCH_THRESHOLD {
                    Match::Partial(best_track.clone())
                } else {
                    Match::None
                }
            })
    }

    async fn get_matches_from_query(&self, query: &str) -> Vec<Track> {
        match search(query).await {
            Ok(results) => results
                .into_iter()
                .filter_map(|item| match item {
                    SearchResultItem::Track(t) => Some(search_result_to_track(t)),
                    _ => None,
                })
                .collect(),
            Err(err) => {
                tracing::warn!("Bandcamp search failed for query {:?}: {}", query, err);
                Vec::new()
            }
        }
    }
}

// ================================================================================================
// Mappers
// ================================================================================================

/// Bandcamp's search results frequently format a track's `name` as `"<band_name> - <title>"`
/// (common on single/track pages), but Soundome already keeps the artist in `Track.artists`
/// and never wants it duplicated inside `Track.title`.
///
/// Strip a leading `"<band_name> - "` prefix (also accepting en/em dash variants), but only
/// when the prefix matches the known band name exactly (case-insensitive, whitespace
/// tolerant). This keeps the cleanup deterministic and safe: a title that legitimately
/// contains " - " for another reason (e.g. "Title - Extended Mix") is left untouched, since
/// its prefix will not match the band name.
fn strip_band_name_prefix(title: &str, band_name: &str) -> String {
    let band_name = band_name.trim();
    if band_name.is_empty() {
        return title.to_string();
    }

    for separator in [" - ", " – ", " — "] {
        if let Some((prefix, rest)) = title.split_once(separator) {
            let rest = rest.trim();
            if !rest.is_empty() && prefix.trim().to_lowercase() == band_name.to_lowercase() {
                return rest.to_string();
            }
        }
    }

    title.to_string()
}

/// Converts a Bandcamp "band name" into one or more `Artist`s.
///
/// Some Bandcamp pages (notably custom collaboration/collective pages) publish a single
/// combined name for what are actually two or more distinct artists, e.g.
/// `"Acidpach, L'Art Cène"` or `"Adharaa & Kobaltik"`. Using that combined string verbatim
/// as a single `Artist` is what causes `Track::transpose_metadata_from_source` (which treats
/// Bandcamp matches as authoritative and fully replaces the artist list) to overwrite two
/// correctly separated source artists (e.g. from SoundCloud) with one incorrect merged one.
///
/// `split_collab_artist_name` splits the name on common collaboration separators. When it
/// yields more than one artist, the Bandcamp band reference is intentionally NOT attached to
/// any of them: that reference identifies the combined band/project page, not any individual
/// artist, and attaching the same external id/url to two different artists could otherwise
/// cause them to be incorrectly deduplicated as the same artist by `ArtistRepository::get_by_url`.
fn band_name_to_artists(band_name: &str, band_id: u64, artist_url: &str) -> Vec<Artist> {
    let names = split_collab_artist_name(band_name);

    if names.len() <= 1 {
        return vec![Artist {
            id: None,
            name: band_name.to_string(),
            icon: None,
            references: vec![Reference {
                id: None,
                ref_type: ReferenceType::Metadata,
                platform: Platform::Bandcamp,
                external_id: Some(band_id.to_string()),
                external_url: Some(artist_url.to_string()),
            }],
        }];
    }

    names
        .into_iter()
        .map(|name| Artist {
            id: None,
            name,
            icon: None,
            references: Vec::new(),
        })
        .collect()
}

fn search_result_to_track(item: bandcamp::SearchResultItemTrack) -> Track {
    // Curate the "<band_name> - <title>" convention before it ever reaches Track.title —
    // see `strip_band_name_prefix` for why this is safe to do unconditionally here.
    let title = strip_band_name_prefix(&item.name, &item.band_name);

    let artists = band_name_to_artists(&item.band_name, item.band_id, &item.url.artist_url);

    let album = item.album_name.as_ref().map(|album_title| Album {
        id: None,
        title: album_title.clone(),
        artists: artists.clone(),
        date: None,
        album_type: AlbumType::Unknown,
        cover: None,
        references: item
            .album_id
            .map(|album_id| {
                vec![Reference {
                    id: None,
                    ref_type: ReferenceType::Metadata,
                    platform: Platform::Bandcamp,
                    external_id: Some(album_id.to_string()),
                    external_url: Some(item.url.artist_url.clone()),
                }]
            })
            .unwrap_or_default(),
    });

    Track {
        id: None,
        needs_validation: false,
        validation_reason: None,
        soundome_id: None,
        title,
        artists,
        album,
        date: None,
        genre: None,
        cover: None,
        duration: None,
        track_number: None,
        disc_number: None,
        label: None,
        file_path: None,
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::Bandcamp,
            external_id: Some(item.track_id.to_string()),
            external_url: Some(item.url.item_url),
        }],
    }
}

// ================================================================================================
// Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_exact_band_name_prefix() {
        assert_eq!(
            strip_band_name_prefix("Boards of Canada - Roygbiv", "Boards of Canada"),
            "Roygbiv"
        );
    }

    #[test]
    fn strips_prefix_case_insensitively_and_trims_whitespace() {
        assert_eq!(
            strip_band_name_prefix("  boards OF canada   -   Roygbiv  ", "Boards of Canada"),
            "Roygbiv"
        );
    }

    #[test]
    fn strips_en_and_em_dash_variants() {
        assert_eq!(strip_band_name_prefix("Artist – Title", "Artist"), "Title");
        assert_eq!(strip_band_name_prefix("Artist — Title", "Artist"), "Title");
    }

    #[test]
    fn leaves_title_untouched_when_prefix_does_not_match_band_name() {
        // The " - " here is part of the title itself (e.g. a remix tag), not the artist name.
        assert_eq!(
            strip_band_name_prefix("Roygbiv - Extended Mix", "Boards of Canada"),
            "Roygbiv - Extended Mix"
        );
    }

    #[test]
    fn leaves_title_untouched_when_there_is_no_separator() {
        assert_eq!(
            strip_band_name_prefix("Roygbiv", "Boards of Canada"),
            "Roygbiv"
        );
    }

    #[test]
    fn leaves_title_untouched_when_band_name_is_empty() {
        assert_eq!(
            strip_band_name_prefix("Boards of Canada - Roygbiv", ""),
            "Boards of Canada - Roygbiv"
        );
    }

    #[test]
    fn leaves_title_untouched_when_remainder_would_be_empty() {
        // Defensive: never collapse a title down to nothing.
        assert_eq!(
            strip_band_name_prefix("Boards of Canada - ", "Boards of Canada"),
            "Boards of Canada - "
        );
    }

    #[test]
    fn band_name_to_artists_splits_collab_name_and_drops_ambiguous_reference() {
        let artists =
            band_name_to_artists("Acidpach, L'Art Cène", 123, "https://example.bandcamp.com");
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].name, "Acidpach");
        assert_eq!(artists[1].name, "L'Art Cène");
        assert!(artists[0].references.is_empty());
        assert!(artists[1].references.is_empty());
    }

    #[test]
    fn band_name_to_artists_keeps_single_artist_with_reference() {
        let artists = band_name_to_artists("Boards of Canada", 456, "https://example.bandcamp.com");
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].name, "Boards of Canada");
        assert_eq!(artists[0].references.len(), 1);
        assert_eq!(
            artists[0].references[0].external_id,
            Some("456".to_string())
        );
    }
}
