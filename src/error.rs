use rocket::{response::Responder, http::{Status, ContentType}, Response, catch, serde::json::Json};
use serde::Serialize;

use std::{io::Cursor, fmt::Display};

#[derive(Serialize)]
pub struct ErrorResponse {
    message :String
}

#[allow(dead_code)]
pub enum ApiError {
    BadRequest(String),     // 400
    Unauthorized(String),   // 401
    Forbidden(String),      // 403
    NotFound(String),       // 404
    Conflict(String),       // 409

    Internal(String),       // 500
    NotImplemented(String)  // 501
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::BadRequest(s) |
            Self::Unauthorized (s) |
            Self::Forbidden(s) |
            Self::NotFound(s) |
            Self::Conflict(s) |
            Self::Internal(s) |
            Self::NotImplemented(s) => s
        })
    }
}

impl ApiError {
    pub fn status(&self) -> Status {
        match &self {
            Self::BadRequest(_) => Status::BadRequest,
            Self::Unauthorized(_) => Status::Unauthorized,
            Self::Forbidden(_) => Status::Forbidden,
            Self::NotFound(_) => Status::NotFound,
            Self::Conflict(_) => Status::Conflict,
            Self::Internal(_) => Status::InternalServerError,
            Self::NotImplemented(_) => Status::NotImplemented
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = self.status();
        let body = serde_json::to_string(&ErrorResponse {
            message: self.to_string()
        }).unwrap();
        
        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
        .ok()
    }
}

#[catch(401)]
pub fn unauthorized() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: String::from("You need to authenticate to access this resource.")
    })
}

#[catch(403)]
pub fn forbidden() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: String::from("You don't have permission to access this resource.")
    })
}

#[catch(404)]
pub fn not_found() -> Json<ErrorResponse> {
    Json(ErrorResponse { 
        message: String::from("URL not found.")
    })
}

#[catch(422)]
pub fn unprocessable() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: String::from("The request was well-formed but was unable to be followed due to semantic errors.")
    })
}

#[catch(500)]
pub fn internal() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: String::from("Something went very wrong, and it most likely isn't your fault.")
    })
}