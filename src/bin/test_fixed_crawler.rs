mod components;
mod infrastructure;
mod application;
mod domain;

use application::services::crawling_fixed::FixedCrawlingOrchestrator;
use infrastructure::llm::models::{MarketplaceStructure, PlatformInfo, PageSignatures, PageSignature, DataExtractors, 
    ProductExtractor, PriceSelectors, ShippingSelectors, SellerSelectors, ListItemExtractor, 
    NavigationPatterns, PaginationPattern, AntiBotIndicators};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 고정 크롤링 테스트 시작");
    println!("==========================");
    
    dotenv::dotenv().ok();
    
    let sites = vec![
        MarketplaceStructure {
            platform: PlatformInfo {
                name: "Coupang".to_string(),
                domain: "https://www.coupang.com".to_string(),
                country: "KR".to_string(),
                detected_by: vec!["coupang.com".to_string()],
            },
            page_signatures: PageSignatures {
                search_results: PageSignature {
                    url_patterns: vec!["np/search".to_string()],
                    required_elements: vec![".search-product".to_string()],
                    confidence_markers: HashMap::new(),
                },
                product_detail: PageSignature {
                    url_patterns: vec!["vp/products".to_string()],
                    required_elements: vec![".prod-buy".to_string()],
                    confidence_markers: HashMap::new(),
                },
                category_list: Some(PageSignature {
                    url_patterns: vec!["np/categories".to_string()],
                    required_elements: vec![".search-product-list".to_string()],
                    confidence_markers: HashMap::new(),
                }),
            },
            data_extractors: DataExtractors {
                product: ProductExtractor {
                    price_selectors: PriceSelectors {
                        current_price: vec![".total-price strong".to_string(), ".price-value".to_string()],
                        original_price: vec![".base-price".to_string(), ".origin-price".to_string()],
                        discount_rate: vec![".discount-percentage".to_string()],
                        currency_pattern: "원".to_string(),
                        priority_order: None,
                    },
                    shipping_selectors: ShippingSelectors {
                        shipping_fee: vec![".shipping-fee".to_string()],
                        delivery_time: vec![".delivery-arrival-info".to_string()],
                        shipping_method: vec![".shipping-method".to_string()],
                        free_shipping_indicator: vec![".free-shipping".to_string()],
                        special_delivery: Some(vec![".rocket".to_string(), ".rocket-fresh".to_string()]),
                    },
                    seller_selectors: SellerSelectors {
                        seller_name: vec![".prod-sale-vendor-name".to_string()],
                        seller_rating: vec![".seller-rating".to_string()],
                        seller_location: vec![".seller-location".to_string()],
                        fulfilled_by: vec![".fulfilled-by".to_string()],
                        seller_id: None,
                    },
                    additional_selectors: None,
                },
                list_item: ListItemExtractor {
                    item_container: vec![".search-product".to_string()],
                    item_link: vec!["a.search-product-link".to_string()],
                    item_price: vec![".price-value".to_string()],
                    item_title: vec![".name".to_string()],
                    ad_badge_selector: Some(vec![".ad-badge".to_string()]),
                    javascript_list: None,
                },
                javascript_extractors: None,
            },
            navigation_patterns: NavigationPatterns {
                pagination: PaginationPattern {
                    next_button: vec![".page-next".to_string()],
                    page_param: "page".to_string(),
                    items_per_page: Some(72),
                    prev_button: None,
                    page_numbers: None,
                    max_pages: None,
                },
                infinite_scroll: false,
                load_more_button: None,
                ajax_endpoints: vec![],
                scroll_config: None,
            },
            anti_bot_indicators: AntiBotIndicators {
                cloudflare_detected: false,
                captcha_present: false,
                requires_login: false,
                rate_limit_indicators: vec![],
                captcha_types: None,
                rate_limit: None,
                requires_javascript: Some(true),
                fingerprinting: None,
                aws_waf: None,
                geographic_restriction: Some("KR focused".to_string()),
                user_agent_check: Some(true),
            },
            javascript_config: None,
            technical_notes: None,
        }
    ];
    
    println!("🚀 고정 크롤링 오케스트레이터 생성 중...");
    let mut orchestrator = match FixedCrawlingOrchestrator::new(sites).await {
        Ok(o) => {
            println!("✅ 고정 오케스트레이터 생성 성공!");
            o
        },
        Err(e) => {
            println!("❌ 고정 오케스트레이터 생성 실패: {}", e);
            return Err(e.into());
        }
    };
    
    println!("🌐 고정 크롤링 시작...");
    let products = orchestrator.crawl_continuously().await?;
    
    println!("\n📊 수집 결과:");
    println!("총 {}개 상품 발견", products.len());
    
    for (idx, product) in products.iter().enumerate() {
        println!("{}. {} - {:?}원", 
                 idx + 1, 
                 product.product_name, 
                 product.current_price);
    }
    
    println!("🏁 테스트 완료");
    Ok(())
}