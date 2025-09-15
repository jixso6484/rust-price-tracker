use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum ErrorType {  // Rust 관례: CamelCase
    Browser,
    Llm,      // 약어도 보통 Llm으로
    Crawl,    // 오타 수정
    Data,
    Block,
}
#[derive(Debug)]
pub struct ErrorS{

    method: String,
    kind : ErrorType ,
    message : String,
    source : Option<Box<dyn Error+ Send + Sync>>
}
impl fmt::Display for ErrorS{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        
        match self.kind{
            ErrorType::Browser => write!(f, "browser error: {}", self.message),
            ErrorType::Llm => write!(f, "LLM error: {}", self.message),
            ErrorType::Crawl => write!(f, "crawl error: {}", self.message),
            ErrorType::Data => write!(f, "data error: {}", self.message),
            ErrorType::Block => write!(f, "blocked: {}", self.message),
        }?;
        write!(f,"(in {} )",self.method)?;

        Ok(())
    }   
}

impl Error for ErrorS{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        //as_ref Option 부분에서 함수는 참조로 소유권을 정리해준다 type casting 부분
        self.source.as_ref().map(|e| e.as_ref() as &(dyn Error + 'static))
    }

}
pub type Result<T> = std::result::Result<T, ErrorS>;

impl ErrorS{
    pub fn new(kind:ErrorType,method:impl Into<String>,message:impl Into<String>) -> Self {
        Self{
            kind,
            method: method.into(),
            message: message.into(),
            source : None
        }
    }
    pub fn browser(method : impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorType::Browser,method,message)
    }
    pub fn crawl(method : impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorType::Crawl,method,message)
    }
    pub fn data(method : impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorType::Data,method,message)
    }
    pub fn llm(method : impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorType::Llm,method,message)
    }
    pub fn with_source(mut self, source: impl Error + Send + Sync +'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

// From 구현들 (외부 라이브러리 에러를 ErrorS로 변환)
impl From<ort::OrtError> for ErrorS {
    fn from(err: ort::OrtError) -> Self {
        ErrorS::llm("ort", format!("ONNX Runtime error: {}", err))
    }
}

// headless_chrome 에러 처리로 변경
impl From<anyhow::Error> for ErrorS {
    fn from(err: anyhow::Error) -> Self {
        ErrorS::browser("headless_chrome", format!("Browser error: {}", err))
    }
}