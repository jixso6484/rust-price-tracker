use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 메인 구조체 추가
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceStructure {
    pub platform: PlatformInfo,
    pub page_signatures: PageSignatures,
    pub data_extractors: DataExtractors,
    pub navigation_patterns: NavigationPatterns,
    pub anti_bot_indicators: AntiBotIndicators,
    
    // 2025년 추가 필드
    #[serde(skip_serializing_if = "Option::is_none")]
    pub javascript_config: Option<JavaScriptConfig>,  // AliExpress용
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_notes: Option<TechnicalNotes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub name: String,
    pub domain: String,
    pub country: String,
    pub detected_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageSignatures {
    pub search_results: PageSignature,
    pub product_detail: PageSignature,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_list: Option<PageSignature>,  // Optional로 변경
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageSignature {
    pub url_patterns: Vec<String>,
    pub required_elements: Vec<String>,
    
    #[serde(default)]
    pub confidence_markers: HashMap<String, f32>,
}

// JavaScript 데이터 처리 (AliExpress 필수)
#[derive(Debug, Serialize, Deserialize)]
pub struct JavaScriptConfig {
    pub requires_javascript: bool,
    pub main_data_variable: String,  // "window.runParams"
    pub data_location: String,       // "script[type='text/javascript']"
    pub parse_method: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub react_based: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataExtractors {
    pub product: ProductExtractor,
    pub list_item: ListItemExtractor,
    
    // 플랫폼별 추가 필드
    #[serde(skip_serializing_if = "Option::is_none")]
    pub javascript_extractors: Option<JavaScriptExtractors>,
}

// JavaScript 기반 추출 (AliExpress)
#[derive(Debug, Serialize, Deserialize)]
pub struct JavaScriptExtractors {
    pub product_data_path: String,    // "window.runParams.data"
    pub price_module: String,         // "priceModule"
    pub shipping_module: String,      // "shippingModule"
    pub seller_module: String,        // "sellerModule"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductExtractor {
    pub price_selectors: PriceSelectors,
    pub shipping_selectors: ShippingSelectors,
    pub seller_selectors: SellerSelectors,
    
    // 플랫폼별 추가 셀렉터
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_selectors: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceSelectors {
    pub current_price: Vec<String>,
    pub original_price: Vec<String>,
    pub discount_rate: Vec<String>,
    pub currency_pattern: String,
    
    // 가격 추출 우선순위
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_order: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShippingSelectors {
    pub shipping_fee: Vec<String>,
    pub delivery_time: Vec<String>,
    pub shipping_method: Vec<String>,
    pub free_shipping_indicator: Vec<String>,
    
    // Coupang 로켓배송 등 특수 배송
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_delivery: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SellerSelectors {
    pub seller_name: Vec<String>,
    pub seller_rating: Vec<String>,
    pub seller_location: Vec<String>,
    pub fulfilled_by: Vec<String>,
    
    // 판매자 ID 추출
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItemExtractor {
    pub item_container: Vec<String>,
    pub item_link: Vec<String>,
    pub item_price: Vec<String>,
    pub item_title: Vec<String>,
    
    // 광고 필터링
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_badge_selector: Option<Vec<String>>,
    
    // JavaScript 리스트 데이터
    #[serde(skip_serializing_if = "Option::is_none")]
    pub javascript_list: Option<JavaScriptListConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaScriptListConfig {
    pub data_location: String,
    pub items_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NavigationPatterns {
    pub pagination: PaginationPattern,
    pub infinite_scroll: bool,
    pub load_more_button: Option<String>,
    
    #[serde(default)]
    pub ajax_endpoints: Vec<String>,
    
    // 스크롤 설정
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_config: Option<ScrollConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrollConfig {
    pub trigger_percentage: String,  // "80%"
    pub lazy_loading: bool,
    pub max_scroll_attempts: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationPattern {
    pub next_button: Vec<String>,
    pub page_param: String,
    pub items_per_page: Option<usize>,
    
    // 추가 필드
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_button: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_numbers: Option<Vec<String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<u32>,  // AliExpress는 60페이지 제한
}

// 확장된 Anti-bot 지표
#[derive(Debug, Serialize, Deserialize)]
pub struct AntiBotIndicators {
    pub cloudflare_detected: bool,
    pub captcha_present: bool,
    pub requires_login: bool,
    pub rate_limit_indicators: Vec<String>,
    
    // 2025년 추가 필드
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha_types: Option<Vec<String>>,  // ["FunCaptcha", "reCAPTCHA v3"]
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<String>,  // "100/min per IP"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_javascript: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprinting: Option<String>,  // "advanced", "basic"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_waf: Option<bool>,  // Amazon 전용
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geographic_restriction: Option<String>,  // "KR focused"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent_check: Option<bool>,
}

// 기술 노트 (디버깅/참고용)
#[derive(Debug, Serialize, Deserialize)]
pub struct TechnicalNotes {
    pub rendering: String,  // "SSR", "CSR", "React.js with SSR + CSR hybrid"
    pub data_loading: String,
    pub primary_data_source: String,
    pub selector_stability: String,  // "stable", "moderate", "unstable"
}

// 헬퍼 구현
impl MarketplaceStructure {
    // JavaScript 실행이 필요한지 확인
    pub fn requires_javascript(&self) -> bool {
        self.javascript_config
            .as_ref()
            .map(|c| c.requires_javascript)
            .unwrap_or(false) ||
        self.anti_bot_indicators.requires_javascript.unwrap_or(false)
    }
    
    // 플랫폼별 특수 처리 필요 여부
    pub fn needs_special_handling(&self) -> bool {
        match self.platform.name.as_str() {
            "aliexpress" => true,  // JavaScript 필수
            "coupang" => self.anti_bot_indicators.cloudflare_detected,
            "amazon" => self.anti_bot_indicators.aws_waf.unwrap_or(false),
            _ => false,
        }
    }
    
    // 최적 셀렉터 가져오기 (우선순위)
    pub fn get_price_selector(&self) -> Option<&String> {
        if let Some(priority) = &self.data_extractors.product.price_selectors.priority_order {
            priority.first()
        } else {
            self.data_extractors.product.price_selectors.current_price.first()
        }
    }
}

// 플랫폼 감지
impl PlatformInfo {
    pub fn detect_from_url(url: &str) -> Option<String> {
        if url.contains("amazon.") {
            Some("amazon".to_string())
        } else if url.contains("coupang.com") {
            Some("coupang".to_string())
        } else if url.contains("aliexpress.") {
            Some("aliexpress".to_string())
        } else {
            None
        }
    }
}