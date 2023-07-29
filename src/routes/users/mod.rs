pub mod access_codes;
pub mod permissions;

use cherrydoor_models::{models::{User, AccessCode, Permission, UserPermission}, full::UserFull, schema::{users, self, users_permissions}, insert::UserInsert, update::UserUpdate};
use diesel::{QueryDsl, SelectableHelper, ExpressionMethods, BelongingToDsl, OptionalExtension, result};
use diesel_async::{RunQueryDsl};
use rocket::{get, post, patch, delete, serde::json::Json, State, response::status::Created};

use crate::{error::ApiError, db::{DB, DbConnection, get_connection}, guards::auth::{Auth, OperatorUser}};

type Error = ApiError;
type UsersResponse = Result<Json<Vec<User>>, Error>;
type UserResponse = Result<Json<UserFull>, Error>;
type UserResponseCreated = Result<Created<Json<UserFull>>, Error>;

#[get("/?<page>")]
pub async fn list(
    _auth :Auth<OperatorUser>,

    page :Option<i64>,
    db :&State<DB>
) -> UsersResponse {
    let mut conn = get_connection(db).await?;

    let users = match users::table
        .select(User::as_select())
        .order(users::columns::id.asc())
        .limit(10)
        .offset(10 * page.unwrap_or(0))
    .load(&mut conn).await {
        Ok(users) => users,
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    Ok(Json(users))
}

#[get("/<name>")]
pub async fn get<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> UserResponse {
    let mut conn = get_connection(db).await?;
    
    match get_full_user(name, &mut conn).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e)
    }
}

#[post("/", format = "application/json", data = "<user>")]
pub async fn create(
    _auth :Auth<OperatorUser>,

    user :Json<UserInsert>,
    db :&State<DB>
) -> UserResponseCreated {
    let mut conn = get_connection(db).await?;
    let name = user.0.name.clone();

    if let Err(e) = diesel::insert_into(users::table)
        .values(user.0)
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("User {} already exists.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    }

    match get_full_user(&name, &mut conn).await {
        Ok(user) => Ok(Created::new(format!("/users/{}", user.user.name)).body(Json(user))),
        Err(e) => Err(e)
    }
}

#[patch("/<name>", format = "application/json", data = "<user>")]
pub async fn update<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    user :Json<UserUpdate>,
    db :&State<DB>
) -> UserResponse {
    let mut conn = get_connection(db).await?;
    let old_user = get_user(name, &mut conn).await?;

    if let Err(e) = diesel::update(&old_user)
        .set(&user.0)
    .execute(&mut conn).await {
        return Err(ApiError::Internal(format!("{}", e)))
    }

    match get_full_user(name, &mut conn).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e)
    }
}

#[delete("/<name>")]
pub async fn delete<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> UserResponse {
    let mut conn = get_connection(db).await?;

    let user = get_full_user(name, &mut conn).await?;

    let tasks = vec![
        diesel::delete(users_permissions::table)
            .filter(users_permissions::columns::user_id.eq(&user.user.id))
            .execute(&mut conn).await,
        diesel::delete(schema::access_codes::table)
            .filter(schema::access_codes::columns::user.eq(&user.user.id))
            .execute(&mut conn).await,
        diesel::delete(&user.user)
            .execute(&mut conn).await
    ];

    for i in tasks {
        if let Err(e) = i {
            return Err(ApiError::Internal(format!("{}", e)));
        }
    }

    Ok(Json(user))
}

async fn get_user<'a, 'v>(
    name :&'v str,
    db :&mut DbConnection<'a>
) -> Result<User, Error> {
    match users::table
        .select(User::as_select())
        .filter(users::columns::name.eq(name))
    .first(db).await.optional() {
        Ok(maybe_user) => {
            match maybe_user {
                Some(user) => Ok(user),
                None => Err(ApiError::NotFound(format!("User {} not found.", name)))
            }
        }
        Err(e) => {
            Err(ApiError::Internal(format!("{}", e)))
        }
    }
}

async fn get_all_access_codes<'a>(
    user :&User,
    db :&mut DbConnection<'a>
) -> Result<Vec<AccessCode>, Error> {
    match AccessCode::belonging_to(user)
        .select(AccessCode::as_select())
    .load(db).await {
        Ok(access_codes) => Ok(access_codes),
        Err(e) => {
            Err(ApiError::Internal(format!("{}", e)))
        }
    }
}

async fn get_all_permissions<'a>(
    user :&User,
    db :&mut DbConnection<'a>
) -> Result<Vec<Permission>, Error> {
    let user_permissions :Vec<(UserPermission, Permission)> = match UserPermission::belonging_to(user)
        .inner_join(schema::permissions::table)
        .select((UserPermission::as_select(), Permission::as_select()))
    .load(db).await {
        Ok(permissions) => permissions,
        Err(e) => {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    Ok(user_permissions.into_iter().map(|v| { v.1 }).collect())
}

async fn get_access_code<'a>(
    user :&User,
    id :i32,
    db :&mut DbConnection<'a>
) -> Result<AccessCode, Error> {
    match AccessCode::belonging_to(user)
        .select(AccessCode::as_select())
        .filter(schema::access_codes::columns::id.eq(id))
    .first(db).await.optional() {
        Ok(maybe_access_code) => {
            match maybe_access_code {
                Some(access_code) => Ok(access_code),
                None => Err(ApiError::NotFound(format!("Access code {} does not exist or does not belong to user {}", id, user.name)))
            }
        }
        Err(e) => {
            Err(ApiError::Internal(format!("{}", e)))
        }
    }
}

async fn get_full_user<'a, 'v>(
    name :&'v str,
    db :&mut DbConnection<'a>
) -> Result<UserFull, Error> {
    let user = get_user(name, db).await?;
    let access_codes = get_all_access_codes(&user, db).await?;
    let permissions = get_all_permissions(&user, db).await?;

    Ok(UserFull {
        user, access_codes, permissions
    })
}