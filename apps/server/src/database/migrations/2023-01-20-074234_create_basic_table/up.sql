-- Your SQL goes here

create table track (

    id integer not null primary key autoincrement,
    title text not null,
    album text,
    year integer,
    cover text,
    duration integer,
    track_number integer,
    disc_number integer,
    comments text,
    label text,
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