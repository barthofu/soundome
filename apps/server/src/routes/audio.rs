use rocket::{
    fs::NamedFile,
    get,
    response::{self, Responder},
    Request, Response,
};
/**
 * Resources:
 * https://rocket.rs/v0.5-rc/guide/responses/#custom-responder
 * https://github.com/SergioBenitez/Rocket/issues/95#issuecomment-354824883
 */
use std::path::Path;

// This is a custom responder that adds the Cache-control and Content-Type headers to the response.
pub struct MediaFile(NamedFile);

impl<'r> Responder<'r, 'r> for MediaFile {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-control", "max-age=86400") //  24h (24*60*60)
            .raw_header("Content-Type", "audio/mpeg")
            .ok()
    }
}

#[get("/audio")]
pub async fn stream() -> Option<MediaFile> {
    let audio_path = Path::new("tmp/song.mp3");

    NamedFile::open(&audio_path).await.ok().map(MediaFile)
}
