use crate::components::error::error_cl::{Result, ErrorS};

/// LLM 응답 파서 - 텍스트 응답을 구조화된 데이터로 변환
/// 
/// # 지원 형식
/// - "action: click, value: 1, reason: 상품 링크 클릭"
/// - "action: scroll, value: 500, reason: 더 많은 상품 보기"
/// - "action: extract, value: 0, reason: 현재 페이지 정보 추출"
pub struct ResponseParser;

impl ResponseParser {
    /// LLM 응답을 파싱하여 (액션, 값, 이유) 튜플로 반환
    /// 
    /// # 입력 예시
    /// ```
    /// "action: click, value: 1, reason: 상품 링크를 클릭하여 상세 정보 확인"
    /// ```
    /// 
    /// # 출력
    /// ```
    /// ("click", 1, "상품 링크를 클릭하여 상세 정보 확인")
    /// ```
    pub fn parse_action_response(response: &str) -> Result<(String, i32, String)> {
        println!("🔍 Parsing LLM response: {}", response);
        
        let response = response.trim();
        let mut action = String::new();
        let mut value = 0i32;
        let mut reason = String::new();
        
        // 콤마로 분리하여 각 부분 파싱
        for part in response.split(',') {
            let part = part.trim();
            
            if part.starts_with("action:") {
                action = part.replace("action:", "").trim().to_string();
            } else if part.starts_with("value:") {
                if let Ok(parsed_value) = part.replace("value:", "").trim().parse::<i32>() {
                    value = parsed_value;
                }
            } else if part.starts_with("reason:") {
                reason = part.replace("reason:", "").trim().to_string();
            }
            // 숫자만 있는 경우도 value로 처리
            else if let Ok(parsed_num) = part.trim().parse::<i32>() {
                if value == 0 { // value가 아직 설정되지 않았다면
                    value = parsed_num;
                }
            }
        }
        
        // 기본값 설정
        if action.is_empty() {
            action = "unknown".to_string();
        }
        if reason.is_empty() {
            reason = "No reason provided".to_string();
        }
        
        println!("✅ Parsed - Action: {}, Value: {}, Reason: {}", action, value, reason);
        Ok((action, value, reason))
    }
    
    /// JSON 형식 응답 파싱 (향후 확장용)
    /// 
    /// # 입력 예시
    /// ```json
    /// {"action": "click", "value": 1, "reason": "상품 클릭"}
    /// ```
    pub fn parse_json_response(json_str: &str) -> Result<(String, i32, String)> {
        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| ErrorS::data("ResponseParser::parse_json_response", format!("JSON parse error: {}", e)))?;
        
        let action = parsed["action"].as_str().unwrap_or("unknown").to_string();
        let value = parsed["value"].as_i64().unwrap_or(0) as i32;
        let reason = parsed["reason"].as_str().unwrap_or("No reason provided").to_string();
        
        Ok((action, value, reason))
    }
    
    /// 다양한 형식 자동 감지 파싱
    pub fn parse_auto(response: &str) -> Result<(String, i32, String)> {
        let response = response.trim();
        
        // JSON 형식 감지
        if response.starts_with('{') && response.ends_with('}') {
            Self::parse_json_response(response)
        } 
        // 일반 텍스트 형식
        else {
            Self::parse_action_response(response)
        }
    }
    
    /// 응답 검증 - 유효한 액션인지 확인
    pub fn validate_action(action: &str) -> bool {
        match action.to_lowercase().as_str() {
            "click" | "scroll" | "extract" | "navigate" | "wait" | "screenshot" => true,
            _ => false,
        }
    }
    
    /// 안전한 파싱 - 실패시 기본값 반환
    pub fn parse_safe(response: &str) -> (String, i32, String) {
        match Self::parse_auto(response) {
            Ok((action, value, reason)) => {
                if Self::validate_action(&action) {
                    (action, value, reason)
                } else {
                    ("scroll".to_string(), 500, "기본 스크롤 액션".to_string())
                }
            },
            Err(_) => {
                println!("⚠️ 파싱 실패, 기본 액션 반환");
                ("scroll".to_string(), 500, "파싱 실패로 인한 기본 액션".to_string())
            }
        }
    }
}