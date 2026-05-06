use okapi::openapi3::Responses;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response, Result};
use rocket::serde::Serialize;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::add_schema_response;
use schemars::JsonSchema;
use std::fmt;
use std::io::Cursor;
use thiserror::Error;

#[derive(JsonSchema, Serialize, Debug, Clone)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CustomError {
    pub status: Status,
    pub code: String,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum Error {
    // base http errrors
    #[error("{}", Status::InternalServerError)]
    Internal,
    #[error("{}", Status::BadRequest)]
    BadRequest,
    #[error("{}", Status::Unauthorized)]
    Unauthorized,
    #[error("{}", Status::Forbidden)]
    Forbidden,
    #[error("{}", Status::NotFound)]
    NotFound,
    #[error("{}", Status::NotModified)]
    NotModified,

    // on the fly error
    #[error("")]
    Custom(CustomError),

    // custom errors
    #[error("This resource already exists")]
    AlreadyExists,
    #[error("An error occured with the database")]
    DieselError(#[from] diesel::result::Error),
}

// get HTTP status from error type
impl Error {
    fn get_http_status(&self) -> Status {
        match self {
            Error::Internal => Status::InternalServerError,
            Error::BadRequest => Status::BadRequest,
            Error::Unauthorized => Status::Unauthorized,
            Error::Forbidden => Status::Forbidden,
            Error::NotFound => Status::NotFound,
            Error::NotModified => Status::NotModified,

            Error::AlreadyExists => Status::Conflict,
            Error::DieselError(_) => Status::InternalServerError,

            _ => Status::InternalServerError,
        }
    }

    fn get_code(&self) -> String {
        match self {
            Error::Internal => "Internal".to_string(),
            Error::BadRequest => "BadRequest".to_string(),
            Error::Unauthorized => "Unauthorized".to_string(),
            Error::Forbidden => "Forbidden".to_string(),
            Error::NotFound => "NotFound".to_string(),
            Error::NotModified => "NotModified".to_string(),

            Error::AlreadyExists => "AlreadyExists".to_string(),
            Error::DieselError(_) => "DieselError".to_string(),

            _ => "Internal".to_string(),
        }
    }
}

// rocket API error
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> Result<'static> {
        match self {
            Error::Custom(err) => {
                let err_response = serde_json::to_string(&ErrorResponse {
                    code: err.code,
                    message: err.message,
                })
                .unwrap();

                Response::build()
                    .status(err.status)
                    .header(ContentType::JSON)
                    .sized_body(err_response.len(), Cursor::new(err_response))
                    .ok()
            }

            _ => {
                let err_response = serde_json::to_string(&ErrorResponse {
                    code: self.get_code(),
                    message: self.to_string(),
                })
                .unwrap();

                Response::build()
                    .status(self.get_http_status())
                    .header(ContentType::JSON)
                    .sized_body(err_response.len(), Cursor::new(err_response))
                    .ok()
            }
        }
    }
}

impl OpenApiResponderInner for Error {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<ErrorResponse>();

        add_schema_response(&mut responses, 500, "application/json", schema.clone())?;
        add_schema_response(&mut responses, 400, "application/json", schema.clone())?;
        add_schema_response(&mut responses, 403, "application/json", schema.clone())?;
        add_schema_response(&mut responses, 404, "application/json", schema.clone())?;
        add_schema_response(&mut responses, 409, "application/json", schema.clone())?;

        Ok(responses)
    }
}

impl From<shared::errors::Error> for Error {
    fn from(err: shared::errors::Error) -> Self {
        Error::Custom(CustomError {
            status: Status::InternalServerError,
            code: "Internal".to_string(),
            message: err.to_string(),
        })
    }
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CustomError: {} - {}", self.code, self.message)
    }
}
