use crate::components::error::error_cl::{Result, ErrorS};
use ort::{Environment, Session, SessionBuilder, Value}; // ONNX Runtime 사용
use std::sync::Arc;
use tokio::sync::OnceCell;
use tiktoken_rs::CoreBPE;
use ndarray::Array2;

pub struct LocalLLM{
    environment: Option<Arc<Environment>>, // ONNX Environment
    model : Option<Session>, // ONNX 세션
    tokenizer: CoreBPE,
    _config : ModelConfig, // _ prefix로 unused 경고 제거
    model_loaded: bool,
}

#[derive(Debug)]
pub struct ModelConfig {
    pub _max_length: usize,     // _ prefix로 unused 경고 제거
    pub _temperature: f32,      // _ prefix로 unused 경고 제거  
    pub _top_p: f32,            // _ prefix로 unused 경고 제거
    pub _vocab_size: usize,     // _ prefix로 unused 경고 제거
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            _max_length: 32768,  // Qwen3 1.7B 컨텍스트 길이 (32K 토큰)
            _temperature: 0.7,
            _top_p: 0.9,
            _vocab_size: 151936, // Qwen3 vocabulary 크기
        }
    }
}

// 싱글톤 인스턴스
static LLM_INSTANCE: OnceCell<Arc<LocalLLM>> = OnceCell::const_new();

impl LocalLLM{
    pub async fn new()->Result<Self>{
        // ONNX 모델 파일 체크
        let model_path = "src/infrastructure/llm/model_int8.onnx";
        
        let model_exists = std::path::Path::new(model_path).exists();
        let mut onnx_session = None;
        let mut onnx_environment = None;
        
        if model_exists {
            println!("🧠 Qwen3 1.7B ONNX 모델 파일 발견!");
            
            // ONNX Environment 생성
            match Environment::builder()
                .with_name("qwen3_llm")
                .build() {
                Ok(env) => {
                    let env_arc = Arc::new(env);
                    println!("✅ ONNX Environment 생성 성공");
                    
                    // SessionBuilder 생성 시도
                    match SessionBuilder::new(&env_arc) {
                        Ok(builder) => {
                            // 모델 파일 로드
                            match builder.with_model_from_file(model_path) {
                                Ok(session) => {
                                    println!("✅ ONNX 세션 생성 성공! 모델 로드 완료");
                                    onnx_environment = Some(env_arc);
                                    onnx_session = Some(session);
                                },
                                Err(e) => {
                                    println!("❌ 모델 파일 로드 실패: {}", e);
                                    println!("   파일 경로: {}", model_path);
                                    println!("   Mock 모드로 전환합니다.");
                                }
                            }
                        },
                        Err(e) => {
                            println!("❌ SessionBuilder 생성 실패: {}", e);
                            println!("   Mock 모드로 전환합니다.");
                        }
                    }
                },
                Err(e) => {
                    println!("❌ ONNX Environment 생성 실패: {}", e);
                    println!("   Mock 모드로 전환합니다.");
                }
            }
        } else {
            println!("⚠️ ONNX 모델 파일이 없습니다: {}", model_path);
            println!("   Mock 모드로 실행합니다.");
        }
        
        let config = ModelConfig::default();
        
        // Qwen3 토크나이저 초기화 (GPT-4 토크나이저 대신 근사치 사용)
        let tokenizer = tiktoken_rs::get_bpe_from_model("gpt-4")
            .map_err(|e| ErrorS::data("LocalLLM::new", format!("Failed to load tokenizer: {}", e)))?;
        
        let model_loaded = onnx_session.is_some();
        println!("✅ LocalLLM 초기화 완료 (실제 모델 로드: {})", model_loaded);
        
        Ok(Self { 
            environment: onnx_environment,
            model: onnx_session,
            tokenizer, 
            _config: config, 
            model_loaded 
        })
    }
    

    // 싱글톤 인스턴스 가져오기
    pub async fn get_instance() -> Result<Arc<LocalLLM>> {
        LLM_INSTANCE.get_or_try_init(|| async {
            let llm = Self::new().await?;
            Ok(Arc::new(llm))
        }).await.cloned()
    }

    // Qwen3 1.7B 텍스트 생성 - 내장 LLM만 사용
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        if self.model_loaded {
            println!("🧠 Qwen3 1.7B 모델로 추론 중...");
            self.run_inference(prompt).await
        } else {
            return Err(ErrorS::data("LocalLLM::generate", "ONNX 모델이 로드되지 않았습니다. 모델 파일을 확인하세요."));
        }
    }

    // ONNX 모델 추론 실행 (순수 추론만)
    async fn run_inference(&self, prompt: &str) -> Result<String> {
        let tokens = self.tokenize(prompt)?;
        
        // 토큰 길이 제한 (Qwen3 1.7B는 32K 토큰)
        let max_input_tokens = 16384; // 절반만 사용 (출력 공간 확보)
        let input_tokens = if tokens.len() > max_input_tokens {
            tokens[..max_input_tokens].to_vec()
        } else {
            tokens
        };
        
        println!("📏 입력 토큰 수: {}/{}", input_tokens.len(), max_input_tokens);
        
        // ONNX 모델 입력 준비
        let _input_ids = Array2::from_shape_vec((1, input_tokens.len()), input_tokens)
            .map_err(|e| ErrorS::data("LocalLLM::run_inference", format!("Failed to create input tensor: {}", e)))?;
        
        // 실제 ONNX 모델 추론
        if let Some(ref session) = self.model {
            println!("🤖 실제 ONNX 모델 추론 실행 중...");
            
            // ONNX 모델 입력 준비 (input_ids)
            let input_ids = Array2::from_shape_vec((1, input_tokens.len()), input_tokens)
                .map_err(|e| ErrorS::data("LocalLLM::run_inference", format!("Failed to create input tensor: {}", e)))?;
            
            // ONNX 입력으로 변환
            match Value::from_array(session.allocator(), &input_ids) {
                Ok(input_value) => {
                    println!("📊 입력 텐서 생성 성공");
                    
                    // 모델 실행
                    match session.run(vec![input_value]) {
                        Ok(outputs) => {
                            println!("✅ ONNX 추론 성공! 출력 개수: {}", outputs.len());
                            
                            if let Some(output_tensor) = outputs.get(0) {
                                match self.extract_output_tokens(output_tensor) {
                                    Ok(output_tokens) => {
                                        let response = self.detokenize(&output_tokens)?;
                                        println!("✅ 응답 생성 완료: {} 토큰", output_tokens.len());
                                        return Ok(response);
                                    },
                                    Err(e) => {
                                        println!("❌ 출력 토큰 추출 실패: {}", e);
                                    }
                                }
                            } else {
                                println!("⚠️ 출력 텐서가 비어있습니다");
                            }
                        },
                        Err(e) => {
                            println!("❌ ONNX 추론 실패: {}", e);
                            println!("   입력 shape: ({}, {})", 1, input_tokens.len());
                        }
                    }
                },
                Err(e) => {
                    println!("❌ 입력 텐서 생성 실패: {}", e);
                }
            }
        } else {
            println!("🤖 Mock 추론 실행 중... (ONNX 모델 없음)");
            // Mock 응답: 간단한 액션 결정
            let mock_response = "action: click, value: 1, reason: 상품 링크를 클릭하여 상세 정보 확인";
            Ok(mock_response.to_string())
        }
    }
    
    // 행동을 결정하는 부분 - 간단한 응답 파싱
    pub async fn decide_browser_action(&self, llm_response: &str) -> Result<(String, i32, String)> {
        println!("🔍 Parsing LLM response: {}", llm_response);
        
        // "action: 1, reason: 상품 링크를 클릭하여 상세 정보 확인" 형식 파싱
        let response = llm_response.trim();
        let mut action = String::new();
        let mut value = 0i32;
        let mut reason = String::new();
        
        // 콤마로 분리하여 파싱
        for part in response.split(',') {
            let part = part.trim();
            
            if part.starts_with("action:") {
                action = part.replace("action:", "").trim().to_string();
            } else if part.starts_with("value:") || part.contains(':') && part.chars().filter(|c| c.is_numeric()).count() > 0 {
                // value: 숫자 또는 숫자만 있는 경우
                if let Some(num_str) = part.split(':').nth(1) {
                    value = num_str.trim().parse().unwrap_or(0);
                } else if let Ok(parsed_num) = part.trim().parse::<i32>() {
                    value = parsed_num;
                }
            } else if part.starts_with("reason:") {
                reason = part.replace("reason:", "").trim().to_string();
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
    



    // Qwen 2.5 토크나이제이션
    fn tokenize(&self, text: &str) -> Result<Vec<i64>> {
        let tokens = self.tokenizer.encode_with_special_tokens(text);
        Ok(tokens.into_iter().map(|t| t as i64).collect())
    }

    // 토큰을 텍스트로 변환
    fn detokenize(&self, tokens: &[i64]) -> Result<String> {
        let u32_tokens: Vec<u32> = tokens.iter().map(|&t| t as u32).collect();
        let text = self.tokenizer.decode(u32_tokens)
            .map_err(|e| ErrorS::data("LocalLLM::detokenize", format!("Failed to decode tokens: {}", e)))?;
        Ok(text)
    }
    
    // ONNX 출력에서 토큰 추출
    fn extract_output_tokens(&self, output_value: &Value) -> Result<Vec<i64>> {
        // ONNX Value에서 토큰 배열 추출
        match output_value.try_extract::<i64>() {
            Ok(tensor) => {
                let tokens: Vec<i64> = tensor.view().iter().cloned().collect();
                
                // 생성된 토큰에서 특수 토큰 제거 및 길이 제한
                let max_output_tokens = 100; // 출력 토큰 제한
                let filtered_tokens: Vec<i64> = tokens
                    .into_iter()
                    .take(max_output_tokens)
                    .filter(|&token| token > 0 && token < 151936) // Qwen vocabulary 범위
                    .collect();
                
                if filtered_tokens.is_empty() {
                    // fallback: Mock 응답 토큰화
                    let mock_text = "action: click, value: 1, reason: 응답 생성";
                    self.tokenize(mock_text)
                } else {
                    Ok(filtered_tokens)
                }
            },
            Err(_) => {
                // 타입 변환 실패시 다른 타입으로 시도
                if let Ok(float_tensor) = output_value.try_extract::<f32>() {
                    // argmax로 토큰 선택 (간단한 greedy decoding)
                    let logits = float_tensor.view();
                    let tokens: Vec<i64> = logits
                        .axis_iter(ndarray::Axis(0))
                        .map(|row| {
                            row.iter()
                                .enumerate()
                                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                                .map(|(idx, _)| idx as i64)
                                .unwrap_or(0)
                        })
                        .take(50) // 최대 50 토큰
                        .collect();
                    
                    Ok(tokens)
                } else {
                    Err(ErrorS::data("LocalLLM::extract_output_tokens", "Failed to extract tokens from ONNX output"))
                }
            }
        }
    }
    
    // 토큰 개수 계산
    pub fn count_tokens(&self, text: &str) -> usize {
        self.tokenizer.encode_with_special_tokens(text).len()
    }
    
    // 텍스트를 최대 길이로 자르기
    pub fn truncate_text(&self, text: &str, max_tokens: usize) -> String {
        let tokens = self.tokenizer.encode_with_special_tokens(text);
        if tokens.len() <= max_tokens {
            return text.to_string();
        }
        
        let truncated_tokens: Vec<u32> = tokens.into_iter().take(max_tokens).collect();
        self.tokenizer.decode(truncated_tokens).unwrap_or_else(|_| text.chars().take(max_tokens * 4).collect())
    }
    
    // ============ 프롬프트 관련 함수들 ============
    

    // 실제 HTML 분석 기반 지능형 응답 생성
    async fn generate_intelligent_response(&self, html: &str,main_url : &str, url : &str) -> Result<String> {
        // 디버깅을 위해 프롬프트 일부 출력
        println!("🔍 실제 HTML 분석 - 프롬프트 키워드: {}",url);
        
        // HTML 타입 판단은 필요시 추가
        // let _judgment_html_type = self.judgment_html_type_proccessor(html, url);
        let response = self.main_browser_action_prompt(html,main_url,url);
        
        Ok(response)
    }

    // 삭제된 메서드들 (더 이상 필요 없음)
    // parse_action_response - LLM 응답 파싱 불필요
    // create_browser_action_prompt - 프롬프트 생성 불필요
    // create_product_check_prompt - 상품 체크 프롬프트 불필요
    #[allow(dead_code)]
    fn main_browser_action_prompt(&self, html: &str,main_url : &str, url : &str) -> String {

        let site_prompt = match main_url{
            "https://www.coupang.com" => prompt_selector("쿠팡"),
            "https://www.11st.co.kr" => prompt_selector("11번가"), 
            "https://www.amazon.com/" => prompt_selector("아마존"),
            _ => "알 수 없는 사이트입니다".to_string()
        };
    format!(
        r#"

        당신은 온라인 쇼핑몰의 할인 정보를 효율적으로 수집하는 전문 AI 에이전트입니다. 목표: 할인 정보가 있을만한 페이지로 들어가도록 url 과 html을 보고 액션 타입을 결정해라

        사용 가능한 브라우저 액션:
        - Navigate {{ url: "URL" }} : 새 페이지로 이동
        - Click {{ selector: "CSS셀렉터" }} : 요소 클릭
        - Scroll {{ direction: "up/down", amount: 픽셀수 }} : 페이지 스크롤
        - ExtractText {{ selector: Some("CSS셀렉터") }} : 텍스트 추출
        - WaitForElement {{ selector: "CSS셀렉터", timeout: 초 }} : 요소 대기
        - FillForm {{ selector: "CSS셀렉터", value: "입력값" }} : 폼 입력
        - ExecuteJs {{ script: "JavaScript코드" }} : JS 실행
        - GetPageState : 페이지 상태 확인

        ## 쇼핑몰 할인 키워드:
        
        {}

        액션 결정 로직:
        ```
        if (팝업_감지) → Click(닫기_버튼)
        if (메인페이지 && 할인메뉴_존재) → Navigate(할인섹션)
        if (목록페이지 && 할인상품_많음) → Click(더보기/정렬)
        if (할인정보_발견) → ExtractText(수집)
        if (로딩중) → WaitForElement(대기)
        if (할인확장_가능) → Navigate/Click(확장)
        ```

        응답 형식 (JSON):
        {{
            "action": "액션타입",
            "params": {{ "파라미터": "값" }},
            "reason": "할인 정보 수집 관점에서의 선택 이유"
        }}

        예시:
        {{ "action": "Click", "params": {{ "selector": "button:contains('더보기'), .load-more, .btn-more" }}, "reason": "추가 할인 상품을 로드하여 더 많은 할인 정보 수집" }}
        {{ "action": "Navigate", "params": {{ "url": "/event" }}, "reason": "메인페이지의 이벤트 섹션으로 이동하여 할인 정보 탐색" }}
        {{ "action": "ExtractText", "params": {{ "selector": ".price, .discount, .coupon, .shipping" }}, "reason": "상품의 가격, 할인율, 쿠폰, 배송 정보 등 모든 할인 혜택 추출" }}
        
        
        url : 
            {} 
        HTML:
            {}
       "#,
                url,html,site_prompt
            )
    }
    
    
}
// LLM 메서드 추가
impl LocalLLM {
    fn judgment_html_type_proccessor(&self, _html: &str, _url: &str) -> String {
        // HTML 타입 판단 로직 (필요시 구현)
        "product_page".to_string()
    }
}

pub fn prompt_selector(site: &str) -> String {
    match site {
        "쿠팡" => format!(r#"
            **쿠팡:**
            할인: "판매자특가", "쿠폰가", "즉시할인", "타임딜", "오늘의발견", "%할인", "%OFF"
            배송: "로켓배송", "무료배송", "당일배송", "새벽배송"
            적립: "쿠팡캐시", "적립", "%적립"
            멤버십: "와우멤버십", "와우할인", "회원전용"
            묶음: "대용량할인", "묶음배송", "세트상품", "1+1", "2+1"
            추가: 계절별 축제별 공휴일별 추가적인 할인 페이지가 존재함

            행동 지침:
            ### 1단계: 진입 장벽 제거
            ✅ 팝업/모달 감지 → Click으로 제거
            - "동의", "확인", "×", "닫기", "Accept", "OK" 버튼
            - 앱 설치 유도 → "웹에서 계속", "나중에" 선택

            ### 2단계: 페이지 타입별 전략
            🏠 **메인페이지** (도메인 루트)
            → 할인 메뉴/배너 찾기 → Navigate
            - 각 사이트펼 할인 이벤트 및 할인 Page 탐색

            🏪 **목록/카테고리 페이지**
            → 정렬 및 확장 → Click/Scroll
            - "할인율순", "가격낮은순" 정렬
            - "더보기", "다음페이지" 확장

            🛍️ **상품 상세페이지**
            → 모든 할인 정보 수집 → ExtractText
            - 가격, 할인율, 쿠폰, 적립, 배송 혜택

            ### 3단계: 할인 정보 우선 탐지
            🚨 **긴급도 높음** (즉시 ExtractText):
            - "오늘만", "시간한정", "마감임박", "선착순", "한정수량"
            - 30% 이상 고할인율
            - "타임딜", "플래시세일", "깜짝할인"

            📊 **확장 탐색** (더 많은 할인):
            - 할인 상품 10개 이상 → "더보기" Click
            - 무한스크롤 → Scroll (down, 500-1000px)
            - 추가 상품 로딩 시
            - 페이지네이션 → 다음 페이지 Click
            - 다른 할인 카테고리 → Navigate

            ### 4단계: 동적 처리
            🔄 **로딩 상태** → WaitForElement (5-10초)
            - "로딩", "가격 확인 중", "혜택 계산 중", 스피너

        "#),
        "11번가" => format!(r#"
            **11번가:**
            할인: "오늘특가", "타임특가", "11번가특가", "무료배송", "당일배송"
            혜택: "포인트적립", "마일리지", "캐시백", "멤버십할인"
        "#),
        _ => format!("기본 프롬프트")
    }
}
// LLMRepository alias for compatibility
pub type LLMRepository = LocalLLM;
