use cherrydoor_models::{models::WebUIUser, insert::WebUIUserInsert, update::WebUIUserUpdate, schema::web_ui_users};
use diesel::{QueryDsl, SelectableHelper, ExpressionMethods, OptionalExtension, result};
use diesel_async::RunQueryDsl;
use rocket::{get, post, patch, delete, serde::json::Json, State, response::status::Created};
use serde::{Serialize, Deserialize};

use crate::{db::{DB, get_connection}, error::ApiError, guards::auth::{Auth, AdminUser, OperatorUser}};

#[derive(Serialize)]
pub struct WebUIUserOutput {
    pub id :i32,
    pub name :String,
    pub is_admin :bool,
    pub ac_does_not_expire :bool
}

#[derive(Deserialize)]
pub struct WebUIUserCreate {
    name :String,
    password :String,
    is_admin :bool,
    ac_does_not_expire :bool
}

#[derive(Deserialize)]
pub struct WebUIUserPatch {
    password :Option<String>,
    is_admin :Option<bool>,
    ac_does_not_expire :Option<bool>
}

// Safety measure
impl From<WebUIUser> for WebUIUserOutput {
    fn from(user :WebUIUser) -> Self {
        Self {
            id: user.id,
            name: user.name,
            is_admin: user.is_admin,
            ac_does_not_expire: user.ac_does_not_expire
        }
    }
}

impl WebUIUserCreate {
    fn into_insert(self) -> WebUIUserInsert {
        WebUIUserInsert { 
            name: self.name, 
            password_hash: sha256::digest(self.password),
            is_admin: self.is_admin,
            ac_does_not_expire: self.ac_does_not_expire
        }
    }
}

impl WebUIUserPatch {
    fn into_update(self) -> WebUIUserUpdate {
        WebUIUserUpdate {
            password_hash: self.password.map(sha256::digest),
            is_admin: self.is_admin,
            ac_does_not_expire: self.ac_does_not_expire
        }
    }
}

type Error = ApiError;
type WebUIUsersResponse = Result<Json<Vec<WebUIUserOutput>>, Error>;
type WebUIUserResponse = Result<Json<WebUIUserOutput>, Error>;
type WebUIUserCreatedResponse = Result<Created<Json<WebUIUserOutput>>, Error>;


#[get("/")]
pub async fn list(
    _auth :Auth<OperatorUser>,

    db :&State<DB>
) -> WebUIUsersResponse {
    let mut conn = get_connection(db).await?;

    let wu_users :Vec<WebUIUser> = match web_ui_users::table
        .select(WebUIUser::as_select())
    .load(&mut conn).await {
        Ok(wu_users) => wu_users,
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    Ok(Json(wu_users.into_iter().map(|v| { v.into() }).collect()))
}

#[get("/<name>")]
pub async fn get<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db: &State<DB>
) -> WebUIUserResponse {
    let mut conn = get_connection(db).await?;

    let wu_user = match web_ui_users::table
        .select(WebUIUser::as_select())
        .filter(web_ui_users::columns::name.eq(name))
    .first(&mut conn).await.optional() {
        Ok(maybe_wu_user) => {
            match maybe_wu_user {
                Some(wu_user) => wu_user,
                None => return Err(ApiError::NotFound(format!("User {} not found.", name)))
            }
        },
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };


    Ok(Json(wu_user.into()))
}

#[post("/", format = "application/json", data = "<wu_user>")]
pub async fn create(
    _auth :Auth<AdminUser>,

    wu_user :Json<WebUIUserCreate>,
    db :&State<DB>
) -> WebUIUserCreatedResponse {
    let mut conn = get_connection(db).await?;
    let name = wu_user.0.name.clone();

    if let Err(e) = diesel::insert_into(web_ui_users::table)
        .values(wu_user.0.into_insert())
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("User {} already exists.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    }

    let wu_user :WebUIUser = match web_ui_users::table
        .select(WebUIUser::as_select())
        .filter(web_ui_users::columns::name.eq(name))
    .first(&mut conn).await {
        Ok(wu_user) => wu_user,
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    Ok(Created::new(format!("/web-ui-users/{}", &wu_user.name)).body(Json(wu_user.into())))
}

#[patch("/<name>", format = "application/json", data = "<wu_user>")]
pub async fn update<'a>(
    _auth :Auth<AdminUser>,

    name :&'a str,
    wu_user :Json<WebUIUserPatch>,
    db :&State<DB>
) -> WebUIUserResponse {
    let mut conn = get_connection(db).await?;

    let old_wu_user = match web_ui_users::table
        .select(WebUIUser::as_select())
        .filter(web_ui_users::columns::name.eq(name))
    .first(&mut conn).await.optional() {
        Ok(maybe_wu_user) => {
            match maybe_wu_user {
                Some(wu_user) => wu_user,
                None => return Err(ApiError::NotFound(format!("User {} not found.", name)))
            }
        },
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    match diesel::update(&old_wu_user)
        .set(wu_user.0.into_update())
    .execute(&mut conn).await {
        Ok(updated) => {
            if updated == 0 {
                return Err(ApiError::NotFound(format!("User {} not found.", name)));
            }
        },
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    let new_wu_user = match web_ui_users::table
        .select(WebUIUser::as_select())
        .filter(web_ui_users::columns::name.eq(name))
    .first(&mut conn).await {
        Ok(new_wu_user) => new_wu_user,
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    Ok(Json(new_wu_user.into()))
}

#[delete("/<name>")]
pub async fn delete<'a>(
    _auth :Auth<AdminUser>,

    name :&'a str,
    db :&State<DB>
) -> WebUIUserResponse {
    let mut conn = get_connection(db).await?;

    let wu_user = match web_ui_users::table
        .select(WebUIUser::as_select())
        .filter(web_ui_users::columns::name.eq(name))
    .first(&mut conn).await.optional() {
        Ok(maybe_wu_user) => {
            match maybe_wu_user {
                Some(wu_user) => wu_user,
                None => return Err(ApiError::NotFound(format!("User {} not found.", name)))
            }
        }
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };


    if let Err(e) = diesel::delete(&wu_user)
    .execute(&mut conn).await {
        return Err(ApiError::Internal(format!("{}", e)))
    };
    

    Ok(Json(wu_user.into()))
}