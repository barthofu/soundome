use musicbrainz_rs::entity::recording::RecordingSearchQuery;
use musicbrainz_rs::prelude::*;
use musicbrainz_rs::entity::{artist_credit::ArtistCredit, recording::Recording, release::Release};
use shared::{models::{album::Album, artist::Artist, track::Track}, utils::date::{format_date, Format}};

use crate::TagProvider;

pub struct MusicBrainz;

impl MusicBrainz {
    pub fn new() -> Self {
        Self
    }
}

impl TagProvider for MusicBrainz {
    async fn search(&self, track: &Track) -> Vec<Track> {
        let mut query_builder = RecordingSearchQuery::query_builder();
        query_builder
            .recording(track.title.clone().as_str())
            .and().artist(track.artists[0].name.clone().as_str());
        track.album.clone().map(|album| query_builder
            .and().release(album.title.clone().as_str())
        );

        let query = query_builder.build();

        let query_result = Recording::search(query).execute().await.unwrap();
        query_result
            .entities
            .iter().map(|recording| convert_to_track(recording)).collect()
    }
}

// ================================================================================================
// Mappers
// ================================================================================================

pub fn convert_to_artist(artist: &ArtistCredit) -> Artist {
    Artist {
        name: artist.name.clone(),
        url: None,
        icon: None,
    }
}

pub fn convert_to_album(release: &Release) -> Album {
    Album {
        title: release.title.clone(),
        artists: release.artist_credit.clone()
            .map(|a| a.iter().map(|artist| convert_to_artist(artist)).collect())
            .unwrap_or_default(),
        date: release.date.clone().map(|date| format_date(&date, Format::DATE)),
        url: None,
        // cover: release.get_coverart().res_1200().execute().await.unwrap().
        cover: None
    }
}

pub fn convert_to_track(recording: &Recording) -> Track {
    let album = recording.releases.clone().map(|release| release.first().map(|first_release| convert_to_album(first_release)).or_else(|| None).unwrap()).or_else(|| None);
    let artists = recording.artist_credit.clone().map(|a| a.iter().map(|artist| convert_to_artist(artist)).collect()).or_else(|| Some(Vec::new())).unwrap();

    Track {
        title: recording.title.clone(),
        artists,
        album,
        genre: None,
        date: recording.first_release_date.clone().map(|date| format_date(&date, Format::DATE)),
        track_number: None,
        disc_number: None,
        url: None,
        cover: None,
        duration: recording.length.map(|length| length as i32),
        label: None,
    }
}
