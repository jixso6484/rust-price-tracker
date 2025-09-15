use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;
use crate::domain::product::{Product, CreateProduct, UpdateProduct, PriceHistory, CreatePriceHistory};
use crate::components::error::error_cl::{Result, ErrorS};

pub struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| ErrorS::data("ProductRepository::new", format!("Database connection failed: {}", e)))?;
        
        println!("✅ Database connection established");
        Ok(Self { pool })
    }
    
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// 새 상품 생성
    pub async fn create(&self, product: CreateProduct) -> Result<Product> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let record = sqlx::query!(
            r#"
            INSERT INTO products (
                id, product_name, current_price, original_price, site, category, 
                url, image_url, coupon_code, valid_until, additional_benefits, 
                is_rocket_delivery, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
            id,
            product.product_name,
            product.current_price,
            product.original_price,
            product.site,
            product.category,
            product.url,
            product.image_url,
            product.coupon_code,
            product.valid_until,
            &product.additional_benefits,
            product.is_rocket_delivery,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::create", format!("Failed to create product: {}", e)))?;
        
        let mut created_product = Product {
            id: Some(record.id),
            product_name: record.product_name,
            current_price: record.current_price,
            original_price: record.original_price,
            discount_rate: None,
            site: record.site,
            category: record.category,
            url: record.url,
            image_url: record.image_url,
            coupon_code: record.coupon_code,
            valid_until: record.valid_until,
            additional_benefits: record.additional_benefits.unwrap_or_default(),
            is_rocket_delivery: record.is_rocket_delivery.unwrap_or(false),
            created_at: record.created_at.unwrap_or(now),
            updated_at: record.updated_at.unwrap_or(now),
        };
        
        created_product.calculate_discount_rate();
        Ok(created_product)
    }
    
    /// URL로 상품 찾기
    pub async fn find_by_url(&self, url: &str) -> Result<Option<Product>> {
        let record = sqlx::query!(
            "SELECT * FROM products WHERE url = $1",
            url
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::find_by_url", format!("Query failed: {}", e)))?;
        
        if let Some(record) = record {
            let mut product = Product {
                id: Some(record.id),
                product_name: record.product_name,
                current_price: record.current_price,
                original_price: record.original_price,
                discount_rate: None,
                site: record.site,
                category: record.category,
                url: record.url,
                image_url: record.image_url,
                coupon_code: record.coupon_code,
                valid_until: record.valid_until,
                additional_benefits: record.additional_benefits.unwrap_or_default(),
                is_rocket_delivery: record.is_rocket_delivery.unwrap_or(false),
                created_at: record.created_at.unwrap_or_else(Utc::now),
                updated_at: record.updated_at.unwrap_or_else(Utc::now),
            };
            product.calculate_discount_rate();
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }
    
    /// ID로 상품 찾기
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>> {
        let record = sqlx::query!(
            "SELECT * FROM products WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::find_by_id", format!("Query failed: {}", e)))?;
        
        if let Some(record) = record {
            let mut product = Product {
                id: Some(record.id),
                product_name: record.product_name,
                current_price: record.current_price,
                original_price: record.original_price,
                discount_rate: None,
                site: record.site,
                category: record.category,
                url: record.url,
                image_url: record.image_url,
                coupon_code: record.coupon_code,
                valid_until: record.valid_until,
                additional_benefits: record.additional_benefits.unwrap_or_default(),
                is_rocket_delivery: record.is_rocket_delivery.unwrap_or(false),
                created_at: record.created_at.unwrap_or_else(Utc::now),
                updated_at: record.updated_at.unwrap_or_else(Utc::now),
            };
            product.calculate_discount_rate();
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }
    
    /// 가격 업데이트
    pub async fn update_price(&self, id: Uuid, new_price: f64, original_price: Option<f64>) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            UPDATE products 
            SET current_price = $1, original_price = COALESCE($2, original_price), updated_at = $3
            WHERE id = $4
            "#,
            new_price,
            original_price,
            now,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::update_price", format!("Update failed: {}", e)))?;
        
        Ok(())
    }
    
    /// 할인 상품 목록 조회
    pub async fn list_discounted(&self, min_discount: f64) -> Result<Vec<Product>> {
        let records = sqlx::query!(
            r#"
            SELECT * FROM products 
            WHERE current_price IS NOT NULL 
              AND original_price IS NOT NULL 
              AND current_price < original_price
              AND ((original_price - current_price) / original_price * 100) >= $1
            ORDER BY ((original_price - current_price) / original_price * 100) DESC
            "#,
            min_discount
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::list_discounted", format!("Query failed: {}", e)))?;
        
        let mut products = Vec::new();
        for record in records {
            let mut product = Product {
                id: Some(record.id),
                product_name: record.product_name,
                current_price: record.current_price,
                original_price: record.original_price,
                discount_rate: None,
                site: record.site,
                category: record.category,
                url: record.url,
                image_url: record.image_url,
                coupon_code: record.coupon_code,
                valid_until: record.valid_until,
                additional_benefits: record.additional_benefits.unwrap_or_default(),
                is_rocket_delivery: record.is_rocket_delivery.unwrap_or(false),
                created_at: record.created_at.unwrap_or_else(Utc::now),
                updated_at: record.updated_at.unwrap_or_else(Utc::now),
            };
            product.calculate_discount_rate();
            products.push(product);
        }
        
        Ok(products)
    }
    
    /// 사이트별 상품 목록
    pub async fn list_by_site(&self, site: &str, limit: Option<i64>) -> Result<Vec<Product>> {
        let limit = limit.unwrap_or(50);
        
        let records = sqlx::query!(
            "SELECT * FROM products WHERE site = $1 ORDER BY updated_at DESC LIMIT $2",
            site,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::list_by_site", format!("Query failed: {}", e)))?;
        
        let mut products = Vec::new();
        for record in records {
            let mut product = Product {
                id: Some(record.id),
                product_name: record.product_name,
                current_price: record.current_price,
                original_price: record.original_price,
                discount_rate: None,
                site: record.site,
                category: record.category,
                url: record.url,
                image_url: record.image_url,
                coupon_code: record.coupon_code,
                valid_until: record.valid_until,
                additional_benefits: record.additional_benefits.unwrap_or_default(),
                is_rocket_delivery: record.is_rocket_delivery.unwrap_or(false),
                created_at: record.created_at.unwrap_or_else(Utc::now),
                updated_at: record.updated_at.unwrap_or_else(Utc::now),
            };
            product.calculate_discount_rate();
            products.push(product);
        }
        
        Ok(products)
    }
    
    /// 상품 업데이트
    pub async fn update(&self, id: Uuid, update_data: UpdateProduct) -> Result<Product> {
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            UPDATE products SET
                product_name = COALESCE($1, product_name),
                current_price = COALESCE($2, current_price),
                original_price = COALESCE($3, original_price),
                site = COALESCE($4, site),
                category = COALESCE($5, category),
                url = COALESCE($6, url),
                image_url = COALESCE($7, image_url),
                coupon_code = COALESCE($8, coupon_code),
                valid_until = COALESCE($9, valid_until),
                additional_benefits = COALESCE($10, additional_benefits),
                is_rocket_delivery = COALESCE($11, is_rocket_delivery),
                updated_at = $12
            WHERE id = $13
            "#,
            update_data.product_name,
            update_data.current_price,
            update_data.original_price,
            update_data.site,
            update_data.category,
            update_data.url,
            update_data.image_url,
            update_data.coupon_code,
            update_data.valid_until,
            update_data.additional_benefits.as_deref(),
            update_data.is_rocket_delivery,
            now,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::update", format!("Update failed: {}", e)))?;
        
        self.find_by_id(id)
            .await?
            .ok_or_else(|| ErrorS::data("ProductRepository::update", "Product not found after update"))
    }
    
    /// 상품 삭제
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM products WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| ErrorS::data("ProductRepository::delete", format!("Delete failed: {}", e)))?;
        
        Ok(())
    }
    
    /// 가격 히스토리 추가
    pub async fn add_price_history(&self, history: CreatePriceHistory) -> Result<PriceHistory> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let record = sqlx::query!(
            r#"
            INSERT INTO price_history (id, product_id, price, original_price, recorded_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            id,
            history.product_id,
            history.price,
            history.original_price,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::add_price_history", format!("Failed to add price history: {}", e)))?;
        
        Ok(PriceHistory {
            id: record.id,
            product_id: record.product_id,
            price: record.price,
            original_price: record.original_price,
            discount_rate: record.discount_rate,
            is_lowest: record.is_lowest,
            recorded_at: record.recorded_at.unwrap_or(now),
        })
    }
    
    /// 가격 히스토리 조회
    pub async fn get_price_history(&self, product_id: Uuid, limit: Option<i64>) -> Result<Vec<PriceHistory>> {
        let limit = limit.unwrap_or(30);
        
        let records = sqlx::query!(
            "SELECT * FROM price_history WHERE product_id = $1 ORDER BY recorded_at DESC LIMIT $2",
            product_id,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ErrorS::data("ProductRepository::get_price_history", format!("Query failed: {}", e)))?;
        
        let mut histories = Vec::new();
        for record in records {
            histories.push(PriceHistory {
                id: record.id,
                product_id: record.product_id,
                price: record.price,
                original_price: record.original_price,
                discount_rate: record.discount_rate,
                is_lowest: record.is_lowest,
                recorded_at: record.recorded_at.unwrap_or_else(Utc::now),
            });
        }
        
        Ok(histories)
    }
}