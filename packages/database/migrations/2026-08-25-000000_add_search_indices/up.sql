-- Indices to keep the paginated/searched/sorted library list endpoints
-- (`/tracks`, `/albums`, `/artists`) fast now that they filter and sort in
-- SQL instead of loading the whole table and filtering client-side.
CREATE INDEX IF NOT EXISTS idx_track_title ON track (title);
CREATE INDEX IF NOT EXISTS idx_track_needs_validation ON track (needs_validation);
CREATE INDEX IF NOT EXISTS idx_track_album_id ON track (album_id);
CREATE INDEX IF NOT EXISTS idx_album_title ON album (title);
CREATE INDEX IF NOT EXISTS idx_artist_name ON artist (name);
CREATE INDEX IF NOT EXISTS idx_playlist_name ON playlist (name);
