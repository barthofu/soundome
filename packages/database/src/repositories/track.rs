use diesel::prelude::*;

use crate::{
    macros,
    models::track::{NewTrack, TrackEntity, UpdateTrack},
};

// basic CRUD operations

macros::resource::find_one!(track, TrackEntity);
macros::resource::find_all!(track, TrackEntity);
macros::resource::create!(track, TrackEntity, NewTrack);
macros::resource::update!(track, TrackEntity, UpdateTrack);
macros::resource::delete!(track);

// custom operations
