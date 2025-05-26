use domain::ports::repositories::ReferenceRepository;

pub struct DieselReferenceRepository {}

impl DieselReferenceRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl ReferenceRepository for DieselReferenceRepository {
    
    fn get_by_url(&self, conn: &mut diesel::SqliteConnection, url: &str) -> shared::types::SoundomeResult<shared::models::Reference> {
        schema::track_ref::table
            .filter(schema::track_ref::url.eq(url))
            .first::<ReferenceEn>(conn)
            .map
            .map_err(|err| {
                shared::errors::Error::Database(format!(
                    "Failed to get resource by url: {}",
                    err
                ))
            })?;
        Ok(reference.convert_to_domain())
    }
}