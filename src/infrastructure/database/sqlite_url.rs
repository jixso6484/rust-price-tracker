use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use once_cell::sync::Lazy;

static DB_POOL: Lazy<PgPool> = Lazy::new(|| {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .expect("Failed to create connection pool");
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS urls (
                id SERIAL PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create table");
        
        pool
    })
});

pub async fn save_url(url: &str) -> Result<bool> {
    let pool = &*DB_POOL;
    
    match sqlx::query("INSERT INTO urls (url) VALUES ($1)")
        .bind(url)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(true),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Ok(false)
        }
        Err(e) => Err(anyhow::anyhow!("Database error: {:?}", e))
    }
}

pub async fn check_url_exists(url: &str) -> Result<bool> {
    let pool = &*DB_POOL;
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM urls WHERE url = $1")
        .bind(url)
        .fetch_one(pool)
        .await?;
    
    Ok(count > 0)
}

pub async fn get_all_urls() -> Result<Vec<String>> {
    let pool = &*DB_POOL;
    
    let urls: Vec<String> = sqlx::query_scalar("SELECT url FROM urls ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    
    Ok(urls)
}

pub async fn delete_url(url: &str) -> Result<bool> {
    let pool = &*DB_POOL;
    
    let result = sqlx::query("DELETE FROM urls WHERE url = $1")
        .bind(url)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}