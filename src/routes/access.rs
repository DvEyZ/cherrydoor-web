use cherrydoor_models::{
    schema::{access_codes, users, permissions, access_profiles}, 
    models::{User, AccessCode, UserPermission, Permission, AccessProfilePermission, AccessProfile}
};
use cherrydoor_command::Command;
use diesel::{QueryDsl, SelectableHelper, ExpressionMethods, OptionalExtension, BelongingToDsl};
use diesel_async::RunQueryDsl;
use reqwest::StatusCode;
use rocket::{post, State, serde::json::Json, response::status::NoContent};
use serde::Deserialize;

use crate::{db::{self, DB, get_connection}, error::ApiError, guards::auth::{Auth, OperatorUser}};

#[derive(Deserialize)]
pub struct AccessCodeAccess {
    code :String
}

pub struct CommandAddress(pub String);

#[post("/open")]
pub async fn open(
    _auth :Auth<OperatorUser>,
    command_addr :&State<CommandAddress>
) -> Result<NoContent, ApiError> {
    let command = &Command::new()
        .open_for(5000)
        .display_text_for("Wejdz".to_string(), 5000)
        .set_color_for(0, 0, 0, 5000)
        .play_sound(1);

    let client = reqwest::Client::new();

    match client.post(format!("{}/", command_addr.0))
        .json(command)
        .send()
    .await {
        Ok(res) => {
            match res.status() {
                StatusCode::NO_CONTENT => {
                    Ok(NoContent)
                },
                _ => {
                    Err(ApiError::Internal(format!("Command server returned {}", res.text().await.unwrap_or("garbage".to_string()))))
                }
            }
        },
        Err(e) => Err(ApiError::Internal(format!("Error while connecting to command server: {}", e)))
    }
}

#[post("/code", format = "application/json", data = "<access>")]
pub async fn code(
    access :Json<AccessCodeAccess>,
    db :&State<DB>
) -> Result<NoContent, ApiError> {
    let mut conn = get_connection(db).await?;

    let apn = db::get_active_profile_name().await?;

    let ac :AccessCode = match access_codes::table
        .select(AccessCode::as_select())
        .filter(access_codes::columns::code.eq(&access.0.code))
    .first(&mut conn).await.optional() {
        Ok(maybe_ac) => match maybe_ac {
            Some(ac) => ac,
            None => return Err(ApiError::NotFound(String::from("Access code not registered.")))
        },
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    let user :User = match users::table
        .select(User::as_select())
        .filter(users::columns::id.eq(ac.user))
    .first(&mut conn).await {
        Ok(user) => user,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    let upwp :Vec<(UserPermission, Permission)> = match UserPermission::belonging_to(&user)
        .inner_join(permissions::table)
        .select((UserPermission::as_select(), Permission::as_select()))
    .load(&mut conn).await {
        Ok(uwpw) => uwpw,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };
    let perms :Vec<Permission> = upwp.into_iter().map(|v| { v.1 }).collect();

    let apwp :Vec<(AccessProfilePermission, AccessProfile)> = match AccessProfilePermission::belonging_to(&perms)
        .inner_join(access_profiles::table)
        .select((AccessProfilePermission::as_select(), AccessProfile::as_select()))
    .load(&mut conn).await {
        Ok(apwp) => apwp,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };
    let aps :Vec<AccessProfile> = apwp.into_iter().map(|v| { v.1 }).collect();

    if aps.iter().any(|prof| {
        prof.name == apn
    }) {
        Ok(NoContent)
    } else {
        Err(ApiError::BadRequest(String::from("You don't have the permission to enter now.")))
    }
}