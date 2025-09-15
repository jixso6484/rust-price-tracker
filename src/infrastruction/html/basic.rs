use anyhow::Result;
use async_trait::async_trait;
use crate::infrastruction::database::models::Product;
use crate::infrastruction::html::coupang::coupangmain::CoupangParse;

pub struct ParserFactory;

#[async_trait]
pub trait HtmlParser: Send + Sync {
    async fn parse_product(&self, html: &str, url: &str) -> Result<Product>;
    async fn parse_product_list(&self, html: &str) -> Result<Vec<String>>;
    async fn interactive_elements(&self, html: &str) -> Result<Vec<String>>;
}
impl ParserFactory {
    pub fn get_parser(url: &str) -> Result<Box<dyn HtmlParser>> {
        if url.contains("coupang.com") {
            Ok(Box::new(CoupangParse::new()))
        } else {
            Err(anyhow::anyhow!("Unsupported site: {}", url))
        }
    }
}