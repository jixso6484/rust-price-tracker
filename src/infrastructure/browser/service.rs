use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use headless_chrome::{Browser, Tab};
use crate::infrastructure::browser::models::{BrowserAction, BrowserState};
use crate::components::error::error_cl::{Result, ErrorS};

pub struct BrowserService {
    browser: Browser,
    tabs: HashMap<String, Arc<Tab>>,  // URL별 탭 캐싱
    max_tabs: usize,
    default_timeout: Duration,
}

impl BrowserService {
    pub async fn new() -> Result<Self> {
        let browser = Browser::default()
            .map_err(|e| ErrorS::data("BrowserService::new", format!("Failed to create browser: {}", e)))?;
        
        println!("🌐 Browser service initialized");
        
        Ok(Self {
            browser,
            tabs: HashMap::new(),
            max_tabs: 10,
            default_timeout: Duration::from_secs(30),
        })
    }
    
    pub fn with_max_tabs(mut self, max_tabs: usize) -> Self {
        self.max_tabs = max_tabs;
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }
    
    /// URL로 이동하고 BrowserState 반환
    pub async fn navigate(&mut self, url: &str) -> Result<BrowserState> {
        let tab = self.get_or_create_tab(url).await?;
        
        // 페이지 로드
        tab.navigate_to(url)
            .map_err(|e| ErrorS::data("BrowserService::navigate", format!("Navigation failed: {}", e)))?;
        
        // 페이지 로드 완료 대기
        tab.wait_until_navigated()
            .map_err(|e| ErrorS::data("BrowserService::navigate", format!("Navigation timeout: {}", e)))?;
        
        // 추가 로딩 시간
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        self.get_page_state(&tab).await
    }
    
    /// 브라우저 액션 실행
    pub async fn execute(&self, url: &str, action: BrowserAction) -> Result<BrowserState> {
        let tab = self.tabs.get(url)
            .ok_or_else(|| ErrorS::data("BrowserService::execute", format!("No tab found for URL: {}", url)))?;
        
        match action {
            BrowserAction::Click { selector } => {
                self.click_element(tab, &selector).await?;
            },
            BrowserAction::Scroll { direction, pixels } => {
                self.scroll_page(tab, direction, pixels).await?;
            },
            BrowserAction::WaitForElement { selector, timeout } => {
                self.wait_for_element(tab, &selector, timeout).await?;
            },
            BrowserAction::ExtractText { selector } => {
                return self.extract_text(tab, &selector).await;
            },
            BrowserAction::GetPageState => {
                return self.get_page_state(tab).await;
            },
            BrowserAction::TakeScreenshot => {
                return self.take_screenshot(tab).await;
            },
        }
        
        // 액션 실행 후 페이지 상태 반환
        self.get_page_state(tab).await
    }
    
    /// 모든 탭 닫기
    pub async fn close_all(&mut self) -> Result<()> {
        for (url, tab) in self.tabs.drain() {
            if let Err(e) = tab.close(true) {
                println!("⚠️ Failed to close tab for {}: {}", url, e);
            }
        }
        println!("🗂️ All tabs closed");
        Ok(())
    }
    
    /// 특정 탭 닫기
    pub async fn close_tab(&mut self, url: &str) -> Result<()> {
        if let Some(tab) = self.tabs.remove(url) {
            tab.close(true)
                .map_err(|e| ErrorS::data("BrowserService::close_tab", format!("Failed to close tab: {}", e)))?;
            println!("🗂️ Tab closed: {}", url);
        }
        Ok(())
    }
    
    // Private helper methods
    
    async fn get_or_create_tab(&mut self, url: &str) -> Result<Arc<Tab>> {
        // 기존 탭이 있으면 재사용
        if let Some(tab) = self.tabs.get(url) {
            return Ok(tab.clone());
        }
        
        // 탭 수 제한 확인
        if self.tabs.len() >= self.max_tabs {
            // 가장 오래된 탭 제거 (간단한 LRU)
            if let Some((oldest_url, _)) = self.tabs.iter().next() {
                let oldest_url = oldest_url.clone();
                self.close_tab(&oldest_url).await?;
            }
        }
        
        // 새 탭 생성
        let tab = self.browser.new_tab()
            .map_err(|e| ErrorS::data("BrowserService::get_or_create_tab", format!("Failed to create tab: {}", e)))?;
        
        let tab_arc = Arc::new(tab);
        self.tabs.insert(url.to_string(), tab_arc.clone());
        
        println!("🆕 New tab created for: {}", url);
        Ok(tab_arc)
    }
    
    async fn get_page_state(&self, tab: &Tab) -> Result<BrowserState> {
        let html = tab.get_content()
            .map_err(|e| ErrorS::data("BrowserService::get_page_state", format!("Failed to get HTML: {}", e)))?;
        
        let url = tab.get_url()
            .unwrap_or_else(|| "unknown".to_string());
        
        let title = tab.get_title()
            .unwrap_or_else(|| "".to_string());
        
        Ok(BrowserState {
            url,
            title,
            html: Some(html),
            screenshot: None,
            extracted_text: None,
            status: "loaded".to_string(),
        })
    }
    
    async fn click_element(&self, tab: &Tab, selector: &str) -> Result<()> {
        tab.wait_for_element(selector)
            .map_err(|e| ErrorS::data("BrowserService::click_element", format!("Element not found: {}", e)))?
            .click()
            .map_err(|e| ErrorS::data("BrowserService::click_element", format!("Click failed: {}", e)))?;
        
        println!("🖱️ Clicked element: {}", selector);
        
        // 클릭 후 잠시 대기
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }
    
    async fn scroll_page(&self, tab: &Tab, direction: crate::infrastructure::browser::models::ScrollDirection, pixels: i32) -> Result<()> {
        let script = match direction {
            crate::infrastructure::browser::models::ScrollDirection::Down => {
                format!("window.scrollBy(0, {})", pixels)
            },
            crate::infrastructure::browser::models::ScrollDirection::Up => {
                format!("window.scrollBy(0, -{})", pixels)
            },
        };
        
        tab.evaluate(&script, false)
            .map_err(|e| ErrorS::data("BrowserService::scroll_page", format!("Scroll failed: {}", e)))?;
        
        println!("📜 Scrolled {} pixels {}", pixels, match direction {
            crate::infrastructure::browser::models::ScrollDirection::Down => "down",
            crate::infrastructure::browser::models::ScrollDirection::Up => "up",
        });
        
        // 스크롤 후 잠시 대기
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }
    
    async fn wait_for_element(&self, tab: &Tab, selector: &str, timeout: Duration) -> Result<()> {
        tab.wait_for_element_with_custom_timeout(selector, timeout)
            .map_err(|e| ErrorS::data("BrowserService::wait_for_element", format!("Element wait timeout: {}", e)))?;
        
        println!("⏳ Element appeared: {}", selector);
        Ok(())
    }
    
    async fn extract_text(&self, tab: &Tab, selector: &str) -> Result<BrowserState> {
        let element = tab.wait_for_element(selector)
            .map_err(|e| ErrorS::data("BrowserService::extract_text", format!("Element not found: {}", e)))?;
        
        let text = element.get_inner_text()
            .map_err(|e| ErrorS::data("BrowserService::extract_text", format!("Text extraction failed: {}", e)))?;
        
        println!("📝 Extracted text: {} characters", text.len());
        
        let mut state = self.get_page_state(tab).await?;
        state.extracted_text = Some(text);
        Ok(state)
    }
    
    async fn take_screenshot(&self, tab: &Tab) -> Result<BrowserState> {
        let screenshot = tab.capture_screenshot(headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|e| ErrorS::data("BrowserService::take_screenshot", format!("Screenshot failed: {}", e)))?;
        
        println!("📷 Screenshot captured: {} bytes", screenshot.len());
        
        let mut state = self.get_page_state(tab).await?;
        state.screenshot = Some(screenshot);
        Ok(state)
    }
}

impl Drop for BrowserService {
    fn drop(&mut self) {
        // 소멸자에서 모든 탭 정리
        for (url, tab) in self.tabs.drain() {
            if let Err(e) = tab.close(true) {
                println!("⚠️ Failed to close tab during cleanup for {}: {}", url, e);
            }
        }
    }
}