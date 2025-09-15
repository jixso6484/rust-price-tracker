-- Products 테이블
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_name VARCHAR(255) NOT NULL,
    current_price DECIMAL(10, 2),
    original_price DECIMAL(10, 2),
    site VARCHAR(100) NOT NULL,
    category VARCHAR(100) NOT NULL,
    url TEXT,
    image TEXT NOT NULL,
    coupon_code VARCHAR(50),
    valid_until TIMESTAMPTZ,
    additional_benefits JSONB DEFAULT '[]',
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Price History 테이블 (독립적인 가격 추적)
CREATE TABLE IF NOT EXISTS price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    price DECIMAL(10, 2) NOT NULL,
    original_price DECIMAL(10, 2),
    discount_rate DECIMAL(5, 2),
    is_lowest BOOLEAN DEFAULT FALSE,
    recorded_at TIMESTAMPTZ DEFAULT NOW()
);

-- 인덱스 생성
CREATE INDEX idx_products_site ON products(site);
CREATE INDEX idx_products_category ON products(category);
CREATE INDEX idx_products_timestamp ON products(timestamp);
CREATE INDEX idx_price_history_product_id ON price_history(product_id);
CREATE INDEX idx_price_history_recorded_at ON price_history(recorded_at);
CREATE INDEX idx_price_history_is_lowest ON price_history(is_lowest) WHERE is_lowest = TRUE;