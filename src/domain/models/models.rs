use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Option<i32>,
    pub product_name: String,
    pub current_price: Option<f64>,
    pub original_price: Option<f64>,
    pub site: String,
    pub category: String,
    pub url: Option<String>,
    pub image: String,
    pub coupon_code: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub additional_benefits: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for Product {
    fn default() -> Self {
        Self {
            id: None,
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
            created_at: None,
            updated_at: None,
        }
    }
}
