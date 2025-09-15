use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub product_name: String,
    pub current_price: Option<f64>,
    pub original_price: Option<f64>,
    pub site: String,
    pub category: String,
    pub url: Option<String>,
    pub image: String,
    pub coupon_code: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub additional_benefits: Option<JsonValue>,
    pub timestamp: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for Product {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            product_name: "Unknown Product".to_string(),
            current_price: None,
            original_price: None,
            site: "unknown".to_string(),
            category: "미분류".to_string(),
            url: None,
            image: String::new(),
            coupon_code: None,
            valid_until: None,
            additional_benefits: None,
            timestamp: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProduct {
    pub product_name: String,
    pub current_price: Option<f64>,
    pub original_price: Option<f64>,
    pub site: String,
    pub category: String,
    pub url: Option<String>,
    pub image: String,
    pub coupon_code: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub additional_benefits: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProduct {
    pub product_name: Option<String>,
    pub current_price: Option<f64>,
    pub original_price: Option<f64>,
    pub site: Option<String>,
    pub category: Option<String>,
    pub url: Option<String>,
    pub image: Option<String>,
    pub coupon_code: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub additional_benefits: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PriceHistory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub price: f64,
    pub original_price: Option<f64>,
    pub discount_rate: Option<f64>,
    pub is_lowest: Option<bool>,
    pub recorded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePriceHistory {
    pub product_id: Uuid,
    pub price: f64,
    pub original_price: Option<f64>,
}