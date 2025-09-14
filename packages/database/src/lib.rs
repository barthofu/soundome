use diesel::{r2d2::{ConnectionManager, Pool}, Connection, SqliteConnection};

#[macro_use]
extern crate diesel;

pub mod entities;
pub mod repositories;
pub mod mappers;
pub mod schema;
pub mod macros;

pub fn init_connection(database_url: &str) -> SqliteConnection {
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn init_pool(database_url: &str) -> SqlitePool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect(&format!("Failed to create pool connection to {}", database_url))
}

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
