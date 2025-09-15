use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use headless_chrome::Tab;
use super::chromiumAdapter::ChromiumAdapter;
use super::models::{BrowserAction, BrowserState};
use crate::components::error::error_cl::Result;

/// 탭 상태를 유지하는 브라우저 매니저
/// 기존 BrowserManager와 달리 탭을 재사용하여 연속적인 상호작용 가능
pub struct StatefulBrowserManager {
    adapter: ChromiumAdapter,
    // URL별로 탭을 캐시해서 재사용
    tab_cache: Arc<RwLock<HashMap<String, Arc<Tab>>>>,
}

impl StatefulBrowserManager {
    pub async fn new() -> Result<Self> {
        let adapter = ChromiumAdapter::new().await?;
        let tab_cache = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            adapter,
            tab_cache,
        })
    }
    
    /// URL에 해당하는 탭을 가져오거나 새로 생성
    async fn get_or_create_tab(&self, url: &str) -> Result<Arc<Tab>> {
        let base_url = self.extract_base_url(url);
        
        // 기존 탭이 있는지 확인
        {
            let cache = self.tab_cache.read().await;
            if let Some(tab) = cache.get(&base_url) {
                // 탭이 아직 유효한지 확인
                let current_url = tab.get_url();
                if current_url.contains(&base_url) {
                    println!("🔄 기존 탭 재사용: {}", base_url);
                    return Ok(tab.clone());
                }
            }
        }
        
        // 새 탭 생성
        println!("🆕 새 탭 생성: {}", url);
        let tab = self.adapter.new_page(url).await?;
        
        // 캐시에 저장
        {
            let mut cache = self.tab_cache.write().await;
            cache.insert(base_url, tab.clone());
        }
        
        Ok(tab)
    }
    
    /// 연속적인 액션 실행 (동일 탭 유지)
    pub async fn execute_action_continuously(&self, base_url: &str, action: BrowserAction) -> Result<BrowserState> {
        let tab = self.get_or_create_tab(base_url).await?;
        
        // 현재 URL 확인
        let current_url = tab.get_url();
        println!("📍 현재 위치: {}", current_url);
        
        // 액션 실행
        match &action {
            BrowserAction::Navigate { url } => {
                println!("🌐 페이지 이동: {} → {}", current_url, url);
                // Navigate의 경우 새로운 URL로 이동하므로 탭 캐시 업데이트 필요
                let result = self.adapter.execute_action(&tab, action.clone()).await?;
                
                // 캐시 업데이트 (새 URL의 base_url로)
                let new_base_url = self.extract_base_url(url);
                if new_base_url != self.extract_base_url(base_url) {
                    let mut cache = self.tab_cache.write().await;
                    cache.remove(&self.extract_base_url(base_url));
                    cache.insert(new_base_url, tab.clone());
                }
                
                Ok(result)
            },
            BrowserAction::Click { selector } => {
                println!("👆 클릭: {}", selector);
                self.adapter.execute_action(&tab, action.clone()).await
            },
            BrowserAction::Scroll { direction, amount } => {
                println!("📜 스크롤: {:?} {}px", direction, amount);
                self.adapter.execute_action(&tab, action.clone()).await
            },
            _ => {
                self.adapter.execute_action(&tab, action.clone()).await
            }
        }
    }
    
    /// URL에서 base URL 추출 (도메인 + 경로 일부)
    fn extract_base_url(&self, url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""))
        } else {
            url.to_string()
        }
    }
    
    /// 현재 상태 조회
    pub async fn get_current_state(&self, url: &str) -> Result<BrowserState> {
        let tab = self.get_or_create_tab(url).await?;
        self.adapter.execute_action(&tab, BrowserAction::GetPageState).await
    }
    
    /// 탭 캐시 정리
    pub async fn clear_cache(&self) {
        let mut cache = self.tab_cache.write().await;
        cache.clear();
        println!("🧹 탭 캐시 정리 완료");
    }
    
    /// 특정 URL의 탭 제거
    pub async fn close_tab(&self, url: &str) {
        let base_url = self.extract_base_url(url);
        let mut cache = self.tab_cache.write().await;
        if cache.remove(&base_url).is_some() {
            println!("❌ 탭 제거: {}", base_url);
        }
    }
}