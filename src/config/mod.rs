use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::components::error::crawler_error::{CrawlerError, CrawlerResult};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub browser: BrowserConfig,
    pub crawler: CrawlerConfig,
    pub llm: LLMConfig,
    pub logging: LoggingConfig,
    pub sites: HashMap<String, SiteConfig>,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub timeout: u64,
    pub user_agent: String,
    pub window_size: WindowSize,
    pub disable_images: bool,
    pub disable_javascript: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlerConfig {
    pub max_retries: u32,
    pub delay_between_requests: u64,
    pub max_concurrent_tabs: usize,
    pub request_timeout: u64,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LLMConfig {
    pub use_api: bool,
    pub api_provider: String,
    pub api_key: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub console: bool,
    pub json_format: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteConfig {
    pub domain: String,
    pub enabled: bool,
    pub rate_limit: u32,
    pub selectors: SiteSelectors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteSelectors {
    pub product_name: Vec<String>,
    pub price: Vec<String>,
    pub original_price: Option<Vec<String>>,
    pub discount: Option<Vec<String>>,
    pub image: Vec<String>,
    pub product_links: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub metrics_interval: u64,
    pub health_check_interval: u64,
    pub memory_threshold_mb: u64,
    pub cpu_threshold_percent: u32,
}

impl Config {
    /// 설정 파일에서 로드
    pub fn load_from_file(path: &str) -> CrawlerResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CrawlerError::config(format!("Failed to read config file {}: {}", path, e)))?;
        
        let mut config: Config = toml::from_str(&content)
            .map_err(|e| CrawlerError::config(format!("Failed to parse config file: {}", e)))?;
        
        // 환경변수로 민감한 정보 오버라이드
        config.override_from_env()?;
        
        // 설정 검증
        config.validate()?;
        
        println!("✅ Configuration loaded from: {}", path);
        Ok(config)
    }
    
    /// 기본 설정 생성
    pub fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgresql://localhost/price_tracker".to_string(),
                max_connections: 10,
                timeout: 30,
            },
            browser: BrowserConfig {
                headless: true,
                timeout: 30,
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
                window_size: WindowSize { width: 1920, height: 1080 },
                disable_images: false,
                disable_javascript: false,
            },
            crawler: CrawlerConfig {
                max_retries: 3,
                delay_between_requests: 1000,
                max_concurrent_tabs: 5,
                request_timeout: 30,
                rate_limit_per_minute: 60,
            },
            llm: LLMConfig {
                use_api: false,
                api_provider: "openai".to_string(),
                api_key: String::new(),
                max_tokens: 1000,
                temperature: 0.7,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: "crawler.log".to_string(),
                console: true,
                json_format: false,
            },
            sites: HashMap::new(),
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_interval: 60,
                health_check_interval: 300,
                memory_threshold_mb: 1000,
                cpu_threshold_percent: 80,
            },
        }
    }
    
    /// 환경변수로 설정 오버라이드
    fn override_from_env(&mut self) -> CrawlerResult<()> {
        // 데이터베이스 URL
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        
        // LLM API 키
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            self.llm.api_key = api_key;
            self.llm.use_api = true;
            self.llm.api_provider = "openai".to_string();
        } else if let Ok(api_key) = std::env::var("CLAUDE_API_KEY") {
            self.llm.api_key = api_key;
            self.llm.use_api = true;
            self.llm.api_provider = "claude".to_string();
        }
        
        // 로그 레벨
        if let Ok(log_level) = std::env::var("LOG_LEVEL") {
            self.logging.level = log_level;
        }
        
        // 헤드리스 모드
        if let Ok(headless) = std::env::var("HEADLESS") {
            self.browser.headless = headless.parse().unwrap_or(true);
        }
        
        Ok(())
    }
    
    /// 설정 검증
    fn validate(&self) -> CrawlerResult<()> {
        // 데이터베이스 URL 검증
        if self.database.url.is_empty() {
            return Err(CrawlerError::validation("database.url", "Database URL cannot be empty"));
        }
        
        // 브라우저 설정 검증
        if self.browser.timeout == 0 {
            return Err(CrawlerError::validation("browser.timeout", "Browser timeout must be greater than 0"));
        }
        
        // 크롤러 설정 검증
        if self.crawler.max_concurrent_tabs == 0 {
            return Err(CrawlerError::validation("crawler.max_concurrent_tabs", "Max concurrent tabs must be greater than 0"));
        }
        
        // LLM 설정 검증
        if self.llm.use_api && self.llm.api_key.is_empty() {
            return Err(CrawlerError::validation("llm.api_key", "API key is required when use_api is true"));
        }
        
        // 사이트 설정 검증
        for (name, site) in &self.sites {
            if site.domain.is_empty() {
                return Err(CrawlerError::validation(&format!("sites.{}.domain", name), "Site domain cannot be empty"));
            }
            if site.selectors.product_name.is_empty() {
                return Err(CrawlerError::validation(&format!("sites.{}.selectors.product_name", name), "Product name selectors cannot be empty"));
            }
        }
        
        println!("✅ Configuration validation passed");
        Ok(())
    }
    
    /// 특정 사이트 설정 가져오기
    pub fn get_site_config(&self, site_name: &str) -> Option<&SiteConfig> {
        self.sites.get(site_name)
    }
    
    /// 활성화된 사이트들만 가져오기
    pub fn get_enabled_sites(&self) -> Vec<(&String, &SiteConfig)> {
        self.sites.iter()
            .filter(|(_, config)| config.enabled)
            .collect()
    }
    
    /// 설정을 파일로 저장
    pub fn save_to_file(&self, path: &str) -> CrawlerResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CrawlerError::config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| CrawlerError::config(format!("Failed to write config file {}: {}", path, e)))?;
        
        println!("✅ Configuration saved to: {}", path);
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.crawler.max_retries, 3);
        assert!(config.browser.headless);
        assert_eq!(config.logging.level, "info");
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.database.url = String::new();
        assert!(config.validate().is_err());
    }
}