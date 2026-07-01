/// Integration tests for the Spotify fetcher.
///
/// Live tests are gated with `#[ignore]` and require valid credentials in `.env`.
/// Run them with:
///   cargo test -p soundome-tests spotify -- --ignored --nocapture
///
/// The unit tests (no `#[ignore]`) reproduce the deserialization bug without
/// any network access, so they always run and act as a regression guard.

// ── unit tests ────────────────────────────────────────────────────────────────

/// These JSON snippets are representative of what the Spotify API sends back
/// when `fields=id,name,images` is used — i.e. neither `items` nor `tracks`
/// is present in the response.

#[cfg(test)]
mod deser {
    use rspotify::model::FullPlaylist;

    /// Minimal JSON that Spotify returns when `fields` omits items/tracks entirely.
    /// This is what causes the `expect("missing items/tracks")` panic in rspotify.
    const PLAYLIST_NO_ITEMS: &str = r#"{
        "collaborative": false,
        "description": "test playlist",
        "external_urls": { "spotify": "https://open.spotify.com/playlist/abc123" },
        "followers": { "href": null, "total": 0 },
        "href": "https://api.spotify.com/v1/playlists/abc123",
        "id": "abc123",
        "images": [],
        "name": "My Test Playlist",
        "owner": {
            "display_name": "testuser",
            "external_urls": { "spotify": "https://open.spotify.com/user/testuser" },
            "href": "https://api.spotify.com/v1/users/testuser",
            "id": "testuser",
            "type": "user",
            "uri": "spotify:user:testuser"
        },
        "public": true,
        "snapshot_id": "snap1",
        "type": "playlist",
        "uri": "spotify:playlist:abc123"
    }"#;

    /// JSON with `tracks` as a pagination envelope (no `items` array inside).
    /// This is the shape returned when using `fields=...tracks(total,href,...)`.
    const PLAYLIST_TRACKS_ENVELOPE_ONLY: &str = r#"{
        "collaborative": false,
        "description": null,
        "external_urls": { "spotify": "https://open.spotify.com/playlist/abc123" },
        "followers": { "href": null, "total": 5 },
        "href": "https://api.spotify.com/v1/playlists/abc123",
        "id": "abc123",
        "images": [{ "url": "https://i.scdn.co/image/abc", "height": 300, "width": 300 }],
        "name": "My Test Playlist",
        "owner": {
            "display_name": "testuser",
            "external_urls": { "spotify": "https://open.spotify.com/user/testuser" },
            "href": "https://api.spotify.com/v1/users/testuser",
            "id": "testuser",
            "type": "user",
            "uri": "spotify:user:testuser"
        },
        "public": true,
        "snapshot_id": "snap1",
        "tracks": { "href": "https://api.spotify.com/v1/playlists/abc123/tracks", "total": 9 },
        "type": "playlist",
        "uri": "spotify:playlist:abc123"
    }"#;

    /// Correct JSON with a proper `items` page (including an empty items array).
    const PLAYLIST_WITH_EMPTY_ITEMS: &str = r#"{
        "collaborative": false,
        "description": null,
        "external_urls": { "spotify": "https://open.spotify.com/playlist/abc123" },
        "followers": { "href": null, "total": 0 },
        "href": "https://api.spotify.com/v1/playlists/abc123",
        "id": "abc123",
        "images": [],
        "name": "My Test Playlist",
        "owner": {
            "display_name": "testuser",
            "external_urls": { "spotify": "https://open.spotify.com/user/testuser" },
            "href": "https://api.spotify.com/v1/users/testuser",
            "id": "testuser",
            "type": "user",
            "uri": "spotify:user:testuser"
        },
        "public": true,
        "snapshot_id": "snap1",
        "tracks": {
            "href": "https://api.spotify.com/v1/playlists/abc123/tracks",
            "items": [],
            "limit": 100,
            "next": null,
            "offset": 0,
            "previous": null,
            "total": 0
        },
        "type": "playlist",
        "uri": "spotify:playlist:abc123"
    }"#;

    /// Reproduces the rspotify bug: `FullPlaylist` panics (via serde `from`)
    /// when both `items` and `tracks` are absent from the JSON.
    ///
    /// Expected: deserialization fails or panics — this test documents the
    /// current broken behavior. Once rspotify is fixed (or we bypass it),
    /// this test should be updated to assert `Ok`.
    #[test]
    fn full_playlist_no_items_fails() {
        let result = std::panic::catch_unwind(|| {
            serde_json::from_str::<FullPlaylist>(PLAYLIST_NO_ITEMS)
        });
        // Either a panic (from `.expect`) or a serde error — both are "broken".
        match result {
            Err(_panic) => {
                // rspotify panicked — the known bug is reproduced.
                println!("rspotify panicked as expected (expect bug confirmed)");
            }
            Ok(Err(e)) => {
                // Serde returned an error — also acceptable as "broken".
                println!("serde error as expected: {}", e);
            }
            Ok(Ok(playlist)) => {
                // This should NOT happen with rspotify ≤ 0.16.1.
                // If it does, rspotify was fixed upstream — update this test.
                println!("Unexpected success: playlist name = {}", playlist.name);
                panic!(
                    "Expected deserialization to fail with no items/tracks, but it succeeded. \
                     rspotify may have been fixed — remove this workaround."
                );
            }
        }
    }

    /// Reproduces the partial-envelope bug: `tracks` is present but only has
    /// `total` and `href` (no `items` array) — as returned by
    /// `fields=...tracks(total,href,...)`.
    #[test]
    fn full_playlist_tracks_envelope_only_fails() {
        let result = std::panic::catch_unwind(|| {
            serde_json::from_str::<FullPlaylist>(PLAYLIST_TRACKS_ENVELOPE_ONLY)
        });
        match result {
            Err(_panic) => println!("rspotify panicked as expected (envelope-only bug confirmed)"),
            Ok(Err(e)) => println!("serde error as expected: {}", e),
            Ok(Ok(playlist)) => {
                println!("Unexpected success: playlist name = {}", playlist.name);
                panic!(
                    "Expected deserialization to fail with partial tracks envelope, but succeeded."
                );
            }
        }
    }

    /// Sanity check: a well-formed playlist with a proper `tracks` page
    /// deserializes correctly.
    #[test]
    fn full_playlist_with_empty_items_succeeds() {
        let playlist: FullPlaylist = serde_json::from_str(PLAYLIST_WITH_EMPTY_ITEMS)
            .expect("well-formed playlist should deserialize");
        assert_eq!(playlist.name, "My Test Playlist");
        assert!(playlist.items.items.is_empty());
    }
}

// ── live API tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod live {
    use fetcher::{Fetcher, Source};

    /// A real public Spotify playlist (not owned by the test app).
    /// This is the playlist that was failing in production.
    const TEST_PLAYLIST_URL: &str =
        "https://open.spotify.com/playlist/44CNTkxQCOUE3X13a88odM";

    fn init() {
        let _ = dotenvy::dotenv();
        let _ = tracing_subscriber::fmt()
            .with_env_filter("fetcher=debug,soundome_tests=debug")
            .try_init();
    }

    /// Fetches playlist metadata from the real Spotify API.
    ///
    /// Run with:
    ///   cargo test -p soundome-tests spotify::live::playlist_metadata -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "requires SOUNDOME__PROVIDERS__SPOTIFY__* credentials in .env"]
    async fn playlist_metadata() {
        init();

        let fetcher = Fetcher::new().await;
        let result = fetcher.get_playlist_from_url(TEST_PLAYLIST_URL).await;

        match result {
            Ok(playlist) => {
                println!("✓ Playlist name: {}", playlist.name);
                println!("  Source: {:?}", playlist.source);
                println!("  Cover: {:?}", playlist.cover);
                assert!(
                    !playlist.name.is_empty(),
                    "Playlist name should not be empty"
                );
                assert_ne!(
                    playlist.name, TEST_PLAYLIST_URL,
                    "Name should be the real playlist name, not the URL"
                );
            }
            Err(e) => {
                panic!(
                    "get_playlist_from_url failed: {}\n\
                     This is the bug we are trying to fix.",
                    e
                );
            }
        }
    }

    /// Fetches playlist tracks from the real Spotify API.
    /// This should work regardless of the metadata bug (uses the /items endpoint).
    ///
    /// Run with:
    ///   cargo test -p soundome-tests spotify::live::playlist_tracks -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "requires SOUNDOME__PROVIDERS__SPOTIFY__* credentials in .env"]
    async fn playlist_tracks() {
        init();

        let fetcher = Fetcher::new().await;
        let result = fetcher.get_playlist_tracks_from_url(TEST_PLAYLIST_URL).await;

        match result {
            Ok(tracks) => {
                println!("✓ Found {} tracks", tracks.len());
                for (i, pt) in tracks.iter().take(3).enumerate() {
                    println!("  [{}] {}", i + 1, pt.track.display());
                }
                assert!(!tracks.is_empty(), "Playlist should have tracks");
            }
            Err(e) => {
                panic!("get_playlist_tracks_from_url failed: {}", e);
            }
        }
    }
}
