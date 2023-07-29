use std::marker::PhantomData;

use rocket::{request::{FromRequest, Outcome}, Request, http::Status, State, outcome};
use serde::{Serialize, Deserialize};

pub struct SecretKeyWrapper {
    pub key :rocket::config::SecretKey
}

#[derive(Serialize, Deserialize)]
pub struct WebUIUserAuthorization {
    pub exp :u64,
    pub name :String,
    pub is_admin :bool
}

pub trait AuthorizationProvider {
    fn authorize(auth :&WebUIUserAuthorization) -> bool;
}

pub struct OperatorUser;
pub struct AdminUser;

impl AuthorizationProvider for OperatorUser {
    fn authorize(_auth :&WebUIUserAuthorization) -> bool {
        true
    }
}

impl AuthorizationProvider for AdminUser {
    fn authorize(auth :&WebUIUserAuthorization) -> bool {
        auth.is_admin
    }
}

pub struct Auth<T :AuthorizationProvider> {
    auth_provider :PhantomData<T>,
    pub claim :WebUIUserAuthorization
}

#[rocket::async_trait]
impl<T :AuthorizationProvider, 'r> FromRequest<'r> for Auth<T> {
    type Error = ();

    async fn from_request(request :&'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_token = match request.headers().get_one("Authorization") {
            Some(token) => {
                let mut spl = token.split(' ');
                if let Some(name) = spl.next() {
                    if name != "Bearer" {
                        return Outcome::Failure((Status::BadRequest, ()))
                    }
                };
                match spl.next() {
                    Some(token) => token,
                    None => return Outcome::Failure((Status::BadRequest, ()))
                }

            }
            None => return Outcome::Failure((Status::Unauthorized, ()))
        };

        let secret = match request.guard::<&State<SecretKeyWrapper>>().await.map(|conf| &conf.key) {
            outcome::Outcome::Success(key) => key,
            _ => return Outcome::Failure((Status::InternalServerError, ()))
        };

        let claim = match jsonwebtoken::decode::<WebUIUserAuthorization>(
            auth_token, 
            &jsonwebtoken::DecodingKey::from_secret(secret.to_string().as_bytes()),
            &jsonwebtoken::Validation::default()
        ) {
            Ok(claim) => claim.claims,
            Err(_) => return Outcome::Failure((Status::Unauthorized, ()))
        };

        if claim.exp < jsonwebtoken::get_current_timestamp() {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        if !T::authorize(&claim) {
            return Outcome::Failure((Status::Forbidden, ()))
        }

        return Outcome::Success(Auth {claim, auth_provider: PhantomData});
    }
}