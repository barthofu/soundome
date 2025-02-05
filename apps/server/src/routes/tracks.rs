use rocket::serde::json::Json;
use rocket::{delete, get, patch, post};
use rocket_okapi::openapi;

use crate::models::track::{NewTrack, Track, UpdateTrack};
use crate::utils::database::Db;
use crate::utils::{macros, response};

// crud handlers

macros::crud_handlers::resource::get!("/tracks/<track_id>", track_id, track, Track);
macros::crud_handlers::resource::get_all!("/tracks", track, Track);
macros::crud_handlers::resource::create!("/tracks", track, Track, NewTrack);
macros::crud_handlers::resource::update!("/tracks/<track_id>", track_id, track, Track, UpdateTrack);
macros::crud_handlers::resource::delete!("/tracks/<track_id>", track_id, track);
