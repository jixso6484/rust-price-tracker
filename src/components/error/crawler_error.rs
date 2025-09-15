use thiserror::Error;

/// 크롤러 전용 에러 타입
#[derive(Debug, Error)]
pub enum CrawlerError {
    #[error("Browser error: {0}")]
    Browser(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Parsing error: {0}")]
    Parser(String),
    
    #[error("LLM error: {0}")]
    LLM(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Timeout error: operation timed out after {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("External service error: {service}: {error}")]
    ExternalService { service: String, error: String },
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl CrawlerError {
    pub fn browser<T: Into<String>>(msg: T) -> Self {
        Self::Browser(msg.into())
    }
    
    pub fn parser<T: Into<String>>(msg: T) -> Self {
        Self::Parser(msg.into())
    }
    
    pub fn llm<T: Into<String>>(msg: T) -> Self {
        Self::LLM(msg.into())
    }
    
    pub fn config<T: Into<String>>(msg: T) -> Self {
        Self::Config(msg.into())
    }
    
    pub fn rate_limit<T: Into<String>>(msg: T) -> Self {
        Self::RateLimit(msg.into())
    }
    
    pub fn auth<T: Into<String>>(msg: T) -> Self {
        Self::Auth(msg.into())
    }
    
    pub fn validation<T: Into<String>>(field: T, message: T) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
    
    pub fn not_found<T: Into<String>>(resource: T) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }
    
    pub fn external<T: Into<String>>(service: T, error: T) -> Self {
        Self::ExternalService {
            service: service.into(),
            error: error.into(),
        }
    }
    
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { timeout_seconds: seconds }
    }
    
    pub fn unknown<T: Into<String>>(msg: T) -> Self {
        Self::Unknown(msg.into())
    }
    
    /// 에러가 재시도 가능한지 판단
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::Timeout { .. } => true,
            Self::RateLimit(_) => true,
            Self::ExternalService { .. } => true,
            Self::Database(_) => false, // DB 에러는 보통 재시도 불가
            Self::Browser(_) => true,   // 브라우저 에러는 재시도 가능
            Self::Parser(_) => false,   // 파싱 에러는 보통 재시도 불가
            Self::LLM(_) => true,       // LLM 에러는 재시도 가능
            Self::Config(_) => false,   // 설정 에러는 재시도 불가
            Self::Auth(_) => false,     // 인증 에러는 재시도 불가
            Self::Validation { .. } => false, // 검증 에러는 재시도 불가
            Self::NotFound { .. } => false,   // 리소스 없음은 재시도 불가
            Self::Io(_) => true,        // IO 에러는 재시도 가능
            Self::Serialization(_) => false,  // 직렬화 에러는 재시도 불가
            Self::Unknown(_) => false,  // 알 수 없는 에러는 재시도 불가
        }
    }
    
    /// 에러 심각도 반환
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Config(_) | Self::Auth(_) | Self::Validation { .. } => ErrorSeverity::Critical,
            Self::Database(_) | Self::Serialization(_) => ErrorSeverity::High,
            Self::Parser(_) | Self::NotFound { .. } => ErrorSeverity::Medium,
            Self::Network(_) | Self::Timeout { .. } | Self::RateLimit(_) => ErrorSeverity::Low,
            Self::Browser(_) | Self::LLM(_) | Self::Io(_) => ErrorSeverity::Medium,
            Self::ExternalService { .. } | Self::Unknown(_) => ErrorSeverity::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Result type alias
pub type CrawlerResult<T> = std::result::Result<T, CrawlerError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = CrawlerError::browser("Tab crashed");
        assert!(matches!(err, CrawlerError::Browser(_)));
        assert!(err.is_retryable());
        assert_eq!(err.severity(), ErrorSeverity::Medium);
    }
    
    #[test]
    fn test_error_retryable() {
        assert!(CrawlerError::network(reqwest::Error::from(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"))).is_retryable());
        assert!(!CrawlerError::config("Invalid config").is_retryable());
    }
    
    #[test]
    fn test_error_severity() {
        assert_eq!(CrawlerError::config("test").severity(), ErrorSeverity::Critical);
        assert_eq!(CrawlerError::parser("test").severity(), ErrorSeverity::Medium);
        assert_eq!(CrawlerError::rate_limit("test").severity(), ErrorSeverity::Low);
    }
}