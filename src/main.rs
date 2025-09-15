mod components;
mod infrastructure;
mod application;
mod domain;
mod config;

use infrastructure::database::repository::ProductRepository;
use infrastructure::browser::service::BrowserService;
use infrastructure::llm::service::LLMService;
use domain::product::{CreateProduct, Product};
use config::Config;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛒 할인 상품 크롤링 시스템 v2.0");
    println!("================================");
    
    // 환경 변수 로드
    dotenv::dotenv().ok();
    
    // 설정 로드
    println!("📋 설정 로드 중...");
    let config = match Config::load_from_file("config.toml") {
        Ok(c) => c,
        Err(_) => {
            println!("⚠️ config.toml을 찾을 수 없어서 기본 설정을 사용합니다.");
            Config::default()
        }
    };
    
    // 데이터베이스 연결
    println!("🔌 데이터베이스 연결 중...");
    let product_repo = ProductRepository::new(&config.database.url).await?;
    println!("✅ 데이터베이스 연결 성공!");
    
    // 브라우저 서비스 초기화
    println!("🌐 브라우저 서비스 초기화 중...");
    let browser_service = BrowserService::new(&config.browser).await?;
    println!("✅ 브라우저 서비스 초기화 성공!");
    
    // LLM 서비스 초기화
    println!("🧠 LLM 서비스 초기화 중...");
    let llm_service = LLMService::new(&config.llm).await?;
    println!("✅ LLM 서비스 초기화 성공!");
    
    // 간단한 테스트 크롤링 실행
    println!("🚀 테스트 크롤링 시작...");
    let test_url = "https://www.coupang.com/np/search?q=노트북";
    
    match browser_service.navigate(test_url).await {
        Ok(tab_id) => {
            println!("✅ 페이지 로드 성공: {}", test_url);
            
            // 페이지 분석
            let page_content = browser_service.execute(&tab_id, "document.body.innerHTML").await?;
            let analysis = llm_service.analyze_page(&page_content).await?;
            
            println!("🔍 페이지 분석 결과:");
            println!("  - 페이지 타입: {:?}", analysis.page_type);
            println!("  - 제품 수: {}", analysis.product_count);
            println!("  - 추천 액션: {:?}", analysis.recommended_action);
            
            // 간단한 제품 정보 추출 (테스트용)
            if analysis.product_count > 0 {
                let products = llm_service.extract_product_info(&page_content).await?;
                println!("📦 추출된 제품 수: {}", products.len());
                
                for (i, product) in products.iter().take(3).enumerate() {
                    println!("  {}. {}", i + 1, product.name);
                    if let Some(price) = product.current_price {
                        println!("     가격: {}원", price);
                    }
                }
            }
            
            browser_service.close_tab(&tab_id).await?;
        },
        Err(e) => {
            println!("❌ 페이지 로드 실패: {}", e);
        }
    }
    
    println!("🧹 브라우저 정리 중...");
    browser_service.close_all().await?;
    
    println!("🏁 크롤링 시스템 종료");
    Ok(())
}

