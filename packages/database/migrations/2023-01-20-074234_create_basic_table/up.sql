-- Mise à jour de la migration pour correspondre aux modèles métier

create table track (
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
    file_path text,
    source text,
    source_url text,
    source_id text,
    provider text,
    provider_url text,
    provider_id text
);

create table track_source (
    id integer not null primary key autoincrement,
    track_id integer not null references track(id),
    external_id text not null,
    platform text not null -- "spotify", "soundcloud", "youtube", etc.
);

create table album (
    id integer not null primary key autoincrement,
    title text not null,
    album_type text not null,
    cover text,
    date text,
    url text
);

create table album_source (
    id integer not null primary key autoincrement,
    album_id integer not null references album(id),
    external_id text not null,
    platform text not null -- "spotify", "soundcloud", "youtube", etc.
);

create table artist (
    id integer not null primary key autoincrement,
    name text not null,
    url text,
    icon text
);

create table artist_source (
    id integer not null primary key autoincrement,
    artist_id integer not null references artist(id),
    external_id text not null,
    platform text not null -- "spotify", "soundcloud", "youtube", etc.
);

create table playlist (
    id integer not null primary key autoincrement,
    name text not null,
    source text not null,
    source_url text,
    cover text,
    last_sync timestamp
);

create table genre (
    id integer not null primary key autoincrement,
    name text not null
);

/* association tables */

create table artist_tracks (
    track_id integer not null,
    artist_id integer not null,
    primary key (track_id, artist_id),
    foreign key (track_id) references track(id) on delete cascade,
    foreign key (artist_id) references artist(id) on delete cascade
);

create table artist_albums (
    album_id integer not null,
    artist_id integer not null,
    primary key (album_id, artist_id),
    foreign key (album_id) references album(id) on delete cascade,
    foreign key (artist_id) references artist(id) on delete cascade
);

create table playlist_tracks (
    track_id integer not null,
    playlist_id integer not null,
    position integer,
    primary key (track_id, playlist_id),
    foreign key (track_id) references track(id) on delete cascade,
    foreign key (playlist_id) references playlist(id) on delete cascade
);

create table track_genres (
    track_id integer not null,
    genre_id integer not null,
    primary key (track_id, genre_id),
    foreign key (track_id) references track(id) on delete cascade,
    foreign key (genre_id) references genre(id) on delete cascade
);
