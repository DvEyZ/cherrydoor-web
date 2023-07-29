use cherrydoor_models::insert::AccessCodeInsert;
use serde::Deserialize;

use crate::db::get_connection;

use super::*;

type AccessCodesResponse = Result<Json<Vec<AccessCode>>, Error>;
type AccessCodeResponse = Result<Json<AccessCode>, Error>;


#[derive(Deserialize)]
pub struct AccessCodeCreate {
    code :String
}

impl AccessCodeCreate {
    pub fn into_insert(self, user_id :i32) -> AccessCodeInsert {
        AccessCodeInsert {
            code: self.code,
            user: user_id
        }
    }
}

#[get("/<name>/access-codes")]
pub async fn list<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> AccessCodesResponse {
    let mut conn = get_connection(db).await?;
    let user = get_user(name, &mut conn).await?;
    Ok(Json(get_all_access_codes(&user, &mut conn).await?))
}

#[get("/<name>/access-codes/<id>")]
pub async fn get<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    id :i32,
    db :&State<DB>
) -> AccessCodeResponse {
    let mut conn = get_connection(db).await?;

    let user = get_user(name, &mut conn).await?;
    match get_access_code(&user, id, &mut conn).await {
        Ok(access_code) => Ok(Json(access_code)),
        Err(e) => Err(e)
    } 
}

#[post("/<name>/access-codes", format = "application/json", data = "<code>")]
pub async fn manual_add<'a>(
    _auth :Auth<OperatorUser>,
    
    name :&'a str,
    code :Json<AccessCodeCreate>,
    db :&State<DB>
) -> UserResponse {
    let mut conn = get_connection(db).await?;
    let user = get_user(name, &mut conn).await?;
    
    if let Err(e) = diesel::insert_into(schema::access_codes::table)
        .values(code.0.into_insert(user.id))
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("This access code is already registered.")))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };
    
    match get_full_user(name, &mut conn).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e)
    }
}

#[post("/<name>/access-codes/register")]
pub async fn register<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> UserResponse {
    Err(ApiError::NotImplemented("Work in progress.".to_string()))
}

#[delete("/<name>/access-codes/<id>")]
pub async fn delete<'a>(
    name :&'a str,
    id :i32,
    db :&State<DB>
) -> UserResponse {
    let mut conn = get_connection(db).await?;
    let user = get_user(name, &mut conn).await?;

    if let Err(e) = diesel::delete(schema::access_codes::table)
        .filter(schema::access_codes::columns::id.eq(id))
        .filter(schema::access_codes::columns::user.eq(user.id))
    .execute(&mut conn).await {
        return Err(ApiError::Internal(format!("{}", e)))
    }

    match get_full_user(name, &mut conn).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e)
    }
}