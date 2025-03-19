use diesel::{Connection, SqliteConnection};

#[macro_use]
extern crate diesel;

pub mod macros;
pub mod models;
pub mod repositories;
pub mod schema;

pub fn get_connection(database_url: &str) -> SqliteConnection {
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
