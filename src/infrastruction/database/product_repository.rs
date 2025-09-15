use super::connection::DbPool;
use crate::domain::models::models::{Product, PriceHistory};
use anyhow::Result;
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct ProductRepository {
    pool: DbPool,
}

impl ProductRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_product(&self, product: &Product) -> Result<Uuid> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO products (
                product_name, current_price, original_price, 
                site, category, url, image, coupon_code, 
                valid_until, additional_benefits
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
            product.product_name,
            product.current_price,
            product.original_price,
            product.site,
            product.category,
            product.url,
            product.image,
            product.coupon_code,
            product.valid_until,
            serde_json::to_value(&product.additional_benefits)?
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn find_product_by_name_and_site(
        &self, 
        name: &str, 
        site: &str
    ) -> Result<Option<Product>> {
        let row = sqlx::query!(
            r#"
            SELECT id, product_name, current_price, original_price,
                   site, category, url, image, coupon_code, 
                   valid_until, additional_benefits, timestamp
            FROM products
            WHERE product_name = $1 AND site = $2
            "#,
            name,
            site
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Product {
            id: Some(r.id.to_string()),
            product_name: r.product_name,
            current_price: r.current_price.map(|d| d.to_string().parse().unwrap_or(0.0)),
            original_price: r.original_price.map(|d| d.to_string().parse().unwrap_or(0.0)),
            site: r.site,
            category: r.category,
            url: r.url,
            image: r.image,
            coupon_code: r.coupon_code,
            valid_until: r.valid_until,
            additional_benefits: serde_json::from_value(r.additional_benefits).unwrap_or_default(),
            timestamp: r.timestamp,
        }))
    }

    pub async fn update_product_price(
        &self, 
        product_id: &Uuid, 
        new_price: f64,
        original_price: Option<f64>
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE products 
            SET current_price = $2, 
                original_price = COALESCE($3, original_price),
                updated_at = NOW()
            WHERE id = $1
            "#,
            product_id,
            new_price as f64,
            original_price as Option<f64>
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_price_history(
        &self,
        product_id: &Uuid,
        price: f64,
        original_price: Option<f64>
    ) -> Result<Uuid> {
        let discount_rate = original_price.map(|orig| {
            if orig > 0.0 {
                ((orig - price) / orig) * 100.0
            } else {
                0.0
            }
        });

        let lowest_price = self.get_lowest_price(product_id).await?;
        let is_lowest = lowest_price.map_or(true, |low| price < low);

        if is_lowest {
            sqlx::query!(
                "UPDATE price_history SET is_lowest = false WHERE product_id = $1",
                product_id
            )
            .execute(&self.pool)
            .await?;
        }

        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO price_history (
                product_id, price, original_price, discount_rate, is_lowest
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            product_id,
            price as f64,
            original_price as Option<f64>,
            discount_rate as Option<f64>,
            is_lowest
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn get_price_history(
        &self,
        product_id: &Uuid,
        limit: i64
    ) -> Result<Vec<PriceHistory>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, product_id, price, original_price, 
                   discount_rate, is_lowest, recorded_at
            FROM price_history
            WHERE product_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2
            "#,
            product_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| PriceHistory {
            id: r.id,
            product_id: r.product_id.to_string(),
            price: r.price.to_string().parse().unwrap_or(0.0),
            original_price: r.original_price.map(|d| d.to_string().parse().unwrap_or(0.0)),
            discount_rate: r.discount_rate.map(|d| d.to_string().parse().unwrap_or(0.0)),
            is_lowest: r.is_lowest,
            recorded_at: r.recorded_at,
        }).collect())
    }

    async fn get_lowest_price(&self, product_id: &Uuid) -> Result<Option<f64>> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT MIN(price) as "min_price"
            FROM price_history
            WHERE product_id = $1
            "#,
            product_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.map(|d| d.to_string().parse().unwrap_or(0.0)))
    }

    pub async fn get_products_with_discounts(&self, min_discount: f64) -> Result<Vec<Product>> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT p.id, p.product_name, p.current_price, p.original_price,
                   p.site, p.category, p.url, p.image, p.coupon_code,
                   p.valid_until, p.additional_benefits, p.timestamp
            FROM products p
            JOIN price_history ph ON p.id = ph.product_id
            WHERE ph.discount_rate >= $1
                AND ph.recorded_at >= NOW() - INTERVAL '24 hours'
            ORDER BY p.timestamp DESC
            "#,
            min_discount as f64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Product {
            id: Some(r.id.to_string()),
            product_name: r.product_name,
            current_price: r.current_price.map(|d| d.to_string().parse().unwrap_or(0.0)),
            original_price: r.original_price.map(|d| d.to_string().parse().unwrap_or(0.0)),
            site: r.site,
            category: r.category,
            url: r.url,
            image: r.image,
            coupon_code: r.coupon_code,
            valid_until: r.valid_until,
            additional_benefits: serde_json::from_value(r.additional_benefits).unwrap_or_default(),
            timestamp: r.timestamp,
        }).collect())
    }
}