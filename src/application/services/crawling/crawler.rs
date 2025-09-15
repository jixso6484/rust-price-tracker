use std::sync::Arc;
use tokio::time::Duration;
use headless_chrome::Tab;

// Infrastructure 의존성
use crate::infrastruction::llm::llmRepository::LocalLLM;
use crate::infrastruction::browser::chromiumAdapter::ChromiumAdapter;
use crate::infrastruction::browser::models::{BrowserAction, BrowserState};
use crate::infrastruction::database::models::Product;
use crate::infrastruction::llm::models::MarketplaceStructure;

// 에러 처리
use crate::components::error::error_cl::Result;

/// 새로운 크롤링 오케스트레이터 - 간단하고 효율적인 설계
/// 
/// # 핵심 원칙
/// - 사이트별 독립 브라우저 인스턴스
/// - 단일 탭 재사용으로 상호작용 연속성 보장
/// - LLM의 간단한 액션 결정
/// - 명확한 에러 처리
pub struct NewCrawler {
    /// LLM 인스턴스 (공유)
    llm: Arc<LocalLLM>,
    
    /// 독립 브라우저 인스턴스
    browser: ChromiumAdapter,
    
    /// 현재 활성 탭 (재사용)
    current_tab: Option<Arc<Tab>>,
    
    /// 크롤링 대상 사이트
    site: MarketplaceStructure,
}

impl NewCrawler {
    /// 사이트별 크롤러 생성
    pub async fn new(site: MarketplaceStructure) -> Result<Self> {
        let llm = LocalLLM::get_instance().await?;
        let browser = ChromiumAdapter::new().await?;
        
        println!("✅ 새 크롤러 생성: {}", site.platform.name);
        
        Ok(Self {
            llm,
            browser,
            current_tab: None,
            site,
        })
    }
    
    /// 단순한 크롤링 실행
    pub async fn crawl(&mut self, max_actions: usize) -> Result<Vec<Product>> {
        let mut products = Vec::new();
        let mut action_count = 0;
        
        println!("🚀 크롤링 시작: {}", self.site.platform.domain);
        
        // 1. 초기 페이지 로드
        let mut current_state = self.navigate_to_site().await?;
        
        // 2. 크롤링 루프
        while action_count < max_actions {
            // HTML 분석 및 액션 결정
            if let Some(html) = &current_state.html {
                // LLM에게 다음 액션 물어보기
                let llm_response = self.ask_llm_for_action(html, &current_state.url).await?;
                
                // 응답 파싱
                let (action_type, value, reason) = self.llm.decide_browser_action(&llm_response).await?;
                
                println!("🤖 LLM 결정: {} (값: {}) - {}", action_type, value, reason);
                
                // 액션 실행
                match self.execute_decided_action(&action_type, value).await {
                    Ok(new_state) => {
                        current_state = new_state;
                        
                        // 상품 정보 수집 체크
                        if self.should_extract_products(&current_state) {
                            println!("📦 상품 추출 가능 페이지 감지");
                        }
                    },
                    Err(e) => {
                        println!("❌ 액션 실행 실패: {}", e);
                        break;
                    }
                }
            }
            
            action_count += 1;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        
        println!("🏁 크롤링 완료: {}개 액션 실행", action_count);
        Ok(products)
    }
    
    /// 사이트 초기 진입
    async fn navigate_to_site(&mut self) -> Result<BrowserState> {
        let url = &self.site.platform.domain;
        
        if self.current_tab.is_none() {
            println!("🆕 새 탭 생성: {}", url);
            let tab = self.browser.new_page(url).await?;
            self.current_tab = Some(tab);
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let tab = self.current_tab.as_ref().unwrap();
        self.browser.execute_action(tab, BrowserAction::GetPageState).await
    }
    
    /// LLM에게 액션 요청
    async fn ask_llm_for_action(&self, html: &str, url: &str) -> Result<String> {
        let prompt = format!(
            r#"
            현재 페이지: {}
            
            다음 중 하나의 액션을 선택하세요:
            - click: 상품 링크나 버튼 클릭
            - scroll: 페이지 스크롤
            - extract: 현재 페이지에서 정보 추출
            
            응답 형식: action: [액션명], value: 1, reason: [이유]
            
            HTML: {}
            "#, 
            url, 
            html.chars().take(1000).collect::<String>()
        );
        
        self.llm.generate(&prompt).await
    }
    
    /// 결정된 액션 실행
    async fn execute_decided_action(&mut self, action_type: &str, _value: i32) -> Result<BrowserState> {
        let tab = self.current_tab.as_ref().unwrap();
        
        let browser_action = match action_type {
            "click" => BrowserAction::Click { 
                selector: "a, button".to_string() 
            },
            "scroll" => BrowserAction::Scroll { 
                direction: crate::infrastruction::browser::models::ScrollDirection::Down, 
                amount: 500 
            },
            "extract" => BrowserAction::ExtractText { 
                selector: None 
            },
            _ => BrowserAction::GetPageState,
        };
        
        self.browser.execute_action(tab, browser_action).await
    }
    
    /// 상품 추출 가능 여부 판단
    fn should_extract_products(&self, state: &BrowserState) -> bool {
        if let Some(html) = &state.html {
            html.contains("product") || html.contains("price") || html.contains("상품")
        } else {
            false
        }
    }
}