use std::time::{Duration, Instant, SystemTime};
use std::collections::HashMap;
use std::sync::Arc;
use headless_chrome::Tab;

// Infrastructure 의존성
use crate::infrastruction::llm::llmRepository::LocalLLM;
use crate::infrastruction::browser::chromiumAdapter::ChromiumAdapter;
use crate::infrastruction::browser::models::{BrowserAction, BrowserState};
use crate::infrastruction::llm::models::MarketplaceStructure;
use crate::infrastruction::llm::response_parser::ResponseParser;
use crate::infrastruction::html::basic::{ParserFactory, HtmlParser};
use crate::infrastruction::database::models::Product;
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
    
    parser : HtmlParser;

    // 현재 크롤링 사이트
    pub site: String,
    //크롤링 여부
    pub crawling: bool,
}


#[derive(Debug)]s
struct monitoring{
    
    // 실행 통계
    pub total_requests: u64,        // 총 요청 수
    pub successful_requests: u64,   // 성공한 요청 수
    pub failed_requests: u32,       // 실패한 요청 수 (기존 crawling_fail)
    pub consecutive_failures: u32,  // 연속 실패 수
    
    // 데이터 수집 통계 DB 저장 
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
}
impl CrawlingOrchestrator {
    /// 사이트별 독립 크롤링 오케스트레이터 생성
    pub async fn new(site_url: &str) -> Result<Self> {
        let llm = LocalLLM::get_instance().await?;
        let browser = ChromiumAdapter::new().await?;
        let parsers = ParserFactory::new();

        
        println!("🚀 독립 크롤링 오케스트레이터 생성: {}", site_url);
        
        Ok(Self {
            llm,
            browser,
            current_tab: None,
            parsers,
            site: site_url.to_string(),
            crawling: false,
        })
    }
    
    /// 크롤링 시작
    pub async fn start_crawling(&mut self) -> Result<()> {
        self.crawling = true;
        self.started_at = Some(SystemTime::now());
        println!("✅ 크롤링 시작: {}", self.site);
        Ok(())

        if self.crawling{}
    }
    
    /// 크롤링 중지
    pub async fn stop_crawling(&mut self) {
        self.crawling = false;
        println!("🛑 크롤링 중지: {}", self.site);
    }
}