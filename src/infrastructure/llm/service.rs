use std::sync::Arc;
use tokio::sync::OnceCell;
use crate::components::error::error_cl::{Result, ErrorS};

#[derive(Debug)]
pub struct LLMService {
    use_api: bool,
    api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PageAnalysis {
    pub page_type: String,
    pub recommended_action: String,
    pub confidence: f32,
    pub reasoning: String,
    pub elements_found: Vec<String>,
}

// 싱글톤 인스턴스
static LLM_INSTANCE: OnceCell<Arc<LLMService>> = OnceCell::const_new();

impl LLMService {
    pub async fn new() -> Result<Self> {
        println!("🧠 LLM Service 초기화");
        
        // 환경변수에서 API 키 확인
        let api_key = std::env::var("OPENAI_API_KEY").ok()
            .or_else(|| std::env::var("CLAUDE_API_KEY").ok());
        
        let use_api = api_key.is_some();
        
        if use_api {
            println!("✅ LLM API 키 발견 - API 모드 사용");
        } else {
            println!("⚠️ LLM API 키 없음 - 규칙 기반 분석 모드 사용");
        }
        
        Ok(Self {
            use_api,
            api_key,
        })
    }
    
    /// 싱글톤 인스턴스 가져오기
    pub async fn get_instance() -> Result<Arc<LLMService>> {
        LLM_INSTANCE.get_or_try_init(|| async {
            let llm = Self::new().await?;
            Ok(Arc::new(llm))
        }).await.cloned()
    }
    
    /// 페이지 분석
    pub async fn analyze_page(&self, html: &str, url: &str) -> Result<PageAnalysis> {
        if self.use_api {
            self.api_analyze_page(html, url).await
        } else {
            self.rule_based_analyze_page(html, url).await
        }
    }
    
    /// 브라우저 액션 추천
    pub async fn recommend_action(&self, html: &str, goal: &str) -> Result<String> {
        if self.use_api {
            self.api_recommend_action(html, goal).await
        } else {
            self.rule_based_recommend_action(html, goal).await
        }
    }
    
    /// 제품 정보 추출 도움
    pub async fn extract_product_info(&self, html: &str) -> Result<Vec<String>> {
        // 제품 정보 추출을 위한 셀렉터 추천
        let selectors = vec![
            ".product-name".to_string(),
            ".product-title".to_string(),
            ".price".to_string(),
            ".sale-price".to_string(),
            ".original-price".to_string(),
            ".discount".to_string(),
            ".product-image".to_string(),
            ".description".to_string(),
        ];
        
        Ok(selectors)
    }
    
    // Private methods
    
    async fn api_analyze_page(&self, html: &str, url: &str) -> Result<PageAnalysis> {
        // TODO: 실제 API 호출 구현 (OpenAI, Claude 등)
        println!("🔍 API 모드로 페이지 분석 중...");
        
        // 현재는 규칙 기반으로 폴백
        self.rule_based_analyze_page(html, url).await
    }
    
    async fn rule_based_analyze_page(&self, html: &str, url: &str) -> Result<PageAnalysis> {
        println!("🔍 규칙 기반 페이지 분석 중...");
        
        let html_lower = html.to_lowercase();
        let url_lower = url.to_lowercase();
        
        // 페이지 타입 결정
        let page_type = if url_lower.contains("product") || url_lower.contains("item") {
            "product_detail"
        } else if url_lower.contains("search") || url_lower.contains("category") {
            "product_list"
        } else if url_lower.contains("cart") {
            "shopping_cart"
        } else if url_lower.contains("checkout") {
            "checkout"
        } else if html_lower.contains("product") && html_lower.contains("price") {
            "product_detail"
        } else if html_lower.contains("search-result") || html_lower.contains("product-list") {
            "product_list"
        } else {
            "main_page"
        }.to_string();
        
        // 액션 추천
        let recommended_action = match page_type.as_str() {
            "product_detail" => {
                if html_lower.contains("add to cart") || html_lower.contains("장바구니") {
                    "extract_product_info"
                } else {
                    "scroll_for_more_info"
                }
            },
            "product_list" => {
                if html_lower.contains("next") || html_lower.contains("다음") {
                    "click_next_page"
                } else {
                    "extract_product_links"
                }
            },
            "main_page" => "navigate_to_categories",
            _ => "extract_text"
        }.to_string();
        
        // 발견된 요소들
        let mut elements_found = Vec::new();
        
        if html_lower.contains("price") {
            elements_found.push("price_element".to_string());
        }
        if html_lower.contains("button") {
            elements_found.push("interactive_button".to_string());
        }
        if html_lower.contains("form") {
            elements_found.push("form_element".to_string());
        }
        if html_lower.contains("pagination") || html_lower.contains("페이지") {
            elements_found.push("pagination".to_string());
        }
        
        let confidence = if page_type == "product_detail" && elements_found.contains(&"price_element".to_string()) {
            0.9
        } else if page_type == "product_list" {
            0.8
        } else {
            0.6
        };
        
        let reasoning = format!(
            "페이지 타입: {}. URL 패턴과 HTML 요소를 기반으로 분석. 발견된 요소: {}개",
            page_type, elements_found.len()
        );
        
        Ok(PageAnalysis {
            page_type,
            recommended_action,
            confidence,
            reasoning,
            elements_found,
        })
    }
    
    async fn api_recommend_action(&self, html: &str, goal: &str) -> Result<String> {
        // TODO: 실제 API 호출 구현
        println!("🎯 API 모드로 액션 추천 중...");
        
        // 현재는 규칙 기반으로 폴백
        self.rule_based_recommend_action(html, goal).await
    }
    
    async fn rule_based_recommend_action(&self, html: &str, goal: &str) -> Result<String> {
        println!("🎯 규칙 기반 액션 추천 중...");
        
        let html_lower = html.to_lowercase();
        let goal_lower = goal.to_lowercase();
        
        let action = if goal_lower.contains("product") || goal_lower.contains("상품") {
            if html_lower.contains("add to cart") || html_lower.contains("장바구니") {
                "action: extract_text, selector: .product-info, reason: 상품 정보 추출"
            } else if html_lower.contains("next") || html_lower.contains("다음") {
                "action: click, selector: .next-button, reason: 다음 페이지로 이동"
            } else {
                "action: scroll, direction: down, pixels: 500, reason: 더 많은 상품 정보 확인"
            }
        } else if goal_lower.contains("search") || goal_lower.contains("검색") {
            "action: click, selector: .search-button, reason: 검색 실행"
        } else if goal_lower.contains("scroll") || goal_lower.contains("스크롤") {
            "action: scroll, direction: down, pixels: 800, reason: 추가 콘텐츠 로드"
        } else {
            "action: extract_text, selector: body, reason: 페이지 정보 수집"
        };
        
        Ok(action.to_string())
    }
}

// 호환성을 위한 alias
pub type LocalLLM = LLMService;