use super::*;
use cherrydoor_models::{insert::UserPermissionInsert, schema::users_permissions};
use rocket::{get, post, delete};
use serde::Deserialize;

type UserPermissionsResponse = Result<Json<Vec<Permission>>, Error>;

#[derive(Deserialize)]
pub struct UserPermissionAppend {
    permission_id :i32
}

impl UserPermissionAppend {
    pub fn into_insert(self, user_id :i32) -> UserPermissionInsert {
        UserPermissionInsert { 
            user_id, 
            permission_id: self.permission_id
        }
    }
}

#[get("/<name>/permissions")]
pub async fn list<'a>(
    _auth :Auth<OperatorUser>,
    
    name :&'a str,
    db :&State<DB>
) -> UserPermissionsResponse {
    let mut conn = get_connection(db).await?;
    let user = get_user(name, &mut conn).await?;

    let perms :Vec<Permission> = match UserPermission::belonging_to(&user)
        .inner_join(schema::permissions::table)
        .select((UserPermission::as_select(), Permission::as_select()))
    .load(&mut conn).await {
        Ok(perms) => perms.into_iter().map(|v :(UserPermission, Permission)| { v.1 }).collect(),
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(Json(perms))
}

#[post("/<name>/permissions", format = "application/json", data = "<permission>")]
pub async fn assign<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    permission :Json<UserPermissionAppend>,
    db :&State<DB>
) -> UserResponse {
    let mut conn = get_connection(db).await?;
    let user = get_user(name, &mut conn).await?;

    if let Err(e) = diesel::insert_into(users_permissions::table)
        .values(permission.0.into_insert(user.id))
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("User {} already has this permission.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    match get_full_user(name, &mut conn).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e)
    }
}

#[delete("/<name>/permissions/<id>")]
pub async fn remove<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    id :i32,
    db :&State<DB>
) -> UserResponse {
    let mut conn = get_connection(db).await?;
    let user = get_user(name, &mut conn).await?;
    
    match diesel::delete(users_permissions::table)
        .filter(users_permissions::columns::permission_id.eq(id))
        .filter(users_permissions::columns::user_id.eq(user.id))
    .execute(&mut conn).await {
        Ok(del_count) => {
            if del_count == 0 {
                return Err(ApiError::NotFound(format!("Permission {} either does not exist, or does not belong to user {}", id, name)));
            }
        }
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    }

    match get_full_user(name, &mut conn).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e)
    }
}