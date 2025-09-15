use anyhow::Result;
use sexy_crawling::infrastructure::browser::chromiumAdapter::ChromiumAdapter;
use sexy_crawling::infrastructure::browser::models::BrowserAction;
use sexy_crawling::infrastructure::html::coupang::CoupangParser;
use sexy_crawling::infrastructure::html::basic::html_parese;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 브라우저 초기화
    let browser = ChromiumAdapter::new().await?;
    
    // 2. 쿠팡 카테고리 페이지 열기
    let url = "https://www.coupang.com/np/categories/194282";
    let tab = browser.new_page(url).await?;
    
    // 3. 페이지 HTML 가져오기
    let state = browser.execute_action(&tab, BrowserAction::GetPageState).await?;
    let html = state.html.unwrap_or_default();
    
    // 4. Parser로 상호작용 가능한 요소들 추출
    let parser = CoupangParser;
    let interactive_elements = parser.interactive_elements(&html).await?;
    
    println!("발견된 상호작용 요소 개수: {}", interactive_elements.len());
    
    // 5. 상품 URL 목록 추출
    let product_urls = parser.parse_productUrl_list(&html).await?;
    println!("발견된 상품 URL 개수: {}", product_urls.len());
    
    // 6. 첫 번째 상품 클릭 (상품 상세 페이지로 이동)
    if !product_urls.is_empty() {
        // URL로 직접 이동하는 방법
        let first_product_url = &product_urls[0];
        let product_tab = browser.new_page(first_product_url).await?;
        
        // 또는 selector로 클릭하는 방법
        // let action = BrowserAction::Click { 
        //     selector: "a.baby-product-link:first-child".to_string() 
        // };
        // let new_state = browser.execute_action(&tab, action).await?;
        
        // 7. 상품 정보 파싱
        let product_state = browser.execute_action(&product_tab, BrowserAction::GetPageState).await?;
        let product_html = product_state.html.unwrap_or_default();
        let product_info = parser.productinfo_parse(&product_html, first_product_url).await?;
        
        println!("상품명: {}", product_info.product_name);
        println!("가격: {:?}", product_info.current_price);
    }
    
    // 8. 페이지네이션 처리 예시
    // 다음 페이지 버튼 찾기
    let next_page_selector = ".pagination a.next-page";
    let action = BrowserAction::WaitForElement {
        selector: next_page_selector.to_string(),
        timeout: 5000,
    };
    
    // 요소가 있으면 클릭
    if browser.execute_action(&tab, action).await.is_ok() {
        let click_action = BrowserAction::Click {
            selector: next_page_selector.to_string(),
        };
        let new_state = browser.execute_action(&tab, click_action).await?;
        println!("다음 페이지로 이동: {}", new_state.url);
        
        // 새 페이지에서 다시 상품 목록 추출
        let new_html = new_state.html.unwrap_or_default();
        let new_products = parser.parse_productUrl_list(&new_html).await?;
        println!("새 페이지 상품 개수: {}", new_products.len());
    }
    
    // 9. 무한 스크롤 처리 예시
    for _ in 0..3 {
        // 페이지 아래로 스크롤
        let scroll_action = BrowserAction::Scroll {
            direction: sexy_crawling::infrastructure::browser::models::ScrollDirection::Down,
            amount: 1000.0,
        };
        browser.execute_action(&tab, scroll_action).await?;
        
        // 새로운 콘텐츠 로드 대기
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 업데이트된 HTML 가져오기
        let updated_state = browser.execute_action(&tab, BrowserAction::GetPageState).await?;
        let updated_html = updated_state.html.unwrap_or_default();
        let all_products = parser.parse_productUrl_list(&updated_html).await?;
        println!("스크롤 후 총 상품 개수: {}", all_products.len());
    }
    
    Ok(())
}