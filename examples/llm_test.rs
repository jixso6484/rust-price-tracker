use sexy_crawling::infrastruction::llm::llmRepository::LocalLLM;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Qwen3 1.7B LLM 테스트 시작...");
    
    // LLM 인스턴스 생성
    println!("📥 LLM 인스턴스 초기화 중...");
    let llm = match LocalLLM::get_instance().await {
        Ok(llm) => llm,
        Err(e) => {
            println!("❌ LLM 초기화 실패: {}", e);
            println!("💡 ONNX 모델 파일이 src/infrastruction/llm/model_int8.onnx 경로에 있는지 확인하세요.");
            return Err(e.into());
        }
    };
    
    println!("✅ LLM 인스턴스 생성 성공!");
    
    // 간단한 텍스트 생성 테스트
    let test_prompt = "안녕하세요! 간단한 인사말을 해주세요.";
    println!("\n🔤 테스트 프롬프트: {}", test_prompt);
    
    match llm.generate(test_prompt).await {
        Ok(response) => {
            println!("✅ 응답 생성 성공!");
            println!("📄 응답: {}", response);
        },
        Err(e) => {
            println!("❌ 응답 생성 실패: {}", e);
        }
    }
    
    // 토큰 계산 테스트
    let token_count = llm.count_tokens(test_prompt);
    println!("\n📏 프롬프트 토큰 수: {}", token_count);
    
    // HTML 분석 테스트 (Mock 데이터 사용)
    let mock_html = r#"
        <div class="product">
            <h1>테스트 상품</h1>
            <span class="price">29,900원</span>
            <button class="btn-more">더보기</button>
        </div>
    "#;
    
    println!("\n🔍 HTML 상품 페이지 분석 테스트...");
    match llm.check_if_page_has_products(mock_html).await {
        Ok(has_products) => {
            println!("📊 상품 페이지 여부: {}", has_products);
        },
        Err(e) => {
            println!("❌ HTML 분석 실패: {}", e);
        }
    }
    
    println!("\n🎉 모든 테스트 완료!");
    Ok(())
}