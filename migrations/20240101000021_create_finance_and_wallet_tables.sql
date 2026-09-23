-- =============================================================================
-- MIGRATION: FINANCIAL INFRASTRUCTURE, WALLETS, LEDGER & TARIFFS
-- =============================================================================

BEGIN;

-- ۱. جدول کیف پول اختصاصی سازمان / شرکت / مغازه
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    cash_balance NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    gift_balance NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    currency VARCHAR(3) NOT NULL DEFAULT 'IRR',
    is_frozen BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_wallet_company UNIQUE (company_id)
);

CREATE INDEX IF NOT EXISTS idx_wallets_company ON wallets (company_id);

-- ۲. دفتر کل حسابداری تغییرات موجودی (Immutable Transaction Ledger)
CREATE TABLE IF NOT EXISTS wallet_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    amount NUMERIC(15, 2) NOT NULL,
    cash_amount NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    gift_amount NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    balance_after NUMERIC(15, 2) NOT NULL,
    tx_type VARCHAR(32) NOT NULL,
    reference_type VARCHAR(64) NOT NULL,
    reference_id UUID,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wallet_tx_wallet ON wallet_transactions (wallet_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_wallet_tx_reference ON wallet_transactions (reference_type, reference_id);

-- ۳. جدول تعرفه‌نامه داینامیک خدمات پلتفرم (Tariffs & Credit Packages)
CREATE TABLE IF NOT EXISTS tariffs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(64) NOT NULL UNIQUE,
    title VARCHAR(150) NOT NULL,
    description TEXT,
    category VARCHAR(32) NOT NULL,
    price NUMERIC(15, 2) NOT NULL,
    gift_credit NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    validity_days INT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- سید کردن بسته‌های اعتباری استاندارد و خدمات پلتفرم
INSERT INTO tariffs (code, title, description, category, price, gift_credit, validity_days)
VALUES 
    ('pkg_2m', 'بسته اعتباری برنزی (۲ میلیون تومانی)', 'مناسب برای ثبت ۱ تا ۲ آگهی شغلی', 'credit_package', 20000000.00, 0.00, 30),
    ('pkg_4m', 'بسته اعتباری نقره‌ای (۴ میلیون تومانی)', 'شامل ۴۰۰ هزار تومان شارژ هدیه رایگان', 'credit_package', 36000000.00, 4000000.00, 45),
    ('pkg_7m', 'بسته اعتباری طلایی (۷ میلیون تومانی)', 'شامل ۱.۱ میلیون تومان شارژ هدیه رایگان', 'credit_package', 59000000.00, 11000000.00, 60),
    ('pkg_10m', 'بسته اعتباری الماس (۱۰ میلیون تومانی)', 'شامل ۲.۱ میلیون تومان شارژ هدیه رایگان', 'credit_package', 79000000.00, 21000000.00, 90),
    ('srv_ad_standard', 'ثبت و انتشار آگهی شغلی استاندارد', 'نمایش ۳۰ روزه آگهی بر روی نقشه و در نتایج جستجو', 'single_service', 2450000.00, 0.00, 30),
    ('srv_ad_urgent', 'نشان استخدام فوری', 'برچسب فوری و اولویت در پیشنهادها به مدت ۴۸ ساعت', 'add_on', 1500000.00, 0.00, 2),
    ('srv_ad_ladder', 'نردبان آگهی روی نقشه', 'بازگشت آگهی به صدر نتایج جستجو و رفرش تاریخ', 'add_on', 500000.00, 0.00, NULL),
    ('srv_featured_pin', 'سنجاق طلایی نقشه (Featured Pin)', 'پین متمایز طلایی در تمام زوم‌های نقشه به مدت ۷ روز', 'add_on', 1500000.00, 0.00, 7),
    ('srv_talent_unlock', 'خرید اطلاعات تماس کارجو از نقشه', 'دسترسی دائمی به شماره موبایل، رزومه و مشخصات کامل کارجو', 'single_service', 700000.00, 0.00, NULL)
ON CONFLICT (code) DO NOTHING;

-- ۴. جدول فاکتورهای رسمی خرید (Invoices)
CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_number VARCHAR(32) NOT NULL UNIQUE,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    tariff_id UUID NOT NULL REFERENCES tariffs(id),
    subtotal NUMERIC(15, 2) NOT NULL,
    tax_amount NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    total_amount NUMERIC(15, 2) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(32) NOT NULL DEFAULT 'mock_gateway',
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_invoices_company ON invoices (company_id, status);

-- ۵. جدول دسترسی‌های بازشده به شماره تماس کارجویان
CREATE TABLE IF NOT EXISTS candidate_contact_unlocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    candidate_id UUID NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
    unlocked_by_user_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_company_candidate_unlock UNIQUE (company_id, candidate_id)
);

CREATE INDEX IF NOT EXISTS idx_cand_unlock_company ON candidate_contact_unlocks (company_id);

COMMIT;