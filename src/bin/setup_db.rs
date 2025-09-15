use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")?;
    println!("Connecting to Neon DB...");
    
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;
    
    println!("✅ Connected to Neon DB!");
    
    // Read SQL file
    let sql = std::fs::read_to_string("create_tables.sql")?;
    
    println!("Creating tables...");
    
    // Execute SQL
    sqlx::raw_sql(&sql)
        .execute(&pool)
        .await?;
    
    println!("✅ Tables created successfully!");
    
    // Verify tables exist
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'public' 
         AND table_name IN ('products', 'price_history')"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("\n📋 Created tables:");
    for (table,) in tables {
        println!("  - {}", table);
    }
    
    Ok(())
}