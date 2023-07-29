use diesel_async::{pooled_connection::{bb8::{Pool, PooledConnection}}, AsyncMysqlConnection};

use crate::error::ApiError;

pub type DbPool = Pool<AsyncMysqlConnection>;
pub type DbConnection<'a> = PooledConnection<'a, AsyncMysqlConnection>;
pub struct DB(pub DbPool);

pub async fn get_connection(db :& DB) -> Result<DbConnection<'_>, ApiError> {
    match db.0.get().await {
        Ok(conn) => Ok(conn),
        Err(e) => Err(ApiError::Internal(format!("{}", e)))
    }
}

pub async fn get_active_profile_name() -> Result<String, ApiError> {
    Ok(String::from("break"))
}