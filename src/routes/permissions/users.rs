use cherrydoor_models::{insert::UserPermissionInsert, schema::users_permissions};
use serde::Deserialize;

use super::*;

type UsersResponse = Result<Json<Vec<User>>, Error>;

#[derive(Deserialize)]
pub struct UserPermissionAppend {
    user_id :i32
}

impl UserPermissionAppend {
    pub fn into_insert(self, permission_id :i32) -> UserPermissionInsert {
        UserPermissionInsert {
            user_id: self.user_id,
            permission_id
        }
    }
}

#[get("/<name>/users")]
pub async fn list<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> UsersResponse {
    let mut conn = get_connection(db).await?;
    let perm = get_permission(name, &mut conn).await?;

    match get_all_users(&perm, &mut conn).await {
        Ok(users) => Ok(Json(users)),
        Err(e) => Err(e)
    }
}

#[post("/<name>/users", format = "application/json", data = "<user>")]
pub async fn assign<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    user :Json<UserPermissionAppend>,
    db :&State<DB>
) -> PermissionResponse {
    let mut conn = get_connection(db).await?;
    let perm = get_permission(name, &mut conn).await?;

    if let Err(e) = diesel::insert_into(schema::users_permissions::table)
        .values(user.0.into_insert(perm.id))
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("The user already has the permission {}.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    match get_full_permission(name, &mut conn).await {
        Ok(perm) => Ok(Json(perm)),
        Err(e) => Err(e)
    }
}

#[delete("/<name>/users/<id>")]
pub async fn remove<'a>(
    _auth :Auth<OperatorUser>,
    
    name :&'a str,
    id :i32,
    db :&State<DB>
) -> PermissionResponse {
    let mut conn = get_connection(db).await?;
    let perm = get_permission(name, &mut conn).await?;

    match diesel::delete(users_permissions::table)
        .filter(users_permissions::columns::user_id.eq(id))
        .filter(users_permissions::columns::permission_id.eq(perm.id))
    .execute(&mut conn).await {
        Ok(del_count) => {
            if del_count == 0 {
                return Err(ApiError::NotFound(format!("User with ID {} either does not exist, or does not have permission {}.", id, name)))
            }
        }
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    match get_full_permission(name, &mut conn).await {
        Ok(perm) => Ok(Json(perm)),
        Err(e) => Err(e)
    }
}