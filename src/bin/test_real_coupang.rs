use sexy_crawling::infrastruction::browser::chromiumAdapter::ChromiumAdapter;
use sexy_crawling::infrastruction::browser::models::BrowserAction;
use sexy_crawling::infrastruction::html::coupang::CoupangParser;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 실제 쿠팡 사이트 테스트 시작...");
    
    // 브라우저 초기화
    let browser = ChromiumAdapter::new().await?;
    let parser = CoupangParser::new();
    
    println!("🌐 쿠팡 메인 페이지 접속 중...");
    let tab = browser.new_page("https://www.coupang.com").await?;
    
    // 페이지 로드 대기
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // 페이지 상태 가져오기
    let state = browser.execute_action(&tab, BrowserAction::GetPageState).await?;
    let html = state.html.unwrap_or_default();
    
    println!("📄 HTML 길이: {} 문자", html.len());
    
    if html.is_empty() {
        println!("❌ HTML이 비어있습니다!");
        return Ok(());
    }
    
    // HTML에서 실제 요소들 찾아보기
    println!("🔍 페이지 요소 분석 중...");
    let interactive_elements = parser.interactive_elements(&html).await?;
    
    println!("📋 발견된 상호작용 요소: {}개", interactive_elements.len());
    for (i, element) in interactive_elements.iter().take(10).enumerate() {
        println!("  {}: {}", i + 1, element);
    }
    
    // 할인 관련 링크 찾기
    println!("\n🎯 할인 관련 요소 찾기...");
    let discount_links: Vec<&String> = interactive_elements
        .iter()
        .filter(|e| e.to_lowercase().contains("할인") || e.to_lowercase().contains("deal") || e.to_lowercase().contains("특가"))
        .collect();
        
    if discount_links.is_empty() {
        println!("❌ 할인 관련 요소를 찾을 수 없습니다.");
    } else {
        println!("✅ 할인 관련 요소 {}개 발견:", discount_links.len());
        for link in discount_links {
            println!("  - {}", link);
        }
    }
    
    // 실제 URL에서 할인 페이지로 직접 이동 테스트
    println!("\n🚀 할인 검색 페이지로 직접 이동 테스트...");
    let search_url = "https://www.coupang.com/np/search?q=할인";
    println!("📍 이동할 URL: {}", search_url);
    
    let search_tab = browser.new_page(search_url).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    let search_state = browser.execute_action(&search_tab, BrowserAction::GetPageState).await?;
    let search_html = search_state.html.unwrap_or_default();
    
    if search_html.is_empty() {
        println!("❌ 검색 페이지 HTML을 가져올 수 없습니다!");
    } else {
        println!("✅ 검색 페이지 로드 성공! HTML 길이: {} 문자", search_html.len());
        
        // 상품 링크 찾기
        let product_urls = parser.parse_productUrl_list(&search_html).await?;
        println!("🛒 발견된 상품 링크: {}개", product_urls.len());
        
        for (i, url) in product_urls.iter().take(5).enumerate() {
            println!("  {}: {}", i + 1, url);
        }
    }
    
    println!("\n✅ 테스트 완료!");
    Ok(())
}