#![macro_use]
extern crate rocket;
mod db;
mod error;

mod guards;
mod routes;

use diesel_async::{pooled_connection::{bb8::Pool, AsyncDieselConnectionManager}, AsyncMysqlConnection};
use guards::auth::SecretKeyWrapper;
use rocket::{launch, routes, http::Method, catchers};

use rocket_cors::{CorsOptions, AllowedOrigins};
use routes::{auth, web_ui_users, users, permissions, access_profiles, access::{self, CommandAddress}, status};

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let app = rocket::build();

    let db_uri = std::env::var("DATABASE_URI").unwrap();
    let key = SecretKeyWrapper { key: rocket::Config::from(app.figment()).secret_key };
    let command_addr = CommandAddress(std::env::var("COMMAND_ADDRESS").unwrap());

    let db = db::DB(
        Pool::builder()
            .build(AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(db_uri))
        .await.unwrap()
    );

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch, Method::Put, Method::Delete]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
    .to_cors().unwrap();

    
    app
        .attach(cors)
        .manage(db)
        .manage(key)
        .manage(command_addr)
        .mount("/auth", routes![
            auth::authenticate,     // POST /
        ])
        .mount("/web-ui-users", routes![
            web_ui_users::list,     // GET /
            web_ui_users::get,      // GET /<name>
            web_ui_users::create,   // POST /
            web_ui_users::update,   // PATCH /<name>
            web_ui_users::delete    // DELETE /<name>
        ])
        .mount("/users", routes![
            users::list,    // GET /
            users::get,     // GET /<name>
            users::create,  // POST /
            users::update,  // PATCH /<name>
            users::delete,  // DELETE /<name>
            users::access_codes::list,          // GET /<name>/access-codes
            users::access_codes::manual_add,    // POST /<name>/access-codes
            users::access_codes::register,      // POST /<name>/access-codes/register
            users::access_codes::get,           // GET /<name>/access-codes/<id>
            users::access_codes::delete,        // DELETE /<name>/access-codes/<id>
            users::permissions::list,       // GET /<name>/permissions
            users::permissions::assign,     // POST /<name>/permissions
            users::permissions::remove      // DELETE /<name>/permissions/<id>
        ])
        .mount("/permissions", routes![
            permissions::list,      // GET /
            permissions::get,       // GET /<name>
            permissions::create,    // POST /
            permissions::update,    // PATCH /<name>
            permissions::delete,    // DELETE /<name>
            permissions::users::list,   // GET /<name>/users
            permissions::users::assign, // POST /<name>/users
            permissions::users::remove, // DELETE /<name>/users/<user-id>
            permissions::access_profiles::list,     // GET /<name>/access-profiles
            permissions::access_profiles::assign,   // POST /<name>/access-profiles
            permissions::access_profiles::remove,   // DELETE /<name>/access-profiles/<profile-id>
        ])
        .mount("/access-profiles", routes![
            access_profiles::list,      // GET /
            access_profiles::get,       // GET /<name>
            access_profiles::create,    // POST /
            access_profiles::update,    // PATCH /<name>
            access_profiles::delete,    // DELETE /<name>
            access_profiles::permissions::list,     // GET /<name>/permissions
            access_profiles::permissions::assign,   // POST /<name>/permissions
            access_profiles::permissions::remove,   // DELETE /<name>/permissions/<id>
        ])
        .mount("/access", routes![
            access::open,   // POST /access/open
            access::code    // POST /access/code
        ])
        .mount("/status", routes![
            status::get
        ])
        .register("/", catchers![
            error::unauthorized,
            error::forbidden,
            error::not_found,
            error::unprocessable,
            error::internal
        ])
}
