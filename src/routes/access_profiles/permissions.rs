use cherrydoor_models::{insert::AccessProfilePermissionInsert, schema::access_profiles_permissions};
use serde::Deserialize;

use super::*;

type PermissionsResponse = Result<Json<Vec<Permission>>, Error>;

#[derive(Deserialize)]
pub struct AccessProfilePermissionAppend {
    permission_id :i32
}

impl AccessProfilePermissionAppend {
    pub fn into_insert(self, access_profile_id :i32) -> AccessProfilePermissionInsert {
        AccessProfilePermissionInsert { 
            access_profile_id, 
            permission_id: self.permission_id 
        }
    }
}

#[get("/<name>/permissions")]
pub async fn list<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> PermissionsResponse {
    let mut conn = get_connection(db).await?;
    let access_profile = get_access_profile(name, &mut conn).await?;

    let perms :Vec<(AccessProfilePermission, Permission)> = match AccessProfilePermission::belonging_to(&access_profile)
        .inner_join(schema::permissions::table)
        .select((AccessProfilePermission::as_select(), Permission::as_select()))
    .load(&mut conn).await {
        Ok(perms) => perms,
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    Ok(Json(perms.into_iter().map(|v| { v.1 }).collect()))
}

#[post("/<name>/permissions", format = "application/json", data = "<permission>")]
pub async fn assign<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    permission :Json<AccessProfilePermissionAppend>,
    db :&State<DB>,
) -> AccessProfileResponse {
    let mut conn = get_connection(db).await?;
    let access_profile = get_access_profile(name, &mut conn).await?;

    if let Err(e) = diesel::insert_into(access_profiles_permissions::table)
        .values(permission.0.into_insert(access_profile.id))
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("The permission is already assigned to profile {}.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    match get_full_access_profile(name, &mut conn).await {
        Ok(access_profile) => Ok(Json(access_profile)),
        Err(e) => Err(e)
    }
}

#[delete("/<name>/permissions/<id>")]
pub async fn remove<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    id :i32,
    db :&State<DB>
) -> AccessProfileResponse {
    let mut conn = get_connection(db).await?;
    let access_profile = get_access_profile(name, &mut conn).await?;

    match diesel::delete(access_profiles_permissions::table)
        .filter(access_profiles_permissions::columns::permission_id.eq(id))
        .filter(access_profiles_permissions::columns::access_profile_id.eq(access_profile.id))
    .execute(&mut conn).await {
        Ok(del_count) => {
            if del_count == 0 {
                return Err(ApiError::NotFound(format!("Permission of id {} either does not exist or is not associated with profile {}.", id, name)))
            }
        }
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    };

    match get_full_access_profile(name, &mut conn).await {
        Ok(access_profile) => Ok(Json(access_profile)),
        Err(e) => Err(e)
    }
}