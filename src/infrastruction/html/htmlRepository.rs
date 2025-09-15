use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Duration};
use crate::infrastruction::html::basic::{HtmlParser, ParserFactory};
use anyhow::Result;

pub struct HtmlRepository {
    // HTML 파서 저장소
}

impl HtmlRepository {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn get_parser(&self, url: &str) -> Result<Box<dyn HtmlParser>> {
        ParserFactory::get_parser(url)
    }
}

