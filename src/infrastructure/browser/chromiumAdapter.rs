use headless_chrome::{Browser, Tab, LaunchOptions};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use std::sync::Arc;
use std::time::Duration;
use std::ffi::OsStr;
use rand::Rng;
use crate::components::error::error_cl::{Result, ErrorS};
use super::models::{BrowserAction, BrowserState, ScrollDirection};

pub struct ChromiumAdapter{
    browser : Option<Browser>,
}

impl ChromiumAdapter{
    pub async fn new()->Result<Self>{
        println!("🔧 브라우저 초기화 중... (headless_chrome 사용)");
        
        let launch_options = LaunchOptions::default_builder()
            .path(Some(std::path::PathBuf::from(r#"C:\chrome-win64\chrome.exe"#)))
            .headless(true)
            .window_size(Some((1366, 768)))
.args(vec![
                OsStr::new("--no-sandbox"),
                OsStr::new("--disable-dev-shm-usage"),
                OsStr::new("--disable-gpu"),
                OsStr::new("--disable-web-security"),
                OsStr::new("--disable-features=VizDisplayCompositor"),
                OsStr::new("--disable-background-timer-throttling"),
                OsStr::new("--disable-backgrounding-occluded-windows"),
                OsStr::new("--disable-renderer-backgrounding"),
                // 봇 탐지 회피 설정
                OsStr::new("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36"),
                OsStr::new("--disable-blink-features=AutomationControlled"),
                OsStr::new("--exclude-switches=enable-automation"),
                OsStr::new("--disable-extensions-http-throttling"),
                OsStr::new("--disable-http2"),  // HTTP2 에러 방지
                OsStr::new("--accept-lang=ko-KR,ko;q=0.9,en;q=0.8"),
                OsStr::new("--disable-ipc-flooding-protection"),
            ])
            .build()
            .map_err(|e| ErrorS::browser("ChromiumAdapter::new", format!("Failed to build launch options: {}", e)))?;
        
        match Browser::new(launch_options) {
            Ok(browser) => {
                println!("✅ headless_chrome 브라우저 초기화 성공");
                tokio::time::sleep(Duration::from_millis(1000)).await;
                
                Ok(Self{
                    browser: Some(browser),
                })
            }
            Err(e) => {
                println!("❌ headless_chrome 브라우저 초기화 실패: {}", e);
                println!("💡 Chrome 경로나 권한을 확인해주세요");
                Err(ErrorS::browser("ChromiumAdapter::new", format!("Browser launch failed: {}", e)))
            }
        }
    }

    pub async fn new_page(&self, url: &str) -> Result<Arc<Tab>> {
        let browser = self.browser.as_ref()
            .ok_or_else(|| ErrorS::browser("ChromiumAdapter::new_page", "Browser not initialized"))?;

        println!("🌐 새 페이지 생성: {}", url);
        
        match browser.new_tab() {
            Ok(tab) => {
                println!("✅ 새 탭 생성 성공");
                
                // 봇 탐지 회피를 위한 JavaScript 실행
                let stealth_script = r#"
                    // webdriver 속성 숨기기
                    Object.defineProperty(navigator, 'webdriver', {get: () => undefined});
                    
                    // 플러그인 정보 설정
                    Object.defineProperty(navigator, 'plugins', {
                        get: () => [1, 2, 3, 4, 5]
                    });
                    
                    // 언어 설정
                    Object.defineProperty(navigator, 'languages', {
                        get: () => ['ko-KR', 'ko', 'en-US', 'en']
                    });
                "#;
                
                // 스텔스 스크립트 실행
                let _ = tab.evaluate(stealth_script, false);
                
                match tab.navigate_to(url) {
                    Ok(_) => {
                        println!("✅ 페이지 로드 성공: {}", url);
                        
                        // 인간처럼 행동하기: 랜덤 딜레이 (2-5초)
                        let mut rng = rand::thread_rng();
                        let delay_ms = rng.gen_range(2000..=5000);
                        println!("   ⏳ 인간 행동 시뮬레이션: {}ms 대기 중...", delay_ms);
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        
                        // 추가 인간 행동: 약간의 스크롤
                        let _ = tab.evaluate("window.scrollBy(0, Math.random() * 300);", false);
                        tokio::time::sleep(Duration::from_millis(rng.gen_range(500..=1500))).await;
                        
                        Ok(tab)
                    }
                    Err(e) => {
                        println!("❌ 페이지 로드 실패: {}", e);
                        Err(ErrorS::browser("ChromiumAdapter::new_page", format!("Failed to navigate to {}: {}", url, e)))
                    }
                }
            }
            Err(e) => {
                println!("❌ 새 탭 생성 실패: {}", e);
                Err(ErrorS::browser("ChromiumAdapter::new_page", format!("Failed to create new tab: {}", e)))
            }
        }
    }
    
    pub async fn close(&mut self)->Result<()>{
        if let Some(_browser) = self.browser.take(){
            // headless_chrome은 Drop trait으로 자동 정리됩니다
            println!("✅ 브라우저 정리 완료");
        }
        Ok(())
    }
    
    pub fn is_available(&self) -> bool {
        self.browser.is_some()
    }
    
    pub async fn execute_action(&self, tab: &Arc<Tab>, action: BrowserAction) -> Result<BrowserState> {
        match action {
            BrowserAction::Navigate { url } => {
                tab.navigate_to(&url)
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to navigate: {}", e)))?;
                self.get_page_state(tab).await
            },
            BrowserAction::Click { selector } => {
                tab.wait_for_element(&selector)
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to find element: {}", e)))?
                    .click()
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to click: {}", e)))?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                self.get_page_state(tab).await
            },
            BrowserAction::ExtractText { selector } => {
                let text = if let Some(sel) = selector {
                    match tab.wait_for_element(&sel) {
                        Ok(element) => element.get_inner_text().unwrap_or_default(),
                        Err(_) => String::new(),
                    }
                } else {
                    tab.get_content().unwrap_or_default()
                };
                
                Ok(BrowserState {
                    url: tab.get_url(),
                    title: tab.get_title().unwrap_or_default(),
                    html: Some(text),
                    screenshot: None,
                    extracted_text: None,
                    error: None,
                    interactive_elements: Vec::new(),
                })
            },
            BrowserAction::Scroll { direction, amount: _ } => {
                let script = match direction {
                    ScrollDirection::Down => "window.scrollBy(0, window.innerHeight);",
                    ScrollDirection::Up => "window.scrollBy(0, -window.innerHeight);",
                    ScrollDirection::Left => "window.scrollBy(-window.innerWidth, 0);",
                    ScrollDirection::Right => "window.scrollBy(window.innerWidth, 0);",
                };
                tab.evaluate(script, false)
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to scroll: {}", e)))?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                self.get_page_state(tab).await
            },
            BrowserAction::Screenshot => {
                let screenshot = tab.capture_screenshot(
                    CaptureScreenshotFormatOption::Png, 
                    None, 
                    None,
                    true
                ).map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to take screenshot: {}", e)))?;
                
                let mut state = self.get_page_state(tab).await?;
                state.screenshot = Some(screenshot);
                Ok(state)
            },
            BrowserAction::WaitForElement { selector, timeout } => {
                let _element = tab.wait_for_element_with_custom_timeout(&selector, std::time::Duration::from_millis(timeout))
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to wait for element: {}", e)))?;
                self.get_page_state(tab).await
            },
            BrowserAction::FillFrom { selector, value } => {
                tab.wait_for_element(&selector)
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to find element: {}", e)))?
                    .type_into(&value)
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to type: {}", e)))?;
                self.get_page_state(tab).await
            },
            BrowserAction::ExecuteJs { script } => {
                tab.evaluate(&script, false)
                    .map_err(|e| ErrorS::browser("ChromiumAdapter::execute_action", format!("Failed to execute script: {}", e)))?;
                self.get_page_state(tab).await
            },
            BrowserAction::GetPageState => {
                self.get_page_state(tab).await
            },
        }
    }

    async fn get_page_state(&self, tab: &Arc<Tab>) -> Result<BrowserState> {
        Ok(BrowserState {
            url: tab.get_url(),
            title: tab.get_title().unwrap_or_default(),
            html: Some(tab.get_content().unwrap_or_default()),
            screenshot: None,
            extracted_text: None,
            error: None,
            interactive_elements: Vec::new(),
        })
    }
}