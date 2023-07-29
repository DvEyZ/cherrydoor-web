use serde::Serialize;
use rocket::{get, serde::json::Json};

use crate::{guards::auth::{Auth, OperatorUser}, error::ApiError};

#[derive(Serialize)]
#[serde(tag = "status")]
pub enum StatusEntry {
    Ok,
    Warn {message :String},
    Err {message :String}
}

#[derive(Serialize)]
pub struct Status {
    controller :StatusEntry,
    lock :StatusEntry,
    rfid :StatusEntry,
    led :StatusEntry,
    speaker :StatusEntry,
    wifi :StatusEntry
}

type Error = ApiError;
type StatusResponse = Result<Json<Status>, Error>;

#[get("/")]
pub async fn get(
    _auth :Auth<OperatorUser>
) -> StatusResponse {
    Ok(Json(Status {
        controller: StatusEntry::Ok,
        lock: StatusEntry::Warn { message: "something is wrong".to_string() },
        rfid: StatusEntry::Ok,
        led: StatusEntry::Err { message: "down".to_string() },
        speaker: StatusEntry::Ok,
        wifi: StatusEntry::Ok
    }))
}