create table track (
    id integer not null primary key autoincrement,
    title text not null,
    album text,
    year integer,
    cover text,
    duration integer,
    track_number integer,
    disc_number integer,
    source_url text,
    download_url
    file_type text not null,
    file_size integer not null,
    file_path text not null
);

create table album (
    id integer not null primary key autoincrement,
    title text not null,
    cover text,
    year integer
);

create table artist (
    id integer not null primary key autoincrement,
    name text not null,
    icon text
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

create table album_tracks (
    id integer not null primary key autoincrement,
    track_id integer not null,
    album_id integer not null,
    foreign key (track_id) references track(id),
    foreign key (album_id) references album(id)
);

create table artist_tracks (
    id integer not null primary key autoincrement,
    track_id integer not null,
    artist_id integer not null,
    foreign key (track_id) references track(id),
    foreign key (artist_id) references artist(id)
);

create table artist_albums (
    id integer not null primary key autoincrement,
    album_id integer not null,
    artist_id integer not null,
    foreign key (album_id) references album(id),
    foreign key (artist_id) references artist(id)
);

create table playlist_tracks (
    id integer not null primary key autoincrement,
    track_id integer not null,
    playlist_id integer not null,
    position integer not null,
    foreign key (track_id) references track(id),
    foreign key (playlist_id) references playlist(id)
);

create table track_genres (
    id integer not null primary key autoincrement,
    track_id integer not null,
    genre_id integer not null,
    foreign key (track_id) references track(id),
    foreign key (genre_id) references genre(id)
);
