use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use crate::infrastruction::html::basic::HtmlParser;
use crate::infrastruction::database::models::Product;
use anyhow::Result;
use scraper::{Html, Selector};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoupangPageType {
    MainPage,
    ProductDetail,
    ProductOptions,
    SearchResults,
    Category,
    Brand,
    Campaign,
    GoldBox,
    EventPages,
    CouponCenter,
    RocketFresh,
    RocketGlobal,
    RocketLuxury,
    CoupangBiz,
    MyCoupang,
    Login,
    MobileMain,
    MobileSearch,
    ProductApi,
    SearchSuggestApi,
    Unknown,
}

pub struct CoupangParse;

impl CoupangParse {
    pub fn new() -> Self {
        CoupangParse
    }
}

#[async_trait]
impl HtmlParser for CoupangParse {
    async fn parse_product(&self, html: &str, url: &str) -> Result<Product> {
        println!("🔍 쿠팡 상품 파싱 시작: {}", url);
        
        let document = Html::parse_document(html);
        let mut product = Product::default();
        product.site = "coupang".to_string();
        product.url = Some(url.to_string());
        
        // 상품명 추출
        let name_selectors = vec![
            ".prod-buy-header__title",
            ".name",
            ".search-product-title",
            ".baby-product-title",
        ];
        
        for selector_str in name_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    product.product_name = element.text().collect::<String>().trim().to_owned();
                    break;
                }
            }
        }
        
        // 가격 추출
        let price_selectors = vec![
            ".total-price strong",
            ".price-value",
            ".sale-price",
            ".price .total",
        ];
        
        for selector_str in price_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let price_text = element.text().collect::<String>();
                    let price = price_text.chars()
                        .filter(|c| c.is_numeric())
                        .collect::<String>();
                    if !price.is_empty() {
                        product.current_price = Some(price.parse::<f64>().unwrap_or(0.0));
                        break;
                    }
                }
            }
        }
        
        // 이미지 URL 추출
        let img_selectors = vec![
            ".prod-image__item img",
            ".baby-product-image img",
            ".search-product-image img",
        ];
        
        for selector_str in img_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(img) = document.select(&selector).next() {
                    if let Some(img_url) = img.value().attr("data-img-src")
                        .or(img.value().attr("data-original-src"))
                        .or(img.value().attr("data-src"))
                        .or(img.value().attr("src")) {
                        product.image = img_url.to_owned();
                        break;
                    }
                }
            }
        }
        
        println!("🎯 파싱 완료 - 상품명: {}, 가격: {:?}, 이미지: {}", 
                 product.product_name, product.current_price, product.image);
        
        Ok(product)
    }

    async fn parse_product_list(&self, html: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let mut urls = Vec::new();
        
        // 다양한 상품 리스트 셀렉터
        let selectors = vec![
            ".search-product",
            ".baby-product",
            ".search-product-item",
            "ul.search-product-list li",
        ];
        
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    // 링크 추출
                    if let Ok(link_selector) = Selector::parse("a") {
                        if let Some(link) = element.select(&link_selector).next() {
                            if let Some(href) = link.value().attr("href") {
                                let full_url = if href.starts_with("http") {
                                    href.to_string()
                                } else {
                                    format!("https://www.coupang.com{}", href)
                                };
                                urls.push(full_url);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(urls)
    }

    async fn interactive_elements(&self, html: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let mut elements = Vec::new();
        
        // React 컴포넌트 찾기
        let react_selector = Selector::parse("[data-react-class], [data-component-type]").unwrap();
        
        for element in document.select(&react_selector) {
            if let Some(component_type) = element.value().attr("data-component-type") {
                elements.push(format!("Component: {}", component_type));
            }
            if let Some(react_class) = element.value().attr("data-react-class") {
                elements.push(format!("React: {}", react_class));
            }
        }
        
        // 페이지네이션 요소
        let pagination_selector = Selector::parse(".search-pagination, .pagination").unwrap();
        for _element in document.select(&pagination_selector) {
            elements.push("Pagination found".to_string());
        }
        
        Ok(elements)
    }
}

// CoupangParse 내부 헬퍼 메서드들
impl CoupangParse {
    #[allow(dead_code)]
    async fn decide_url_type(&self, url: &str) -> Result<CoupangPageType> {
        // 모바일 도메인 체크
        if url.contains("m.coupang.com") || url.contains("mc.coupang.com") {
            if url.contains("/search") {
                return Ok(CoupangPageType::MobileSearch);
            }
            return Ok(CoupangPageType::MobileMain);
        }
        
        // 이벤트 페이지 서브도메인 체크
        if url.contains("pages.coupang.com") {
            return Ok(CoupangPageType::EventPages);
        }
        
        // API 엔드포인트 체크
        if url.contains("/api/") {
            if url.contains("/products") {
                return Ok(CoupangPageType::ProductApi);
            }
            if url.contains("/suggest") || url.contains("/search-suggest") {
                return Ok(CoupangPageType::SearchSuggestApi);
            }
        }
        
        // 메인 도메인 URL 파싱
        let clean_url = url.replace("https://www.coupang.com", "")
            .replace("https://coupang.com", "")
            .replace("http://www.coupang.com", "")
            .replace("http://coupang.com", "");
        
        // 루트 페이지
        if clean_url.is_empty() || clean_url == "/" {
            return Ok(CoupangPageType::MainPage);
        }
        
        // URL 경로를 세그먼트로 분리
        let parts: Vec<&str> = clean_url.trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.is_empty() {
            return Ok(CoupangPageType::MainPage);
        }
        
        // 첫 번째 경로 세그먼트로 페이지 타입 결정
        match parts[0] {
            // 상품 관련
            "vp" => {
                if parts.len() > 1 && parts[1] == "products" {
                    Ok(CoupangPageType::ProductDetail)
                } else {
                    Ok(CoupangPageType::ProductOptions)
                }
            },
            "products" => Ok(CoupangPageType::ProductDetail),
            
            // 검색 및 카테고리
            "np" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "search" => Ok(CoupangPageType::SearchResults),
                        "categories" => Ok(CoupangPageType::Category),
                        "brands" => Ok(CoupangPageType::Brand),
                        "goldbox" => Ok(CoupangPageType::GoldBox),
                        "campaigns" => Ok(CoupangPageType::Campaign),
                        _ => Ok(CoupangPageType::Category),
                    }
                } else if clean_url.contains("search") {
                    Ok(CoupangPageType::SearchResults)
                } else {
                    Ok(CoupangPageType::Category)
                }
            },
            
            // 특별 섹션
            "goldbox" => Ok(CoupangPageType::GoldBox),
            "campaign" | "campaigns" => Ok(CoupangPageType::Campaign),
            "event" | "events" => Ok(CoupangPageType::EventPages),
            "coupon" | "coupons" => Ok(CoupangPageType::CouponCenter),
            
            // 로켓 서비스
            "fresh" | "rocket-fresh" => Ok(CoupangPageType::RocketFresh),
            "global" | "rocket-global" => Ok(CoupangPageType::RocketGlobal),
            "luxury" | "rocket-luxury" => Ok(CoupangPageType::RocketLuxury),
            
            // 비즈니스
            "biz" | "business" => Ok(CoupangPageType::CoupangBiz),
            
            // 사용자 계정
            "my" | "mycoupang" | "my-coupang" => Ok(CoupangPageType::MyCoupang),
            "login" | "member" | "signin" => Ok(CoupangPageType::Login),
            
            // 검색
            "search" => Ok(CoupangPageType::SearchResults),
            
            // 카테고리명이 직접 올 수도 있음
            "category" => Ok(CoupangPageType::Category),
            "brand" => Ok(CoupangPageType::Brand),
            
            // 알 수 없는 경우
            _ => {
                // URL에 특정 키워드가 포함되어 있는지 추가 체크
                if clean_url.contains("product") {
                    Ok(CoupangPageType::ProductDetail)
                } else if clean_url.contains("search") {
                    Ok(CoupangPageType::SearchResults)
                } else if clean_url.contains("category") {
                    Ok(CoupangPageType::Category)
                } else {
                    Ok(CoupangPageType::Unknown)
                }
            }
        }
    }
}