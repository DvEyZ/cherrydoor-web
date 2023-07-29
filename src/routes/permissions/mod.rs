pub mod users;
pub mod access_profiles;

use cherrydoor_models::{models::{Permission, UserPermission, User, AccessProfile, AccessProfilePermission}, full::PermissionFull, schema::{self, users_permissions, access_profiles_permissions}, insert::PermissionInsert, update::PermissionUpdate};
use diesel::{QueryDsl, SelectableHelper, ExpressionMethods, BelongingToDsl, OptionalExtension, result};
use diesel_async::RunQueryDsl;
use rocket::{get, post, patch, delete, serde::json::Json, State, response::status::Created};

use crate::{error::ApiError, db::{DB, DbConnection, get_connection}, guards::auth::{Auth, OperatorUser}};

type Error = ApiError;
type PermissionsResponse = Result<Json<Vec<Permission>>, Error>;
type PermissionResponse = Result<Json<PermissionFull>, Error>;
type PermissionResponseCreated = Result<Created<Json<PermissionFull>>, Error>;


#[get("/")]
pub async fn list(
    _auth :Auth<OperatorUser>,

    db :&State<DB>
) -> PermissionsResponse {
    let mut conn = get_connection(db).await?;

    let perms :Vec<Permission> = match schema::permissions::table
        .select(Permission::as_select())
    .load(&mut *conn).await {
        Ok(perms) => perms,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(Json(perms))
}

#[get("/<name>")]
pub async fn get<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> PermissionResponse {
    let mut conn = get_connection(db).await?;

    match get_full_permission(name, &mut conn).await {
        Ok(perm) => Ok(Json(perm)),
        Err(e) => Err(e)
    }
}

#[post("/", format = "application/json", data = "<permission>")]
pub async fn create(
    _auth :Auth<OperatorUser>,

    permission :Json<PermissionInsert>,
    db :&State<DB>
) -> PermissionResponseCreated {
    let mut conn = get_connection(db).await?;
    let name = permission.0.name.clone();

    if let Err(e) = diesel::insert_into(schema::permissions::table)
        .values(permission.0)
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("Permission {} already exists.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    match get_full_permission(&name, &mut conn).await {
        Ok(perm) => Ok(Created::new(format!("/permissions/{}", perm.permission.name)).body(Json(perm))),
        Err(e) => Err(e)
    }
}

#[patch("/<name>", format = "application/json", data = "<permission>")]
pub async fn update<'a>(
    _auth :Auth<OperatorUser>,

    permission :Json<PermissionUpdate>,
    name :&'a str,
    db :&State<DB>
) -> PermissionResponse {
    let mut conn = get_connection(db).await?;
    let old_permission = get_permission(name, &mut conn).await?;

    if let Err(e) = diesel::update(&old_permission)
        .set(&permission.0)
    .execute(&mut conn).await {
        return Err(ApiError::Internal(format!("{}", e)))
    };

    match get_full_permission(name, &mut conn).await {
        Ok(perm) => Ok(Json(perm)),
        Err(e) => Err(e)
    }
}

#[delete("/<name>")]
pub async fn delete<'a>(
    _auth :Auth<OperatorUser>,
    
    name :&'a str,
    db :&State<DB>
) -> PermissionResponse {
    let mut conn = get_connection(db).await?;

    let permission = get_full_permission(name, &mut conn).await?;

    let tasks = [
        diesel::delete(users_permissions::table)
            .filter(users_permissions::columns::permission_id.eq(permission.permission.id))
            .execute(&mut conn).await,
        diesel::delete(access_profiles_permissions::table)
            .filter(access_profiles_permissions::columns::permission_id.eq(permission.permission.id))
            .execute(&mut conn).await,
        diesel::delete(&permission.permission).execute(&mut conn).await
    ];

    for i in tasks {
        if let Err(e) = i {
            return Err(ApiError::Internal(format!("{}", e)));
        }
    }

    Ok(Json(permission))
}

async fn get_full_permission<'a, 'v>(
    name :&'v str,
    db :&mut DbConnection<'a>
) -> Result<PermissionFull, Error> {
    let permission = get_permission(name, db).await?;
    let users = get_all_users(&permission, db).await?;
    let access_profiles = get_all_access_profiles(&permission, db).await?;

    Ok(PermissionFull { 
        permission, 
        users,
        access_profiles 
    })
}

async fn get_permission<'a, 'v>(
    name :&'v str,
    db :&mut DbConnection<'a>
) -> Result<Permission, Error> {
    match schema::permissions::table
        .select(Permission::as_select())
        .filter(schema::permissions::columns::name.eq(name))
    .first(db).await.optional() {
        Ok(maybe_perm) => {
            match maybe_perm {
                Some(perm) => Ok(perm),
                None => Err(ApiError::NotFound(format!("Permission {} not found.", name)))
            }
        },
        Err(e) => Err(ApiError::Internal(format!("{}", e)))
    }
}

async fn get_all_users<'a>(
    perm :&Permission,
    db :&mut DbConnection<'a>
) -> Result<Vec<User>, Error> {
    let users :Vec<(UserPermission, User)> = match UserPermission::belonging_to(&perm)
        .inner_join(schema::users::table)
        .select((UserPermission::as_select(), User::as_select()))
    .load(db).await {
        Ok(users) => users,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(users.into_iter().map(|v| { v.1 }).collect())
}

async fn get_all_access_profiles<'a>(
    perm :&Permission,
    db :&mut DbConnection<'a>
) -> Result<Vec<AccessProfile>, Error> {
    let access_profiles :Vec<(AccessProfilePermission, AccessProfile)> = match AccessProfilePermission::belonging_to(&perm)
        .inner_join(schema::access_profiles::table)
        .select((AccessProfilePermission::as_select(), AccessProfile::as_select()))
    .load(db).await {
        Ok(access_profiles) => access_profiles,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(access_profiles.into_iter().map(|v| { v.1 }).collect())
}