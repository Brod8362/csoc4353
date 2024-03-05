use std::{error::Error, num::NonZeroU32};

use ring::{digest, pbkdf2::{self, Algorithm}, rand::{self, SecureRandom}};
use rocket::futures;
use sqlx::{sqlite::SqlitePoolOptions, Executor, Pool, Row, Sqlite};
use futures::TryStreamExt;
use thiserror::Error;

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
static ALGORITHM: Algorithm = pbkdf2::PBKDF2_HMAC_SHA512;

#[derive(Error, Debug)]
pub enum AccountCreationError {
    #[error("Username already exists")]
    UsernameInUse,
    #[error("Empty passwords are not permitted")]
    EmptyPassword
}

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Username does not exist")]
    UsernameDoesNotExist,
    #[error("Incorrect password")]
    IncorrectPassword
}

/// Make sure all of the required tables exist in the database.
/// Tables will only be created if they do not exist. 
/// Should a table's schema ever be modified, the database will need to be re-made from scratch.
pub async fn init_tables(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    tx.execute("
        CREATE TABLE IF NOT EXISTS auth(
            user_id TEXT PRIMARY KEY NOT NULL,
            username TEXT UNIQUE NOT NULL,
            password_hash BLOB NOT NULL,
            password_salt BLOB NOT NULL
        );
    ").await?;
    tx.commit().await?;
    Ok(())
}

/// Initialize a connection to the database.
/// In this case, using SQLite, this is a local file.
/// The file must already exist, though it may be empty.
/// `database_uri` is expected to be in `sqlite://path/to/file.db` format.
/// sqlite does not natively support multithreading, so it may be wise to keep max_connections to 1.
pub async fn init_connection(database_uri: &str, max_connections: u32) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect(database_uri)
        .await?;
    init_tables(&pool).await?;
    Ok(pool)
}

/// Register a new user in the database.
/// This function will ensure the username is not duplicated. 
/// It also handles hashing and salting the password.
/// The ID of the created user will be returned.
pub async fn register_user(pool: &Pool<Sqlite>, username: &str, password: &str) -> Result<String, Box<dyn Error>> {
    let rs = sqlx::query("SELECT * FROM auth WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    if rs.is_some() {
        //this means at least one row exists with the same username, so we need to error out
        //TODO: make an actual error type here, do NOT just return Ok(())!
        return Err(Box::new(AccountCreationError::UsernameInUse))
    }

    if password.is_empty() {
        return Err(Box::new(AccountCreationError::EmptyPassword))
    }

    //hash and salt the password
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let rng = rand::SystemRandom::new();

    let mut salt = [0u8; CREDENTIAL_LEN];
    let mut hash = [0u8; CREDENTIAL_LEN];

    rng.fill(&mut salt).expect("Failed to generate salt");
    pbkdf2::derive(ALGORITHM, n_iter, &salt, password.as_bytes(), &mut hash);
    let vec_hash = Vec::from(hash);
    let vec_salt = Vec::from(salt);

    let user_id = "asd"; //TODO generate uuid here!!
    sqlx::query("INSERT INTO auth VALUES(?, ?, ?, ?)")
        .bind(user_id)
        .bind(username)
        .bind(vec_hash)
        .bind(vec_salt)
        .execute(pool)
        .await?;

    Ok(user_id.to_owned())
}

pub async fn authenticate_user(pool: &Pool<Sqlite>, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
    let rs_maybe = sqlx::query("SELECT password_hash, password_salt FROM auth WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    if rs_maybe.is_none() {
       //username does not exist
       return Err(Box::new(AuthenticationError::UsernameDoesNotExist))
    }

    let rs = rs_maybe.unwrap();

    let expected_hash: Vec<u8> = rs.try_get("password_hash")?;
    let salt: Vec<u8> = rs.try_get("password_salt")?;

    let n_iter = NonZeroU32::new(100_000).unwrap();

    let verified = pbkdf2::verify(ALGORITHM, n_iter, &salt, password.as_bytes(), &expected_hash);

    if verified.is_ok() {
        Ok(())
    } else {
        Err(Box::new(AuthenticationError::IncorrectPassword)) 
    }

}


mod tests {
    use rocket::tokio;

    use super::init_connection;

    #[tokio::test]
    pub async fn test_init_database() {
        let _: sqlx::Pool<sqlx::Sqlite> = init_connection("sqlite://:memory:", 1).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_create_user() {
        let pool = init_connection("sqlite://:memory:", 1).await.unwrap();
        let username = "test";
        let password = "test";

        let success = crate::database::register_user(&pool, username, password).await;
        assert!(success.is_ok());
        //creating a second user with the same ID should fail
        let should_fail_dupe_username = crate::database::register_user(&pool, username, password).await;
        assert!(should_fail_dupe_username.is_err()); //TODO: figure out a way to check the specific error
        //creating a with an empty password should fail
        let should_fail_empty_password = crate::database::register_user(&pool, "test2", "").await;
        assert!(should_fail_empty_password.is_err());
    }

    #[tokio::test]
    pub async fn test_authenticate_user() {
        let pool = init_connection("sqlite://:memory:", 1).await.unwrap();

        let create_user_res = crate::database::register_user(&pool, "test", "password").await;
        assert!(create_user_res.is_ok());

        let auth_ok_res = crate::database::authenticate_user(&pool, "test", "password").await;
        assert!(auth_ok_res.is_ok());

        let auth_bad_password = crate::database::authenticate_user(&pool, "test", "wrongpassword").await;
        assert!(auth_bad_password.is_err());

        let auth_bad_username = crate::database::authenticate_user(&pool, "wrongusername", "password").await;
        assert!(auth_bad_username.is_err());
    }
}