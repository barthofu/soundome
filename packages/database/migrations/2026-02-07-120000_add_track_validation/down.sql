-- SQLite doesn't support DROP COLUMN; recreate the table without the new columns.

PRAGMA foreign_keys=off;
BEGIN TRANSACTION;

CREATE TABLE track_new (
    id integer not null primary key autoincrement,
    title text not null,
    duration integer,
    album_id integer references album(id),
    track_number integer,
    disc_number integer,
    label text,
    date text,
    genre text,
    cover text,
    file_path text
);

INSERT INTO track_new (
    id,
    title,
    duration,
    album_id,
    track_number,
    disc_number,
    label,
    date,
    genre,
    cover,
    file_path
)
SELECT
    id,
    title,
    duration,
    album_id,
    track_number,
    disc_number,
    label,
    date,
    genre,
    cover,
    file_path
FROM track;

DROP TABLE track;
ALTER TABLE track_new RENAME TO track;

COMMIT;
PRAGMA foreign_keys=on;
