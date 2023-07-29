use cherrydoor_models::{insert::AccessProfilePermissionInsert, schema::access_profiles_permissions};
use serde::Deserialize;

use super::*;

type AccessProfilesResponse = Result<Json<Vec<AccessProfile>>, Error>;

#[derive(Deserialize)]
pub struct AccessProfilePermissionAppend {
    access_profile_id :i32
}

impl AccessProfilePermissionAppend {
    pub fn into_insert(self, permission_id :i32) -> AccessProfilePermissionInsert {
        AccessProfilePermissionInsert {
            access_profile_id: self.access_profile_id,
            permission_id
        }
    }
}

#[get("/<name>/access-profiles")]
pub async fn list<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    db :&State<DB>
) -> AccessProfilesResponse {
    let mut conn = get_connection(db).await?;
    let perm = get_permission(name, &mut conn).await?;

    match get_all_access_profiles(&perm, &mut conn).await {
        Ok(perms) => Ok(Json(perms)),
        Err(e) => Err(e)
    }
}

#[post("/<name>/access-profiles", format = "application/json", data="<access_profile>")]
pub async fn assign<'a>(
    _auth :Auth<OperatorUser>,

    name :&'a str,
    access_profile :Json<AccessProfilePermissionAppend>,
    db :&State<DB>
) -> PermissionResponse {
    let mut conn = get_connection(db).await?;
    let perm = get_permission(name, &mut conn).await?;

    if let Err(e) = diesel::insert_into(schema::access_profiles_permissions::table)
        .values(access_profile.0.into_insert(perm.id))
    .execute(&mut conn).await {
        if let result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _) = e {
            return Err(ApiError::Conflict(format!("The access profile is already assigned to permission {}.", &name)))
        } else {
            return Err(ApiError::Internal(format!("{}", e)))
        }
    };

    match get_full_permission(name, &mut conn).await {
        Ok(perm) => Ok(Json(perm)),
        Err(e) => Err(e)
    }
}

#[delete("/<name>/access-profiles/<id>")]
pub async fn remove<'a>(
    _auth :Auth<OperatorUser>,
    
    name :&'a str,
    id :i32,
    db :&State<DB>
) -> PermissionResponse {
    let mut conn = get_connection(db).await?;
    let perm = get_permission(name, &mut conn).await?;

    match diesel::delete(access_profiles_permissions::table)
        .filter(access_profiles_permissions::columns::access_profile_id.eq(id))
        .filter(access_profiles_permissions::columns::permission_id.eq(perm.id))
    .execute(&mut conn).await {
        Ok(del_count) => {
            if del_count == 0 {
                return Err(ApiError::NotFound(format!("User with ID {} either does not exist, or does not have permission {}.", id, name)))
            }
        }
        Err(e) => return Err(ApiError::Internal(format!("{}", e)))
    }

    match get_full_permission(name, &mut conn).await {
        Ok(perm) => Ok(Json(perm)),
        Err(e) => Err(e)
    }
}