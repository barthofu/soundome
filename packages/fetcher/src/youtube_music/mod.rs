pub mod mappers;

use async_trait::async_trait;
use futures::future::join_all;
use mappers::convert_track;
use rustypipe::{client::RustyPipe, model::{MusicArtist, TrackItem}};
use shared::{errors::Error, models::{album::Album, artist::Artist, playlist::PlaylistTrack, track::Track}};

use crate::Source;

pub struct YoutubeMusic {
    client: RustyPipe,
}

impl YoutubeMusic {

    pub fn new() -> Self {
        Self {
            client: RustyPipe::new(),
        }
    }

    // =================
    // Utils
    // =================

    /**
     * Converts a MusicTrack into a Track
     */
    async fn get_complete_track_from_music_track(&self, track: TrackItem) -> Track {
        let mut artists: Vec<MusicArtist> = Vec::new();
        for artist in track.artists.iter() {
            let artist = self.client.query().music_artist(&artist.id.clone().unwrap_or("".to_string()), false).await.ok();
            artist.map(|artist| artists.push(artist));
        }
        let album = self.client.query().music_album(&track.album.as_ref().map_or("", |album| &album.id)).await.ok();

        convert_track(track, artists, album)
    }

    /**
     * Extracts the id from a youtube music track url (e.g: https://music.youtube.com/watch?v=U0ZoqmyGJo8&si=KsVobimXN6uao4s4 -> xxxxxxx)
     */
    fn get_id_from_url(&self, url: &str) -> String {
        let url = url.replace("https://music.youtube.com/watch?v=", "");
        let url = url.split("&").collect::<Vec<&str>>()[0];
        url.to_string()
    }

    /**
     * Extracts the id from a youtube music artist url (e.g: https://music.youtube.com/channel/UCfeJiV0Xu-C4z4DApRcznig -> xxxxxxx)
     */
    fn get_artist_id_from_url(&self, url: &str) -> String {
        let url = url.replace("https://music.youtube.com/channel/", "");
        let url = url.split("&").collect::<Vec<&str>>()[0];
        url.to_string()
    }

    /**
     * Extracts the id from a youtube music album url (e.g: https://music.youtube.com/playlist?list=OLAK5uy_nEnkIMbtqesDReZnKM61c9Xo24Sgos8hA -> xxxxxxx)
     */
    fn get_album_id_from_url(&self, url: &str) -> String {
        let url = url.replace("https://music.youtube.com/playlist?list=", "");
        let url = url.split("&").collect::<Vec<&str>>()[0];
        url.to_string()
    }

    /**
     * Extracts the id from a youtube music playlist url (e.g: https://music.youtube.com/watch?v=YvI_FNrczzQ&list=RDCLAK5uy_mHkFNBTuR8DZUj61H5XY2onS7nRujVFx8 -> xxxxxxx)
     * It should be the `list` thing
     */
    fn _get_playlist_id_from_url(&self, url: &str) -> String {
        let url = url.replace("https://music.youtube.com/watch?v=", "");
        let url = url.split("&").collect::<Vec<&str>>()[1];
        let url = url.replace("list=", "");
        url.to_string()
    }
}

#[async_trait]
impl Source for YoutubeMusic {

    async fn get_track_from_url(&self, url: &str) -> Result<Track, Error> {
        let track = self.client.query().music_details(self.get_id_from_url(url)).await.map_err(|_| Error::NotFound(format!("Youtube Music track from {}", url).to_string()))?;
        Ok(self.get_complete_track_from_music_track(track.track).await)
    }

    async fn get_tracks_from_query(&self, query: &str) -> Result<Vec<Track>, Error> {
        let results = self.client
            .query()
            .music_search_tracks(query)
            .await
            .map_err(mappers::convert_error)?;

        let tracks = join_all(results.items.items
            .iter()
            .map(|track| self.get_complete_track_from_music_track(track.clone()))
        ).await;
        Ok(tracks)
    }

    async fn get_playlist_tracks_from_url(&self, _url: &str) -> Result<Vec<PlaylistTrack>, Error> {
        todo!()
    }

    async fn get_artist_from_url(&self, url: &str) -> Result<Artist, Error> {
        let artist = self.client
            .query()
            .music_artist(&self.get_artist_id_from_url(url), true)
            .await
            .map_err(|_| Error::NotFound(format!("Youtube Music artist from {}", url).to_string()))?;
        Ok(mappers::convert_artist(&artist))
    }

    async fn get_artists_from_query(&self, search: &str) -> Result<Vec<Artist>, Error> {
        let results = self.client
            .query()
            .music_search_artists(search)
            .await
            .map_err(mappers::convert_error)?;

        Ok(results.items.items.iter().map(|artist| mappers::convert_artist_item(artist)).collect())
    }

    async fn get_album_from_url(&self, url: &str) -> Result<Album, Error> {
        let album = self.client
            .query()
            .music_album(&self.get_album_id_from_url(url))
            .await
            .map_err(|_| Error::NotFound(format!("Youtube Music album from {}", url).to_string()))?;
        Ok(mappers::convert_album(&album))
    }

    async fn get_albums_from_query(&self, search: &str) -> Result<Vec<Album>, Error> {
        let results = self.client
            .query()
            .music_search_albums(search)
            .await
            .map_err(mappers::convert_error)?;

        Ok(results.items.items.iter().map(|album| mappers::convert_album_item(album)).collect())
    }

    async fn get_album_tracks_from_url(&self, _: &str) -> Result<Vec<Track>, Error> {
        todo!()
    }

    fn is_valid_track_url(url: &str) -> bool {
        url.contains("music.youtube.com/watch?v=")
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        url.contains("music.youtube.com") && url.contains("list=")
    }
}
