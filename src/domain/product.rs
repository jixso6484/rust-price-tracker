use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Option<Uuid>,
    pub product_name: String,
    pub current_price: Option<f64>,
    pub original_price: Option<f64>,
    pub discount_rate: Option<f64>,  // 계산된 할인율
    pub site: String,
    pub category: String,
    pub url: String,                 // 필수 필드
    pub image_url: String,           // image → image_url
    pub coupon_code: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub additional_benefits: Vec<String>,  // JSON 대신 Vec<String>
    pub is_rocket_delivery: bool,    // 쿠팡 특화 필드
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Product {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: None,
            product_name: "Unknown Product".to_string(),
            current_price: None,
            original_price: None,
            discount_rate: None,
            site: "unknown".to_string(),
            category: "미분류".to_string(),
            url: String::new(),
            image_url: String::new(),
            coupon_code: None,
            valid_until: None,
            additional_benefits: Vec::new(),
            is_rocket_delivery: false,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Product {
    pub fn new(name: String, url: String, site: String) -> Self {
        let now = Utc::now();
        Self {
            id: Some(Uuid::new_v4()),
            product_name: name,
            url,
            site,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }
    
    /// 할인율 계산
    pub fn calculate_discount_rate(&mut self) {
        if let (Some(current), Some(original)) = (self.current_price, self.original_price) {
            if original > 0.0 {
                let discount = ((original - current) / original) * 100.0;
                self.discount_rate = Some(discount.round());
            }
        }
    }
    
    /// 가격 업데이트
    pub fn update_price(&mut self, new_price: f64, original_price: Option<f64>) {
        self.current_price = Some(new_price);
        if let Some(original) = original_price {
            self.original_price = Some(original);
        }
        self.calculate_discount_rate();
        self.updated_at = Utc::now();
    }
    
    /// 쿠팡 로켓배송 여부 설정
    pub fn set_rocket_delivery(&mut self, is_rocket: bool) {
        self.is_rocket_delivery = is_rocket;
    }
    
    /// 혜택 추가
    pub fn add_benefit(&mut self, benefit: String) {
        if !self.additional_benefits.contains(&benefit) {
            self.additional_benefits.push(benefit);
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
    pub url: String,
    pub image_url: String,
    pub coupon_code: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub additional_benefits: Vec<String>,
    pub is_rocket_delivery: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProduct {
    pub product_name: Option<String>,
    pub current_price: Option<f64>,
    pub original_price: Option<f64>,
    pub site: Option<String>,
    pub category: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub coupon_code: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub additional_benefits: Option<Vec<String>>,
    pub is_rocket_delivery: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PriceHistory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub price: f64,
    pub original_price: Option<f64>,
    pub discount_rate: Option<f64>,
    pub is_lowest: Option<bool>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePriceHistory {
    pub product_id: Uuid,
    pub price: f64,
    pub original_price: Option<f64>,
}