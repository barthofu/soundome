use rocket::http::Status;
use rocket::{catch, Request};

#[catch(default)]
pub fn default(status: Status, req: &Request) -> String {
    format!("{} ({})", status, req.uri())
}
