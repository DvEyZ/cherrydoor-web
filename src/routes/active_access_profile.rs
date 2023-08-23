use std::{sync::Arc, fmt::Display};
use async_mutex::Mutex;
use cherrydoor_command::Command;
use cherrydoor_models::{schema::{self, AccessProfileAccessMode}, models::AccessProfile};
use diesel::{QueryDsl, SelectableHelper, OptionalExtension, ExpressionMethods};
use diesel_async::RunQueryDsl;
use reqwest::StatusCode;
use rocket::{serde::json::Json, get, post, response::status::NoContent, State};
use std::error::Error;
use serde::{Serialize, Deserialize};

use crate::{db::{DB, get_connection}, guards::auth::{Auth, OperatorUser}, error::ApiError};

#[derive(Debug)]
struct CommandResponseError {
    message :String
}

impl Error for CommandResponseError {}

impl Display for CommandResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct ActiveAccessProfile {
    active_profile_name :Arc<Mutex<String>>,
    command_addr :String
}

impl ActiveAccessProfile {
    pub async fn new(db :&DB, command_addr :String) -> Self {
        let mut conn = get_connection(db).await.unwrap();

        let profile = schema::access_profiles::table
            .select(AccessProfile::as_select())
        .first(&mut conn).await.unwrap();

        let self_prototype = Self {
            active_profile_name :Arc::new(Mutex::new(String::from(""))),
            command_addr
        };

        self_prototype.set(profile).await.unwrap();

        self_prototype
    }

    pub async fn set(&self, access_profile :AccessProfile) -> Result<(), Box<dyn Error>> {
        let mut s = self.active_profile_name.lock().await;
        *s = access_profile.name;

        let color = Rgb::from_hex_string(&access_profile.color)?;

        let mut command = Command::new()
            .display_text(access_profile.display_text)
            .set_color(color.r, color.g, color.b);

        if access_profile.access_mode == AccessProfileAccessMode::OpenLock {
            command = command.open()
        }

        let client = reqwest::Client::new();

        // Close the door
        let close_res = client.post(format!("{}/", self.command_addr))
            .json(&Command::new().close())
            .send()
        .await?;
    
        match close_res.status() {
                StatusCode::NO_CONTENT => {},
                _ => {
                    return Err(Box::new(CommandResponseError {
                        message: format!("Command server returned {}", close_res.text().await.unwrap_or("garbage".to_string()))
                    }))
                }
        };

        // Execute the actual set command
        let res = client.post(format!("{}/", self.command_addr))
            .json(&command)
            .send()
        .await?;
    
        match res.status() {
            StatusCode::NO_CONTENT => {},
            _ => {
                return Err(Box::new(CommandResponseError {
                    message: format!("Command server returned {}", res.text().await.unwrap_or("garbage".to_string()))
                }))
            }
        }
        
        Ok(())
    }

    pub async fn get(&self) -> String {
        let v = self.active_profile_name.lock().await;
        v.clone()
    }
}

struct Rgb {
    pub r :u8,
    pub g :u8,
    pub b :u8
}

#[derive(Debug, Clone)]
struct RgbParseError;

impl std::error::Error for RgbParseError {}

impl Display for RgbParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid RGB string.")
    }
}

impl Rgb {
    fn parse_one(part :&str) -> Result<u8, RgbParseError> {
        u8::from_str_radix(part, 16).or(Err(RgbParseError))
    }

    pub fn from_hex_string(string :&str) -> Result<Self, RgbParseError> {
        if string.len() == 7 {
            let r = Rgb::parse_one(&string[1..3])?;
            let g = Rgb::parse_one(&string[3..5])?;
            let b = Rgb::parse_one(&string[5..7])?;

            Ok(Rgb {r, g, b})          
        }
        else if string.len() == 4 {
            let r = Rgb::parse_one(&string[1..3])?;
            let g = Rgb::parse_one(&string[3..5])?;
            let b = Rgb::parse_one(&string[5..7])?;

            Ok(Rgb {r: r + 16 * r, g: g + 16 * g, b: b + 16 * b})
        }
        else {
            Err(RgbParseError)
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct ActiveAccessProfileModel {
    name :String
}

#[post("/", format = "application/json", data = "<data>")]
pub async fn set(
    _auth :Auth<OperatorUser>,
    db :&State<DB>,
    aacp :&State<ActiveAccessProfile>,

    data :Json<ActiveAccessProfileModel> 
) -> Result<NoContent, ApiError> {
    let mut conn = get_connection(db).await?;

    let ap :AccessProfile = match schema::access_profiles::table
        .select(AccessProfile::as_select())
        .filter(schema::access_profiles::columns::name.eq(&data.0.name))
    .first(&mut conn).await.optional() {
        Ok(maybe_ap) => {
            match maybe_ap {
                Some(ap) => ap,
                None => return Err(ApiError::NotFound(format!("Access profile {} not found.", &data.0.name)))
            }
        },
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    match aacp.set(ap).await {
        Ok(_) => Ok(NoContent),
        Err(e) => Err(ApiError::Internal(format!("{}", e)))
    }
}

#[get("/")]
pub async fn get(
    _auth :Auth<OperatorUser>,
    aacp :&State<ActiveAccessProfile>
) -> Json<ActiveAccessProfileModel> {
    Json(ActiveAccessProfileModel {
        name: aacp.get().await
    })
}