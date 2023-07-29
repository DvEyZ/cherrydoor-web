pub mod permissions;

use cherrydoor_models::{models::{AccessProfile, Permission, AccessProfilePermission}, full::AccessProfileFull, schema::{self, access_profiles}, insert::AccessProfileInsert, update::AccessProfileUpdate};
use diesel::{QueryDsl, SelectableHelper, ExpressionMethods, OptionalExtension, BelongingToDsl, result};
use diesel_async::RunQueryDsl;
use rocket::{get, post, patch, delete, serde::json::Json, State, response::status::Created};
use crate::{guards::auth::{Auth, OperatorUser}, db::get_connection};

use crate::{error::ApiError, db::{DB, DbConnection}};

type Error = ApiError;
type AccessProfilesResponse = Result<Json<Vec<AccessProfile>>, Error>;
type AccessProfileResponse = Result<Json<AccessProfileFull>, Error>;
type AccessProfileResponseCreated = Result<Created<Json<AccessProfileFull>>, Error>;

#[get("/")]
pub async fn list(
    _auth :Auth<OperatorUser>,

    db :&State<DB>
) -> AccessProfilesResponse {
    let mut conn = get_connection(db).await?;

    let access_profiles :Vec<AccessProfile> = match schema::access_profiles::table
        .select(AccessProfile::as_select())
    .load(&mut conn).await {
        Ok(profiles) => profiles,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(Json(access_profiles))
}

#[get("/<name>")]
pub async fn get<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> AccessProfileResponse {
    let mut conn = get_connection(db).await?;

    match get_full_access_profile(name, &mut conn).await {
        Ok(access_profile) => Ok(Json(access_profile)),
        Err(e) => Err(e)
    }
}

#[post("/", format = "application/json", data = "<access_profile>")]
pub async fn create(
    _auth :Auth<OperatorUser>,

    access_profile :Json<AccessProfileInsert>,
    db :&State<DB>
) -> AccessProfileResponseCreated {
    let mut conn = get_connection(db).await?;
    let name = access_profile.name.clone();

    if let Err(e) = diesel::insert_into(access_profiles::table)
        .values(access_profile.0)
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("Access profile {} already exists.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    match get_full_access_profile(&name, &mut conn).await {
        Ok(access_profile) => Ok(Created::new(format!("{}", access_profile.access_profile.name)).body(Json(access_profile))),
        Err(e) => Err(e)
    }
}

#[patch("/<name>", format = "application/json", data = "<access_profile>")]
pub async fn update<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    access_profile :Json<AccessProfileUpdate>,
    db :&State<DB>
) -> AccessProfileResponse {
    let mut conn = get_connection(db).await?;
    let old_access_profile = get_access_profile(name, &mut conn).await?;

    if let Err(e) = diesel::update(&old_access_profile)
        .set(&access_profile.0)
    .execute(&mut conn).await {
        return Err(ApiError::Internal(format!("{}", e))) 
    }

    match get_full_access_profile(name, &mut conn).await {
        Ok(access_profile) => Ok(Json(access_profile)),
        Err(e) => Err(e)
    }
}

#[delete("/<name>")]
pub async fn delete<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> AccessProfileResponse {
    let mut conn = get_connection(db).await?;
    let access_profile = match get_full_access_profile(name, &mut conn).await {
        Ok(access_profile) => access_profile,
        Err(e) => return Err(e)
    };

    let tasks = vec![
        diesel::delete(schema::access_profiles_permissions::table)
            .filter(schema::access_profiles_permissions::columns::access_profile_id.eq(&access_profile.access_profile.id))
            .execute(&mut conn).await,
        diesel::delete(&access_profile.access_profile)
            .execute(&mut conn).await
    ];

    for i in tasks {
        if let Err(e) = i {
            return Err(ApiError::Internal(format!("{}", e)));
        }
    }

    Ok(Json(access_profile))
}

async fn get_full_access_profile<'a, 'v>(
    name :&'v str,
    db :&mut DbConnection<'a>
) -> Result<AccessProfileFull, Error> {
    let access_profile = get_access_profile(name, db).await?;
    let permissions = get_all_permissions(&access_profile, db).await?;

    Ok(AccessProfileFull { 
        access_profile,
        permissions
    })
}

async fn get_access_profile<'a,'v>(
    name :&'v str,
    db :&mut DbConnection<'a>
) -> Result<AccessProfile, Error> {
    match schema::access_profiles::table
        .select(AccessProfile::as_select())
        .filter(schema::access_profiles::columns::name.eq(name))
    .first(db).await.optional() {
        Ok(maybe_profile) => {
            match maybe_profile {
                Some(profile) => Ok(profile),
                None => Err(ApiError::NotFound(format!("Access profile {} not found.", name)))
            }
        },
        Err(e) => Err(ApiError::Internal(format!("/access-profiles/{}", e)))
    }
}

async fn get_all_permissions<'a>(
    access_profile :&AccessProfile,
    db :&mut DbConnection<'a>
) -> Result<Vec<Permission>, Error> {
    let perms :Vec<(AccessProfilePermission, Permission)> = match AccessProfilePermission::belonging_to(&access_profile)
        .inner_join(schema::permissions::table)
        .select((AccessProfilePermission::as_select(), Permission::as_select()))
    .load(db).await {
        Ok(perms) => perms,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(perms.into_iter().map(|v| { v.1 }).collect())
}