use serde::{Deserialize,Serialize};
use std::collections::HashMap;

// Tagged Union
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction{
    Navigate {url : String },
    Click{ selector : String },
    Scroll{ direction:ScrollDirection, amount: i32 },
    ExtractText { selector : Option<String> },
    Screenshot,
    WaitForElement{ selector:String, timeout:u64 },
    FillFrom{ selector : String, value:String },
    ExecuteJs{ script:String },
    GetPageState,  
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection{
    Up, Down, Left, Right
}


#[derive(Debug,Serialize,Deserialize)]
pub struct BrowserState{
    pub url : String,
    pub title : String,
    pub html : Option<String>,
    pub screenshot: Option<Vec<u8>>,
    pub extracted_text: Option<String>,
    pub error: Option<String>,
    pub interactive_elements: Vec<InteractiveElement>, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub selector: String,
    pub element_type: String,  // button, link, input 등
    pub text: Option<String>,
    pub attributes: HashMap<String, String>,
}

