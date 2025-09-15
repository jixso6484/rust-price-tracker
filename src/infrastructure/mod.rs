pub mod llm;
pub mod database;
pub mod browser;
pub mod html;

// Re-exports for convenience
pub use database::ProductRepository;
pub use browser::BrowserService;
pub use llm::{LLMService, LocalLLM};