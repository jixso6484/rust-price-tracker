mod components;
mod infrastruction;
mod application;
mod domain;

use application::services::crawling::CrawlingOrchestrator;
use infrastruction::llm::models::{MarketplaceStructure, PlatformInfo, PageSignatures, PageSignature, DataExtractors, 
    ProductExtractor, PriceSelectors, ShippingSelectors, SellerSelectors, ListItemExtractor, 
    NavigationPatterns, PaginationPattern, AntiBotIndicators};
use infrastruction::database::connection::ProductRepository;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛒 할인 상품 크롤링 시스템 v1.0");
    println!("================================");
    
    // 환경 변수 로드
    dotenv::dotenv().ok();
    
    // 데이터베이스 연결
    println!("🔌 데이터베이스 연결 중...");
    let product_repo = ProductRepository::new().await?;
    println!("✅ 데이터베이스 연결 성공!");
    
    // 크롤링 대상 사이트 설정 (쿠팡)
    let sites = vec![
        MarketplaceStructure {
            platform: PlatformInfo {
                name: "Coupang".to_string(),
                domain: "https://www.coupang.com".to_string(), // 노트북/태블릿 카테고리
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
    
    // 크롤링 오케스트레이터 생성 및 실행
    println!("🚀 크롤링 오케스트레이터 생성 중...");
    let mut orchestrator = match CrawlingOrchestrator::new(sites).await {
        Ok(o) => {
            println!("✅ 오케스트레이터 생성 성공!");
            o
        },
        Err(e) => {
            println!("❌ 오케스트레이터 생성 실패: {}", e);
            return Err(e.into());
        }
    };
    
    println!("🌐 크롤링 시작 (stop 입력으로 중단 가능)...");
    let products = orchestrator.crawl_continuously().await?;
    
    // 수집된 상품 DB 저장
    if !products.is_empty() {
        println!("💾 {}개 상품을 데이터베이스에 저장 중...", products.len());
        let mut saved_count = 0;
        let mut skipped_count = 0;
        
        for product in &products {
            // 필수 필드 검증
            if product.product_name.is_empty() || product.product_name == "Unknown Product" {
                println!("⚠️  상품명이 없어서 건너뜀");
                skipped_count += 1;
                continue;
            }
            
            if product.current_price.is_none() || product.current_price == Some(0.0) {
                println!("⚠️  가격 정보가 없어서 건너뜀: {}", product.product_name);
                skipped_count += 1;
                continue;
            }
            
            if product.url.is_none() || !product.url.as_ref().unwrap_or(&String::new()).starts_with("http") {
                println!("⚠️  유효한 URL이 없어서 건너뜀: {}", product.product_name);
                skipped_count += 1;
                continue;
            }
            
            let create_product = infrastruction::database::models::CreateProduct {
                product_name: product.product_name.clone(),
                current_price: product.current_price,
                original_price: product.original_price,
                site: if product.site.is_empty() { 
                    "coupang".to_string() 
                } else { 
                    product.site.clone() 
                },
                category: if product.category.is_empty() { 
                    "미분류".to_string() 
                } else { 
                    product.category.clone() 
                },
                url: product.url.clone(),
                image: if product.image.is_empty() { 
                    "".to_string() 
                } else { 
                    product.image.clone() 
                },
                coupon_code: product.coupon_code.clone(),
                valid_until: product.valid_until,
                additional_benefits: product.additional_benefits.clone(),
            };
            match product_repo.create_product(create_product).await {
                Ok(saved_product) => {
                    println!("✅ 저장됨: {} (ID: {})", saved_product.product_name, saved_product.id);
                    saved_count += 1;
                    
                    // 가격 히스토리도 추가
                    if let Some(current_price) = saved_product.current_price {
                        product_repo.add_price_history(
                            saved_product.id,
                            current_price,
                            saved_product.original_price
                        ).await?;
                    }
                },
                Err(e) => println!("❌ 저장 실패: {} - {}", product.product_name, e),
            }
        }
        println!("📊 저장 결과: 성공 {}개, 건너뜀 {}개", saved_count, skipped_count);
    } else {
        println!("⚠️ 수집된 상품이 없습니다.");
    }
    
    println!("🏁 크롤링 시스템 종료");
    Ok(())
}

