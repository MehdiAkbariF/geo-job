-- Full-Text Search column on Title (weight A) and Description (weight B)
ALTER TABLE opportunities ADD COLUMN IF NOT EXISTS search_vector tsvector
GENERATED ALWAYS AS (
    setweight(to_tsvector('simple', coalesce(title, '')), 'A') ||
    setweight(to_tsvector('simple', coalesce(description, '')), 'B')
) STORED;

-- GIN Index for sub-millisecond Full-Text Search
CREATE INDEX IF NOT EXISTS idx_opportunities_search_vector_gin 
ON opportunities USING GIN (search_vector);

-- Compound Index for public discovery visibility and cursor pagination
CREATE INDEX IF NOT EXISTS idx_opportunities_discovery 
ON opportunities (status, published_at DESC, id DESC) 
WHERE status = 'published';