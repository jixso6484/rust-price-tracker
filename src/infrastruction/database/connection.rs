use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use uuid::Uuid;
use crate::components::error::error_cl::{Result, ErrorS};

use super::models::{Product, CreateProduct, UpdateProduct, PriceHistory};

pub struct ProductRepository{
    db_pool : Pool<Postgres>,
}
impl ProductRepository{
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|e| ErrorS::data("ProductRepository::new", format!("Failed to get DATABASE_URL: {}", e)))?;
        
        // Neon DB requires SSL
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&database_url)
            .await
            .map_err(|e| ErrorS::data("ProductRepository::new", format!("Failed to connect to Neon database: {}", e)).with_source(e))?;

        Ok(Self { db_pool: pool })
    }

    pub async fn create_product(&self, product: CreateProduct) -> Result<Product> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (
                product_name, current_price, original_price, 
                site, category, url, image, 
                coupon_code, valid_until, additional_benefits
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(product.product_name)
        .bind(product.current_price)
        .bind(product.original_price)
        .bind(product.site)
        .bind(product.category)
        .bind(product.url)
        .bind(product.image)
        .bind(product.coupon_code)
        .bind(product.valid_until)
        .bind(product.additional_benefits)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::create_product", format!("Failed to create product: {}", e)).with_source(e))?;

        Ok(product)
    }

    pub async fn get_product(&self, id: Uuid) -> Result<Option<Product>> {
        let product = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::get_product", format!("Failed to get product: {}", e)).with_source(e))?;

        Ok(product)
    }

    pub async fn update_product(&self, id: Uuid, update: UpdateProduct) -> Result<Option<Product>> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            UPDATE products 
            SET 
                product_name = COALESCE($2, product_name),
                current_price = COALESCE($3, current_price),
                original_price = COALESCE($4, original_price),
                site = COALESCE($5, site),
                category = COALESCE($6, category),
                url = COALESCE($7, url),
                image = COALESCE($8, image),
                coupon_code = COALESCE($9, coupon_code),
                valid_until = COALESCE($10, valid_until),
                additional_benefits = COALESCE($11, additional_benefits),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(update.product_name)
        .bind(update.current_price)
        .bind(update.original_price)
        .bind(update.site)
        .bind(update.category)
        .bind(update.url)
        .bind(update.image)
        .bind(update.coupon_code)
        .bind(update.valid_until)
        .bind(update.additional_benefits)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::update_product", format!("Failed to update product: {}", e)).with_source(e))?;

        Ok(product)
    }

    pub async fn delete_product(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM products WHERE id = $1"
        )
        .bind(id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::delete_product", format!("Failed to delete product: {}", e)).with_source(e))?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_products(&self, limit: i32, offset: i32) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            "SELECT * FROM products ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::list_products", format!("Failed to list products: {}", e)).with_source(e))?;

        Ok(products)
    }

    pub async fn list_products_by_site(&self, site: &str) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE site = $1 ORDER BY timestamp DESC"
        )
        .bind(site)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::list_products_by_site", format!("Failed to list products by site: {}", e)).with_source(e))?;

        Ok(products)
    }

    pub async fn list_products_by_category(&self, category: &str) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE category = $1 ORDER BY timestamp DESC"
        )
        .bind(category)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::list_products_by_category", format!("Failed to list products by category: {}", e)).with_source(e))?;

        Ok(products)
    }

    pub async fn add_price_history(&self, product_id: Uuid, price: f64, original_price: Option<f64>) -> Result<PriceHistory> {
        let discount_rate = match (price, original_price) {
            (current, Some(original)) if original > 0.0 => {
                Some(((original - current) / original * 100.0).round())
            }
            _ => None,
        };

        let history = sqlx::query_as::<_, PriceHistory>(
            r#"
            INSERT INTO price_history (product_id, price, original_price, discount_rate)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(product_id)
        .bind(price)
        .bind(original_price)
        .bind(discount_rate)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::add_price_history", format!("Failed to add price history: {}", e)).with_source(e))?;

        Ok(history)
    }

    pub async fn get_price_history(&self, product_id: Uuid, limit: i32) -> Result<Vec<PriceHistory>> {
        let history = sqlx::query_as::<_, PriceHistory>(
            "SELECT * FROM price_history WHERE product_id = $1 ORDER BY recorded_at DESC LIMIT $2"
        )
        .bind(product_id)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::get_price_history", format!("Failed to get price history: {}", e)).with_source(e))?;

        Ok(history)
    }

    pub async fn get_lowest_price(&self, product_id: Uuid) -> Result<Option<PriceHistory>> {
        let history = sqlx::query_as::<_, PriceHistory>(
            "SELECT * FROM price_history WHERE product_id = $1 AND is_lowest = true LIMIT 1"
        )
        .bind(product_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::get_lowest_price", format!("Failed to get lowest price: {}", e)).with_source(e))?;

        Ok(history)
    }
    pub async fn get_existing_product(&self,url : String)->Result<bool>{

        let exists = sqlx::query_as::<_,Product>(
            "SELECT * FROM products WHERE url = $1"
        )
        .bind(url)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::get_existing_product",format!("Failed to get product : {}",e)))?;

        Ok(exists.is_some())
    }
}