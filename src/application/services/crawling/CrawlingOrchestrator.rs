use std::time::{Duration, Instant, SystemTime};
use std::collections::HashMap;
use std::sync::Arc;
use headless_chrome::Tab;

// Infrastructure 의존성
use crate::infrastruction::llm::llmRepository::LocalLLM;
use crate::infrastruction::browser::chromiumAdapter::ChromiumAdapter;
use crate::infrastruction::browser::models::{BrowserAction, BrowserState};
use crate::infrastruction::html::basic::{ParserFactory, HtmlParser};
use crate::infrastruction::database::models::Product;
use crate::infrastruction::llm::models::MarketplaceStructure;

// 에러 처리
use crate::components::error::error_cl::Result;

#[derive(Debug)]
pub struct CrawlingOrchestrator {
    /// 독립 LLM 인스턴스 
    llm: Arc<LocalLLM>,
    
    /// 독립 브라우저 인스턴스 (사이트별 전용)
    browser: ChromiumAdapter,
    
    /// 현재 활성 탭 (재사용)
    current_tab: Option<Arc<Tab>>,
    
    /// HTML 파서
    parser: Box<dyn HtmlParser>,

    /// 모니터링 정보
    monitoring: Monitoring,
    
    /// 크롤링 대상 사이트들
    sites: Vec<MarketplaceStructure>,
}

#[derive(Debug)]
struct Monitoring {
    // 실행 통계
    pub total_requests: u64,        // 총 요청 수
    pub successful_requests: u64,   // 성공한 요청 수
    pub failed_requests: u32,       // 실패한 요청 수
    pub consecutive_failures: u32,  // 연속 실패 수
    
    // 데이터 수집 통계
    pub items_collected: u64,       // 수집된 아이템 수
    pub duplicate_items: u64,       // 중복 아이템 수
    pub new_items: u64,            // 새로운 아이템 수
    
    // 성능 메트릭
    pub avg_response_time: Duration,    // 평균 응답 시간
    pub requests_per_minute: f64,       // 분당 요청 수
    pub last_request_time: Option<Instant>, // 마지막 요청 시간
    
    // 시간 정보
    pub started_at: Option<SystemTime>,     // 시작 시간
    pub last_success_at: Option<SystemTime>, // 마지막 성공 시간
    pub last_failure_at: Option<SystemTime>, // 마지막 실패 시간
    pub next_scheduled_at: Option<SystemTime>, // 다음 예정 시간
    
    // 시스템 리소스
    pub memory_usage: u64,          // 현재 메모리 사용량 (bytes)
    pub peak_memory_usage: u64,     // 최대 메모리 사용량
    
    // 에러 세부 정보
    pub error_counts: HashMap<String, u32>, // 에러 타입별 카운트
    pub last_error: Option<String>,         // 마지막 에러 메시지
    
    // 설정 정보
    pub crawl_interval: Duration,   // 크롤링 간격
    pub timeout: Duration,          // 타임아웃
    pub max_retries: u32,          // 최대 재시도 횟수
    pub current_retry: u32,        // 현재 재시도 카운트
    
    // 크롤링 상태
    pub is_crawling: bool,         // 크롤링 중 여부
}

impl Default for Monitoring {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            consecutive_failures: 0,
            items_collected: 0,
            duplicate_items: 0,
            new_items: 0,
            avg_response_time: Duration::from_secs(0),
            requests_per_minute: 0.0,
            last_request_time: None,
            started_at: None,
            last_success_at: None,
            last_failure_at: None,
            next_scheduled_at: None,
            memory_usage: 0,
            peak_memory_usage: 0,
            error_counts: HashMap::new(),
            last_error: None,
            crawl_interval: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            current_retry: 0,
            is_crawling: false,
        }
    }
}

impl CrawlingOrchestrator {
    /// 사이트별 독립 크롤링 오케스트레이터 생성
    pub async fn new(sites: Vec<MarketplaceStructure>) -> Result<Self> {
        let llm = LocalLLM::get_instance().await?;
        let browser = ChromiumAdapter::new().await?;
        
        // 첫 번째 사이트를 기준으로 파서 생성
        let parser = if !sites.is_empty() {
            ParserFactory::get_parser(&sites[0].platform.domain)?
        } else {
            return Err(crate::components::error::error_cl::ErrorS::data(
                "CrawlingOrchestrator::new",
                "No sites provided"
            ));
        };
        
        println!("🚀 독립 크롤링 오케스트레이터 생성 완료");
        
        Ok(Self {
            llm,
            browser,
            current_tab: None,
            parser,
            monitoring: Monitoring::default(),
            sites,
        })
    }
    
    /// 크롤링 시작
    pub async fn start_crawling(&mut self) -> Result<()> {
        self.monitoring.is_crawling = true;
        self.monitoring.started_at = Some(SystemTime::now());
        println!("✅ 크롤링 시작");
        
        // 크롤링 로직 구현
        Ok(())
    }
    
    /// 크롤링 중지
    pub async fn stop_crawling(&mut self) {
        self.monitoring.is_crawling = false;
        println!("🛑 크롤링 중지");
    }
    
    /// 연속 크롤링 실행
    pub async fn crawl_continuously(&mut self) -> Result<Vec<Product>> {
        let mut all_products = Vec::new();
        
        for site in self.sites.clone() {
            println!("🌐 사이트 크롤링 시작: {}", site.platform.name);
            
            // 사이트 URL로 이동
            let tab = self.browser.new_page(&site.platform.domain).await?;
            self.current_tab = Some(tab.clone());
            
            // 페이지 로드 대기
            tokio::time::sleep(Duration::from_secs(3)).await;
            
            // HTML 가져오기
            let state = self.browser.execute_action(&tab, BrowserAction::GetPageState).await?;
            
            if let Some(html) = state.html {
                // 상품 목록 파싱
                let product_urls = self.parser.parse_product_list(&html).await?;
                println!("📦 {}개 상품 URL 발견", product_urls.len());
                
                // 각 상품 상세 페이지 크롤링 (최대 10개만)
                for url in product_urls.iter().take(10) {
                    println!("🔍 상품 페이지 크롤링: {}", url);
                    
                    let product_tab = self.browser.new_page(url).await?;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    
                    let product_state = self.browser.execute_action(&product_tab, BrowserAction::GetPageState).await?;
                    
                    if let Some(product_html) = product_state.html {
                        match self.parser.parse_product(&product_html, url).await {
                            Ok(product) => {
                                println!("✅ 상품 수집: {}", product.product_name);
                                all_products.push(product);
                                self.monitoring.items_collected += 1;
                            },
                            Err(e) => {
                                println!("❌ 상품 파싱 실패: {}", e);
                                self.monitoring.failed_requests += 1;
                            }
                        }
                    }
                    
                    // Rate limiting
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        
        println!("📊 총 {}개 상품 수집 완료", all_products.len());
        Ok(all_products)
    }
    
    /// 모니터링 정보 조회
    pub fn get_monitoring(&self) -> &Monitoring {
        &self.monitoring
    }
}