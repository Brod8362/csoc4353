use std::{error::Error, num::NonZeroU32};

use ring::{digest, pbkdf2::{self, Algorithm}, rand::{self, SecureRandom}};
use uuid::Uuid;
use sqlx::{sqlite::SqlitePoolOptions, Executor, Pool, Row, Sqlite};
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
            password_salt BLOB NOT NULL,
            full_name TEXT NOT NULL,
            address1 TEXT NOT NULL,
            city1 TEXT NOT NULL,
            state1 TEXT NOT NULL,
            zip1 TEXT NOT NULL,
            address2 TEXT NOT NULL,
            city2 TEXT NOT NULL,
            state2 TEXT NOT NULL,
            zip2 TEXT NOT NULL
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

    let user_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO auth VALUES(?, ?, ?, ?)")
        .bind(&user_id)
        .bind(username)
        .bind(vec_hash)
        .bind(vec_salt)
        .execute(pool)
        .await?;

    Ok(user_id)
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

// update profile function
pub async fn update_profile(pool: &Pool<Sqlite>, user_id: &str, full_name: Option<&str>, address1: Option<&str>, city1: Option<&str>, state1: Option<&str>, zip1: Option<&str>, address2: Option<&str>, city2:Option<&str>, state2: Option<&str>, zip2: Option<&str>) -> Result<(), Box<dyn Error>> {
    // check if user exists
    let rs_maybe = sqlx::query("SELECT * FROM profile WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    // if user does not exist, return error
    if rs_maybe.is_none() {
        return Err(Box::new(AuthenticationError::UsernameDoesNotExist))
    }

    // check if required fields are empty
    let required_fields = [full_name, address1, city1, state1, zip1];
    if let Some(missing_field) = required_fields.iter().find(|x| x.is_none()) {
        return Err(Box::new(AccountCreationError::EmptyPassword))
    } else { // update profile
        sqlx::query("UPDATE profile SET full_name = ?, address1 = ?, city1 = ?, state1 = ?, zip1 = ? WHERE user_id = ?")
            .bind(full_name). bind(address1). bind(city1). bind(state1). bind(zip1). bind(user_id). execute(pool).await?;
    }

    // the info for address2 is optional, but all fields must be filled if any are filled
    if address2.is_some() {
        let required_fields = [address2, city2, state2, zip2];
        if let Some(missing_field) = required_fields.iter().find(|x| x.is_none()) {
            return Err(Box::new(AccountCreationError::EmptyPassword))
        } else { // update profile
            sqlx::query("UPDATE profile SET address2 = ?, city2 = ?, state2 = ?, zip2 = ? WHERE user_id = ?")
                .bind(address2). bind(city2). bind(state2). bind(zip2). bind(user_id). execute(pool).await?;
        }
    }

    Ok(())
}

/*
pub async fn store_quote(pool: &Pool<Sqlite>, gallons: &i32, address: &str, date: &str) -> Result<String, Box<dyn Error>>{
    let rs
}
*/


mod tests {
    use rocket::tokio;
    #[tokio::test]
    pub async fn test_init_database() {
        let _: sqlx::Pool<sqlx::Sqlite> = crate::database::init_connection("sqlite://:memory:", 1).await.unwrap();
    }

    #[tokio::test]
    pub async fn test_create_user() {
        let pool = crate::database::init_connection("sqlite://:memory:", 1).await.unwrap();
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
        let pool = crate::database::init_connection("sqlite://:memory:", 1).await.unwrap();

        let create_user_res = crate::database::register_user(&pool, "test", "password").await;
        assert!(create_user_res.is_ok());

        let auth_ok_res = crate::database::authenticate_user(&pool, "test", "password").await;
        assert!(auth_ok_res.is_ok());

        let auth_bad_password = crate::database::authenticate_user(&pool, "test", "wrongpassword").await;
        assert!(auth_bad_password.is_err());

        let auth_bad_username = crate::database::authenticate_user(&pool, "wrongusername", "password").await;
        assert!(auth_bad_username.is_err());
    }

    // test update profile
    #[tokio::test]
    pub async fn test_update_profile() {
        let pool = care::database::init_connection("sqlite://memory:", 1).await.unwrap();
        let full_name = "John Doe";
        let address1 = "5113 Rainflower Circle S";
        let city1 = "League City";
        let state1 = "TX";
        let zip1 = "77573";
        let address2 = "1600 Pennsylvania Ave NW";
        let city2 = "Washington";
        let state2 = "DC";
        let zip2 = "20500";
        let user_id = "test";

        let create_user_res = crate::database::register_user(&pool, "test", "password").await;
        assert!(create_user_res.is_ok());

        // test update profile with all fields filled
        let update_profile_res = crate::database::update_profile(&pool, full_name, address1, city1, state1, zip1, address2, city2, state2, zip2).await;
        assert!(update_profile_res.is_ok());

        // test update profile with all fields except address2 fields
        let update_profile_res = crate::database::update_profile(&pool, full_name, address1, city1, state1, zip1, None, None, None, None).await;
        assert!(update_profile_res.is_ok());

        // test update profile with missing required field
        let update_profile_res = crate::database::update_profile(&pool, full_name, address1, city1, state1, "").await;
        assert!(update_profile_res.is_err());

        // test update profile with missing required field
        let update_profile_res = crate::database::update_profile(&pool, full_name, address1, city1, state1, zip1, address2, city2, state2, "").await;
        assert!(update_profile_res.is_err());
    }
}