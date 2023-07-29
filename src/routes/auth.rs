use cherrydoor_models::{schema::web_ui_users, models::WebUIUser};
use diesel::{QueryDsl, SelectableHelper, ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use rocket::{post, State, serde::json::Json};
use serde::{Deserialize, Serialize};

use crate::{db::{DB, get_connection}, error::ApiError, guards::auth::{SecretKeyWrapper, WebUIUserAuthorization}};

#[derive(Deserialize)]
pub struct WebUIUserLogin {
    name :String,
    password :String
}

#[derive(Serialize)]
pub struct WebUIUserToken {
    token :String
}

type Error = ApiError;
type AuthResponse = Result<Json<WebUIUserToken>, Error>;

#[post("/", format = "application/json", data = "<auth>")]
pub async fn authenticate(
    auth :Json<WebUIUserLogin>,
    db :&State<DB>,
    secret :&State<SecretKeyWrapper>
) -> AuthResponse {
    let mut conn = get_connection(db).await?;

    let user :WebUIUser = match web_ui_users::table
        .select(WebUIUser::as_select())
        .filter(web_ui_users::columns::name.eq(&auth.0.name))
    .first(&mut conn).await.optional() {
        Ok(maybe_user) => {
            match maybe_user {
                Some(user) => user,
                None => return  Err(ApiError::NotFound(format!("User {} not found.", &auth.0.name)))
            }
        },
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    let password_hash = sha256::digest(auth.0.password);

    if password_hash != user.password_hash {
        return Err(ApiError::Forbidden("Bad password.".to_string()))
    }

    let auth = WebUIUserAuthorization {
        exp: jsonwebtoken::get_current_timestamp() + 3600,
        name: user.name,
        is_admin: user.is_admin
    };

    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(), 
        &auth, 
        &jsonwebtoken::EncodingKey::from_secret(secret.key.to_string().as_bytes())
    ) {
        Ok(token) => token,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(Json(WebUIUserToken { token }))
}