#!/usr/bin/env bash
set -euo pipefail

# Permanently removes every track marked for manual validation and its staged
# audio file. Stop Soundome before running this script.
#
# Usage: ./scripts/purge_pending_validations.sh [database] [temp-directory]
# Defaults: $DATABASE_URL or data/soundome.db; $SOUNDOME_TEMP_DOWNLOAD_DIR or ./temp

database_url="${1:-${DATABASE_URL:-data/soundome.db}}"
temp_dir="${2:-${SOUNDOME_TEMP_DOWNLOAD_DIR:-./temp}}"
database_path="${database_url#sqlite://}"

if [[ ! -f "$database_path" ]]; then
    printf 'Database not found: %s\n' "$database_path" >&2
    exit 1
fi

mapfile -t staged_files < <(
    sqlite3 -noheader -batch "$database_path" \
        'SELECT file_path FROM track WHERE needs_validation = 1 AND file_path IS NOT NULL;'
)

sqlite3 -batch "$database_path" <<'SQL'
PRAGMA foreign_keys = ON;
BEGIN IMMEDIATE;

CREATE TEMP TABLE pending_validation_ids AS
    SELECT id FROM track WHERE needs_validation = 1;

DELETE FROM playlist_tracks WHERE track_id IN (SELECT id FROM pending_validation_ids);
DELETE FROM artist_tracks WHERE track_id IN (SELECT id FROM pending_validation_ids);
DELETE FROM track_genres WHERE track_id IN (SELECT id FROM pending_validation_ids);
DELETE FROM track_ref WHERE track_id IN (SELECT id FROM pending_validation_ids);
DELETE FROM track WHERE id IN (SELECT id FROM pending_validation_ids);

DROP TABLE pending_validation_ids;
COMMIT;
SQL

for file_path in "${staged_files[@]}"; do
    [[ -z "$file_path" ]] && continue
    if [[ -f "$file_path" ]]; then
        rm -- "$file_path"
        printf 'Deleted staged file: %s\n' "$file_path"
    elif [[ -f "$temp_dir/$(basename "$file_path")" ]]; then
        rm -- "$temp_dir/$(basename "$file_path")"
        printf 'Deleted staged file: %s\n' "$temp_dir/$(basename "$file_path")"
    fi
done

# A failed or interrupted workflow may leave a staged audio file without a
# file_path row. Since this script is an explicit purge operation, remove all
# audio files directly in the configured staging directory as well.
if [[ -d "$temp_dir" ]]; then
    shopt -s nullglob
    for file_path in "$temp_dir"/*.{mp3,m4a,flac,ogg,opus,wav,aac}; do
        [[ -f "$file_path" ]] || continue
        rm -- "$file_path"
        printf 'Deleted staged file: %s\n' "$file_path"
    done
    shopt -u nullglob
fi

printf 'Pending validation tracks purged from %s.\n' "$database_path"
