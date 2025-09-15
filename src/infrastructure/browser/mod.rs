pub mod models;
pub mod service;
pub mod chromiumAdapter;
pub mod BrowserManager;
pub mod StatefulBrowserManager;

// Re-export for convenience
pub use service::BrowserService;