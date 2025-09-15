// Simple test to verify LLM action validation
use serde_json;

#[derive(Debug, PartialEq)]
pub enum BrowserAction {
    Navigate { url: String },
    Click { selector: String },
    Scroll { direction: ScrollDirection, amount: u32 },
    Type { selector: String, text: String },
    Wait { milliseconds: u64 },
    Screenshot { path: String },
}

#[derive(Debug, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left, 
    Right,
}

fn parse_action_response(response: &str) -> Result<BrowserAction, String> {
    println!("🔍 LLM 응답 파싱 중: {}", response.chars().take(200).collect::<String>());
    
    // JSON 응답인지 체크
    let json_value: serde_json::Value = match serde_json::from_str(response.trim()) {
        Ok(json) => json,
        Err(_) => {
            // JSON이 아니면 기본 스크롤
            println!("❌ JSON 파싱 실패, 기본 스크롤로 복구");
            return Ok(BrowserAction::Scroll { direction: ScrollDirection::Down, amount: 500 });
        }
    };
    
    // action 필드 확인
    let action_type = match json_value.get("action") {
        Some(action) => action.as_str().unwrap_or("scroll"),
        None => {
            println!("❌ 'action' 필드가 없음");
            return Err("Missing action field".to_string());
        }
    };
    
    println!("✅ 파싱된 액션 타입: {}", action_type);
    
    // 액션에 따라 파라미터 추출 및 검증
    match action_type {
        "navigate" => {
            let url = json_value.get("url")
                .and_then(|u| u.as_str())
                .ok_or("Missing url field")?
                .to_string();
            
            // URL 검증
            if !url.starts_with("http") && !url.starts_with("/") {
                return Err(format!("Invalid URL format: {}", url));
            }
            
            println!("🔗 내비게이션 URL 검증 완료: {}", url);
            Ok(BrowserAction::Navigate { url })
        },
        "click" => {
            let selector = json_value.get("selector")
                .and_then(|s| s.as_str())
                .ok_or("Missing selector field")?
                .to_string();
                
            // 기본적인 CSS 선택자 검증
            if selector.trim().is_empty() {
                return Err("Empty selector".to_string());
            }
            
            println!("👆 클릭 선택자 검증 완료: {}", selector);
            Ok(BrowserAction::Click { selector })
        },
        "scroll" => {
            let direction_str = json_value.get("direction")
                .and_then(|d| d.as_str())
                .unwrap_or("down");
                
            let direction = match direction_str.to_lowercase().as_str() {
                "up" => ScrollDirection::Up,
                "down" => ScrollDirection::Down,
                "left" => ScrollDirection::Left,
                "right" => ScrollDirection::Right,
                _ => ScrollDirection::Down,
            };
            
            let amount = json_value.get("amount")
                .and_then(|a| a.as_u64())
                .unwrap_or(500) as u32;
                
            println!("📜 스크롤 검증 완료: {:?} {}px", direction, amount);
            Ok(BrowserAction::Scroll { direction, amount })
        },
        _ => {
            println!("❓ 알 수 없는 액션 타입: {}, 기본 스크롤로 복구", action_type);
            Ok(BrowserAction::Scroll { direction: ScrollDirection::Down, amount: 500 })
        }
    }
}

fn main() {
    println!("🧪 LLM 액션 검증 테스트 시작\n");
    
    // 테스트 케이스 1: 유효한 Navigate 액션
    let test1 = r#"{"action": "navigate", "url": "https://example.com"}"#;
    match parse_action_response(test1) {
        Ok(action) => println!("✅ 테스트 1 성공: {:?}\n", action),
        Err(e) => println!("❌ 테스트 1 실패: {}\n", e),
    }
    
    // 테스트 케이스 2: 유효한 Click 액션
    let test2 = r#"{"action": "click", "selector": ".product-link"}"#;
    match parse_action_response(test2) {
        Ok(action) => println!("✅ 테스트 2 성공: {:?}\n", action),
        Err(e) => println!("❌ 테스트 2 실패: {}\n", e),
    }
    
    // 테스트 케이스 3: 유효한 Scroll 액션
    let test3 = r#"{"action": "scroll", "direction": "down", "amount": 800}"#;
    match parse_action_response(test3) {
        Ok(action) => println!("✅ 테스트 3 성공: {:?}\n", action),
        Err(e) => println!("❌ 테스트 3 실패: {}\n", e),
    }
    
    // 테스트 케이스 4: 잘못된 JSON (복구 테스트)
    let test4 = "invalid json response";
    match parse_action_response(test4) {
        Ok(action) => println!("✅ 테스트 4 성공 (복구): {:?}\n", action),
        Err(e) => println!("❌ 테스트 4 실패: {}\n", e),
    }
    
    // 테스트 케이스 5: action 필드 없음
    let test5 = r#"{"url": "https://example.com"}"#;
    match parse_action_response(test5) {
        Ok(action) => println!("✅ 테스트 5 성공: {:?}\n", action),
        Err(e) => println!("❌ 테스트 5 실패: {}\n", e),
    }
    
    println!("🏁 테스트 완료!");
}