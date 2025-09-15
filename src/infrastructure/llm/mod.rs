pub mod service;
pub mod models;
pub mod llmRepository;
pub mod response_parser;

// Re-export for convenience
pub use service::{LLMService, LocalLLM, PageAnalysis};