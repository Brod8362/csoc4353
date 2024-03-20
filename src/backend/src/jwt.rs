use std::{collections::BTreeMap, time::Duration};

use hmac::{digest::KeyInit, Hmac};
use jwt::{SignWithKey, VerifyWithKey};
use rocket::{http::CookieJar, time::OffsetDateTime};
use sha2::Sha256;

pub fn generate_jwt(user_id: &str, duration: Duration, secret: &[u8]) -> String {
    //TODO actual secret
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();

    let expiry_time = OffsetDateTime::now_utc() + duration;
    let mut claims = BTreeMap::new();
    claims.insert("user_id", user_id.to_string());
    claims.insert("expires", format!("{}", expiry_time.unix_timestamp()));

    claims.sign_with_key(&key).unwrap()
}

pub fn validate_jwt(token: &str, secret: &[u8]) -> Option<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();

    let claims: BTreeMap<String, String> = match token.verify_with_key(&key) {
        Ok(b) => b,
        Err(_) => return None
    };

    let user_id_opt = claims.get("user_id");

    if user_id_opt.is_none() {
        return None;
    }

    if let Some(expires_str) = claims.get("expires") {
        //parse timestamp string into an OffsetDateTime
        let timestamp_epoch: i64 = match expires_str.parse() {
            Ok(t) => t,
            Err(_) => return None
        };
        let dt = OffsetDateTime::from_unix_timestamp(timestamp_epoch).unwrap();

        //if token not expired
        if dt > OffsetDateTime::now_utc() {
            return Some(user_id_opt.unwrap().clone());
        }
    }
    None
}

pub fn validate_from_cookie(cookies: &CookieJar<'_>, secret: &[u8]) -> Option<String> {
    match cookies.get("Authorization") {
        Some(cookie) => validate_jwt(cookie.value(), secret),
        None => None
    }
}