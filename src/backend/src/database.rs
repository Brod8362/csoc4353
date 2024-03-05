use sqlx::{sqlite::SqlitePoolOptions, Executor, Pool, Sqlite};

/// Make sure all of the required tables exist in the database.
/// Tables will only be created if they do not exist. 
/// Should a table's schema ever be modified, the database will need to be re-made from scratch.
pub async fn init_tables(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    tx.execute("CREATE TABLE IF NOT EXISTS auth(a TEXT);").await?;
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

async fn get_user(pool: &Pool<Sqlite>) -> Result<String, sqlx::Error> {
    Ok("<TEMPORARY>".to_owned())
}