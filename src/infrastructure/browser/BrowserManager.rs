use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use super::chromiumAdapter::ChromiumAdapter;
use super::models::{BrowserAction, BrowserState};
use crate::components::error::error_cl::Result;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// 브라우저 인스턴스와 메트릭스를 관리하는 싱글톤 매니저
/// 
/// # 주요 기능
/// - ChromiumAdapter의 생명주기 관리 (생성, 재사용, 종료)
/// - 브라우저 액션 실행 시 성능 메트릭스 수집
/// - 멀티탭 환경에서의 리소스 모니터링
/// 
/// # 설계 원칙
/// - 싱글톤 패턴: 시스템 전체에서 하나의 브라우저만 사용
/// - 지연 초기화: 실제 필요할 때만 브라우저 생성
/// - 스레드 안전: 여러 스레드에서 동시 사용 가능
pub struct BrowserManager {
    /// ChromiumAdapter 인스턴스를 감싸는 이중 잠금 구조
    /// - 외부 RwLock: 어댑터 생성/삭제 시에만 쓰기 잠금
    /// - 내부 RwLock: 어댑터 사용 시 읽기/쓰기 잠금
    adapter: Arc<RwLock<Option<Arc<RwLock<ChromiumAdapter>>>>>,
    
    /// 브라우저 성능 및 상태 메트릭스
    /// AtomicU64/AtomicUsize 사용으로 락 없이 빠른 카운터 조작
    metrics: BrowserMetrics,
}

/// 브라우저 모니터링 메트릭스
/// 
/// # 수집 항목
/// - 성능: 페이지 로드 횟수, 액션 실행 시간
/// - 상태: 활성 탭 수, 에러 발생 횟수
/// - 통계: 평균 응답 시간 (계산됨)
/// 
/// # 스레드 안전성
/// 모든 필드가 Atomic 타입으로 락 없이 멀티스레드에서 안전하게 사용 가능
/// Ordering::Relaxed 사용으로 최고 성능 보장 (순서 보장 불필요)
pub struct BrowserMetrics {
    /// 총 페이지 로드 수 (Navigate 액션 카운트)
    pub page_loads_total: AtomicU64,
    
    /// 총 브라우저 액션 실행 수 (모든 액션 포함)
    pub actions_total: AtomicU64,
    
    /// 총 에러 발생 수 (실패한 액션 카운트)
    pub errors_total: AtomicU64,
    
    /// 현재 활성 탭 수 (동적으로 증감)
    pub active_tabs: AtomicUsize,
    
    /// 총 액션 실행 시간 누적 (밀리초 단위)
    /// 평균 계산: total_action_time_ms / actions_total
    pub total_action_time_ms: AtomicU64,
}

impl Default for BrowserMetrics {
    fn default() -> Self {
        Self {
            page_loads_total: AtomicU64::new(0),
            actions_total: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            active_tabs: AtomicUsize::new(0),
            total_action_time_ms: AtomicU64::new(0),
        }
    }
}

pub static BROWSER_MANAGER: Lazy<BrowserManager> = Lazy::new(|| {
    BrowserManager::new()
});

impl BrowserManager {
    fn new() -> Self {
        Self {
            adapter: Arc::new(RwLock::new(None)),
            metrics: BrowserMetrics::default(),
        }
    }
    
    pub async fn get_adapter(&self) -> Result<Arc<RwLock<ChromiumAdapter>>> {
        let mut adapter_lock = self.adapter.write().await;
        
        if adapter_lock.is_none() {
            let new_adapter = ChromiumAdapter::new().await?;
            *adapter_lock = Some(Arc::new(RwLock::new(new_adapter)));
        }
        
        Ok(adapter_lock.as_ref().unwrap().clone())
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        let mut adapter_lock = self.adapter.write().await;
        
        if let Some(adapter) = adapter_lock.take() {
            let mut adapter = adapter.write().await;
            adapter.close().await?;
        }
        
        Ok(())
    }
    
    /// 브라우저 액션을 실행하면서 성능 메트릭스를 수집
    /// 
    /// # 메트릭스 수집 항목
    /// - 액션 실행 횟수 (actions_total)
    /// - 페이지 로드 횟수 (Navigate 액션인 경우)
    /// - 실행 시간 (밀리초 단위로 누적)
    /// - 에러 발생 횟수 (실패한 액션)
    /// 
    /// # 실행 과정
    /// 1. 시작 시간 기록 및 카운터 증가
    /// 2. 브라우저 어댑터 획득 (지연 초기화)
    /// 3. 새 페이지 생성 및 액션 실행
    /// 4. 실행 시간 및 결과 메트릭스 업데이트
    /// 
    /// # Parameters
    /// - `url`: 대상 페이지 URL
    /// - `action`: 실행할 브라우저 액션
    /// 
    /// # Returns
    /// 액션 실행 후의 브라우저 상태 또는 에러
    pub async fn execute_action_with_monitoring(&self, url: &str, action: BrowserAction) -> Result<BrowserState> {
        // ⏱️ 성능 측정 시작
        let start_time = Instant::now();
        
        // 📊 총 액션 수 증가 (모든 액션 카운트)
        self.metrics.actions_total.fetch_add(1, Ordering::Relaxed);
        
        // 📄 페이지 로드 카운터 (Navigate 액션만)
        if matches!(action, BrowserAction::Navigate { .. }) {
            self.metrics.page_loads_total.fetch_add(1, Ordering::Relaxed);
        }
        
        // 🌐 실제 브라우저 액션 실행
        let result = async {
            let adapter = self.get_adapter().await?;           // 어댑터 획득 (지연 초기화)
            let adapter_lock = adapter.read().await;            // 읽기 잠금 (동시 사용 가능)
            let tab = adapter_lock.new_page(url).await?;        // 새 탭 생성 (headless_chrome)
            adapter_lock.execute_action(&tab, action).await     // 액션 실행
        }.await;
        
        // ⏱️ 실행 시간 측정 및 누적
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.metrics.total_action_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        // 🚨 에러 발생 시 에러 카운터 증가
        if result.is_err() {
            self.metrics.errors_total.fetch_add(1, Ordering::Relaxed);
        }
        
        result
    }
    
    /// 현재 수집된 모든 메트릭스를 스냅샷으로 조회
    /// 
    /// # 제공 정보
    /// - 페이지 로드 총 횟수
    /// - 브라우저 액션 총 실행 횟수  
    /// - 에러 총 발생 횟수
    /// - 현재 활성 탭 수
    /// - 총 실행 시간 및 평균 실행 시간
    /// 
    /// # 성능
    /// - Atomic 타입의 load() 연산으로 매우 빠름
    /// - 락 없이 실행되어 성능 영향 최소
    /// - 평균 시간은 실시간으로 계산됨
    /// 
    /// # Returns
    /// 현재 시점의 메트릭스 스냅샷 (읽기 전용)
    pub fn get_metrics(&self) -> BrowserMetricsSnapshot {
        BrowserMetricsSnapshot {
            page_loads_total: self.metrics.page_loads_total.load(Ordering::Relaxed),
            actions_total: self.metrics.actions_total.load(Ordering::Relaxed),
            errors_total: self.metrics.errors_total.load(Ordering::Relaxed),
            active_tabs: self.metrics.active_tabs.load(Ordering::Relaxed),
            total_action_time_ms: self.metrics.total_action_time_ms.load(Ordering::Relaxed),
            
            // 📈 평균 액션 실행 시간 계산 (0으로 나누기 방지)
            avg_action_time_ms: {
                let total_time = self.metrics.total_action_time_ms.load(Ordering::Relaxed);
                let total_actions = self.metrics.actions_total.load(Ordering::Relaxed);
                if total_actions > 0 { total_time / total_actions } else { 0 }
            },
        }
    }
    
    /// 활성 탭 수를 1 증가 (새 탭 생성 시 호출)
    /// 
    /// # 사용 시점
    /// - 새로운 페이지/탭 생성 시
    /// - 멀티탭 크롤링 시작 시
    /// 
    /// # 스레드 안전성
    /// fetch_add()로 원자적 연산 보장
    pub fn increment_active_tabs(&self) {
        self.metrics.active_tabs.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 활성 탭 수를 1 감소 (탭 종료 시 호출)
    /// 
    /// # 사용 시점  
    /// - 페이지/탭 종료 시
    /// - 크롤링 완료 후 정리 시
    /// 
    /// # 주의사항
    /// 0 이하로 내려가지 않도록 호출측에서 관리 필요
    pub fn decrement_active_tabs(&self) {
        self.metrics.active_tabs.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 브라우저 메트릭스의 불변 스냅샷 (조회 전용)
/// 
/// # 목적
/// - 특정 시점의 메트릭스 상태를 안전하게 조회
/// - Grafana, Prometheus 등 모니터링 도구로 전송
/// - 로깅 및 디버깅용 데이터 제공
/// 
/// # 데이터 일관성
/// get_metrics() 호출 시점의 원자적 스냅샷
/// 개별 필드들이 동일한 시점의 데이터임을 보장하지는 않음
/// (Atomic 연산들이 각각 독립적으로 실행되기 때문)
#[derive(Debug)]
pub struct BrowserMetricsSnapshot {
    /// 총 페이지 로드 횟수 (Navigate 액션)
    pub page_loads_total: u64,
    
    /// 총 브라우저 액션 실행 횟수
    pub actions_total: u64,
    
    /// 총 에러 발생 횟수  
    pub errors_total: u64,
    
    /// 현재 활성 탭 수
    pub active_tabs: usize,
    
    /// 총 액션 실행 시간 (밀리초)
    pub total_action_time_ms: u64,
    
    /// 평균 액션 실행 시간 (밀리초)
    /// 계산식: total_action_time_ms / actions_total
    pub avg_action_time_ms: u64,
}